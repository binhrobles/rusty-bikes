import L from 'leaflet';
import 'leaflet/dist/leaflet.css';
import './style.css'

import control from './modules/control.mjs';

const container = document.getElementById('map');
if (!container) throw new Error('no `map` div!');

const map = L.map(container).setView([40.70, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

control.render(map);
