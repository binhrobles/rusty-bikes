export const RUSTY_BASE_URL = import.meta.env.PROD
  ? 'https://mt5baplmo6.execute-api.us-east-1.amazonaws.com/Prod'
  : 'http://localhost:9000/lambda-url/lambda-handler';

export const RADAR_API_KEY = import.meta.env.VITE_RADAR_API_KEY as string;

// NYC center for biasing geocoding results
export const NYC_CENTER = { latitude: 40.7, longitude: -73.98 };

// Off-route detection threshold in meters
export const OFF_ROUTE_THRESHOLD_METERS = 30;
