/**
 * Handles painting traversals / routes, given raw geojson availability
 */
import { computed } from 'nanostores';
import L from 'leaflet';
import Handlebars from 'handlebars';
import { Feature } from 'geojson';
import { Mode } from '../consts.ts';

import Rainbow from 'rainbowvis.js';
const rainbow = new Rainbow();

import { $clickTime } from './map.ts';
import { $mode } from './mode.ts';
import { $paint, $depth } from './traversal.ts';
import { $raw } from './geojson.ts';

const OSMKeys = ['from', 'to', 'way'];
Handlebars.registerHelper('isNotOSMKey', (key) => !OSMKeys.includes(key));
Handlebars.registerHelper('isStartNode', (id) => id === -1);
Handlebars.registerHelper('isEndNode', (id) => id === -2);

import debugPopupTemplate from '../templates/debugPopup.hbs?raw';
const compiledDebugPopupTemplate = Handlebars.compile(debugPopupTemplate);

export const addDebugClick = (feature: Feature, layer: L.Layer) => {
  const featurePopupDiv = L.DomUtil.create('div', 'feature-popup');

  L.DomEvent.disableClickPropagation(featurePopupDiv).disableScrollPropagation(
    featurePopupDiv
  );

  if (feature.properties) {
    featurePopupDiv.innerHTML = compiledDebugPopupTemplate(feature);
    layer.bindPopup(featurePopupDiv);
  }
};

const $routeStyle = computed($mode, (mode) => {
  return (feature: Feature | undefined) => {
    if (!feature?.properties) {
      console.error(`unable to style feature: ${JSON.stringify(feature)}`);
      return {};
    }

    let className = `route-depth-${feature.properties.depth} step-${feature.properties.idx}`;
    if (mode === Mode.RouteViz) className += ' svg-path';

    return {
      opacity: mode === Mode.RouteViz ? 0 : 1,
      weight: mode === Mode.RouteViz ? 5 : 3,
      className,
    };
  };
});

const $traversalStyle = computed(
  [$mode, $paint, $depth],
  (mode, paint, depth) => {
    let color = (_properties: Record<string, number>): string => '#F26F75';

    if (mode === Mode.Traverse) {
      switch (paint) {
        case 'depth':
          rainbow.setNumberRange(1, depth);
          break;
        case 'length':
          rainbow.setNumberRange(1, 300);
          break;
        case 'distance_so_far':
          rainbow.setNumberRange(1, depth * 50);
          break;
        default:
      }

      color = (properties) => `#${rainbow.colourAt(properties[paint])}`;
    }

    return (feature: Feature | undefined) => {
      if (!feature?.properties) {
        console.error(`unable to style feature: ${JSON.stringify(feature)}`);
        return {};
      }

      return {
        color: color(feature.properties),
        opacity: 0, // start off invisible
        className: `svg-path depth-${feature.properties.depth}`,
      };
    };
  }
);

export const $featureGroup = computed([$clickTime, $raw], (clickTime, json) => {
  const featureGroup = new L.FeatureGroup([]);

  // if no geojson or if the new map click happened recently,
  // return an empty feature group / clear the map
  if (!json || Date.now() - clickTime < 10) return featureGroup;

  // if traversal exists, paint it
  if (json.traversal) {
    L.geoJSON(json.traversal, {
      style: $traversalStyle.get(),
      onEachFeature: addDebugClick,
      bubblingMouseEvents: false,
    }).addTo(featureGroup);
  }

  // if route exists, paint it
  if (json.route) {
    L.geoJSON(json.route, {
      style: $routeStyle.get(),
      onEachFeature: addDebugClick,
      bubblingMouseEvents: false,
    }).addTo(featureGroup);
  }

  return featureGroup;
});

// callback invoked after the feature group has been added to the Dom
// animates traversals
export const onFeatureGroupAdded = async () => {
  const mode = $mode.get();
  let depth;
  let steps;

  // if RouteViz, get the depth and step count from the last step in the route response
  if (mode === Mode.RouteViz) {
    const features = $raw.get()?.route.features;
    if (!features) return;
    depth = features[features?.length - 1].properties?.depth;
    steps = features[features?.length - 1].properties?.idx;
    if (!depth) return;
  } else if (mode === Mode.Traverse) {
    depth = $depth.get();
  } else {
    return;
  }

  for (let i = 0; i <= depth; i++) {
    const collection = document.getElementsByClassName(
      `depth-${i}`
    ) as HTMLCollectionOf<HTMLElement>;
    for (let j = 0; j < collection.length; j++) {
      const feature = collection.item(j);
      feature?.style.setProperty('animation-delay', `${i * 75}ms`);
      feature?.style.setProperty('animation-play-state', 'running');
    }
  }

  // when doing route visualization, also paint route after traversal, in reverse
  if (mode === Mode.RouteViz) {
    for (let i = 0; i <= steps; i++) {
      const collection = document.getElementsByClassName(
        `step-${steps - i}`
      ) as HTMLCollectionOf<HTMLElement>;
      for (let j = 0; j < collection.length; j++) {
        const feature = collection.item(j);
        feature?.style.setProperty(
          'animation-delay',
          `${depth * 75 + i * 75}ms`
        );
        feature?.style.setProperty('animation-play-state', 'running');
      }
    }
  }
};

document.pause = () => {
  const collection = document.getElementsByClassName(
    `svg-path`
  ) as HTMLCollectionOf<HTMLElement>;

  const state = collection
    .item(0)
    ?.style.getPropertyValue('animation-play-state');
  const desired = state === 'running' ? 'paused' : 'running';

  for (let i = 0; i <= collection.length; i++) {
    const feature = collection.item(i);
    feature?.style.setProperty('animation-play-state', desired);
  }
};
