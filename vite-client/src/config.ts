// available app Modes, tied to their queryParam id
export enum Mode {
  Route = 'route',
  Traverse = 'traverse',
  RouteViz = 'route-viz',
};

// holds mode-specific information
export const ModeMeta = {
  [Mode.Traverse]: {
    label: 'Traverse',
  },
  [Mode.Route]: {
    label: 'Route',
  },
  [Mode.RouteViz]: {
    label: 'Route + Visualization',
  },
};

// available paint options, tied to their GeoJSON property key
export enum PaintOptions {
  Depth = 'depth',
  Length = 'length',
  DistanceSoFar = 'distance_so_far',
}
