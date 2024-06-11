import L from 'leaflet';
import Handlebars from 'handlebars';

import controlTemplate from '../templates/cost_control.hbs?raw';
import { CostDefaults, HtmlElementId } from '../consts';
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
}

export default {
  render,
};
