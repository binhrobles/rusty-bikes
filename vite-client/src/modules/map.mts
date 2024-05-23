import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $marker as $traversalMarker } from '../store/traversal.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import { $traversalGeoJson } from '../store/geojson.ts';

const container = document.getElementById('map');
if (!container) throw new Error('no `map` div!');

const map = L.map(container).setView([40.70, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

const addToMap = (something: L.Layer | null) => {
  if (!something) return;
  something.addTo(map);
}

// pub some clicks
map.on('click', $click.set);

// sub some objects
// updated markers just get added to the map
[$traversalMarker, $startMarker, $endMarker].forEach($marker =>
  $marker.listen(m => addToMap(m as unknown as L.Layer))
)

export default map;
