<script lang="ts">
  import { onMount } from 'svelte';
  import MapView from './components/MapView.svelte';
  import InstructionPanel from './components/InstructionPanel.svelte';
  import OffRoutePrompt from './components/OffRoutePrompt.svelte';
  import SearchInput from './components/SearchInput.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import { startGPS } from './store/gps.ts';
  import { loadRoute, loadEndpoints } from './lib/cache.ts';
  import {
    $route as route,
    $routeMeta as routeMeta,
    $startLatLng as startLatLng,
    $endLatLng as endLatLng,
    $startAddress as startAddress,
    $endAddress as endAddress,
  } from './store/route.ts';
  import { fitRoute } from './modules/map.mts';
  // Side-effect import: activates the batched fetch watcher
  import './store/fetch.ts';

  let settingsOpen = false;

  function toggleSettings() {
    settingsOpen = !settingsOpen;
    // Fit the full route on screen when opening settings
    if (settingsOpen) {
      const r = route.get();
      if (r) fitRoute(r);
    }
  }

  onMount(() => {
    startGPS();

    // Restore last session from localStorage
    const cached = loadRoute();
    if (cached) {
      route.set(cached.route);
      routeMeta.set(cached.meta);
    }

    const endpoints = loadEndpoints();
    if (endpoints) {
      if (endpoints.startLatLng) startLatLng.set(endpoints.startLatLng);
      if (endpoints.endLatLng) endLatLng.set(endpoints.endLatLng);
      if (endpoints.startAddress) startAddress.set(endpoints.startAddress);
      if (endpoints.endAddress) endAddress.set(endpoints.endAddress);
    }
  });
</script>

<div class="app">
  <header>
    <SearchInput />
    <button class="settings-btn" on:click={toggleSettings} aria-label="Settings">
      ⚙️
    </button>
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer>
    {#if settingsOpen}
      <SettingsPanel on:close={() => (settingsOpen = false)} />
    {:else}
      <InstructionPanel />
    {/if}
  </footer>

  <OffRoutePrompt />
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
    height: 100dvh;
    background: #0f172a;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    width: 100vw;
  }

  header {
    flex: 0 0 auto;
    z-index: 10;
    display: flex;
    align-items: flex-start;
  }

  .settings-btn {
    background: none;
    border: none;
    font-size: 1.4rem;
    padding: 0.6rem;
    cursor: pointer;
    align-self: center;
    flex-shrink: 0;
  }

  .map-area {
    flex: 1 1 auto;
    position: relative;
    overflow: hidden;
  }

  footer {
    flex: 0 0 auto;
    z-index: 10;
  }
</style>
