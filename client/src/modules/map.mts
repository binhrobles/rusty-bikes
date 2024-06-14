import L from 'leaflet';

import { $click } from '../store/map.ts';
import { $startMarker, $endMarker } from '../store/route.ts';
import {
  $traversalLayer,
  $routeLayer,
  onGeoJsonAdded,
} from '../store/render.ts';
import { ReadableAtom } from 'nanostores';

let mapInstance: L.Map | null = null;

export const createMap = (container: string): L.Map => {
  if (!mapInstance) {
    mapInstance = L.map(container, {
      zoomControl: false,
    }).setView([40.7, -73.98], 13);

    L.tileLayer(
      'https://{s}.basemaps.cartocdn.com/rastertiles/voyager/{z}/{x}/{y}{r}.png',
      {
        attribution: `&copy;<a href="https://www.openstreetmap.org/copyright" target="_blank">OpenStreetMap</a>,
              &copy;<a href="https://carto.com/attributions" target="_blank">CARTO</a>`,
        maxZoom: 19,
      }
    ).addTo(mapInstance);
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
