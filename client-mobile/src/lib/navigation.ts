// Navigation logic stubs — implemented in rb-1.3
import type { NavigationInstruction, MobileRouteStep } from '../types/index.ts';

export function computeTurnDirection(
  _prevStep: MobileRouteStep,
  _nextStep: MobileRouteStep,
): 'left' | 'right' | 'straight' | 'uturn' {
  // TODO rb-1.3: use @turf/bearing to classify heading delta
  return 'straight';
}

export function generateInstruction(
  stepIndex: number,
  step: MobileRouteStep,
  nextStep?: MobileRouteStep,
): NavigationInstruction {
  // TODO rb-1.3
  const isLast = nextStep == null;
  return {
    action: isLast ? 'arrive' : 'continue',
    direction: null,
    distance: step.properties.distance,
    wayName: step.properties.way_name,
    stepIndex,
  };
}

export function checkOffRoute(
  _userPosition: [number, number],
  _currentStep: MobileRouteStep,
  _thresholdMeters: number = 30,
): { offRoute: boolean; distanceOff: number } {
  // TODO rb-1.3: use @turf/nearest-point-on-line + @turf/distance
  return { offRoute: false, distanceOff: 0 };
}
