import maplibregl from 'maplibre-gl';
import { RADAR_API_KEY, NYC_CENTER } from '../lib/config.ts';
import type { MobileRoute, CorridorData } from '../types/index.ts';

const CORRIDOR_SOURCE = 'corridor';
const CORRIDOR_LAYER = 'corridor-line';
const ROUTE_SOURCE = 'route';
const ROUTE_LAYER = 'route-line';
const GPS_SOURCE = 'gps-marker';
const GPS_LAYER = 'gps-dot';

let map: maplibregl.Map | null = null;
let sourcesReady = false;
let pendingCorridor: CorridorData | null = null;
let pendingRoute: MobileRoute | null = null;
let pendingGPS: [number, number] | null = null;
let endMarker: maplibregl.Marker | null = null;
let onEndMarkerDrag: ((lat: number, lon: number) => void) | null = null;

export function createMap(container: string): maplibregl.Map {
  if (map) return map;

  map = new maplibregl.Map({
    container,
    style: `https://api.radar.io/maps/styles/radar-default-v1?publishableKey=${RADAR_API_KEY}`,
    center: [NYC_CENTER.longitude, NYC_CENTER.latitude],
    zoom: 14,
    bearing: 0,
    pitchWithRotate: false,
    attributionControl: false,
  });

  map.addControl(new maplibregl.AttributionControl({ compact: true }), 'bottom-right');

  map.on('load', () => {
    // Corridor source + layer (added first so it renders beneath route)
    map!.addSource(CORRIDOR_SOURCE, {
      type: 'geojson',
      data: { type: 'FeatureCollection', features: [] },
    });

    map!.addLayer({
      id: CORRIDOR_LAYER,
      type: 'line',
      source: CORRIDOR_SOURCE,
      layout: { 'line-join': 'round', 'line-cap': 'round' },
      paint: {
        'line-color': [
          'case',
          ['==', ['at', 2, ['get', 'labels']], true],
          '#e84393',
          ['any',
            ['==', ['at', 0, ['get', 'labels']], 'Lane'],
            ['==', ['at', 0, ['get', 'labels']], 'Track'],
          ],
          '#00b894',
          '#2563eb',
        ],
        'line-width': 4,
        'line-opacity': 0.35,
      },
    });

    // Route line source + layer
    map!.addSource(ROUTE_SOURCE, {
      type: 'geojson',
      data: { type: 'FeatureCollection', features: [] },
    });

    map!.addLayer({
      id: ROUTE_LAYER,
      type: 'line',
      source: ROUTE_SOURCE,
      layout: { 'line-join': 'round', 'line-cap': 'round' },
      paint: {
        'line-color': [
          'case',
          // salmon segments → pink
          ['==', ['at', 2, ['get', 'labels']], true],
          '#e84393',
          // any bike infra (Shared, Lane, Track) → green
          ['any',
            ['==', ['at', 0, ['get', 'labels']], 'Lane'],
            ['==', ['at', 0, ['get', 'labels']], 'Track'],
          ],
          '#00b894',
          // default → blue
          '#2563eb',
        ],
        'line-width': 6,
        'line-opacity': 0.85,
      },
    });

    // GPS dot source + layer
    map!.addSource(GPS_SOURCE, {
      type: 'geojson',
      data: { type: 'FeatureCollection', features: [] },
    });

    map!.addLayer({
      id: GPS_LAYER,
      type: 'circle',
      source: GPS_SOURCE,
      paint: {
        'circle-radius': 10,
        'circle-color': '#2563eb',
        'circle-stroke-width': 3,
        'circle-stroke-color': '#fff',
      },
    });

    sourcesReady = true;

    // Flush any data that arrived before the map was ready
    if (pendingCorridor) {
      const corridorSrc = map!.getSource(CORRIDOR_SOURCE) as maplibregl.GeoJSONSource;
      corridorSrc.setData(pendingCorridor);
      pendingCorridor = null;
    }
    if (pendingRoute) {
      const src = map!.getSource(ROUTE_SOURCE) as maplibregl.GeoJSONSource;
      src.setData(pendingRoute);
      pendingRoute = null;
    }
    if (pendingGPS) {
      updateGPSMarker(pendingGPS[0], pendingGPS[1]);
      pendingGPS = null;
    }
  });

  return map;
}

