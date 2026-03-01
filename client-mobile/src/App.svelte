<script lang="ts">
  import { onMount } from 'svelte';
  import MapView from './components/MapView.svelte';
  import InstructionPanel from './components/InstructionPanel.svelte';
  import OffRoutePrompt from './components/OffRoutePrompt.svelte';
  import SearchInput from './components/SearchInput.svelte';
  import CostSlider from './components/CostSlider.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import { startGPS } from './store/gps.ts';

  let settingsOpen = false;

  onMount(() => {
    startGPS();
  });
</script>

<div class="app">
  <header>
    <SearchInput />
    <button class="settings-btn" on:click={() => (settingsOpen = !settingsOpen)} aria-label="Settings">
      ⚙️
    </button>
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer>
    <InstructionPanel />
    <CostSlider />
  </footer>

  <OffRoutePrompt />
  <SettingsPanel bind:open={settingsOpen} />
</div>

<style>
  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
    height: 100dvh;
    background: #0f172a;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    width: 100vw;
  }

  header {
    flex: 0 0 auto;
    z-index: 10;
    display: flex;
    align-items: flex-start;
  }

  .settings-btn {
    background: none;
    border: none;
    font-size: 1.4rem;
    padding: 0.6rem;
    cursor: pointer;
    align-self: center;
    flex-shrink: 0;
  }

  .map-area {
    flex: 1 1 auto;
    position: relative;
    overflow: hidden;
  }

  footer {
    flex: 0 0 auto;
    z-index: 10;
  }
</style>
