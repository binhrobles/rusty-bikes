import L from 'leaflet';
import ModePanel from '../components/ModePanel.svelte';

import { HtmlElementId } from '../consts.ts';

import mode from '../store/mode.ts';
import traverse from '../store/traverse.ts';
import { $startMarker, $endMarker, $selectedInput } from '../store/route.ts';

// when the marker changes, ensure that the lonLat display
// is tied to the initial and changing values
const onMarkerChange = (id: HtmlElementId) => {
  const inputElement = document.getElementById(id) as HTMLInputElement;
  if (!inputElement) throw `${id} not ready`;

  return (marker: Readonly<L.Marker> | null) => {
    if (!marker) return;

    const { lng, lat } = marker.getLatLng();
    inputElement.value = `(${lng.toFixed(5)}, ${lat.toFixed(5)})`;

    marker.on('move', (event: L.LeafletEvent) => {
      const {
        latlng: { lng, lat },
      } = event as L.LeafletMouseEvent;
      inputElement.value = `(${lng.toFixed(5)}, ${lat.toFixed(5)})`;
    });
  };
};

const addReactivity = () => {
  // updates markers should tie them to the relevant element
  $startMarker.listen(onMarkerChange(HtmlElementId.StartInput));
  $endMarker.listen(onMarkerChange(HtmlElementId.EndInput));

  // bind elements to publish to state
  mode.bind();
  traverse.bind();

  // when the Routing start / end inputs are clicked,
  // queue them up to be changed on the next map click
  document
    .getElementById(HtmlElementId.PanelParent)
    ?.addEventListener('click', (event: Event) => {
      const target = event.target as HTMLElement;
      switch (target.id) {
        case HtmlElementId.StartInput:
        case HtmlElementId.EndInput:
          $selectedInput.set(target.id);
          break;
        default:
      }
    });
};

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

  addReactivity();
};

export default {
  render,
};
