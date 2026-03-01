import { atom } from 'nanostores';
import type { MobileRoute, RouteMeta } from '../types/index.ts';

export const $route = atom<MobileRoute | null>(null);
export const $routeMeta = atom<RouteMeta | null>(null);

// Start/end coordinates for the current route request
export const $startLatLng = atom<[number, number] | null>(null); // [lat, lon]
export const $endLatLng = atom<[number, number] | null>(null); // [lat, lon]
