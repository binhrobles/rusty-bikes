export enum Mode {
  Route = 'route',
  Traverse = 'traverse',
  RouteViz = 'route-viz',
};

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
