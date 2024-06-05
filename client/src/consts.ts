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
  From = 'f',
  To = 't',
  Way = 'w',
  Depth = 'd',
  DistanceSoFar = 'di',
  Length = 'l',
  Cost = 'c',
  Index = 'i',
}

// typo mitigation
export enum HtmlElementId {
  DepthRange = 'depth-range',
  DepthValue = 'depth-value',
  EndInput = 'end-input',
  ModeSelect = 'mode-select',
  PaintSelect = 'paint-select',
  PanelParent = 'panel-parent',
  RoutePanel = 'route-panel',
  StartInput = 'start-input',
  TraversalLonLat = 'traversal-lon-lat',
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
}

export const TraversalDefaults = {
  depth: 20,
  paint: PaintOptions.Depth,
  clickHint: 'Click Somewhere!',
  stepDelayMs: 75,
};
