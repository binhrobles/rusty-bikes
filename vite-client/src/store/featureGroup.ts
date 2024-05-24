/**
 * Handles painting traversals / routes, given raw geojson availability
 */
import { computed } from 'nanostores';
import L from 'leaflet';
import Handlebars from 'handlebars';
import { Feature, Geometry } from 'geojson';
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

export const addDebugClick = (
  feature: Feature<Geometry, any>,
  layer: L.Layer,
) => {
  const featurePopupDiv = L.DomUtil.create('div', 'feature-popup');

  L.DomEvent.disableClickPropagation(featurePopupDiv).disableScrollPropagation(
    featurePopupDiv,
  );

  if (feature.properties) {
    featurePopupDiv.innerHTML = compiledDebugPopupTemplate(feature);
    layer.bindPopup(featurePopupDiv);
  }
};

export const $style = computed(
  [$mode, $paint, $depth],
  (mode, paint, depth) => {
    let style;

    switch (mode) {
      case Mode.RouteViz:
        {
          style = () => ({
            color: '#F26F75',
          });
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

          style = (feature: Feature<Geometry, any> | undefined) => ({
            color: feature?.properties
              ? `#${rainbow.colourAt(feature.properties[paint])}`
              : '#000000', // if black is painted...we got issues!
          });
        }
        break;
      default:
    }

    return style;
  },
);

export const $featureGroup = computed([$raw, $style], (json, style) => {
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
