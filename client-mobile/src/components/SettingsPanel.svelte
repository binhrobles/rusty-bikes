<script lang="ts">
  import {
    $routePriority as routePriority,
    $hillPenalty as hillPenalty,
    $salmonPenalty as salmonPenalty,
    $avoidMajorRoads as avoidMajorRoads,
  } from '../store/cost.ts';
  import { $route as route, $startLatLng as startLatLng, $endLatLng as endLatLng, $startAddress as startAddress, $endAddress as endAddress, $corridor as corridor, $routeMeta as routeMeta } from '../store/route.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { resetNav, getRouteStepBearing } from '../store/nav.ts';
  import { removeEndMarker, enterNavMode } from '../modules/map.mts';
  import { $userPosition as userPosition, $userBearing as userBearing } from '../store/gps.ts';
  import { clearRoute } from '../lib/cache.ts';

  const HILL_LABELS = ['None', 'Standard', 'Strong'] as const;
  const SALMON_LABELS = ['Never', 'Sometimes', 'Often'] as const;

  let debounceTimer: ReturnType<typeof setTimeout>;
  function debouncedSet(value: number) {
    clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => routePriority.set(value), 400);
  }

  function startNavigation() {
    resetNav();
    appView.set('navigating');

    // Immediately tilt + zoom to user's location
    const pos = userPosition.get();
    if (pos) {
      const { latitude, longitude } = pos.coords;
      const bearing = userBearing.get() || getRouteStepBearing();
      enterNavMode(latitude, longitude, bearing);
    }
  }

  function clearAll() {
    startLatLng.set(null);
    endLatLng.set(null);
    startAddress.set('');
    endAddress.set('');
    route.set(null);
    routeMeta.set(null);
    corridor.set(null);
    resetNav();
    removeEndMarker();
    clearRoute();
  }
</script>

<div class="panel">
  <div class="section-label">Route Priority</div>
  <label class="slider-row">
    <span class="slider-end">Comfort</span>
    <input type="range" min="0" max="1" step="0.05" value={1 - $routePriority}
      on:input={(e) => debouncedSet(1 - parseFloat(e.currentTarget.value))} />
    <span class="slider-end">Speed</span>
  </label>

  <div class="section-label">Hill Penalty</div>
  <div class="segmented-row">
    {#each HILL_LABELS as label, i}
      <button
        class="seg-btn"
        class:active={$hillPenalty === i}
        on:click={() => hillPenalty.set(i)}
      >{label}</button>
    {/each}
  </div>

  <div class="section-label">Rule Breaker</div>
  <div class="segmented-row">
    {#each SALMON_LABELS as label, i}
      <button
        class="seg-btn"
        class:active={$salmonPenalty === 2 - i}
        on:click={() => salmonPenalty.set(2 - i)}
      >{label}</button>
    {/each}
  </div>

  <div class="section-label">Major Roads</div>
  <div class="segmented-row">
    <button class="seg-btn" class:active={$avoidMajorRoads}
      on:click={() => avoidMajorRoads.set(true)}>Avoid</button>
    <button class="seg-btn" class:active={!$avoidMajorRoads}
      on:click={() => avoidMajorRoads.set(false)}>Allow</button>
  </div>

  <div class="action-row">
    <button class="clear-btn" on:click={clearAll}>Clear</button>
    <button class="go-btn" disabled={!$route} on:click={startNavigation}>
      Go!
    </button>
  </div>
</div>

<style>
  .panel {
    background: #1e293b;
    padding: 1rem 1.25rem 1.25rem;
    border-top: 1px solid #334155;
  }

  .section-label {
    font-size: 0.75rem;
    font-weight: 600;
    color: #64748b;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-top: 1rem;
    margin-bottom: 0.25rem;
  }

  .section-label:first-of-type { margin-top: 0; }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0;
    font-size: 0.95rem;
    color: #f8fafc;
  }

  .slider-end {
    font-size: 0.8rem;
    color: #94a3b8;
    flex-shrink: 0;
  }

  .slider-row input[type='range'] { flex: 1; accent-color: #2563eb; }

  .segmented-row {
    display: flex;
    gap: 0;
    border-radius: 0.5rem;
    overflow: hidden;
    border: 1px solid #334155;
  }

  .seg-btn {
    flex: 1;
    padding: 0.5rem 0;
    background: transparent;
    color: #94a3b8;
    border: none;
    border-right: 1px solid #334155;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .seg-btn:last-child { border-right: none; }

  .seg-btn.active {
    background: #2563eb;
    color: #fff;
    font-weight: 600;
  }


  .action-row {
    display: flex;
    gap: 0.5rem;
    margin-top: 1rem;
  }

  .clear-btn {
    padding: 0.75rem 1rem;
    background: #334155;
    color: #94a3b8;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
  }

  .go-btn {
    flex: 1;
    padding: 0.75rem;
    background: #2563eb;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-size: 1.1rem;
    font-weight: 700;
    cursor: pointer;
    letter-spacing: 0.02em;
  }

  .go-btn:disabled {
    background: #334155;
    color: #64748b;
    cursor: not-allowed;
  }
</style>