export function updateCorridor(corridor: CorridorData | null): void {
  const data: GeoJSON.FeatureCollection = corridor ?? { type: 'FeatureCollection', features: [] };

  if (!map || !sourcesReady) {
    pendingCorridor = corridor;
    return;
  }

  const src = map.getSource(CORRIDOR_SOURCE) as maplibregl.GeoJSONSource | undefined;
  if (!src) return;
  src.setData(data);
}

export function updateRoute(route: MobileRoute | null): void {
  const data = route ?? { type: 'FeatureCollection', features: [] };

  if (!map || !sourcesReady) {
    pendingRoute = route;
    return;
  }

  const src = map.getSource(ROUTE_SOURCE) as maplibregl.GeoJSONSource | undefined;
  if (!src) return;
  src.setData(data);
}

export function updateGPSMarker(lat: number, lon: number): void {
  if (!map || !sourcesReady) {
    pendingGPS = [lat, lon];
    return;
  }
  const src = map.getSource(GPS_SOURCE) as maplibregl.GeoJSONSource | undefined;
  if (!src) return;
  src.setData({
    type: 'FeatureCollection',
    features: [{ type: 'Feature', geometry: { type: 'Point', coordinates: [lon, lat] }, properties: {} }],
  });
}

export function setEndMarkerDragHandler(handler: (lat: number, lon: number) => void): void {
  onEndMarkerDrag = handler;
}

export function updateEndMarker(lat: number, lon: number): void {
  if (!map) return;

  if (!endMarker) {
    const el = document.createElement('div');
    el.textContent = '📍';
    el.style.fontSize = '2rem';
    el.style.lineHeight = '1';
    el.style.cursor = 'grab';

    endMarker = new maplibregl.Marker({ element: el, draggable: true, anchor: 'bottom' })
      .setLngLat([lon, lat])
      .addTo(map);

    endMarker.on('dragend', () => {
      const lngLat = endMarker!.getLngLat();
      if (onEndMarkerDrag) onEndMarkerDrag(lngLat.lat, lngLat.lng);
    });
  } else {
    endMarker.setLngLat([lon, lat]);
  }
}

export function removeEndMarker(): void {
  if (endMarker) {
    endMarker.remove();
    endMarker = null;
  }
}

export function centerOn(lat: number, lon: number, zoom = 14): void {
  if (!map) return;
  map.easeTo({ center: [lon, lat], zoom, duration: 600 });
}

export function resizeMap(): void {
  map?.resize();
}

export function fitRoute(route: MobileRoute): void {
  if (!map || route.features.length === 0) return;
  const allCoords = route.features.flatMap((f) => f.geometry.coordinates);
  const bounds = allCoords.reduce(
    (b, c) => b.extend(c as [number, number]),
    new maplibregl.LngLatBounds(allCoords[0] as [number, number], allCoords[0] as [number, number]),
  );
  map.fitBounds(bounds, { padding: 60, duration: 600 });
}

/**
 * Navigation-mode camera: pitched, bearing-rotated, user at bottom-center.
 * Padding pushes the logical center upward so the user dot sits in the lower third.
 */
export function followGPSNavMode(lat: number, lon: number, bearing: number): void {
  if (!map) return;
  map.easeTo({
    center: [lon, lat],
    bearing,
    pitch: 45,
    padding: { top: Math.round(map.getContainer().clientHeight * 0.55), bottom: 0, left: 0, right: 0 },
    duration: 500,
  });
}

/**
 * Reset camera to top-down planning mode: no pitch, north-up.
 */
export function resetCamera(): void {
  if (!map) return;
  map.easeTo({
    pitch: 0,
    bearing: 0,
    padding: { top: 0, bottom: 0, left: 0, right: 0 },
    duration: 400,
  });
}
