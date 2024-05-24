import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $marker as $traversalMarker } from '../store/traversal.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import { $featureGroup } from '../store/featureGroup.ts';

const container = document.getElementById('map');
if (!container) throw new Error('no `map` div!');

const map = L.map(container).setView([40.7, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution:
    '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

// pub some clicks
map.on('click', $click.set);

// sub some things that can be added to the map
[$traversalMarker, $startMarker, $endMarker, $featureGroup].forEach(($layer) =>
  $layer.listen((layer, oldLayer) => {
    // handles removing and adding everything to the map on change
    oldLayer?.remove();
    layer?.addTo(map);
  })
);

export default map;
