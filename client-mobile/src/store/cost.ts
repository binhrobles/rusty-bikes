import { atom, computed } from 'nanostores';
import { Cycleway, Road } from '../types/index.ts';
import type { CostModel } from '../types/index.ts';

// Comfort (0..1): how much bike infrastructure and road quality matter
// Speed  (0..1): how much raw distance matters (and de-weights road type preference)
export const $comfortSlider = atom<number>(0.7);
export const $speedSlider = atom<number>(0.5);
export const $hillSlider = atom<number>(0.5);

const SALMON_COEFFICIENTS = [1.1, 1.25, 1.5, 1.8] as const;
export const $salmonSlider = atom<number>(2);

// Fixed weight spreads — only coefficients vary based on sliders
const CYCLEWAY_WEIGHTS = {
  [Cycleway.Track]: 0.5,
  [Cycleway.Lane]: 1.0,
  [Cycleway.Shared]: 1.5,
  [Cycleway.No]: 1.7,
};
const ROAD_WEIGHTS = {
  [Road.Bike]: 0.5,
  [Road.Pedestrian]: 0.9,
  [Road.Local]: 1.2,
  [Road.Collector]: 1.4,
  [Road.Arterial]: 2.0,
};

export const $costModel = computed(
  [$comfortSlider, $speedSlider, $salmonSlider, $hillSlider],
  (comfort, speed, salmon, hill): CostModel => {
    // Comfort: how much infrastructure quality matters (floor 0.1 — never fully zeroes out)
    const cycleway_coefficient = 0.1 + comfort * 0.7;
    const road_coefficient = 0.1 + Math.max(0, comfort * 0.5 - speed * 0.2);
    // Speed: how much raw distance matters
    const distance_coefficient = speed * 0.5;
    // Hill: 0..1 mapped to elevation_coefficient 0..2
    const elevation_coefficient = hill * 2;

    return {
      cycleway_coefficient,
      road_coefficient,
      salmon_coefficient: SALMON_COEFFICIENTS[salmon],
      distance_coefficient,
      elevation_coefficient,
      cycleway_weights: CYCLEWAY_WEIGHTS,
      road_weights: ROAD_WEIGHTS,
    };
  },
);
