<script lang="ts">
  import {
    $comfortSlider as comfortSlider,
    $speedSlider as speedSlider,
    $hillSlider as hillSlider,
    $salmonSlider as salmonSlider,
  } from '../store/cost.ts';
  import { $route as route } from '../store/route.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { resetNav } from '../store/nav.ts';
  import type { WritableAtom } from 'nanostores';

  const debounce = (store: WritableAtom<number>, parse: (v: string) => number, ms = 400) => {
    let timer: ReturnType<typeof setTimeout>;
    return (e: Event) => {
      clearTimeout(timer);
      timer = setTimeout(() => store.set(parse((e.target as HTMLInputElement).value)), ms);
    };
  };

  function startNavigation() {
    resetNav();
    appView.set('navigating');
  }
</script>

<div class="panel">
  <div class="section-label">Routing</div>
  <label class="slider-row">
    <span class="slider-label">Comfort</span>
    <input type="range" min="0" max="1" step="0.05" value={$comfortSlider}
      on:input={debounce(comfortSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Speed</span>
    <input type="range" min="0" max="1" step="0.05" value={$speedSlider}
      on:input={debounce(speedSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Avoid Hills</span>
    <input type="range" min="0" max="1" step="0.05" value={$hillSlider}
      on:input={debounce(hillSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Rules</span>
    <input type="range" min="0" max="3" step="1" value={$salmonSlider}
      on:input={debounce(salmonSlider, parseInt)} />
  </label>
  <div class="slider-ticks">
    <span>Ignore</span>
    <span>Sometimes</span>
    <span>Mostly</span>
    <span>Always</span>
  </div>

  <button class="go-btn" disabled={!$route} on:click={startNavigation}>
    Go!
  </button>
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

  .slider-label { width: 4rem; flex-shrink: 0; }

  .slider-row input[type='range'] { flex: 1; accent-color: #2563eb; }

  .slider-ticks {
    display: flex;
    justify-content: space-between;
    font-size: 0.7rem;
    color: #64748b;
    padding: 0 0.25rem 0.25rem;
  }

  .go-btn {
    display: block;
    width: 100%;
    margin-top: 1rem;
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
