// Mappings for the property keys being returned from the backend
// moved to using single letter prop keys to save some space
// see TraversalSegment's serialization
export enum PropKey {
  From = 'from',
  To = 'to',
  Way = 'way',
  Depth = 'depth',
  DistanceSoFar = 'distance_so_far',
  DistanceRemaining = 'distance_remaining',
  Length = 'length',
  Cost = 'cost',
  CostFactor = 'cost_factor',
  CostSoFar = 'cost_so_far',
  Index = 'idx',
}

// typo mitigation
export enum HtmlElementId {
  CyclewayCoefficientRange = 'cycleway-coefficient-range',
  EndInput = 'end-input',
  HeuristicWeightRange = 'heuristic-weight-range',
  RoadCoefficientRange = 'road-coefficient-range',
  StartInput = 'start-input',
  SalmonCoefficientRange = 'salmon-coefficient-range',
}

export const TraversalDefaults = {
  depth: 20,
  stepDelayMs: 75,
};

export const CostDefaults = {
  CyclewayCoefficient: 0.3,
  RoadCoefficient: 0.4,
  SalmonCoefficient: 1.3,
  HeuristicWeight: 0.75,
};
