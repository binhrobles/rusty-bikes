import { atom, computed } from 'nanostores';
import { CostDefaults } from '../consts';

enum Cycleway {
  Track = 'Track',
  Lane = 'Lane',
  Shared = 'Shared',
  No = 'No',
}

enum Road {
  Bike = 'Bike',
  Pedestrian = 'Pedestrian',
  Local = 'Local',
  Collector = 'Collector',
  Arterial = 'Arterial',
}

export type CostModel = {
  cycleway_coefficient: number;
  road_coefficient: number;
  salmon_coefficient: number;
  cycleway_weights: Record<Cycleway, number>;
  road_weights: Record<Road, number>;
};

// directly passed into the cost model
export const $heuristicWeight = atom<number>(CostDefaults.HeuristicWeight);
export const $salmonCoefficient = atom<number>(CostDefaults.SalmonCoefficient);

// preferences dictate how these parts of the cost model are generated
export const $cyclewayPreference = atom<number>(
  CostDefaults.CyclewayPreference
);
export const $roadPreference = atom<number>(CostDefaults.RoadPreference);

export const $costModel = computed(
  [$salmonCoefficient, $cyclewayPreference, $roadPreference],
  (salmonCoefficient, cyclewayPreference, roadPreference): CostModel => {
    // preference values affect the "spread" bw different road / cycleway types
    const cyclewayWeights = {
      [Cycleway.Track]: 1.0 - (cyclewayPreference / 10.0),
      [Cycleway.Lane]: 1.0,
      [Cycleway.Shared]: 1.0 + (cyclewayPreference / 20.0),
      [Cycleway.No]: 1.0 + (cyclewayPreference / 10.0),
    };

    const roadWeights = {
      [Road.Bike]: 1.0 - (roadPreference / 10.0),
      [Road.Pedestrian]: 1.0 - (roadPreference / 20.0),
      [Road.Local]: 1.0,
      [Road.Collector]: 1.0 + (roadPreference / 20.0),
      [Road.Arterial]: 1.0 + (roadPreference / 10.0),
    };

    const model = {
      cycleway_coefficient: 0.5,
      road_coefficient: 0.5,
      salmon_coefficient: salmonCoefficient,
      cycleway_weights: cyclewayWeights,
      road_weights: roadWeights,
    };

    console.log(`generated cost model: ${JSON.stringify(model, null, 2)}`);

    return model;
  }
);
