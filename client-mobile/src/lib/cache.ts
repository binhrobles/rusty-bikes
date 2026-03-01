// LocalStorage helpers for route caching — implemented in rb-1.8
import type { MobileRoute, RouteMeta } from '../types/index.ts';

const ROUTE_KEY = 'rusty-mobile-route';
const META_KEY = 'rusty-mobile-meta';

export function saveRoute(route: MobileRoute, meta: RouteMeta): void {
  localStorage.setItem(ROUTE_KEY, JSON.stringify(route));
  localStorage.setItem(META_KEY, JSON.stringify(meta));
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

export function clearRoute(): void {
  localStorage.removeItem(ROUTE_KEY);
  localStorage.removeItem(META_KEY);
}
