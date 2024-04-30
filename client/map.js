const RUSTY_BASE_URL = 'http://localhost:3000';

const map = L.map('map').setView([40.70, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);

const nodeCircleOptions = {
  radius: 4,
  fillColor: "#ff7800",
  color: "#000",
  weight: 1,
  opacity: 1,
  fillOpacity: 0.8
};

let currentMarker;
let currentGeo;
map.on('click', async (e) => {
  if (currentMarker) currentMarker.remove();
  if (currentGeo) currentGeo.remove();

  currentMarker = L.marker(e.latlng);
  currentMarker.addTo(map);

  const { lat, lng } = e.latlng;
  const depth = 35; // TODO: extract into text field

  const res = await fetch(`${RUSTY_BASE_URL}/graph?lat=${lat}&lon=${lng}&depth=${depth}`);
  const json = await res.json();
  console.log(json);

  // TODO: paint different depths differently: https://leafletjs.com/examples/geojson/
  currentGeo = L.geoJSON(json, {
    pointToLayer: (_feature, latlng) => {
      console.log(`building for ${latlng}`);
      return L.circleMarker(latlng, nodeCircleOptions);
    },
  })
  currentGeo.addTo(map);

});

