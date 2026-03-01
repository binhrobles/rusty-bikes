<script lang="ts">
  import {
    $offRoutePromptVisible as offRoutePromptVisible,
    $distanceOffRoute as distanceOffRoute,
  } from '../store/nav.ts';
  import { $userPosition as userPosition } from '../store/gps.ts';
  import { $startLatLng as startLatLng } from '../store/route.ts';
  import { $autoReroute as autoReroute } from '../store/settings.ts';

  function dismiss() {
    offRoutePromptVisible.set(false);
  }

  function recalculate() {
    const pos = userPosition.get();
    if (pos) {
      startLatLng.set([pos.coords.latitude, pos.coords.longitude]);
    }
    offRoutePromptVisible.set(false);
  }

  // Silent auto-reroute when setting is on
  offRoutePromptVisible.listen((visible) => {
    if (visible && autoReroute.get()) recalculate();
  });
</script>

{#if $offRoutePromptVisible && !$autoReroute}
  <div class="backdrop" role="dialog" aria-modal="true" aria-label="Off route">
    <div class="prompt">
      <p class="title">You're off route</p>
      <p class="sub">{Math.round($distanceOffRoute)} m from the route</p>
      <div class="actions">
        <button class="btn btn--secondary" on:click={dismiss}>Keep going</button>
        <button class="btn btn--primary" on:click={recalculate}>Recalculate</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: flex-end;
    z-index: 100;
  }

  .prompt {
    width: 100%;
    background: #1e293b;
    color: #f8fafc;
    border-radius: 1rem 1rem 0 0;
    padding: 1.5rem 1.5rem 2rem;
  }

  .title { font-size: 1.2rem; font-weight: 700; margin-bottom: 0.25rem; }
  .sub { font-size: 0.9rem; color: #94a3b8; margin-bottom: 1.25rem; }

  .actions { display: flex; gap: 0.75rem; }

  .btn {
    flex: 1;
    padding: 0.75rem;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn--secondary { background: #334155; color: #f8fafc; }
  .btn--primary { background: #2563eb; color: #fff; }
</style>
