import { atom, batched, task } from 'nanostores';
import { RUSTY_BASE_URL } from '../lib/config.ts';
import { saveRoute } from '../lib/cache.ts';
import { $startLatLng, $endLatLng, $route, $routeMeta } from './route.ts';
import { $costModel } from './cost.ts';
import type { NavigateResponse } from '../types/index.ts';

const COORD_SIG_FIGS = 7;

export const $isLoading = atom<boolean>(false);
export const $error = atom<string | null>(null);

async function fetchNavigate(
  start: [number, number],
  end: [number, number],
  costModel: ReturnType<(typeof $costModel)['get']>,
): Promise<NavigateResponse | null> {
  $isLoading.set(true);
  $error.set(null);

  let response: Response;
  try {
    response = await fetch(`${RUSTY_BASE_URL}/navigate`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({
        start: {
          lat: Number(start[0].toFixed(COORD_SIG_FIGS)),
          lon: Number(start[1].toFixed(COORD_SIG_FIGS)),
        },
        end: {
          lat: Number(end[0].toFixed(COORD_SIG_FIGS)),
          lon: Number(end[1].toFixed(COORD_SIG_FIGS)),
        },
        heuristic_weight: 0.75,
        cost_model: costModel,
      }),
    });
  } catch (e) {
    $isLoading.set(false);
    $error.set('Network error — check your connection.');
    console.error('fetch /navigate failed:', e);
    return null;
  }

  $isLoading.set(false);

  if (!response.ok) {
    $error.set(`Server error (${response.status})`);
    console.error('/navigate non-OK status:', response.status);
    return null;
  }

  return (await response.json()) as NavigateResponse;
}

// Auto-fires whenever start, end, and cost model are all set
export const $raw = batched(
  [$startLatLng, $endLatLng, $costModel],
  (start, end, costModel) =>
    task(async () => {
      if (!start || !end) return null;

      const data = await fetchNavigate(start, end, costModel);
      if (!data) return null;

      $route.set(data.route);
      $routeMeta.set(data.meta);
      saveRoute(data.route, data.meta);

      return data;
    }),
);
