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

export const $traversalStyle = computed(
  [$mode, $paint, $depth],
  (mode, paint, depth) => {
    let style;

    switch (mode) {
      case Mode.RouteViz:
        {
          style = (feature: Feature | undefined) => {
            if (!feature?.properties) {
              console.error(
                `unable to style feature: ${JSON.stringify(feature)}`
              );
              return {};
            }

            return {
              color: '#F26F75',
              className: `depth-${feature.properties.depth}`,
            };
          };
        }
        break;
      case Mode.Traverse:
        {
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

          style = (feature: Feature | undefined) => {
            if (!feature?.properties) {
              console.error(
                `unable to style feature: ${JSON.stringify(feature)}`
              );
              return {};
            }

            return {
              color: `#${rainbow.colourAt(feature.properties[paint])}`,
              className: `depth-${feature.properties.depth}`,
            };
          };
        }
        break;
      default:
    }

    return style;
  }
);

export const $featureGroup = computed([$raw, $traversalStyle], (json, style) => {
  const featureGroup = new L.FeatureGroup([]);
  if (!json) return featureGroup;

  // if traversal exists, paint it
  if (json.traversal) {
    L.geoJSON(json.traversal, {
      style,
      onEachFeature: addDebugClick,
      bubblingMouseEvents: false,
    }).addTo(featureGroup);
  }

  // if route exists, paint it
  if (json.route) {
    L.geoJSON(json.route, {
      onEachFeature: addDebugClick,
      bubblingMouseEvents: false,
    }).addTo(featureGroup);
  }

  return featureGroup;
});

// callback invoked after the feature group has been added to the Dom
// parses out SVG component depths into a map to be used for animation
export const onFeatureGroupAdded = async () => {
  for (let i = 0; i <= $depth.get(); i++) {
    const collection = document.getElementsByClassName(`depth-${i}`);
    for (let j = 0; j < collection.length; j++) {
      const feature = collection.item(j);
      feature?.setAttribute('visibility', 'hidden');

      setTimeout(
        () => feature?.setAttribute('visibility', 'visible'),
        i * 100
      );
    }
  }
};
