<script lang="ts">
  import { onMount } from 'svelte';
  import MapView from './components/MapView.svelte';
  import InstructionPanel from './components/InstructionPanel.svelte';
  import OffRoutePrompt from './components/OffRoutePrompt.svelte';
  import SearchInput from './components/SearchInput.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import { startGPS } from './store/gps.ts';
  import { $settingsOpen as settingsOpen } from './store/settings.ts';
  import { loadRoute, loadEndpoints } from './lib/cache.ts';
  import {
    $route as route,
    $routeMeta as routeMeta,
    $startLatLng as startLatLng,
    $endLatLng as endLatLng,
    $startAddress as startAddress,
    $endAddress as endAddress,
  } from './store/route.ts';
  import { fitRoute, resizeMap } from './modules/map.mts';
  // Side-effect import: activates the batched fetch watcher
  import './store/fetch.ts';
  import { tick } from 'svelte';
  import { RUSTY_BASE_URL } from './lib/config.ts';

  let lambdaReady = false;
  let hasCachedRoute = false;

  async function toggleSettings() {
    settingsOpen.set(!settingsOpen.get());
    // Wait for footer to resize, then tell MapLibre before fitting route
    if ($settingsOpen) {
      await tick();
      resizeMap();
      const r = route.get();
      if (r) fitRoute(r);
    }
  }

  onMount(async () => {
    startGPS();

    // Restore last session from localStorage immediately
    const cached = loadRoute();
    if (cached) {
      hasCachedRoute = true;
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

    // Ping lambda — if no cached route, user can't do anything until it's ready
    let retries = 0;
    while (!lambdaReady && retries < 10) {
      try {
        await fetch(`${RUSTY_BASE_URL}/ping`);
        lambdaReady = true;
      } catch (e) {
        retries++;
        console.error(`received ${e} from /ping`);
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    }
  });
</script>

<div class="app">
  <header>
    {#if lambdaReady}
      <SearchInput />
      <button class="settings-btn" on:click={toggleSettings} aria-label="Settings">
        ⚙️
      </button>
    {:else}
      <div class="connecting">Connecting...</div>
    {/if}
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer>
    {#if $settingsOpen}
      <SettingsPanel on:close={() => settingsOpen.set(false)} />
    {:else}
      <InstructionPanel />
    {/if}
  </footer>

  <OffRoutePrompt />
</div>

<style>
  .connecting {
    padding: 0.75rem 1rem;
    color: #94a3b8;
    font-size: 0.9rem;
  }

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
