<script lang="ts">
  import { createMap, configureBindings } from './modules/map.mts';
  import mode_control from './modules/mode_control.mts';
  import cost_control from './modules/cost_control.mts';

  let map;
  const mapAction = (container) => {
    map = createMap(container);
    configureBindings(map);

    mode_control.render(map);
    cost_control.render(map);

    return {
      destroy: () => {
        map.remove();
        map = null;
      }
    };
  }
</script>

<style>
  #map {
    height: 90vh;
    width: 100vw;
  }
</style>

<header>
  <span>⚠️  Very beta ⚠️  </span>
  <span>Follow dev </span> <a href="https://github.com/binhrobles/rusty-bikes">here</a>
</header>
<br>
<div id="map" use:mapAction></div>
