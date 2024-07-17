<script lang="ts">
  import { onMount } from 'svelte';
  import Icon from './components/Icon.svelte';

  import {
    addLayerControl,
    createMap,
    configureBindings,
  } from './modules/map.mts';
  import {
    addPathfindingControl,
    addLoadingIndicator,
  } from './modules/control.mts';
  import { RUSTY_BASE_URL } from './config.ts';

  let map: L.Map | null;
  const mapAction = (container: string) => {
    map = createMap(container);

    configureBindings(map);
    addPathfindingControl(map);
    addLayerControl(map);
    addLoadingIndicator(map);

    return {
      destroy: () => {
        map?.remove();
        map = null;
      },
    };
  };

  // some loading indicator until lambda is ready to do stuff
  let lambdaReady = false;
  onMount(async () => {
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

<header>
  <div>
    <span class="title">Rusty Bikes <Icon type="bike" /></span>
    <span><i>An urban bike route explorer</i></span>
  </div>
  <a href="https://github.com/binhrobles/rusty-bikes"><Icon type="github" /></a>
</header>

{#if lambdaReady}
  <div id="map" use:mapAction></div>
{:else}
  <span>Waiting on a lambda cold start... <Icon type="robot" /></span>
{/if}

<style>
  header {
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    padding-bottom: 1rem;
    font-size: 0.9rem;
    display: flex;
    justify-content: space-between;
    margin-inline: 1vw;
  }

  .title {
    font-size: 1.125rem;
    line-height: 1.75rem;
    font-weight: 600;
    padding-right: 0.5rem;
  }

  #map {
    height: 90vh;
    width: 98vw;
    margin: 0 auto;
  }
</style>
