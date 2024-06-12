import L from 'leaflet';
import ModePanel from '../components/ModePanel.svelte';

/**
 * Creates a Leaflet control, creates the html element representing it,
 * and instantiates all the html
 */
const render = (map: L.Map) => {
  const control = new L.Control({ position: 'topleft' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'control');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    new ModePanel({
      target: controlDiv,
    });

    return controlDiv;
  };

  control.addTo(map);
};

export default {
  render,
};
