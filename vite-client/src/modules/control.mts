import L from 'leaflet';
import Handlebars from 'handlebars';

import { Mode, ModeMeta, PaintOptions, HtmlElementId, TraversalInitialState } from '../consts.ts';

import $mode from '../store/mode.ts';
import { $depth, $paint, $marker } from '../store/traversal.ts';

import routePanelPartial from '../templates/routePanel.hbs?raw';
import traversalPanelPartial from '../templates/traversalPanel.hbs?raw';
import controlTemplate from '../templates/control.hbs?raw';

Handlebars.registerPartial('routePanel', routePanelPartial);
Handlebars.registerPartial('traversalPanel', traversalPanelPartial);

// compile control template (which includes partials)
// and generate HTML with static configs on load
const compiledControlTemplate = Handlebars.compile(controlTemplate);
const controlHtml = compiledControlTemplate({
  ModeMeta,
  PaintOptions,
  TraversalInitialState,
  HtmlElementId
});

// just for first paint
// not intended to be used for dynamic mode changing atm
const setSelectedMode = (mode: Mode) => {
  const modeOptionElement = document.getElementById(mode) as HTMLOptionElement;
  if (!modeOptionElement) {
    return console.error(`modeOption element ${mode} couldn't be found!`);
  }
  modeOptionElement.selected = true;
}

// when the marker changes, ensure that the lonLat display
// is tied to the initial and changing values
const onTraversalMarkerChange = (marker: Readonly<L.Marker<any>> | null) => {
  if (!marker) return;

  const lonLatSpan = document.getElementById(HtmlElementId.TraversalLonLat);
  if (!lonLatSpan) {
    return console.error('traversal-lon-lat not present');
  }

  const { lng, lat } = marker.getLatLng();
  lonLatSpan.innerText = `(${lng}, ${lat})`;

  marker.on('move', (event: L.LeafletEvent) => {
    const { latlng: { lng, lat }} = event as L.LeafletMouseEvent;
    lonLatSpan.innerText = `(${lng}, ${lat})`;
  });
}

// sets the visibility of the selected panel
// panels are already in the DOM, with `hidden` attributes
const renderPanel = (mode: Mode, oldMode: Mode | null) => {
  const modePanel = document.getElementById(ModeMeta[mode].htmlElementId);
  if (!modePanel) {
    return console.error('modePanel wasn\'t ready');
  }

  if (oldMode) {
    const oldModePanel = document.getElementById(ModeMeta[oldMode].htmlElementId);
    if (!oldModePanel) {
      return console.error('modePanel wasn\'t ready');
    }
    oldModePanel.hidden = true;
  }

  // assign static HTML
  modePanel.hidden = false;
}

/**
 * Creates a Leaflet control, creates the html element representing it,
 * and instantiates all the html
 */
const render = (map: L.Map) => {
  // scaffold the leaflet control w/ a static initial div
  const control = new L.Control({ position: 'topleft' });

  control.onAdd = () => {
    const controlDiv = L.DomUtil.create('div', 'control');
    L.DomEvent
      .disableClickPropagation(controlDiv)
      .disableScrollPropagation(controlDiv);

    controlDiv.innerHTML = controlHtml;
    return controlDiv;
  };

  control.addTo(map);

  // now, set up the initial view + render the appropriate panel
  setSelectedMode($mode.get());
  renderPanel($mode.get(), null);

  // bind mode select changes to $mode state
  document.getElementById(HtmlElementId.ModeSelect)?.addEventListener('change', (event: Event) => {
    $mode.set((event.target as HTMLSelectElement).value as Mode);
  });

  // bind the panel's bubbled up change events to the appropriate state changes
  document.getElementById(HtmlElementId.PanelParent)?.addEventListener('change', (event: Event) => {
    console.log(event);
    const target = event.target as HTMLElement;

    switch (target.id) {
      // Traversal DOM event handlers
      case HtmlElementId.DepthRange:
        const value = (target as HTMLInputElement).value;

        const depthValue = document.getElementById(HtmlElementId.DepthValue);
        if (!depthValue) return;

        depthValue.innerText = value;
        $depth.set(Number(value));
        break;
      case HtmlElementId.PaintSelect:
        const paint = (target as HTMLSelectElement).value as PaintOptions;
        $paint.set(paint);
        break;

      // Routing

      default:
        console.error(`no onChange event handler for ${target.id}`);
    }
  });
}

// subscribe control to state changes
// updates to the mode should cause the appropriate panel to be rendered
$mode.listen(renderPanel);
// updates to the traversal marker should connect the lon/lat debugger
$marker.listen(onTraversalMarkerChange);

export default {
  render,
};
