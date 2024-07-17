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
  CyclewayRange = 'cycleway-range',
  EndInput = 'end-input',
  HeuristicRange = 'heuristic-range',
  RoadRange = 'road-range',
  StartInput = 'start-input',
  SalmonRange = 'salmon-range',
}

export const TraversalDefaults = {
  depth: 20,
  stepDelayMs: 50,
};

export const CostDefaults = {
  CyclewayPreference: 5, // scale based (1..10)
  RoadPreference: 5, // scale based (1..10)
  SalmonCoefficient: 1.3, // % based (1.3 = 130%)
  HeuristicWeight: 0.75, // % based (0.75 = 75%)
};

