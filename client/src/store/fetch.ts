/**
 * Handles formatting and making the request to Rusty Backend
 */
import { atom, batched, task } from 'nanostores';
import L from 'leaflet';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../config.ts';

import {
  $startMarkerLatLng,
  $endMarkerLatLng,
  $withTraversal,
} from './route.ts';
import { $clickTime } from './map.ts';
import { $coefficients, $heuristicWeight, CostModel } from './cost.ts';

// number of significant figs to truncate our coords to
// the OSM data only has up to 7 figures precision
// using more might be making our spatial queries wack
const COORD_SIG_FIGS = 7;

// maybe make these configurable
const HARDCODED_WEIGHTS = {
  cycleway_weights: {
    Shared: 1.5,
    Lane: 1.0,
    Track: 0.5,
  },
  road_weights: {
    Pedestrian: 1.2,
    Bike: 0.5,
    Local: 1.2,
    Collector: 1.4,
    Arterial: 2,
  },
};

export const $isLoading = atom<boolean>(false);
export const $isSuccess = atom<boolean>(true);

export type RouteMetadata = {
  max_depth: number;
  cost_range: number[];
};

type ServerResponse = {
  traversal: FeatureCollection;
  route: FeatureCollection;
  meta: RouteMetadata;
};

const fetchRoute = async (
  startLatLng: L.LatLng,
  endLatLng: L.LatLng,
  withTraversal: boolean,
  costModel: CostModel,
  heuristicWeight: number
): Promise<ServerResponse | undefined> => {
  const { lng: startLon, lat: startLat } = startLatLng;
  const { lng: endLon, lat: endLat } = endLatLng;

  $isLoading.set(true);
  try {
    const res = await fetch(`${RUSTY_BASE_URL}/route`, {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({
        start: {
          lat: Number(startLat.toFixed(COORD_SIG_FIGS)),
          lon: Number(startLon.toFixed(COORD_SIG_FIGS)),
        },
        end: {
          lat: Number(endLat.toFixed(COORD_SIG_FIGS)),
          lon: Number(endLon.toFixed(COORD_SIG_FIGS)),
        },
        with_traversal: Boolean(withTraversal), // ensure this gets sent as a bool, not stringified
        heuristic_weight: heuristicWeight,
        cost_model: {
          ...costModel,
          ...HARDCODED_WEIGHTS,
        },
      }),
    });
    $isLoading.set(false);
    $isSuccess.set(true);

    console.log(`raw fetched @ ${Date.now() - $clickTime.get()}`);
    return await res.json();
  } catch (e) {
    $isLoading.set(false);
    $isSuccess.set(false);

    console.error(`failed to fetch ${e}`);
    return undefined;
  }
};

// make a fetch request whenever all conditions have been met
export const $raw = batched(
  [
    $startMarkerLatLng,
    $endMarkerLatLng,
    $withTraversal,
    $coefficients,
    $heuristicWeight,
  ],
  (startLatLng, endLatLng, withTraversal, costModel, heuristicWeight) =>
    task(async () => {
      console.log(`fetch beginning @ ${Date.now() - $clickTime.get()}`);

      if (startLatLng && endLatLng) {
        return await fetchRoute(
          startLatLng,
          endLatLng,
          withTraversal,
          costModel,
          heuristicWeight
        );
      }

      return null;
    })
);
