import { atom } from 'nanostores';
import type { NavigationInstruction } from '../types/index.ts';

export const $currentStepIndex = atom<number>(0);
export const $instructions = atom<NavigationInstruction[]>([]);
export const $isOnRoute = atom<boolean>(true);
export const $distanceOffRoute = atom<number>(0); // meters
export const $offRoutePromptVisible = atom<boolean>(false);
