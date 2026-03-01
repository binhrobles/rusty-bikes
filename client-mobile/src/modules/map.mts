import maplibregl from 'maplibre-gl';
import { RADAR_API_KEY, NYC_CENTER } from '../lib/config.ts';
import type { MobileRoute } from '../types/index.ts';

const ROUTE_SOURCE = 'route';
const ROUTE_LAYER = 'route-line';
const GPS_SOURCE = 'gps-marker';
const GPS_LAYER = 'gps-dot';

let map: maplibregl.Map | null = null;
let sourcesReady = false;
let pendingRoute: MobileRoute | null = null;

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
        'line-color': '#2563eb',
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

    // Flush any route that arrived before the map was ready
    if (pendingRoute) {
      const src = map!.getSource(ROUTE_SOURCE) as maplibregl.GeoJSONSource;
      src.setData(pendingRoute);
      pendingRoute = null;
    }
  });

  return map;
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
  if (!map || !map.isStyleLoaded()) return;
  const src = map.getSource(GPS_SOURCE) as maplibregl.GeoJSONSource | undefined;
  if (!src) return;
  src.setData({
    type: 'FeatureCollection',
    features: [{ type: 'Feature', geometry: { type: 'Point', coordinates: [lon, lat] }, properties: {} }],
  });
}

export function followGPS(lat: number, lon: number, bearing: number): void {
  if (!map) return;
  map.easeTo({ center: [lon, lat], bearing, duration: 500 });
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
