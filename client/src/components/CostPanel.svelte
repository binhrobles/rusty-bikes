<script lang="ts">
  import { CostDefaults, HtmlElementId } from '../consts.ts';
  import {
    $heuristicWeight as hw,
    $coefficients as cd,
    updateCycleway,
    updateRoad,
    updateSalmon,
  } from '../store/cost.ts';

  // one way binding to init value...two way binding using update helpers
  // did this because I wanted to keep $coefficients as an object
  const { cycleway_coefficient, road_coefficient, salmon_coefficient } = cd.get();
</script>

<h4>Pathfinding Configuration</h4>
<br />

<span>A* Heuristic Greediness</span>
<br />
<input
  class="slider"
  id={HtmlElementId.HeuristicWeightRange}
  type="range"
  min="0.3"
  max="1.5"
  step="0.1"
  bind:value={$hw}>
<br />

<span>Prefer Bike Lanes:</span>
<br />
<input
  class="slider"
  id={HtmlElementId.CyclewayCoefficientRange}
  type="range"
  min="0"
  max="1"
  step="0.1"
  on:change={updateCycleway}
  value={cycleway_coefficient}>
<br />

<span>Prefer Quiet Streets:</span>
<br />
<input
  class="slider"
  id={HtmlElementId.RoadCoefficientRange}
  type="range"
  min="0"
  max="1"
  step="0.1"
  on:change={updateRoad}
  value={road_coefficient}>
<br />

<span>I follow rules:</span>
<br />
<input
  class="slider"
  id={HtmlElementId.SalmonCoefficientRange}
  type="range"
  min="1"
  max="2"
  step="0.1"
  on:change={updateSalmon}
  value={salmon_coefficient}>
