import { batched, task } from 'nanostores';
import L from 'leaflet';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../config.ts';
import { Mode } from '../consts.ts';

import { $markerLatLng as $traversalMarkerLatLng, $depth, $geoJsonRenderOptions as $traversalRenderOptions } from './traversal.ts';
import { $startMarkerLatLng, $endMarkerLatLng } from './route.ts';
import { $mode } from './mode.ts';

type ServerResponse = {
  traversal: FeatureCollection,
  route: FeatureCollection,
}

// TODO: into modules/http?
const fetchTraversal = async (latLng: L.LatLng, depth: number): Promise<ServerResponse> => {
    const { lat, lng } = latLng;
    const res = await fetch(`${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lng}&depth=${depth}`);
    return await res.json();
}

const fetchRoute = async (startLatLng: L.LatLng, endLatLng: L.LatLng, withTraversal: boolean): Promise<ServerResponse> => {
    const { lng: startLon, lat: startLat } = startLatLng;
    const { lng: endLon, lat: endLat } = endLatLng;

    const res = await fetch(`${RUSTY_BASE_URL}/route?start=${startLon},${startLat}&end=${endLon},${endLat}&with_traversal=${withTraversal}`);
    return await res.json();
}

// when traversal details change, refetch traversal geojson
export const $raw = batched(
  [
    $mode,
    $traversalMarkerLatLng, $depth,
    $startMarkerLatLng, $endMarkerLatLng,
  ],
  (mode, traversalLatLng, depth, startLatLng, endLatLng) => task(async () => {
    if (mode === Mode.Traverse && traversalLatLng) {
      try {
        return await fetchTraversal(traversalLatLng, depth);
      } catch (e) {
        console.error('failed to fetch traversal: ', e);
        return null;
      }
    } else if (
      (mode === Mode.Route || mode === Mode.RouteViz) &&
      startLatLng && endLatLng
    ) {
      try {
        return await fetchRoute(startLatLng, endLatLng, mode === Mode.RouteViz);
      } catch (e) {
        console.error('failed to fetch traversal: ', e);
        return null;
      }
    }

    return null;
}));

export const $featureGroup = batched([$raw, $traversalRenderOptions], (json, options) => {
  if (!json) return;

  const featureGroup = new L.FeatureGroup([]);

  // if traversal exists, paint it
  if (json.traversal) {
    L.geoJSON(json.traversal, options).addTo(featureGroup);
  }

  // if route exists, paint it
  if (json.route) {
    L.geoJSON(json.route, /*getGeoJsonOptions(MODE.ROUTE)*/).addTo(featureGroup);
  }

  return featureGroup;
});
