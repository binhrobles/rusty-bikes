import { bearing } from '@turf/bearing';
import { distance } from '@turf/distance';
import { nearestPointOnLine } from '@turf/nearest-point-on-line';
import { point, lineString } from '@turf/helpers';
import type { NavigationInstruction, MobileRouteStep } from '../types/index.ts';

// Bearing delta thresholds for turn classification (degrees)
const STRAIGHT_THRESHOLD = 30;
const UTURN_THRESHOLD = 150;

/**
 * Compute the turn direction needed to go from the end of prevStep into nextStep.
 * Uses the final segment of prevStep as the incoming heading and the first segment
 * of nextStep as the outgoing heading.
 */
export function computeTurnDirection(
  prevStep: MobileRouteStep,
  nextStep: MobileRouteStep,
): 'left' | 'right' | 'straight' | 'uturn' {
  const prevCoords = prevStep.geometry.coordinates;
  const nextCoords = nextStep.geometry.coordinates;

  // Need at least 2 coords in each step to compute a heading
  if (prevCoords.length < 2 || nextCoords.length < 2) return 'straight';

  const incomingStart = point(prevCoords[prevCoords.length - 2]);
  const incomingEnd = point(prevCoords[prevCoords.length - 1]);
  const outgoingEnd = point(nextCoords[1]);

  const incomingBearing = bearing(incomingStart, incomingEnd);
  const outgoingBearing = bearing(incomingEnd, outgoingEnd);

  // Normalize delta to -180..180
  const delta = ((outgoingBearing - incomingBearing + 540) % 360) - 180;

  if (Math.abs(delta) < STRAIGHT_THRESHOLD) return 'straight';
  if (Math.abs(delta) > UTURN_THRESHOLD) return 'uturn';
  return delta > 0 ? 'right' : 'left';
}

/**
 * Generate a NavigationInstruction for a route step.
 * Turn direction is computed by looking ahead at the next step.
 */
export function generateInstruction(
  stepIndex: number,
  step: MobileRouteStep,
  nextStep?: MobileRouteStep,
): NavigationInstruction {
  if (!nextStep) {
    return {
      action: 'arrive',
      direction: null,
      distance: step.properties.distance,
      wayName: step.properties.way_name,
      stepIndex,
    };
  }

  const direction = computeTurnDirection(step, nextStep);
  const action = direction === 'straight' ? 'continue' : 'turn';

  return {
    action,
    direction,
    distance: step.properties.distance,
    wayName: step.properties.way_name,
    stepIndex,
  };
}

/**
 * Check whether the user has strayed off the current step geometry.
 * Returns the perpendicular snap distance in meters.
 */
export function checkOffRoute(
  userPosition: [number, number], // [lat, lon]
  currentStep: MobileRouteStep,
  thresholdMeters: number = 30,
): { offRoute: boolean; distanceOff: number } {
  const userPoint = point([userPosition[1], userPosition[0]]); // [lon, lat]
  const line = lineString(currentStep.geometry.coordinates);
  const snapped = nearestPointOnLine(line, userPoint, { units: 'kilometers' });
  const distanceOff = (snapped.properties.dist ?? Infinity) * 1000; // km → meters

  return { offRoute: distanceOff > thresholdMeters, distanceOff };
}

/**
 * Returns how far along the current step the user is, in meters.
 * Used to decide when to advance to the next step.
 */
export function getStepProgress(
  currentStep: MobileRouteStep,
  userPosition: [number, number], // [lat, lon]
): number {
  const coords = currentStep.geometry.coordinates;
  const userPoint = point([userPosition[1], userPosition[0]]);

  const line = lineString(coords);
  const snapped = nearestPointOnLine(line, userPoint, { units: 'kilometers' });

  // `location` is distance along the line to the snapped point in km
  const locationKm = snapped.properties.location ?? 0;

  // Cross-check: if user is past the step end, clamp to full step distance
  const stepEndPoint = point(coords[coords.length - 1]);
  const distToEnd = distance(userPoint, stepEndPoint, { units: 'meters' });
  const stepLength = currentStep.properties.distance;

  return distToEnd < 15 ? stepLength : locationKm * 1000;
}
