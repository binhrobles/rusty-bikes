<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { createMap, updateRoute, updateGPSMarker, followGPS, fitRoute } from '../modules/map.mts';
  import { $route as route } from '../store/route.ts';
  import { $userPosition as userPosition, $userBearing as userBearing } from '../store/gps.ts';

  let container: HTMLDivElement;
  const unsubs: Array<() => void> = [];

  onMount(() => {
    const map = createMap(container.id);

    unsubs.push(
      route.subscribe((r) => {
        updateRoute(r);
        if (r) fitRoute(r);
      }),
    );

    unsubs.push(
      userPosition.subscribe((pos) => {
        if (!pos) return;
        const { latitude: lat, longitude: lon } = pos.coords;
        updateGPSMarker(lat, lon);
        followGPS(lat, lon, userBearing.get());
      }),
    );

    return () => map.remove();
  });

  onDestroy(() => unsubs.forEach((u) => u()));
</script>

<div id="map" bind:this={container}></div>

<style>
  div {
    width: 100%;
    height: 100%;
  }
</style>
