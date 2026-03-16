<script lang="ts">
  import {
    $currentInstruction as currentInstruction,
    $upcomingInstructions as upcomingInstructions,
  } from '../store/nav.ts';
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
  }
</script>

<div class="nav-footer">
  {#if $currentInstruction}
    <div class="current">
      <span class="current-arrow">{ARROW[$currentInstruction.direction ?? 'straight'] ?? '↑'}</span>
      <div class="current-info">
        <div class="current-street">{$currentInstruction.wayName || 'Continue'}</div>
        <div class="current-dist">{formatDist($currentInstruction.distance)}</div>
      </div>
    </div>
  {/if}

  <div class="upcoming">
    {#each $upcomingInstructions as instr}
      <div class="turn-row">
        <span class="turn-arrow">{ARROW[instr.direction ?? 'straight'] ?? '↑'}</span>
        <span class="turn-street">{instr.wayName || 'Continue'}</span>
        <span class="turn-dist">{formatDist(instr.distance)}</span>
      </div>
    {:else}
      <div class="turn-row empty">Arriving soon</div>
    {/each}
  </div>

  <div class="footer-bottom">
    {#if $routeMeta}
      <div class="route-meta">
        <span>{formatDist($routeMeta.total_distance)}</span>
        <span class="sep">·</span>
        <span>{formatTime($routeMeta.total_time_estimate)}</span>
      </div>
    {/if}
    <button class="exit-btn" on:click={exitNavigation}>Exit</button>
  </div>
</div>

<style>
  .nav-footer {
    background: #1e293b;
    border-top: 1px solid #334155;
    max-height: 50dvh;
    display: flex;
    flex-direction: column;
  }

  .current {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1.25rem;
    border-bottom: 1px solid #475569;
  }

  .current-arrow {
    font-size: 2rem;
    line-height: 1;
    min-width: 2rem;
    text-align: center;
    color: #f8fafc;
  }

  .current-info { flex: 1; min-width: 0; }

  .current-street {
    font-size: 1.1rem;
    font-weight: 600;
    color: #f8fafc;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .current-dist { font-size: 0.85rem; color: #94a3b8; margin-top: 0.1rem; }

  .upcoming {
    flex: 1;
    overflow-y: auto;
    padding: 0 1.25rem;
  }

  .turn-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0;
    color: #cbd5e1;
    font-size: 0.85rem;
    border-bottom: 1px solid #334155;
  }

  .turn-row:last-child { border-bottom: none; }

  .turn-row.empty {
    color: #94a3b8;
    justify-content: center;
    font-style: italic;
  }

  .turn-arrow {
    font-size: 1.1rem;
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
    font-size: 0.75rem;
    color: #94a3b8;
    white-space: nowrap;
  }

  .footer-bottom {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.5rem 1.25rem;
    border-top: 1px solid #334155;
  }

  .route-meta {
    display: flex;
    align-items: center;
    gap: 0.4rem;
    font-size: 0.8rem;
    color: #94a3b8;
    flex: 1;
  }

  .sep { color: #475569; }

  .exit-btn {
    padding: 0.5rem 1.25rem;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-size: 0.9rem;
    font-weight: 600;
    cursor: pointer;
    white-space: nowrap;
  }
</style>
