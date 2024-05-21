import L from 'leaflet';
import Handlebars from 'handlebars';

import { Mode, ModeMeta, PaintOptions } from '../config.ts';

// compile templates and generate HTML with static configs on load
import controlTemplate from '../templates/control.hbs?raw';
const compiledControlTemplate = Handlebars.compile(controlTemplate);
const controlHtml = compiledControlTemplate(ModeMeta);

import traversalPanelTemplate from '../templates/traversalPanel.hbs?raw';
const compiledTraversalTemplate = Handlebars.compile(traversalPanelTemplate);

import routePanelHtml from '../templates/routePanel.hbs?raw';

const modeToHtmlMap = {
  [Mode.Traverse]: compiledTraversalTemplate(PaintOptions),
  [Mode.Route]: routePanelHtml,
  [Mode.RouteViz]: routePanelHtml, // TODO: eventually, a distinct panel
};

/*
 * Checks if the `mode` queryParam has been set to a valid Mode option,
 * otherwise returns Mode.Route
 */
const determineFirstMode = (): Mode => {
  const params = new URLSearchParams(document.location.search);
  const mode = params.get('mode');

  if (!(mode && Object.values<string>(Mode).includes(mode))) return Mode.Route;

  return mode as Mode;
};

const setSelectedMode = (mode: Mode) => {
  const modeOptionElement = document.getElementById(mode) as HTMLOptionElement;
  if (!modeOptionElement) {
    return console.error(`modeOption element ${mode} couldn't be found!`);
  }
  modeOptionElement.selected = true;
}

const wireModeChange = () => {
  // NOTE: yeah I could add this as a property in the html template but I
  //       think I like having the JS in JS rather than in the hbs file
  const modeSelectDiv = document.getElementById('mode-select');
  if (!modeSelectDiv) {
    return console.error('modeSelectDiv wasn\'t ready');
  }

  modeSelectDiv.onchange = (event: Event) => {
    const mode = (event.target as HTMLSelectElement).value as Mode;
    console.log(`selected: ${mode}`);

    // TODO: tie this back to some state update
  }
}

const renderControlPanel = (mode: Mode) => {
  const panelDiv = document.getElementById('panel');
  if (!panelDiv) {
    return console.error('panelDiv wasn\'t ready');
  }

  panelDiv.innerHTML = modeToHtmlMap[mode];
}

/**
 * Creates a Leaflet control, creates the html element representing it,
 * and instantiates all the html
 */
const render = (map: L.Map) => {
  // scaffold the leaflet control w/ a nice initial div
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
  const mode = determineFirstMode();

  setSelectedMode(mode);
  renderControlPanel(mode);

  // add event listeners
  wireModeChange();
}

// TODO: subscribe to update on `state.mode` and update `panel` html

export default {
  render,
};
