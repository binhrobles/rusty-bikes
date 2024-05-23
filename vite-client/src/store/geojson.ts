import { atom } from 'nanostores';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../consts.ts';

import { $marker as $traversalMarker, $depth } from './traversal.ts';

export const $traversalGeoJson = atom<FeatureCollection | null>(null);
export const $routeGeoJson = atom<FeatureCollection | null>(null);

// when marker changes, fetch updated geojson
// TODO: client-side rate limit? only once every 100ms max ?
$traversalMarker.listen(async marker => {
  console.log('fired');
  if (!marker) return;

  try {
    const { lat, lng } = marker.getLatLng();

    const res = await fetch(`${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lng}&depth=${$depth.get()}`);
    const json = await res.json();

    console.log(json);
    $traversalGeoJson.set(json.traversal);
  } catch (e) {
    console.error('failed to fetch traversal: ', e);
  }
});

export default {
  $traversalGeoJson,
  $routeGeoJson,
}
