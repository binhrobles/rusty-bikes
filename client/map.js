/* global L, Rainbow */

// ------ global inits / imports ------ //
const RUSTY_BASE_URL = 'http://localhost:3000';
const START_NODE_ID = -1;
const END_NODE_ID = -2;

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

const PAINTABLE_METADATA_KEYS = ['depth', 'distance', 'distance_so_far'];

const getModeFromUrl = () => {
  const params = new URLSearchParams(document.location.search);
  const mode = params.get('mode');
  return mode && MODE[mode.toUpperCase()];
};

const state = {
  currentGeoJson: null,

  mode: getModeFromUrl() || MODE.ROUTE,
  paint: PAINTABLE_METADATA_KEYS[0],

  // traversal state
  currentMarker: null,
  depth: 20,

  // routing state
  startMarker: null,
  endMarker: null,

  reset: () => {
    if (state.currentGeoJson) state.currentGeoJson.remove();
    state.currentGeoJson = null;

    // traversal state
    if (state.currentMarker) state.currentMarker.remove();
    state.currentMarker = null;
    state.depth = 20;

    // routing state
    if (state.startMarker) state.startMarker.remove();
    state.startMarker = null;
    if (state.endMarker) state.endMarker.remove();
    state.endMarker = null;
  },
};

// generates the html shown in the popup div when a feature is clicked
const generateTraversalPopupHtml = (feature) => {
  const [from, fromWrapper] = feature.properties.from === START_NODE_ID
    ? ['Start', '<span>']
    : [
      feature.properties.from,
      `<a
        href="https://www.openstreetmap.org/node/${feature.properties.from}"
        target="_blank"
        rel="noopener noreferrer">`,
    ];

  const [to, toWrapper] = feature.properties.to === END_NODE_ID
    ? ['End', '<span>']
    : [
      feature.properties.to,
      `<a
        href="https://www.openstreetmap.org/node/${feature.properties.from}"
        target="_blank"
        rel="noopener noreferrer">`,
    ];
  let html = `
            <h4>Segment</h4>
            <hr>
              <table>
                <tr>
                  <td><strong>from</strong></td>
                  <td>${fromWrapper}${from}</td>
                </tr>
                <tr>
                  <td><strong>to</strong></td>
                  <td>${toWrapper}${to}</td>
                </tr>
                <tr>
                  <td><strong>way</strong></td>
                  <td><a
                    href="https://www.openstreetmap.org/way/${feature.properties.way}"
                    target="_blank"
                    rel="noopener noreferrer"
                  >${feature.properties.way}</td>
                </tr>`;

  Object.keys(feature.properties).forEach((key) => {
    if (!['from', 'to', 'way'].includes(key)) {
      const value = feature.properties[key] % 1
        ? feature.properties[key].toFixed(2)
        : feature.properties[key];
      html += `
                <tr>
                  <td><strong>${key}</strong></td>
                  <td>${value}</td>
                </tr>`;
    }
  });

  html += '</table>';

  return html;
};

const makeFeatureClickable = (feature, layer) => {
  const featurePopupDiv = L.DomUtil.create('div', 'feature-popup');
  L.DomEvent
    .disableClickPropagation(featurePopupDiv)
    .disableScrollPropagation(featurePopupDiv);
  if (feature.properties) {
    featurePopupDiv.innerHTML = generateTraversalPopupHtml(feature);
    layer.bindPopup(featurePopupDiv);
  }
};

// instructs leaflet to paint each geojson feature a color, programmatically
const geoJsonStyleFeatureFn = (rainbowInstance, paint) => {
  // ensure the current paint conditions are honored
  switch (state.paint) {
    case 'depth':
      rainbowInstance.setNumberRange(1, state.depth);
      break;
    case 'distance':
      rainbowInstance.setNumberRange(1, 300);
      break;
    case 'distance_so_far':
      rainbowInstance.setNumberRange(1, state.depth * 50);
      break;
    default:
  }

  return (feature) => ({
    color: `#${rainbowInstance.colourAt(feature.properties[paint])}`,
  });
};

