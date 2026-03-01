import { atom } from 'nanostores';
import type { UserPosition } from '../types/index.ts';

export const $userPosition = atom<UserPosition | null>(null);
export const $userBearing = atom<number>(0); // degrees, 0 = north
