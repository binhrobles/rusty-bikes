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
    height: 88vh;
    width: 100%;
  }

  .title {
    font-size: 1.125rem;
    line-height: 1.75rem;
    font-weight: 600;
    padding-right: 0.5rem;
  }

  header {
    font-family: "Helvetica Neue", Helvetica, Arial, sans-serif;
    padding-bottom: 0.5rem;
    font-size: 0.9rem;
  }

  footer {
    position: absolute;
    bottom: 0;
    width: 100%;
    display: flex;
    justify-content: center;
  }
</style>

<header>
  <span class="title">Rusty Bikes 🚲</span>
  <span><i>An interactive urban bike route explorer</i></span>
</header>

{#if lambdaReady}
<div id="map" use:mapAction></div>
{:else}
<span>Waiting on a lambda cold start... 🤖</span>
{/if}
<footer>
  <a href="https://github.com/binhrobles/rusty-bikes"><svg fill="none" stroke="#000000" stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" shape-rendering="geometricPrecision" viewBox="0 0 24 24" height="24" width="24" style="color: currentcolor;"><path d="M9 19c-5 1.5-5-2.5-7-3m14 6v-3.87a3.37 3.37 0 00-.94-2.61c3.14-.35 6.44-1.54 6.44-7A5.44 5.44 0 0020 4.77 5.07 5.07 0 0019.91 1S18.73.65 16 2.48a13.38 13.38 0 00-7 0C6.27.65 5.09 1 5.09 1A5.07 5.07 0 005 4.77a5.44 5.44 0 00-1.5 3.78c0 5.42 3.3 6.61 6.44 7A3.37 3.37 0 009 18.13V22"></path></svg></a>
</footer>
