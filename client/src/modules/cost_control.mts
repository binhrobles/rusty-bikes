import L from 'leaflet';

import CostPanel from '../components/CostPanel.svelte';

/**
 * Creates a Leaflet control for the cost model config and adds it to the map
 */
const render = (map: L.Map) => {
  const control = new L.Control({ position: 'topleft' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'control');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    new CostPanel({
      target: controlDiv,
    });

    return controlDiv;
  };

  control.addTo(map);
}

export default {
  render,
};