const getGeoJsonOptions = (mode) => {
  switch (mode) {
    case MODE.TRAVERSE: {
      return {
        style: geoJsonStyleFeatureFn(rainbow, state.paint),
        onEachFeature: makeFeatureClickable,
        bubblingMouseEvents: false,
      };
    }
    case MODE.ROUTE: {
      return {
        onEachFeature: makeFeatureClickable,
        bubblingMouseEvents: false,
      };
    }
    default:
      return {};
  }
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
      const { lng: startLon, lat: startLat } = state.startMarker.getLatLng();
      const { lng: endLon, lat: endLat } = state.endMarker.getLatLng();
      res = await fetch(`${RUSTY_BASE_URL}/route?start=${startLon},${startLat}&end=${endLon},${endLat}`);
      break;
    }
    default:
      console.error(`bad state -- mode=${state.mode}`);
      return;
  }

  const json = await res.json();

  if (state.currentGeoJson) state.currentGeoJson.remove();
  state.currentGeoJson = L.geoJSON(json, getGeoJsonOptions(state.mode));
  state.currentGeoJson.addTo(map);
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
  let header = `
              <h4>Rusty Bikes</h4>
              <label for="mode-select">Mode:</label>
              <select name="mode-select" id="mode-select" onchange="updateMode(this.value)">
                `;

  Object.keys(MODE).forEach((mode) => {
    header += `<option ${state.mode === mode && 'selected'} value="${mode}">${modeMeta[mode].label}</option>`;
  });

  header += '</select><hr />';

  let content;
  // only repaint this on mode change
  // update content using updates to HTMLElements
  switch (state.mode) {
    case MODE.TRAVERSE:
      // slider
      content = `
              <label for="depthValue">Traversal Depth:</label>
              <span id="depthValue"></span>
              <br />

              <input class="slider" id="depthRange" type="range" min="0" max="100" value=${state.depth} onchange="updateDepth(this.value)">
              <br />`;

      // paint selection
      content += `
              <label for="paint-select">Paint with:</label>
              <select
                name="paint-select"
                id="paint-select"
                onchange="updatePaint(this.value)"
              > `;
      PAINTABLE_METADATA_KEYS.forEach((key) => {
        content += `<option ${key === state.paint && 'selected'} value="${key}">${key}</option>`;
      });
      content += '</select>';

      // lon-lat label
      content += `
              <br />
              <br />

              <label for="traversalLon">Clicked (lon, lat):</label>
              <br />
              (<span id="traversalLon"></span>, <span id="traversalLat"></span>)
              `;

      break;
    case MODE.ROUTE:
      // TODO: highlight last selected text field? to indicate the
      //       field that will populate when map clicked
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
              </table >
              `;
      break;
    default:
      break;
  }

  controlDiv.innerHTML = `${header} ${content} `;
};

control.addTo(map);

// ------ view controls ------ //
// eslint-disable-next-line no-unused-vars
const updateMode = (mode) => {
  state.mode = mode;

  // reset leaflet state
  state.reset();

  control.update();
};

// eslint-disable-next-line no-unused-vars
const updatePaint = (paint) => {
  state.paint = paint;

  state.currentGeoJson.setStyle(geoJsonStyleFeatureFn(rainbow, state.paint));
};

// Update the depth value on slider change
// eslint-disable-next-line no-unused-vars
const updateDepth = (value) => {
  state.depth = Number(value);

  document.getElementById('depthValue').innerText = state.depth;

  // if a paint exists, repaint it
  if (state.currentGeoJson) fetchAndPaintGraph();
};

// ------ traversal mode handlers ------ //
rainbow.setNumberRange(1, state.depth);

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
    if (state.startMarker) state.startMarker.remove();
    state.startMarker = L.marker(clickEvent.latlng, { draggable: true });
    state.startMarker.addTo(map);

    // reacts to dragging
    state.startMarker.on('move', () => {
      startInput.value = JSON.stringify(state.startMarker.getLatLng());
      fetchAndPaintGraph();
    });

    startInput.value = JSON.stringify(clickEvent.latlng);
  } else {
    if (state.endMarker) state.endMarker.remove();
    state.endMarker = L.marker(clickEvent.latlng, { draggable: true });
    state.endMarker.addTo(map);

    // reacts to dragging
    state.endMarker.on('move', () => {
      endInput.value = JSON.stringify(state.startMarker.getLatLng());
      fetchAndPaintGraph();
    });

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
