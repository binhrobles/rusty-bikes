import { atom } from 'nanostores';
import { CostDefaults } from '../consts';

export type CostModel = {
  cycleway_coefficient: number;
  road_coefficient: number;
  salmon_coefficient: number;
};

export const $heuristicWeight = atom<number>(CostDefaults.HeuristicWeight);
export const $cyclewayPreference = atom<number>(CostDefaults.CyclewayPreference);
export const $roadPreference = atom<number>(CostDefaults.RoadPreference);
export const $salmonCoefficient = atom<number>(CostDefaults.SalmonCoefficient);
