import L from 'leaflet';

import LoadingIndicator from '../components/LoadingIndicator.svelte';

/**
 * Creates a Leaflet control for the cost model config and adds it to the map
 */
export const addLoadingControl = (map: L.Map) => {
  const control = new L.Control({ position: 'topright' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    new LoadingIndicator({
      target: controlDiv,
    });

    return controlDiv;
  };

  control.addTo(map);
}
