// LocalStorage helpers for route caching
import type { MobileRoute, RouteMeta } from '../types/index.ts';

const ROUTE_KEY = 'rusty-mobile-route';
const META_KEY = 'rusty-mobile-meta';
const ENDPOINTS_KEY = 'rusty-mobile-endpoints';

type CachedEndpoints = {
  startLatLng: [number, number] | null;
  endLatLng: [number, number] | null;
  startAddress: string;
  endAddress: string;
};

export function saveRoute(route: MobileRoute, meta: RouteMeta): void {
  localStorage.setItem(ROUTE_KEY, JSON.stringify(route));
  localStorage.setItem(META_KEY, JSON.stringify(meta));
}

export function saveEndpoints(endpoints: CachedEndpoints): void {
  localStorage.setItem(ENDPOINTS_KEY, JSON.stringify(endpoints));
}

export function loadRoute(): { route: MobileRoute; meta: RouteMeta } | null {
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
}
