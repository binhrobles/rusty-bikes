<script lang="ts">
  import MapView from './components/MapView.svelte';
  import InstructionPanel from './components/InstructionPanel.svelte';
  import OffRoutePrompt from './components/OffRoutePrompt.svelte';
  import SearchInput from './components/SearchInput.svelte';
</script>

<div class="app">
  <header class="search-bar">
    <SearchInput />
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer class="instruction-bar">
    <InstructionPanel />
  </footer>

  <OffRoutePrompt />
</div>

<style>
  :global(*) {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
  }

  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
    height: 100dvh;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    width: 100vw;
  }

  .search-bar {
    flex: 0 0 auto;
    z-index: 10;
  }

  .map-area {
    flex: 1 1 auto;
    position: relative;
    overflow: hidden;
  }

  .instruction-bar {
    flex: 0 0 auto;
    z-index: 10;
    max-height: 40dvh;
    overflow-y: auto;
  }
</style>
