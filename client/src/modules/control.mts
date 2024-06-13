import L from 'leaflet';

import Control from '../components/Control.svelte';

/**
 * Creates a Leaflet control for the cost model config and adds it to the map
 */
export const addPathfindingControl = (map: L.Map) => {
  const control = new L.Control({ position: 'topleft' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'control');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    new Control({
      target: controlDiv,
    });

    return controlDiv;
  };

  control.addTo(map);
}
