// available app Modes, tied to their queryParam id
export enum Mode {
  Route = 'route',
  Traverse = 'traverse',
  RouteViz = 'route-viz',
}

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
  CostSoFar = 'cost_so_far',
  Index = 'idx',
}

// typo mitigation
export enum HtmlElementId {
  CostConfigParent = 'cost-config-parent',
  CyclewayCoefficientRange = 'cycleway-coefficient-range',
  DepthRange = 'depth-range',
  DepthValue = 'depth-value',
  EndInput = 'end-input',
  HeuristicWeightRange = 'heuristic-weight-range',
  ModeSelect = 'mode-select',
  PaintSelect = 'paint-select',
  PanelParent = 'panel-parent',
  RoadCoefficientRange = 'road-coefficient-range',
  RoutePanel = 'route-panel',
  StartInput = 'start-input',
  SalmonCoefficientRange = 'salmon-coefficient-range',
  TraversePanel = 'traverse-panel',
}

// holds mode-specific information
export const ModeMeta = {
  [Mode.Traverse]: {
    label: 'Traverse',
    htmlElementId: HtmlElementId.TraversePanel,
  },
  [Mode.Route]: {
    label: 'Route',
    htmlElementId: HtmlElementId.RoutePanel,
  },
  [Mode.RouteViz]: {
    label: 'Route + Visualization',
    htmlElementId: HtmlElementId.RoutePanel, // TODO: eventually, a distinct panel
  },
};

// available paint options, tied to their PropKey
export enum PaintOptions {
  Depth = 'Depth',
  DistanceSoFar = 'DistanceSoFar',
  Length = 'Length',
  Cost = 'Cost',
  CostSoFar = 'CostSoFar',
}

export const TraversalDefaults = {
  depth: 20,
  paint: PaintOptions.Depth,
  stepDelayMs: 75,
};

export const CostDefaults = {
  CyclewayCoefficient: 0.3,
  RoadCoefficient: 0.4,
  SalmonCoefficient: 1.3,
  HeuristicWeight: 0.75,
};
