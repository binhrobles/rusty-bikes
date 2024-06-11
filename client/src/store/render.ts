/**
 * Handles painting traversals / routes, given raw geojson availability
 */
import { computed } from 'nanostores';
import L from 'leaflet';
import Handlebars from 'handlebars';
import { Feature } from 'geojson';
import { Mode, PaintOptions, PropKey, TraversalDefaults } from '../consts.ts';
const { stepDelayMs } = TraversalDefaults;

import Rainbow from 'rainbowvis.js';

import { $clickTime } from './map.ts';
import { $mode } from './mode.ts';
import { $paint, $depth } from './traverse.ts';
import { $raw } from './fetch.ts';

const OSMKeys = [PropKey.From, PropKey.To, PropKey.Way];
Handlebars.registerHelper('isNotOSMKey', (key) => !OSMKeys.includes(key));
Handlebars.registerHelper('getProp', (properties, key) => properties[key]);
Handlebars.registerHelper('isStartNode', (id) => id === -1);
Handlebars.registerHelper('isEndNode', (id) => id === -2);
Handlebars.registerHelper('abs', (id) => Math.abs(Number(id)));

import debugPopupTemplate from '../templates/debug_popup.hbs?raw';
const compiledDebugPopupTemplate = Handlebars.compile(debugPopupTemplate);

export const addDebugClick = (feature: Feature, layer: L.Layer) => {
  const featurePopupDiv = L.DomUtil.create('div', 'feature-popup');

  L.DomEvent.disableClickPropagation(featurePopupDiv).disableScrollPropagation(
    featurePopupDiv
  );

  if (feature.properties) {
    featurePopupDiv.innerHTML = compiledDebugPopupTemplate({
      PropKey,
      properties: feature.properties,
    });
    layer.bindPopup(featurePopupDiv);
  }
};

// get the depth and step count from the last step in the route response
// TODO: this assumes the last feature is the deepest
const getRouteDepthAndSteps = () => {
  const features = $raw.get()?.route?.features;
  if (!features) return {};

  const properties = features[features.length - 1].properties;

  if (!properties) return {};
  const depth: number = properties[PropKey.Depth];
  const steps: number = properties[PropKey.Index];

  return { depth, steps };
};

const $routeStyle = computed($mode, (mode) => {
  return (feature: Feature | undefined) => {
    if (!feature?.properties) {
      console.error(`unable to style feature: ${JSON.stringify(feature)}`);
      return {};
    }

    let className = `route-depth-${feature.properties[PropKey.Depth]} step-${
      feature.properties[PropKey.Index]
    }`;
    if (mode === Mode.RouteViz) className += ' svg-path';

    return {
      opacity: mode === Mode.RouteViz ? 0 : 1,
      weight: mode === Mode.RouteViz ? 5 : 3,
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

const $traversalStyle = computed(
  [$mode, $paint, $depth],
  (mode, paint, depth) => {
    const rainbow = new Rainbow();

    let color = (_properties: Record<string, number>): string => '#F26F75';

    if (mode === Mode.RouteViz) {
      rainbow.setNumberRange(0.6, 1.5); // TODO: somewhat arbitrary view of cost, may need to be dynamic w/ custom model
      rainbow.setSpectrum('#2BEA01', '#A9A9A9', 'red');
      color = (properties) =>
        `#${rainbow.colourAt(
          properties[PropKey.Cost] / properties[PropKey.Length]
        )}`;
    }

    if (mode === Mode.Traverse) {
      switch (paint) {
        case PaintOptions.Depth:
          rainbow.setNumberRange(1, depth);
          break;
        case PaintOptions.DistanceSoFar:
          rainbow.setNumberRange(1, depth * 50);
          break;
        case PaintOptions.Cost:
          rainbow.setNumberRange(0, 250);
          rainbow.setSpectrum('green', 'yellow', 'orange', 'red');
          break;
        case PaintOptions.CostSoFar:
          rainbow.setSpectrum('green', 'yellow', 'orange', 'red');
          rainbow.setNumberRange(0, 300 * 10);
          break;
        default:
      }

      color = (properties) =>
        `#${rainbow.colourAt(properties[PropKey[paint]])}`;
    }

    ensureDepthAnimationClassesExist(depth);

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
  }
);

export const $featureGroup = computed(
  [$clickTime, $raw, $paint],
  (clickTime, json, _paint) => {
    const featureGroup = new L.FeatureGroup([]);

    // if no geojson or if the new map click happened recently,
    // return an empty feature group / clear the map
    if (!json || Date.now() - clickTime < 10) return featureGroup;

    // if we're doing the fancy route viz thingy, just quickly ensure that classes to this depth exist
    // we need to do this _before_ traversal gets created / added to the DOM
    if ($mode.get() === Mode.RouteViz) {
      const { depth } = getRouteDepthAndSteps();
      console.log(`got depth ${depth} from getRouteDepthAndSteps`);
      // there's probably a catastrophic failure mode here but hey
      if (depth) ensureDepthAnimationClassesExist(depth);
    }

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
  }
);

// callback invoked after the feature group has been added to the Dom
// animates traversals
export const onFeatureGroupAdded = async () => {
  const mode = $mode.get();

  // if RouteViz, get the depth and step count from the last step in the route response
  // and paint route animation in reverse from end to start
  if (mode === Mode.RouteViz) {
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
