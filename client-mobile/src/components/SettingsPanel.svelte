<script lang="ts">
  import { $autoReroute as autoReroute } from '../store/settings.ts';

  export let open = false;
</script>

{#if open}
  <div class="panel">
    <h2 class="title">Settings</h2>
    <label class="row">
      <span>Auto-reroute when off route</span>
      <input
        type="checkbox"
        checked={$autoReroute}
        on:change={(e) => autoReroute.set(e.currentTarget.checked)}
      />
    </label>
  </div>
{/if}

<style>
  .panel {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    background: #1e293b;
    border-radius: 1rem 1rem 0 0;
    padding: 1.25rem 1.5rem 2rem;
    z-index: 90;
    border-top: 1px solid #334155;
  }

  .title { font-size: 1rem; font-weight: 700; color: #f8fafc; margin-bottom: 1rem; }

  .row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0.5rem 0;
    color: #f8fafc;
    font-size: 0.95rem;
    cursor: pointer;
  }

  input[type='checkbox'] { width: 1.25rem; height: 1.25rem; accent-color: #2563eb; }
</style>
