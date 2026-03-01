import Radar from 'radar-sdk-js';
import { RADAR_API_KEY } from './lib/config.ts';
import App from './App.svelte';

Radar.initialize(RADAR_API_KEY);

const app = new App({
  target: document.getElementById('app')!,
});

export default app;
