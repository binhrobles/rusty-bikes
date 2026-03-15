<script lang="ts">
  import { $upcomingInstructions as upcomingInstructions } from '../store/nav.ts';
  import { $routeMeta as routeMeta } from '../store/route.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { resetCamera } from '../modules/map.mts';

  const ARROW: Record<string, string> = {
    left: '↰', right: '↱', straight: '↑', uturn: '↶',
  };

  function formatDist(m: number) {
    return m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`;
  }

  function formatTime(s: number) {
    const mins = Math.round(s / 60);
    return mins < 60 ? `${mins} min` : `${Math.floor(mins / 60)}h ${mins % 60}m`;
  }

  function exitNavigation() {
    appView.set('planning');
    resetCamera();
    // App.svelte's appView.listen() handles resizeMap + fitRoute
  }
</script>

<div class="nav-footer">
  <div class="upcoming">
    {#each $upcomingInstructions as instr, i}
      <div class="turn-row" class:first={i === 0}>
        <span class="turn-arrow">{ARROW[instr.direction ?? 'straight'] ?? '↑'}</span>
        <span class="turn-street">{instr.wayName || 'Continue'}</span>
        <span class="turn-dist">{formatDist(instr.distance)}</span>
      </div>
    {:else}
      <div class="turn-row empty">Arriving soon</div>
    {/each}
  </div>
  {#if $routeMeta}
    <div class="route-meta">
      <span>{formatDist($routeMeta.total_distance)}</span>
      <span class="sep">·</span>
      <span>{formatTime($routeMeta.total_time_estimate)}</span>
    </div>
  {/if}
  <button class="exit-btn" on:click={exitNavigation}>Exit</button>
</div>

<style>
  .nav-footer {
    background: #1e293b;
    border-top: 1px solid #334155;
    padding: 0.5rem 0;
  }

  .upcoming {
    padding: 0 1.25rem;
  }

  .turn-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0;
    color: #cbd5e1;
    font-size: 0.9rem;
    border-bottom: 1px solid #334155;
  }

  .turn-row.first {
    color: #f8fafc;
    font-weight: 500;
  }

  .turn-row.empty {
    color: #94a3b8;
    justify-content: center;
    font-style: italic;
  }

  .turn-row:last-child { border-bottom: none; }

  .turn-arrow {
    font-size: 1.3rem;
    min-width: 1.5rem;
    text-align: center;
  }

  .turn-street {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .turn-dist {
    font-size: 0.8rem;
    color: #94a3b8;
    white-space: nowrap;
  }

  .route-meta {
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.4rem;
    padding: 0.5rem 1.25rem;
    font-size: 0.8rem;
    color: #94a3b8;
    border-top: 1px solid #334155;
  }

  .sep { color: #475569; }

  .exit-btn {
    display: block;
    width: calc(100% - 2.5rem);
    margin: 0.75rem 1.25rem;
    padding: 0.7rem;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
