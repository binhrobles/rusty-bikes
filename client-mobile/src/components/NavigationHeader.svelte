<script lang="ts">
  import { $currentInstruction as currentInstruction } from '../store/nav.ts';
  import { $routeMeta as routeMeta } from '../store/route.ts';

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

<div class="nav-header">
  {#if $currentInstruction}
    <span class="arrow">{ARROW[$currentInstruction.direction ?? 'straight'] ?? '↑'}</span>
    <div class="info">
      <div class="street">{$currentInstruction.wayName || 'Continue'}</div>
      <div class="dist">{formatDist($currentInstruction.distance)}</div>
    </div>
    {#if $routeMeta}
      <div class="meta">
        <span>{formatDist($routeMeta.total_distance)}</span>
        <span>{formatTime($routeMeta.total_time_estimate)}</span>
      </div>
    {/if}
  {/if}
</div>

<style>
  .nav-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    background: #1e293b;
    color: #f8fafc;
    min-height: 4rem;
    border-bottom: 1px solid #334155;
  }

  .arrow {
    font-size: 2.5rem;
    line-height: 1;
    min-width: 2.5rem;
    text-align: center;
  }

  .info { flex: 1; min-width: 0; }

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
</style>
