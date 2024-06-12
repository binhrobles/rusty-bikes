import { atom, map } from 'nanostores'
import { CostDefaults } from '../consts';

export type CostModel = {
  cycleway_coefficient: number,
  road_coefficient: number,
  salmon_coefficient: number,
}

export const $coefficients = map<CostModel>({
  cycleway_coefficient: CostDefaults.CyclewayCoefficient,
  road_coefficient: CostDefaults.RoadCoefficient,
  salmon_coefficient: CostDefaults.SalmonCoefficient,
});

export const $heuristicWeight = atom<number>(CostDefaults.HeuristicWeight);
