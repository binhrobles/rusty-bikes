import { atom, computed } from 'nanostores';
import type { CostModel } from '../types/index.ts';

// 0 = speed-optimized, 1 = comfort-optimized
export const $comfortSlider = atom<number>(0.5);
// false = ignore traffic direction, true = penalize salmoning
export const $salmonToggle = atom<boolean>(true);

export const $costModel = computed(
  [$comfortSlider, $salmonToggle],
  (comfort, salmon): CostModel => ({
    road_weight: comfort * 10,
    cycleway_weight: comfort * 10,
    road_coeff: 1.0,
    cycleway_coeff: 1.0,
    salmon_multiplier: salmon ? 1.3 : 1.0,
  }),
);
