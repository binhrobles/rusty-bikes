const RUSTY_BASE_URL = 'http://localhost:3000';

const map = L.map('map').setView([40.70, -73.98], 13);

L.tileLayer('https://tile.openstreetmap.org/{z}/{x}/{y}.png', {
  maxZoom: 19,
  attribution: '&copy; <a href="http://www.openstreetmap.org/copyright">OpenStreetMap</a>'
}).addTo(map);

let currentMarker;
map.on('click', async (e) => {
  if (currentMarker) currentMarker.remove();

  currentMarker = L.marker(e.latlng);
  currentMarker.addTo(map);

  const { lat, lng } = e.latlng;
  const depth = 5; // TODO: extract into text field

  const res = await fetch(`${RUSTY_BASE_URL}/graph?lat=${lat}&lon=${lng}&depth=${depth}`);
  console.log(res.status);
});

