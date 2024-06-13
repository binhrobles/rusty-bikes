<script lang="ts">
  import { onMount } from 'svelte';

  import { createMap, configureBindings } from './modules/map.mts';
  import mode_control from './modules/mode_control.mts';
  import cost_control from './modules/cost_control.mts';
  import { RUSTY_BASE_URL } from './config.ts';

  let map;
  const mapAction = (container) => {
    map = createMap(container);
    configureBindings(map);

    mode_control.render(map);
    cost_control.render(map);

    return {
      destroy: () => {
        map.remove();
        map = null;
      }
    };
  }

  let lambdaReady = false;
  onMount(async () => {
    try {
      const res = await fetch(`${RUSTY_BASE_URL}/ping`);
      lambdaReady = true;
    } catch (e) {
      console.error(e);
    }
  });
</script>

<style>
  #map {
    height: 90vh;
    width: 100vw;
  }
</style>

<header>
  <span>⚠️  Very beta ⚠️  </span>
  <span>Follow dev </span> <a href="https://github.com/binhrobles/rusty-bikes">here</a>
</header>
<br>
{#if lambdaReady}
<div id="map" use:mapAction></div>
{:else}
<span>Loading...</span>
{/if}
