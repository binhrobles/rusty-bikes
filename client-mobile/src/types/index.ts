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

export type CorridorData = GeoJSON.FeatureCollection<GeoJSON.LineString>;

export interface NavigateResponse {
  route: MobileRoute;
  meta: RouteMeta;
  corridor?: CorridorData;
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

/// High-level mobile cost model — resolved to full CostModel on the backend.
export interface MobileCostModel {
  priority: number;        // 0 = speed, 1 = comfort
  hill_penalty: number;    // 0 = ignore, 1 = avoid, 2 = strongly avoid
  salmon_penalty: number;  // 0 = ignore, 1 = avoid, 2 = strongly avoid
  avoid_major_roads: boolean;
}
