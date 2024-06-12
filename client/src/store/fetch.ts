/**
 * Handles formatting and making the request to Rusty Backend
 */
import { batched, task } from 'nanostores';
import L from 'leaflet';
import { FeatureCollection } from 'geojson';
import { RUSTY_BASE_URL } from '../config.ts';
import { Mode } from '../consts.ts';

import { $markerLatLng as $traversalMarkerLatLng, $depth } from './traverse.ts';
import { $startMarkerLatLng, $endMarkerLatLng } from './route.ts';
import { $clickTime } from './map.ts';
import { $mode } from './mode.ts';
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

type ServerResponse = {
  traversal: FeatureCollection;
  route: FeatureCollection;
};

const fetchTraversal = async (
  latLng: L.LatLng,
  depth: number,
  costModel: CostModel,
  heuristicWeight: number
): Promise<ServerResponse> => {
  const { lat, lng } = latLng;
  const res = await fetch(`${RUSTY_BASE_URL}/traverse`, {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({
      lat: Number(lat.toFixed(COORD_SIG_FIGS)),
      lon: Number(lng.toFixed(COORD_SIG_FIGS)),
      depth,
      heuristic_weight: heuristicWeight,
      cost_model: {
        ...costModel,
        ...HARDCODED_WEIGHTS,
      },
    }),
  });
  console.log(`raw fetched @ ${Date.now() - $clickTime.get()}`);
  return await res.json();
};

const fetchRoute = async (
  startLatLng: L.LatLng,
  endLatLng: L.LatLng,
  withTraversal: boolean,
  costModel: CostModel,
  heuristicWeight: number
): Promise<ServerResponse> => {
  const { lng: startLon, lat: startLat } = startLatLng;
  const { lng: endLon, lat: endLat } = endLatLng;

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
      with_traversal: withTraversal,
      heuristic_weight: heuristicWeight,
      cost_model: {
        ...costModel,
        ...HARDCODED_WEIGHTS,
      },
    }),
  });
  console.log(`raw fetched @ ${Date.now() - $clickTime.get()}`);
  return await res.json();
};

// make a fetch request whenever all conditions for the $mode have been met
export const $raw = batched(
  [
    $mode,
    $traversalMarkerLatLng,
    $depth,
    $startMarkerLatLng,
    $endMarkerLatLng,
    $coefficients,
    $heuristicWeight,
  ],
  (
    mode,
    traversalLatLng,
    depth,
    startLatLng,
    endLatLng,
    costModel,
    heuristicWeight
  ) =>
    task(async () => {
      console.log(`fetch beginning @ ${Date.now() - $clickTime.get()}`);

      if (mode === Mode.Traverse && traversalLatLng) {
        try {
          return await fetchTraversal(
            traversalLatLng,
            depth,
            costModel,
            heuristicWeight
          );
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
            mode === Mode.RouteViz,
            costModel,
            heuristicWeight
          );
        } catch (e) {
          console.error('failed to fetch traversal: ', e);
          return null;
        }
      }

      return null;
    })
);
