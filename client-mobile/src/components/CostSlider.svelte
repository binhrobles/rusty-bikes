<script lang="ts">
  import { $comfortSlider as comfortSlider, $salmonToggle as salmonToggle } from '../store/cost.ts';
</script>

<div class="cost-controls">
  <label class="slider-label">
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

  <label class="toggle-label">
    <input
      type="checkbox"
      checked={$salmonToggle}
      on:change={(e) => salmonToggle.set(e.currentTarget.checked)}
    />
    <span>Respect traffic direction</span>
  </label>
</div>

<style>
  .cost-controls {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
    padding: 0.6rem 1rem;
    background: #0f172a;
    border-top: 1px solid #334155;
  }

  .slider-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 1rem;
    color: #94a3b8;
  }

  .slider-label input[type='range'] { flex: 1; accent-color: #2563eb; }

  .toggle-label {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: #94a3b8;
    cursor: pointer;
  }

  input[type='checkbox'] { accent-color: #2563eb; }
</style>
