<script lang="ts">
  import { onDestroy } from 'svelte';
  import { $elevationProfile as profileStore } from '../store/elevation.ts';

  let hasData = false;
  let totalGain = 0;
  let totalLoss = 0;
  let linePath = '';
  let areaPath = '';
  let yTicks: {label: string; y: number}[] = [];

  const WIDTH = 280;
  const HEIGHT = 80;
  const PADDING = { top: 4, right: 8, bottom: 16, left: 28 };
  const plotW = WIDTH - PADDING.left - PADDING.right;
  const plotH = HEIGHT - PADDING.top - PADDING.bottom;

  const unsub = profileStore.subscribe((profile) => {
    hasData = profile.hasData;
    totalGain = profile.totalGain;
    totalLoss = profile.totalLoss;

    if (!profile.hasData) {
      linePath = '';
      areaPath = '';
      yTicks = [];
      return;
    }

    const { elevations, totalDistance } = profile;
    const dist = totalDistance || 1;
    const minE = Math.min(...elevations.map((e) => e.elevation));
    const maxE = Math.max(...elevations.map((e) => e.elevation));
    const range = Math.max(maxE - minE, 1);

    const xPos = (d: number) => PADDING.left + (d / dist) * plotW;
    const yPos = (e: number) => PADDING.top + plotH - ((e - minE) / range) * plotH;

    linePath = elevations
      .map((e, i) => `${i === 0 ? 'M' : 'L'}${xPos(e.distance).toFixed(1)},${yPos(e.elevation).toFixed(1)}`)
      .join(' ');
    areaPath = linePath +
      ` L${xPos(dist).toFixed(1)},${yPos(minE).toFixed(1)}` +
      ` L${xPos(0).toFixed(1)},${yPos(minE).toFixed(1)} Z`;

    yTicks = [minE, minE + range / 2, maxE].map((v) => ({
      label: `${Math.round(v)}`,
      y: yPos(v),
    }));
  });
  onDestroy(unsub);
</script>

{#if hasData}
  <div class="elevation-profile">
    <div class="header">
      <span class="stat up">+{totalGain}m</span>
      <span class="stat down">-{totalLoss}m</span>
    </div>
    <svg width={WIDTH} height={HEIGHT} viewBox="0 0 {WIDTH} {HEIGHT}">
      <line
        x1={PADDING.left} y1={PADDING.top + plotH}
        x2={PADDING.left + plotW} y2={PADDING.top + plotH}
        stroke="#ccc" stroke-width="0.5"
      />
      {#each yTicks as tick}
        <text x={PADDING.left - 3} y={tick.y + 3} class="tick">{tick.label}</text>
      {/each}
      <path d={areaPath} fill="rgba(105, 145, 255, 0.2)" />
      <path d={linePath} fill="none" stroke="#6991ff" stroke-width="1.5" />
    </svg>
  </div>
{/if}

<style>
  .elevation-profile {
    background: white;
    border-radius: 8px;
    box-shadow: 0 0 10px rgba(0, 0, 0, 0.15);
    padding: 6px 8px 4px;
    font-family: 'Helvetica Neue', Arial, sans-serif;
    font-size: 11px;
  }

  .header {
    display: flex;
    gap: 8px;
    margin-bottom: 2px;
    font-weight: 600;
  }

  .stat.up { color: #d44; }
  .stat.down { color: #4a4; }

  .tick {
    font-size: 9px;
    fill: #888;
    text-anchor: end;
  }
</style>
