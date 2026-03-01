import { atom } from 'nanostores';
import { startGPSTracking } from '../lib/gps.ts';
import { processGPSUpdate } from './nav.ts';
import type { UserPosition } from '../types/index.ts';

export const $userPosition = atom<UserPosition | null>(null);
export const $userBearing = atom<number>(0); // degrees, 0 = north
export const $gpsError = atom<GeolocationPositionError | null>(null);
export const $gpsActive = atom<boolean>(false);

let stopTracking: (() => void) | null = null;

export function startGPS(): void {
  if (stopTracking) return; // already running

  $gpsError.set(null);
  $gpsActive.set(true);

  stopTracking = startGPSTracking(
    (pos, bearing) => {
      $userPosition.set(pos);
      $userBearing.set(bearing);
      processGPSUpdate([pos.coords.latitude, pos.coords.longitude]);
    },
    (err) => {
      $gpsError.set(err);
      $gpsActive.set(false);
    },
  );
}

export function stopGPS(): void {
  stopTracking?.();
  stopTracking = null;
  $gpsActive.set(false);
}
