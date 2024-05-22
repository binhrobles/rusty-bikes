// available app Modes, tied to their queryParam id
export enum Mode {
  Route = 'route',
  Traverse = 'traverse',
  RouteViz = 'route-viz',
};

export enum HtmlElementId {
  DepthRange = 'depth-range',
  DepthValue = 'depth-value',
  ModeSelect = 'mode-select',
  PaintSelect = 'paint-select',
  PanelParent = 'panel-parent',
  TraversalLonLat = 'traversal-lon-lat',
  TraversePanel = 'traverse-panel',
  RoutePanel = 'route-panel',
};

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

// available paint options, tied to their GeoJSON property key
export enum PaintOptions {
  Depth = 'depth',
  Length = 'length',
  DistanceSoFar = 'distance_so_far',
}

export const TraversalInitialState = {
  depth: 20,
  paint: PaintOptions.Depth,
  clickHint: 'Click Somewhere!',
};
