import L from 'leaflet';
import { SvelteComponent } from 'svelte';

import CostPanel from '../components/CostPanel.svelte';

/**
 * Creates a Leaflet control for the cost model config and adds it to the map
 */
const render = (map: L.Map) => {
  const control = new L.Control({ position: 'topleft' });

  let costComponent: SvelteComponent;
  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'control');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    costComponent = new CostPanel({
      target: controlDiv,
    });

    return controlDiv;
  };

  control.onRemove = () => {
    if (costComponent) {
      costComponent.$destroy();
    }
  }

  control.addTo(map);
}

export default {
  render,
};
