import { computed } from 'nanostores';
import { $raw } from './fetch.ts';

export type ElevationProfile = {
  hasData: boolean;
  totalGain: number;
  totalLoss: number;
  elevations: { distance: number; elevation: number }[];
  totalDistance: number;
};

const EMPTY: ElevationProfile = {
  hasData: false,
  totalGain: 0,
  totalLoss: 0,
  elevations: [],
  totalDistance: 0,
};

export const $elevationProfile = computed([$raw], (raw): ElevationProfile => {
  if (!raw?.route?.features) return EMPTY;

  let cumDistance = 0;
  let gain = 0;
  let loss = 0;
  let elev = 0;
  const elevations = [{ distance: 0, elevation: 0 }];

  for (const feature of raw.route.features) {
    const props = feature.properties;
    if (!props) continue;
    cumDistance += props.distance || 0;
    const g = props.elevation_gain ?? 0;
    const l = props.elevation_loss ?? 0;
    gain += g;
    loss += l;
    elev += g - l;
    elevations.push({ distance: cumDistance, elevation: elev });
  }

  if (elevations.length < 2 || (gain === 0 && loss === 0)) return EMPTY;

  return {
    hasData: true,
    totalGain: gain,
    totalLoss: loss,
    elevations,
    totalDistance: cumDistance,
  };
});
