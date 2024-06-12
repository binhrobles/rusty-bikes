import L from 'leaflet';
import Handlebars from 'handlebars';

import controlTemplate from '../templates/cost_control.hbs?raw';
import { CostDefaults, HtmlElementId } from '../consts';
import { $coefficients, $heuristicWeight } from '../store/cost';
const compiledControlTemplate = Handlebars.compile(controlTemplate);

/**
 * Creates a Leaflet control for the cost model config
 * creates the html element representing it and instantiates all the html
 */
const render = (map: L.Map) => {
  // scaffold the leaflet control w/ a static initial div
  const control = new L.Control({ position: 'topleft' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'cost-control control');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    controlDiv.innerHTML = compiledControlTemplate({
      HtmlElementId,
      CostDefaults,
    });
    return controlDiv;
  };

  control.addTo(map);

  // bind the panel's bubbled up change events to the appropriate state changes
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
  render,
};
