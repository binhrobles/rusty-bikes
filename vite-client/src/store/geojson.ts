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
const fetchTraversal = async (lat: number, lon: number, depth: number): Promise<ServerResponse> => {
    const res = await fetch(`${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lon}&depth=${depth}`);
    return await res.json();
}

// when traversal details change, refetch traversal geojson
export const $raw = batched(
  [
    $mode,
    $traversalMarkerLatLng, $depth,
    $startMarkerLatLng, $endMarkerLatLng,
  ],
  (mode, latLng, depth) => task(async () => {
    if (mode !== Mode.Traverse || !latLng) return;

    try {
      const { lat, lng } = latLng;
      return await fetchTraversal(lat, lng, depth);
    } catch (e) {
      console.error('failed to fetch traversal: ', e);
      return null;
    }
}));

export const $featureGroup = batched([$raw, $traversalRenderOptions], (json, options) => {
  if (!json) return;

  const featureGroup = new L.FeatureGroup([]);

  // if traversal exists, paint it
  if (json.traversal) {
    L.geoJSON(json.traversal, options).addTo(featureGroup);
  }

  // if route exists, paint it
  // if (json.route) {
  //   L.geoJSON(json.route, getGeoJsonOptions(MODE.ROUTE)).addTo(state.currentGeoJson);
  // }

  return featureGroup;
});
