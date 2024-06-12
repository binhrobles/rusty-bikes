import { atom, map } from 'nanostores';
import { CostDefaults } from '../consts';

export type CostModel = {
  cycleway_coefficient: number;
  road_coefficient: number;
  salmon_coefficient: number;
};

const defaultCostModel: CostModel = {
  cycleway_coefficient: CostDefaults.CyclewayCoefficient,
  road_coefficient: CostDefaults.RoadCoefficient,
  salmon_coefficient: CostDefaults.SalmonCoefficient,
};

export const $heuristicWeight = atom<number>(CostDefaults.HeuristicWeight);
export const $coefficients = map<CostModel>(defaultCostModel);

// helper functions to just update the key in the model
export const updateCycleway = (event: Event) => {
  const value = Number((event.target as HTMLInputElement).value);
  $coefficients.setKey('cycleway_coefficient', value);
};
export const updateRoad = (event: Event) => {
  const value = Number((event.target as HTMLInputElement).value);
  $coefficients.setKey('road_coefficient', value);
};
export const updateSalmon = (event: Event) => {
  const value = Number((event.target as HTMLInputElement).value);
  $coefficients.setKey('salmon_coefficient', value);
};
