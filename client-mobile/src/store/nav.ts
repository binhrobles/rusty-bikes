import { atom, computed } from 'nanostores';
import type { NavigationInstruction } from '../types/index.ts';
import { checkOffRoute, getStepProgress } from '../lib/navigation.ts';
import { $route } from './route.ts';
import { OFF_ROUTE_THRESHOLD_METERS } from '../lib/config.ts';

// How close to a step's end (meters) before advancing to the next step
const STEP_ADVANCE_THRESHOLD = 20;

export const $currentStepIndex = atom<number>(0);
export const $instructions = atom<NavigationInstruction[]>([]);
export const $isOnRoute = atom<boolean>(true);
export const $distanceOffRoute = atom<number>(0); // meters
export const $offRoutePromptVisible = atom<boolean>(false);

export const $currentInstruction = computed(
  [$instructions, $currentStepIndex],
  (instructions, idx) => instructions[idx] ?? null,
);

export const $nextInstruction = computed(
  [$instructions, $currentStepIndex],
  (instructions, idx) => instructions[idx + 1] ?? null,
);

export function advanceStep(): void {
  const next = $currentStepIndex.get() + 1;
  if (next < $instructions.get().length) {
    $currentStepIndex.set(next);
  }
}

export function resetNav(): void {
  $currentStepIndex.set(0);
  $isOnRoute.set(true);
  $distanceOffRoute.set(0);
  $offRoutePromptVisible.set(false);
}

/**
 * Called on each GPS fix. Checks off-route status and advances step when
 * the user reaches the end of the current step.
 */
export function processGPSUpdate(userPosition: [number, number]): void {
  const route = $route.get();
  const idx = $currentStepIndex.get();
  if (!route || idx >= route.features.length) return;

  const currentStep = route.features[idx];

  // Off-route check
  const { offRoute, distanceOff } = checkOffRoute(
    userPosition,
    currentStep,
    OFF_ROUTE_THRESHOLD_METERS,
  );
  $distanceOffRoute.set(distanceOff);

  if (offRoute) {
    $isOnRoute.set(false);
    $offRoutePromptVisible.set(true);
    return;
  }

  $isOnRoute.set(true);

  // Step advancement: move to next step when near the end of current one
  const progress = getStepProgress(currentStep, userPosition);
  const remaining = currentStep.properties.distance - progress;
  if (remaining < STEP_ADVANCE_THRESHOLD) {
    advanceStep();
  }
}
