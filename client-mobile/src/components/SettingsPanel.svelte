<script lang="ts">
  import { $autoReroute as autoReroute } from '../store/settings.ts';
  import { $comfortSlider as comfortSlider, $salmonToggle as salmonToggle } from '../store/cost.ts';
  import { createEventDispatcher } from 'svelte';

  const dispatch = createEventDispatcher();

</script>

  <div class="panel">
    <div class="header">
      <h2 class="title">Settings</h2>
      <button class="close-btn" on:click={() => dispatch('close')} aria-label="Close settings">&times;</button>
    </div>

    <div class="section-label">Routing</div>
    <label class="slider-row">
      <span>⚡</span>
      <input
        type="range"
        min="0"
        max="1"
        step="0.05"
        value={$comfortSlider}
        on:input={(e) => comfortSlider.set(parseFloat(e.currentTarget.value))}
      />
      <span>🛡</span>
    </label>
    <label class="row">
      <span>Respect traffic direction</span>
      <input
        type="checkbox"
        checked={$salmonToggle}
        on:change={(e) => salmonToggle.set(e.currentTarget.checked)}
      />
    </label>

    <div class="section-label">Navigation</div>
    <label class="row">
      <span>Auto-reroute when off route</span>
      <input
        type="checkbox"
        checked={$autoReroute}
        on:change={(e) => autoReroute.set(e.currentTarget.checked)}
      />
    </label>
  </div>

<style>
  .panel {
    background: #1e293b;
    padding: 1rem 1.25rem 1.25rem;
    border-top: 1px solid #334155;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
  }

  .title { font-size: 1rem; font-weight: 700; color: #f8fafc; }

  .close-btn {
    background: none;
    border: none;
    color: #475569;
    font-size: 1.4rem;
    line-height: 1;
    cursor: pointer;
    padding: 0.2rem;
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

  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    color: #f8fafc;
    font-size: 0.95rem;
    cursor: pointer;
  }

  .slider-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.5rem 0;
    font-size: 1rem;
    color: #94a3b8;
  }

  .slider-row input[type='range'] { flex: 1; accent-color: #2563eb; }

  input[type='checkbox'] { width: 1.25rem; height: 1.25rem; accent-color: #2563eb; }
</style>
