<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { createMap, updateRoute, updateCorridor, updateGPSMarker, followGPSNavMode, fitRoute, updateEndMarker, setEndMarkerDragHandler, centerOn } from '../modules/map.mts';
  import Radar from 'radar-sdk-js';
  import { $route as route, $corridor as corridor, $routeMeta as routeMeta, $endLatLng as endLatLng, $endAddress as endAddress } from '../store/route.ts';
  import { $userPosition as userPosition, $userBearing as userBearing, startGPS } from '../store/gps.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { getRouteStepBearing } from '../store/nav.ts';

  // Track whether we've already centered on the user this planning session
  let hasCenteredOnUser = false;
  // Track whether we've already fit the route bounds (avoid re-fitting on settings changes)
  let hasFitRoute = false;

  // Reset flags when route is cleared (so next GPS fix re-centers, next route re-fits)
  $: if (!$route) { hasCenteredOnUser = false; hasFitRoute = false; }

  let container: HTMLDivElement;
  const unsubs: Array<() => void> = [];

  onMount(() => {
    const map = createMap(container.id);

    // Start GPS early so we can center on user
    startGPS();

    unsubs.push(
      corridor.subscribe((c) => updateCorridor(c)),
    );

    unsubs.push(
      route.subscribe((r) => {
        updateRoute(r);
        // Fit route bounds once when a new route first appears, not on every settings tweak
        if (r && appView.get() === 'planning' && !hasFitRoute) {
          fitRoute(r);
          hasFitRoute = true;
        }
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

        if (appView.get() === 'navigating') {
          // Use GPS bearing when moving, fall back to route step bearing when stationary
          const bearing = userBearing.get() || getRouteStepBearing();
          followGPSNavMode(lat, lon, bearing);
        } else if (!route.get() && !hasCenteredOnUser) {
          // No route: center map on user's location once
          centerOn(lat, lon);
          hasCenteredOnUser = true;
        }
      }),
    );

    return () => map.remove();
  });

  onDestroy(() => unsubs.forEach((u) => u()));
</script>

<div id="map" bind:this={container}></div>

{#if $routeMeta}
  <div class="route-info">
    <span class="distance">{($routeMeta.total_distance / 1000).toFixed(1)} km</span>
    <span class="sep">&middot;</span>
    <span class="time">{Math.round($routeMeta.total_time_estimate / 60)} min</span>
  </div>
{/if}

<style>
  div#map {
    width: 100%;
    height: 100%;
  }

  .route-info {
    position: absolute;
    bottom: 2.5rem;
    right: 0.7rem;
    background: rgba(15, 23, 42, 0.85);
    backdrop-filter: blur(4px);
    color: #f1f5f9;
    padding: 0.4rem 0.75rem;
    border-radius: 0.5rem;
    font-size: 0.85rem;
    font-weight: 600;
    pointer-events: none;
    z-index: 1;
  }

  .sep {
    color: #64748b;
    margin: 0 0.25rem;
  }
</style>
