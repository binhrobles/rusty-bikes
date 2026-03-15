<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { createMap, updateRoute, updateCorridor, updateGPSMarker, followGPSNavMode, fitRoute, updateEndMarker, setEndMarkerDragHandler } from '../modules/map.mts';
  import Radar from 'radar-sdk-js';
  import { $route as route, $corridor as corridor, $endLatLng as endLatLng, $endAddress as endAddress } from '../store/route.ts';
  import { $userPosition as userPosition, $userBearing as userBearing } from '../store/gps.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { getRouteStepBearing } from '../store/nav.ts';

  let container: HTMLDivElement;
  const unsubs: Array<() => void> = [];

  onMount(() => {
    const map = createMap(container.id);

    unsubs.push(
      corridor.subscribe((c) => updateCorridor(c)),
    );

    unsubs.push(
      route.subscribe((r) => {
        updateRoute(r);
        // Only auto-fit in planning mode
        if (r && appView.get() === 'planning') fitRoute(r);
      }),
    );

    // Draggable destination marker
    setEndMarkerDragHandler((lat, lon) => {
      endLatLng.set([lat, lon]);
      endAddress.set('Dropped pin');
      Radar.reverseGeocode({ latitude: lat, longitude: lon })
        .then((res) => {
          const addr = res.addresses?.[0];
          if (addr) {
            endAddress.set(addr.formattedAddress ?? addr.street ?? 'Dropped pin');
          }
        })
        .catch(() => {});
    });

    unsubs.push(
      endLatLng.subscribe((coords) => {
        if (coords) updateEndMarker(coords[0], coords[1]);
      }),
    );

    unsubs.push(
      userPosition.subscribe((pos) => {
        if (!pos) return;
        const { latitude: lat, longitude: lon } = pos.coords;
        updateGPSMarker(lat, lon);
        // Only auto-follow camera in navigating mode
        if (appView.get() === 'navigating') {
          // Use GPS bearing when moving, fall back to route step bearing when stationary
          const bearing = userBearing.get() || getRouteStepBearing();
          followGPSNavMode(lat, lon, bearing);
        }
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
