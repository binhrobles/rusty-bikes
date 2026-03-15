import { atom, computed } from 'nanostores';
import type { MobileCostModel } from '../types/index.ts';

// Route priority: 0 = speed, 1 = comfort
export const $routePriority = atom<number>(0.6);

// 3-position: 0 = ignore, 1 = avoid, 2 = strongly avoid
export const $hillPenalty = atom<number>(1);
export const $salmonPenalty = atom<number>(1);

// Toggle
export const $avoidMajorRoads = atom<boolean>(false);

export const $mobileCostModel = computed(
  [$routePriority, $hillPenalty, $salmonPenalty, $avoidMajorRoads],
  (priority, hill, salmon, avoidMajor): MobileCostModel => ({
    priority,
    hill_penalty: hill,
    salmon_penalty: salmon,
    avoid_major_roads: avoidMajor,
  }),
);
