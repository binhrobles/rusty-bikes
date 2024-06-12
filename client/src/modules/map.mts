import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $marker as $traversalMarker } from '../store/traverse.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import {
  $traversalLayer,
  $routeLayer,
  onGeoJsonAdded,
} from '../store/render.ts';
import { ReadableAtom } from 'nanostores';

const map = L.map('map', {
  zoomControl: false,
}).setView([40.7, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution:
    '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);
const layerControl = L.control.layers().addTo(map);

// pub some clicks
map.on('click', (click) => {
  $click.set(click);
});

// sub some things that can be added to the map
[$traversalMarker, $startMarker, $endMarker].forEach(($layer) =>
  $layer.listen((layer, oldLayer) => {
    // handles removing and adding everything to the map on change
    oldLayer?.remove();
    layer?.addTo(map);
  })
);

const geojsonAtoms: [string, ReadableAtom][] = [
  ['Traversal', $traversalLayer],
  ['Route', $routeLayer],
];

geojsonAtoms.forEach(([name, $layer]) =>
  $layer.listen((layer: L.Layer | null, oldLayer: L.Layer | null) => {
    if (oldLayer) {
      oldLayer.remove();
      layerControl.removeLayer(oldLayer);
    }
    if (layer) {
      layer.addTo(map);
      layerControl.addOverlay(layer, name);
      // on geojson add, also trigger render store callback
      onGeoJsonAdded();
    }
  })
);

export default map;
