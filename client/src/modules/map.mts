import L from 'leaflet';
import '@maplibre/maplibre-gl-leaflet';

import { $click } from '../store/map.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import {
  $traversalLayer,
  $routeLayer,
  onGeoJsonAdded,
} from '../store/render.ts';
import { ReadableAtom } from 'nanostores';
import { RADAR_API_KEY } from '../config.ts';

let mapInstance: L.Map | null = null;

export const createMap = (container: string): L.Map => {
  if (!mapInstance) {
    mapInstance = L.map(container, {
      zoomControl: false,
    }).setView([40.7, -73.98], 13);

    // Radar serves vector styles only, so the basemap renders through a
    // MapLibre GL canvas wrapped as a Leaflet layer. Attribution is derived
    // from the style's sources by the wrapper.
    L.maplibreGL({
      style: `https://api.radar.io/maps/styles/radar-default-v1?publishableKey=${RADAR_API_KEY}`,
    }).addTo(mapInstance);
  }

  return mapInstance;
};

export const configureBindings = (map: L.Map) => {
  // pub some clicks
  map.on('click', (click) => {
    $click.set(click);
  });

  // sub some things that can be added to the map
  [$startMarker, $endMarker].forEach(($layer) =>
    $layer.listen((layer, oldLayer) => {
      // handles removing and adding everything to the map on change
      oldLayer?.remove();
      layer?.addTo(map);
    })
  );
};

export const fitMarkers = (a: L.LatLng, b?: L.LatLng | null) => {
  if (!mapInstance) return;

  if (b) {
    mapInstance.fitBounds(L.latLngBounds(a, b), { padding: [50, 50], animate: true });
  } else {
    mapInstance.setView(a, 15, { animate: true });
  }
};

// allow hiding of traversal / route layers independently
export const addLayerControl = (map: L.Map) => {
  const layerControl = L.control.layers({}, {}, { position: 'topright', collapsed: false }).addTo(map);
  const geojsonAtoms: [string, ReadableAtom][] = [
    ['Pathfinding', $traversalLayer],
    ['Route', $routeLayer],
  ];

  geojsonAtoms.forEach(([name, $layer]) =>
    $layer.listen((layer: L.Layer | null, oldLayer: L.Layer | null) => {
      if (oldLayer) {
        oldLayer.remove();
        layerControl.removeLayer(oldLayer);
      }
      if (layer) {
        layer.addTo(map);
        layerControl.addOverlay(layer, name);
        // on geojson add, also trigger render store callback
        onGeoJsonAdded();
      }
    })
  );
};
