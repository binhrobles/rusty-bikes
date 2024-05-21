import L from 'leaflet';
import Handlebars from 'handlebars';

import { Mode, ModeMeta } from '../config.ts';
import template from '../templates/control.hbs?raw';

// compile template and generate HTML with static config on load
const compiled = Handlebars.compile(template);
const html = compiled(ModeMeta);

/*
 * Checks if the `mode` queryParam has been set, otherwise returns Mode.Route
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
    console.error(`modeOption element ${mode} couldn't be found!`);
    return;
  }
  modeOptionElement.selected = true;
}

const wireModeChange = () => {
  // NOTE: yeah I could add this as a property in the html template but I
  //       think I like having the JS in JS rather than in the hbs file
  const modeSelectDiv = document.getElementById('mode-select');
  if (!modeSelectDiv) {
    console.error('modeSelectDiv wasn\'t ready');
    return;
  }

  modeSelectDiv.onchange = (event: Event) => {
    const mode = (event.target as HTMLSelectElement).value as Mode;
    console.log(`selected: ${mode}`);

    // TODO: tie this back to some state update
  }
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

    controlDiv.innerHTML = html;
    return controlDiv;
  };

  control.addTo(map);

  // now, fill in with the appropriate panel
  const mode = determineFirstMode();
  setSelectedMode(mode);

  // TODO: paint relevant control panel

  // add event listeners
  wireModeChange();
}

// TODO: subscribe to update on `state.mode` and update `panel` html

export default {
  render,
};
