// LocalStorage helpers for route caching
import type { MobileRoute, RouteMeta } from '../types/index.ts';

const ROUTE_KEY = 'rusty-mobile-route';
const META_KEY = 'rusty-mobile-meta';
const ENDPOINTS_KEY = 'rusty-mobile-endpoints';
const ROUTE_TIMESTAMP_KEY = 'rusty-mobile-route-timestamp';
const ROUTE_TTL_MS = 3 * 60 * 60 * 1000; // 3 hours in milliseconds

type CachedEndpoints = {
  startLatLng: [number, number] | null;
  endLatLng: [number, number] | null;
  startAddress: string;
  endAddress: string;
};

export function saveRoute(route: MobileRoute, meta: RouteMeta): void {
  localStorage.setItem(ROUTE_KEY, JSON.stringify(route));
  localStorage.setItem(META_KEY, JSON.stringify(meta));
  localStorage.setItem(ROUTE_TIMESTAMP_KEY, Date.now().toString());
}

export function saveEndpoints(endpoints: CachedEndpoints): void {
  localStorage.setItem(ENDPOINTS_KEY, JSON.stringify(endpoints));
}

export function loadRoute(): { route: MobileRoute; meta: RouteMeta } | null {
  const timestampStr = localStorage.getItem(ROUTE_TIMESTAMP_KEY);
  if (timestampStr) {
    const timestamp = parseInt(timestampStr, 10);
    const now = Date.now();
    if (now - timestamp > ROUTE_TTL_MS) {
      clearRoute();
      return null;
    }
  }

  const routeJson = localStorage.getItem(ROUTE_KEY);
  const metaJson = localStorage.getItem(META_KEY);
  if (!routeJson || !metaJson) return null;
  return {
    route: JSON.parse(routeJson) as MobileRoute,
    meta: JSON.parse(metaJson) as RouteMeta,
  };
}

export function loadEndpoints(): CachedEndpoints | null {
  const json = localStorage.getItem(ENDPOINTS_KEY);
  if (!json) return null;
  return JSON.parse(json) as CachedEndpoints;
}

export function clearRoute(): void {
  localStorage.removeItem(ROUTE_KEY);
  localStorage.removeItem(META_KEY);
  localStorage.removeItem(ENDPOINTS_KEY);
  localStorage.removeItem(ROUTE_TIMESTAMP_KEY);
}
