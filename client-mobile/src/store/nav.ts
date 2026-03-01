import { atom, computed } from 'nanostores';
import type { NavigationInstruction } from '../types/index.ts';

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
