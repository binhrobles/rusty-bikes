<script lang="ts">
  import { afterUpdate } from 'svelte';
  import { $currentInstruction as currentInstruction, $nextInstruction as nextInstruction } from '../store/nav.ts';
  import { $routeMeta as routeMeta } from '../store/route.ts';

  let listEl: HTMLElement;

  afterUpdate(() => {
    listEl?.querySelector<HTMLElement>('.active')?.scrollIntoView({ block: 'nearest', behavior: 'smooth' });
  });

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
</script>

<div bind:this={listEl}>
  {#if $currentInstruction}
    <div class="hud">
      <span class="arrow">{ARROW[$currentInstruction.direction ?? 'straight'] ?? '↑'}</span>
      <div class="hud-text">
        <div class="street">{$currentInstruction.wayName || 'Continue'}</div>
        <div class="dist">{formatDist($currentInstruction.distance)}</div>
      </div>
      {#if $routeMeta}
        <div class="meta">
          <span>{formatDist($routeMeta.total_distance)}</span>
          <span>{formatTime($routeMeta.total_time_estimate)}</span>
        </div>
      {/if}
    </div>
    {#if $nextInstruction}
      <div class="then">
        Then: {ARROW[$nextInstruction.direction ?? 'straight'] ?? '↑'}
        {$nextInstruction.wayName || 'Continue'}
      </div>
    {/if}
  {:else}
    <div class="hud hud--empty">
      <span>Set a destination to start navigation</span>
    </div>
  {/if}
</div>

<style>
  .hud {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0.75rem 1rem;
    background: #1e293b;
    color: #f8fafc;
    min-height: 4rem;
  }

  .hud--empty {
    justify-content: center;
    color: #94a3b8;
    font-size: 0.9rem;
  }

  .arrow {
    font-size: 2.5rem;
    line-height: 1;
    min-width: 2.5rem;
    text-align: center;
  }

  .hud-text { flex: 1; min-width: 0; }

  .street {
    font-size: 1.1rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dist { font-size: 0.85rem; color: #94a3b8; margin-top: 0.1rem; }

  .meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 0.8rem;
    color: #94a3b8;
    gap: 0.1rem;
    white-space: nowrap;
  }

  .then {
    padding: 0.35rem 1rem;
    background: #0f172a;
    color: #94a3b8;
    font-size: 0.85rem;
    border-top: 1px solid #334155;
  }
</style>
