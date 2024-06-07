import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $marker as $traversalMarker } from '../store/traverse.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import { $featureGroup, onFeatureGroupAdded } from '../store/render.ts';

const map = L.map('map').setView([40.7, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution:
    '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

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

// on feature group add, also trigger render store callback
$featureGroup.listen((layer, oldLayer) => {
  oldLayer?.remove();
  layer?.addTo(map);
  onFeatureGroupAdded();
})

export default map;
