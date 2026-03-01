import type * as GeoJSON from 'geojson';

// Lean route step from /navigate endpoint (mobile-optimized, no from/to/way IDs)
export interface MobileRouteStep extends GeoJSON.Feature<GeoJSON.LineString> {
  properties: {
    distance: number; // meters for this step
    way_name: string; // street name
    labels: [number, number, boolean]; // [cycleway, road, salmon]
  };
}

export interface MobileRoute {
  type: 'FeatureCollection';
  features: MobileRouteStep[];
}

export interface RouteMeta {
  total_distance: number; // meters
  total_time_estimate: number; // seconds
}

export interface NavigateResponse {
  route: MobileRoute;
  meta: RouteMeta;
}

// Computed turn instruction for UI display
export interface NavigationInstruction {
  action: 'turn' | 'continue' | 'arrive';
  direction: 'left' | 'right' | 'straight' | 'uturn' | null;
  distance: number; // meters
  wayName: string;
  stepIndex: number;
}

export interface UserPosition {
  coords: {
    latitude: number;
    longitude: number;
    accuracy: number;
    heading: number | null;
    speed: number | null;
  };
  timestamp: number;
}

// Cost model matching backend CostModel shape
export interface CostModel {
  road_weight: number;
  cycleway_weight: number;
  road_coeff: number;
  cycleway_coeff: number;
  salmon_multiplier: number;
}
