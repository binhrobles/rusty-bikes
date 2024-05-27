/**
 * Handles formatting and making the request to Rusty Backend
 */
import { batched, task } from 'nanostores';
import L from 'leaflet';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../config.ts';
import { Mode } from '../consts.ts';

import {
  $markerLatLng as $traversalMarkerLatLng,
  $depth,
} from './traversal.ts';
import { $startMarkerLatLng, $endMarkerLatLng } from './route.ts';
import { $clickTime } from './map.ts';
import { $mode } from './mode.ts';

type ServerResponse = {
  traversal: FeatureCollection;
  route: FeatureCollection;
};

const fetchTraversal = async (
  latLng: L.LatLng,
  depth: number
): Promise<ServerResponse> => {
  const { lat, lng } = latLng;
  const res = await fetch(
    `${RUSTY_BASE_URL}/traverse?lat=${lat}&lon=${lng}&depth=${depth}`
  );
  console.log(`raw fetched @ ${Date.now() - $clickTime.get()}`)
  return await res.json();
};

const fetchRoute = async (
  startLatLng: L.LatLng,
  endLatLng: L.LatLng,
  withTraversal: boolean
): Promise<ServerResponse> => {
  const { lng: startLon, lat: startLat } = startLatLng;
  const { lng: endLon, lat: endLat } = endLatLng;

  const res = await fetch(
    `${RUSTY_BASE_URL}/route?start=${startLon},${startLat}&end=${endLon},${endLat}&with_traversal=${withTraversal}`
  );
  console.log(`raw fetched @ ${Date.now() - $clickTime.get()}`)
  return await res.json();
};

// make a fetch request whenever all conditions for the $mode have been met
export const $raw = batched(
  [$mode, $traversalMarkerLatLng, $depth, $startMarkerLatLng, $endMarkerLatLng],
  (mode, traversalLatLng, depth, startLatLng, endLatLng) =>
    task(async () => {
      console.log(`fetch beginning @ ${Date.now() - $clickTime.get()}`)

      if (mode === Mode.Traverse && traversalLatLng) {
        try {
          return await fetchTraversal(traversalLatLng, depth);
        } catch (e) {
          console.error('failed to fetch traversal: ', e);
          return null;
        }
      } else if (
        (mode === Mode.Route || mode === Mode.RouteViz) &&
        startLatLng &&
        endLatLng
      ) {
        try {
          return await fetchRoute(
            startLatLng,
            endLatLng,
            mode === Mode.RouteViz
          );
        } catch (e) {
          console.error('failed to fetch traversal: ', e);
          return null;
        }
      }

      return null;
    })
);
