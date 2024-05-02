/* global L, Rainbow */

const RUSTY_BASE_URL = 'http://localhost:3000';

const state = {
  currentMarker: null,
  currentGeo: null,
  currentCoord: null,
};

const map = L.map('map').setView([40.70, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

const rainbow = new Rainbow();
const slider = document.getElementById('depthRange');
const depthOutput = document.getElementById('depthValue');

let depth = Number(slider.value);
depthOutput.innerText = slider.value;
rainbow.setNumberRange(1, depth);

// Uses global state to fetch and paint graph from starting loc
const fetchAndPaintGraph = async () => {
  const { lat, lng } = state.currentCoord;
  const res = await fetch(`${RUSTY_BASE_URL}/graph?lat=${lat}&lon=${lng}&depth=${depth}`);
  const json = await res.json();
  console.log(json);

  if (state.currentGeo) state.currentGeo.remove();
  state.currentGeo = L.geoJSON(json, {
    // paint different depths differently: https://leafletjs.com/examples/geojson/
    style: (feature) => ({
      color: `#${rainbow.colourAt(feature.properties.depth)}`,
    }),
  });
  state.currentGeo.addTo(map);
};

// Update the depth value on slider change
// eslint-disable-next-line no-unused-vars
const updateDepth = (value) => {
  depth = Number(value);
  depthOutput.innerText = value;
  rainbow.setNumberRange(1, depth);

  // if a paint exists, repaint it
  if (state.currentGeo) fetchAndPaintGraph();
};

map.on('click', (clickEvent) => {
  if (state.currentMarker) state.currentMarker.remove();

  state.currentMarker = L.marker(clickEvent.latlng, { draggable: true });
  state.currentMarker.addTo(map);

  // reacts to dragging
  state.currentMarker.on('move', (markerEvent) => {
    state.currentCoord = markerEvent.latlng;
    fetchAndPaintGraph();
  });

  state.currentCoord = clickEvent.latlng;

  fetchAndPaintGraph();
});
