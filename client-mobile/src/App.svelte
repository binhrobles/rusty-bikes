<script lang="ts">
  import { onMount } from 'svelte';
  import MapView from './components/MapView.svelte';
  import SearchInput from './components/SearchInput.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import NavigationFooter from './components/NavigationFooter.svelte';
  import { startGPS } from './store/gps.ts';
  import { $appView as appView } from './store/settings.ts';
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

  onMount(async () => {
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

    // Ping lambda
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

  // When switching back to planning, resize map and fit route
  appView.listen(async (view) => {
    if (view === 'planning') {
      await tick();
      resizeMap();
      const r = route.get();
      if (r) fitRoute(r);
    }
  });
</script>

<div class="app">
  <header>
    {#if $appView === 'planning'}
      {#if lambdaReady}
        <SearchInput />
      {:else}
        <div class="connecting">Connecting...</div>
      {/if}
    {/if}
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer>
    {#if $appView === 'planning'}
      <SettingsPanel />
    {:else}
      <NavigationFooter />
    {/if}
  </footer>
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
