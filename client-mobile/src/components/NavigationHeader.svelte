<script lang="ts">
  import { $currentInstruction as currentInstruction } from '../store/nav.ts';

  const ARROW: Record<string, string> = {
    left: '↰', right: '↱', straight: '↑', uturn: '↶',
  };

  function formatDist(m: number) {
    return m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`;
  }
</script>

<div class="nav-header">
  {#if $currentInstruction}
    <span class="arrow">{ARROW[$currentInstruction.direction ?? 'straight'] ?? '↑'}</span>
    <div class="info">
      <div class="street">{$currentInstruction.wayName || 'Continue'}</div>
      <div class="dist">{formatDist($currentInstruction.distance)}</div>
    </div>
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
    width: 100%;
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
</style>
