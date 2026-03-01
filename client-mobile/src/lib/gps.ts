import type { UserPosition } from '../types/index.ts';

// Minimum speed (m/s) before we trust bearing updates (~walking pace)
const MIN_SPEED_FOR_BEARING = 1.0;

export function startGPSTracking(
  onPositionUpdate: (pos: UserPosition, bearing: number) => void,
  onError: (error: GeolocationPositionError) => void,
): () => void {
  let prevPosition: UserPosition | null = null;
  let lastBearing = 0;

  const watchId = navigator.geolocation.watchPosition(
    (rawPos) => {
      const pos: UserPosition = {
        coords: {
          latitude: rawPos.coords.latitude,
          longitude: rawPos.coords.longitude,
          accuracy: rawPos.coords.accuracy,
          heading: rawPos.coords.heading,
          speed: rawPos.coords.speed,
        },
        timestamp: rawPos.timestamp,
      };

      const speed = rawPos.coords.speed ?? 0;
      const isMoving = speed >= MIN_SPEED_FOR_BEARING;

      // Only update bearing when moving — stationary GPS jitter produces wild headings
      if (isMoving) {
        if (rawPos.coords.heading != null) {
          lastBearing = rawPos.coords.heading;
        } else if (prevPosition != null) {
          lastBearing = computeBearing(
            prevPosition.coords.latitude,
            prevPosition.coords.longitude,
            pos.coords.latitude,
            pos.coords.longitude,
          );
        }
      }

      prevPosition = pos;
      onPositionUpdate(pos, lastBearing);
    },
    onError,
    { enableHighAccuracy: true },
  );

  return () => navigator.geolocation.clearWatch(watchId);
}

function computeBearing(lat1: number, lon1: number, lat2: number, lon2: number): number {
  const toRad = (d: number) => (d * Math.PI) / 180;
  const toDeg = (r: number) => (r * 180) / Math.PI;
  const dLon = toRad(lon2 - lon1);
  const y = Math.sin(dLon) * Math.cos(toRad(lat2));
  const x =
    Math.cos(toRad(lat1)) * Math.sin(toRad(lat2)) -
    Math.sin(toRad(lat1)) * Math.cos(toRad(lat2)) * Math.cos(dLon);
  return (toDeg(Math.atan2(y, x)) + 360) % 360;
}
