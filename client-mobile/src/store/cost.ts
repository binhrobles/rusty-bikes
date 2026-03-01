import { atom, computed } from 'nanostores';
import { Cycleway, Road } from '../types/index.ts';
import type { CostModel } from '../types/index.ts';

// 0 = speed-optimized, 1 = comfort-optimized (maps to preference 0..10)
export const $comfortSlider = atom<number>(0.5);
// false = ignore traffic direction (fastest), true = penalize salmoning
export const $salmonToggle = atom<boolean>(true);

export const $costModel = computed(
  [$comfortSlider, $salmonToggle],
  (comfort, salmon): CostModel => {
    // Map 0..1 slider to 0..10 preference scale (same as desktop)
    const pref = comfort * 10;

    return {
      cycleway_coefficient: 0.5,
      road_coefficient: 0.5,
      salmon_coefficient: salmon ? 1.3 : 1.0,
      cycleway_weights: {
        [Cycleway.Track]: 1.0 - pref / 10.0,
        [Cycleway.Lane]: 1.0,
        [Cycleway.Shared]: 1.0 + pref / 20.0,
        [Cycleway.No]: 1.0 + pref / 10.0,
      },
      road_weights: {
        [Road.Bike]: 1.0 - pref / 10.0,
        [Road.Pedestrian]: 1.0 - pref / 20.0,
        [Road.Local]: 1.0,
        [Road.Collector]: 1.0 + pref / 20.0,
        [Road.Arterial]: 1.0 + pref / 10.0,
      },
    };
  },
);
