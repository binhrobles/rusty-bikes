import './style.css';
import Radar from 'radar-sdk-js';
import { createAutocompletePlugin } from '@radarlabs/plugin-autocomplete';
import '@radarlabs/plugin-autocomplete/dist/radar-autocomplete.css';
import { RADAR_API_KEY } from './config.ts';
import App from './App.svelte'

Radar.initialize(RADAR_API_KEY);
Radar.registerPlugin(createAutocompletePlugin());

const app = new App({
  target: document.getElementById('app')!,
})

export default app;
