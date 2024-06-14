import L, { ControlPosition } from 'leaflet';
import { ComponentType } from 'svelte';

import Control from '../components/Control.svelte';
import LoadingIndicator from '../components/LoadingIndicator.svelte';

const createControlComponent = (
  Component: ComponentType,
  position: ControlPosition
) => {
  const control = new L.Control({ position });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div');
    L.DomEvent.disableClickPropagation(controlDiv).disableScrollPropagation(
      controlDiv
    );

    new Component({
      target: controlDiv,
    });

    return controlDiv;
  };

  return control;
};

export const addPathfindingControl = (map: L.Map) => {
  const control = createControlComponent(Control, 'topleft');
  control.addTo(map);
};

export const addLoadingIndicator = (map: L.Map) => {
  const control = createControlComponent(LoadingIndicator, 'topright');
  control.addTo(map);
};
