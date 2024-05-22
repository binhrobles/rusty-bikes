import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $marker as $traversalMarker } from '../store/traversal.ts';

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

// forward map clicks to our state mgmt
map.on('click', $click.set);

// respond to state updates
$traversalMarker.listen(marker => addToMap(marker as unknown as L.Layer));

export default map;
