/**
 * Handles painting traversals / routes, given raw geojson availability
 */
import { computed } from 'nanostores';
import L from 'leaflet';
import { Feature } from 'geojson';
import { PropKey, TraversalDefaults } from '../consts.ts';
import DebugPopup from '../components/DebugPopup.svelte';
const { stepDelayMs } = TraversalDefaults;

import Rainbow from 'rainbowvis.js';

import { $raw, RouteMetadata } from './fetch.ts';
import { $withTraversal } from './route.ts';

export const addDebugClick = (feature: Feature, layer: L.Layer) => {
  const featurePopupDiv = L.DomUtil.create('div', 'feature-popup');

  L.DomEvent.disableClickPropagation(featurePopupDiv).disableScrollPropagation(
    featurePopupDiv
  );

  if (feature.properties) {
    new DebugPopup({
      target: featurePopupDiv,
      props: {
        properties: feature.properties,
      },
    });
    layer.bindPopup(featurePopupDiv);
  }
};

const getRouteDepthAndSteps = () => {
  const features = $raw.get()?.route?.features;
  if (!features) return {};

  const properties = features[features.length - 1].properties;

  if (!properties) return {};
  const depth: number = properties[PropKey.Depth];
  const steps: number = properties[PropKey.Index];

  return { depth, steps };
};

const $routeStyle = computed($withTraversal, (withTraversal) => {
  return (feature: Feature | undefined) => {
    if (!feature?.properties) {
      console.error(`unable to style feature: ${JSON.stringify(feature)}`);
      return {};
    }

    let className = `route-depth-${feature.properties[PropKey.Depth]} step-${feature.properties[PropKey.Index]
      }`;
    if (withTraversal) className += ' svg-path';

    return {
      opacity: withTraversal ? 0 : 1,
      weight: withTraversal ? 5 : 3,
      className,
    };
  };
});

// the number of CSS classes created for the `depth` animation
let depthClassesCreated = 0;

const ensureDepthAnimationClassesExist = (depth: number) => {
  if (depth > depthClassesCreated) {
    console.log(`generating classes from ${depthClassesCreated} to ${depth}`);
    const styleSheet = document.createElement('style');

    let classes = '';
    for (let i = depthClassesCreated + 1; i <= depth; i++) {
      classes += `.depth-${i} { animation-delay: ${i * stepDelayMs}ms; }\n`;
    }

    styleSheet.innerText = classes;
    document.head.appendChild(styleSheet);

    depthClassesCreated = depth;
  }
};

const getTraversalStyleFunc = (meta: RouteMetadata) => {
  const { max_depth, cost_range } = meta;
  const rainbow = new Rainbow();

  let color = (_properties: Record<string, number>): string => '#F26F75';

  rainbow.setNumberRange(cost_range[0], cost_range[1]);
  rainbow.setSpectrum('#2BEA01', '#A9A9A9', '#A9A9A9', 'red');
  color = (properties) =>
    `#${rainbow.colourAt(
      properties[PropKey.Cost] / properties[PropKey.Length]
    )}`;

  ensureDepthAnimationClassesExist(max_depth);

  return (feature: Feature | undefined) => {
    if (!feature?.properties) {
      console.error(`unable to style feature: ${JSON.stringify(feature)}`);
      return {};
    }

    return {
      color: color(feature.properties),
      opacity: 0, // start off invisible
      className: `svg-path depth-${feature.properties[PropKey.Depth]}`,
    };
  };
};

export const $traversalLayer = computed([$raw], (json) => {
  // if no geojson
  // return an empty feature group / clear the map
  if (!json || !json.traversal) return null;

  // if traversal exists, paint it
  return L.geoJSON(json.traversal, {
    style: getTraversalStyleFunc(json.meta),
    onEachFeature: addDebugClick,
    bubblingMouseEvents: false,
  });
});

export const $routeLayer = computed([$raw], (json) => {
  // if no geojson
  // return an empty feature group / clear the map
  if (!json || !json.route) return null;

  return L.geoJSON(json.route, {
    style: $routeStyle.get(),
    onEachFeature: addDebugClick,
    bubblingMouseEvents: false,
  });
});

// callback invoked after the feature group has been added to the Dom
// animates traversals
export const onGeoJsonAdded = async () => {
  const withTraversal = $withTraversal.get();

  // if visualizing traversal, get the depth and step count from the last step in the route response
  // and paint route animation in reverse from end to start
  if (withTraversal) {
    const features = $raw.get()?.route?.features;
    if (!features) return;
    const { depth, steps } = getRouteDepthAndSteps();
    if (!depth || !steps) return;

    for (let i = 0; i <= steps; i++) {
      const collection = document.getElementsByClassName(
        `step-${steps - i}`
      ) as HTMLCollectionOf<HTMLElement>;
      for (let j = 0; j < collection.length; j++) {
        const feature = collection.item(j);
        feature?.style.setProperty(
          'animation-delay',
          `${depth * stepDelayMs + i * stepDelayMs}ms`
        );
      }
    }
  }
};
