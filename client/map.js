/* global L, Rainbow */

// ------ global inits / imports ------ //
const RUSTY_BASE_URL = 'http://localhost:3000';

const rainbow = new Rainbow();
const map = L.map('map').setView([40.70, -73.98], 13);
L.control.locate().addTo(map);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>',
}).addTo(map);

// ------ global state / functions ------ //
const MODE = {
  ROUTE: 'ROUTE',
  TRAVERSE: 'TRAVERSE',
};

const modeMeta = {
  [MODE.TRAVERSE]: {
    label: 'Traverse',
  },
  [MODE.ROUTE]: {
    label: 'Routing',
  },
};

const state = {
  currentMarker: null,
  currentGeo: null,

  mode: MODE.ROUTE,

  // traversal state
  depth: 20,
};

// Uses global state to fetch and paint graph from starting loc
const fetchAndPaintGraph = async () => {
  let res;
  switch (state.mode) {
    case MODE.TRAVERSE: {
      const { lat, lng } = state.currentMarker.getLatLng();
      res = await fetch(`${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lng}&depth=${state.depth}`);
      break;
    }
    case MODE.ROUTE: {
      const { lat, lng } = JSON.parse(document.getElementById('startInput').value);
      res = await fetch(`${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lng}&depth=${state.depth}`);
      break;
    }
    default:
  }
  const json = await res.json();

  if (state.currentGeo) state.currentGeo.remove();
  state.currentGeo = L.geoJSON(json, {
    // paint different depths differently: https://leafletjs.com/examples/geojson/
    style: (feature) => ({
      color: `#${rainbow.colourAt(feature.properties.depth)}`,
    }),
  });
  state.currentGeo.addTo(map);
};

// ------ control initialization ------ //
const control = L.control({ position: 'topleft' });
const controlDiv = L.DomUtil.create('div', 'control');
L.DomEvent
  .disableClickPropagation(controlDiv)
  .disableScrollPropagation(controlDiv);

control.onAdd = () => {
  control.update();
  return controlDiv;
};

control.update = () => {
  const header = `
    <h4>Rusty Bikes</h4>
    <label for="mode-select">Mode:</label>
    <select name="mode-select" id="mode-select" onchange="updateMode(this.value)">
      ${Object.keys(MODE).map((mode) => `<option ${state.mode === mode && 'selected'} value="${mode}">${modeMeta[mode].label}</option>`)}
    </select >
    <hr />
  `;

  let content;
  // only repaint this on mode change
  // update content using updates to HTMLElements
  switch (state.mode) {
    case MODE.TRAVERSE:
      content = `
        <label for="depthRange">Traversal Depth:</label>
        <span id="depthValue"></span>
        <br/>
        <input class="slider" id="depthRange" type="range" min="0" max="100" value="${state.depth}" onchange="updateDepth(this.value)">
        <br/>

        <label>Clicked (lon, lat):</label>
        <br/>
        (<span id="traversalLon"></span>, <span id="traversalLat"></span>)
      `;
      break;
    case MODE.ROUTE:
      content = `
        <table class="route-table">
          <tr>
            <td><label for="startInput">Start:</label></td>
            <td><input type="text" id="startInput" placeholder="Click to select start point"></td>
          </tr>
          <tr>
            <td><label for="endInput">End:</label></td>
            <td><input type="text" id="endInput" placeholder="Click to select end point"></td>
          </tr>
        </table>
      `;
      break;
    default:
      break;
  }

  controlDiv.innerHTML = `${header} ${content}`;
};

control.addTo(map);

// ------ mode control ------ //
// eslint-disable-next-line no-unused-vars
const updateMode = (mode) => {
  state.mode = mode;

  // reset leaflet state
  if (state.currentMarker) state.currentMarker.remove();
  state.currentMarker = null;
  if (state.currentGeo) state.currentGeo.remove();
  state.currentGeo = null;

  control.update();
};

// ------ traversal mode handlers ------ //
rainbow.setNumberRange(1, state.depth);

// Update the depth value on slider change
// eslint-disable-next-line no-unused-vars
const updateDepth = (value) => {
  state.depth = Number(value);

  document.getElementById('depthValue').innerText = state.depth;
  rainbow.setNumberRange(1, state.depth);

  // if a paint exists, repaint it
  if (state.currentGeo) fetchAndPaintGraph();
};

// clicks will update the marker location and fetch a graph traversal from that location
const handleTraversalClick = (clickEvent) => {
  if (state.currentMarker) state.currentMarker.remove();

  state.currentMarker = L.marker(clickEvent.latlng, { draggable: true });
  state.currentMarker.addTo(map);

  // reacts to dragging
  state.currentMarker.on('move', () => {
    fetchAndPaintGraph();
  });

  document.getElementById('traversalLon').innerText = clickEvent.latlng.lng;
  document.getElementById('traversalLat').innerText = clickEvent.latlng.lat;

  fetchAndPaintGraph();
};

// ------ routing mode handlers ------- //
// clicks will set the focused input to the LatLng of the click
// if both inputs have values, attempt to fetch route
const handleRouteClick = (clickEvent) => {
  const startInput = document.getElementById('startInput');
  const endInput = document.getElementById('endInput');

  if (!startInput.value) {
    startInput.value = JSON.stringify(clickEvent.latlng);
  } else {
    endInput.value = JSON.stringify(clickEvent.latlng);
  }

  if (startInput.value && endInput.value) {
    fetchAndPaintGraph();
  }
};

// ------ map interaction handlers ------ //
map.on('click', (clickEvent) => {
  switch (state.mode) {
    case MODE.TRAVERSE:
      handleTraversalClick(clickEvent);
      break;
    case MODE.ROUTE:
      handleRouteClick(clickEvent);
      break;
    default:
  }
});
