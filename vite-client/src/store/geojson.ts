import { atom, batched, task } from 'nanostores';
import L from 'leaflet';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../config.ts';
import { Mode } from '../consts.ts';

import { $markerLatLng as $traversalMarkerLatLng, $depth, $geoJsonRenderOptions as $traversalRenderOptions } from './traversal.ts';
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
// TODO: client-side rate limit?
export const $raw = batched(
  [$mode, $traversalMarkerLatLng, $depth],
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

// whenever a new json response is loaded, reinitialize the feature group
// $raw.listen(json => {
//   if (!json) return;

//   // remove the group if it exists
//   $featureGroup.get()?.remove();

//   const featureGroup = new L.FeatureGroup([]);

//   // if traversal exists, paint it
//   if (json.traversal) {
//     L.geoJSON(json.traversal, $traversalRenderOptions.get()).addTo(featureGroup);
//   }

//   // if route exists, paint it
//   // if (json.route) {
//   //   L.geoJSON(json.route, getGeoJsonOptions(MODE.ROUTE)).addTo(state.currentGeoJson);
//   // }

//   $featureGroup.set(featureGroup);
// });

export const $featureGroup = batched([$raw, $traversalRenderOptions], (json, options) => {
  if (!json) return;

  // remove the group if it exists
  // $featureGroup.get()?.remove();

  const featureGroup = new L.FeatureGroup([]);

  // if traversal exists, paint it
  if (json.traversal) {
    L.geoJSON(json.traversal, options).addTo(featureGroup);
  }

  // if route exists, paint it
  // if (json.route) {
  //   L.geoJSON(json.route, getGeoJsonOptions(MODE.ROUTE)).addTo(state.currentGeoJson);
  // }

  // $featureGroup.set(featureGroup);
  return featureGroup;
});
