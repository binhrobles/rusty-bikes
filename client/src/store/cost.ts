import { atom, map } from 'nanostores'
import { CostDefaults, HtmlElementId } from '../consts';

export type CostModel = {
  cycleway_coefficient: number,
  road_coefficient: number,
  salmon_coefficient: number,
}

export const $coefficients = map<CostModel>({
  cycleway_coefficient: CostDefaults.CyclewayCoefficient,
  road_coefficient: CostDefaults.RoadCoefficient,
  salmon_coefficient: CostDefaults.SalmonCoefficient,
});

export const $heuristicWeight = atom<number>(CostDefaults.HeuristicWeight);

// bind the panel's bubbled up change events to the appropriate state changes
export const bind = () => {
  document
    .getElementById(HtmlElementId.CostConfigParent)
    ?.addEventListener('change', (event: Event) => {
      const target = event.target as HTMLElement;

      switch (target.id) {
        case HtmlElementId.HeuristicWeightRange:
          {
            const value = (target as HTMLInputElement).value;
            $heuristicWeight.set(Number(value));
          }
          break;
        case HtmlElementId.CyclewayCoefficientRange:
          {
            const value = (target as HTMLInputElement).value;
            $coefficients.setKey('cycleway_coefficient', Number(value));
          }
          break;
        case HtmlElementId.RoadCoefficientRange:
          {
            const value = (target as HTMLInputElement).value;
            $coefficients.setKey('road_coefficient', Number(value));
          }
          break;
        case HtmlElementId.SalmonCoefficientRange:
          {
            const value = (target as HTMLInputElement).value;
            $coefficients.setKey('salmon_coefficient', Number(value));
          }
          break;
      }

      console.log(`updated cost model: ${JSON.stringify($coefficients.get(), null, 2)}`);
    });
}

export default {
  bind,
}
