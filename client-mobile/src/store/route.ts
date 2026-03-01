import { atom } from 'nanostores';
import type { MobileRoute, RouteMeta } from '../types/index.ts';
import { generateInstruction } from '../lib/navigation.ts';
import { $instructions, resetNav } from './nav.ts';

export const $route = atom<MobileRoute | null>(null);
export const $routeMeta = atom<RouteMeta | null>(null);

// Geocoded address strings for display
export const $startAddress = atom<string>('');
export const $endAddress = atom<string>('');

// Raw [lat, lon] coordinates for the route request
export const $startLatLng = atom<[number, number] | null>(null);
export const $endLatLng = atom<[number, number] | null>(null);

// When a new route arrives, recompute all turn instructions and reset nav state
$route.listen((route) => {
  if (!route) {
    $instructions.set([]);
    return;
  }

  const steps = route.features;
  const instructions = steps.map((step, i) =>
    generateInstruction(i, step, steps[i + 1]),
  );

  resetNav();
  $instructions.set(instructions);
});
