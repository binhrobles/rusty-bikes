<script lang="ts">
  import { HtmlElementId } from '../consts.ts';
  import {
    $startMarkerLatLng as startLatLng,
    $endMarkerLatLng as endLatLng,
    $selectedInput as selected,
    $withTraversal as wt,
  } from '../store/route.ts';
  import {
    $heuristicWeight as hw,
    $coefficients as cd,
    updateCycleway,
    updateRoad,
    updateSalmon,
  } from '../store/cost.ts';

  // one way binding to init value...two way binding using update helpers
  // did this because I wanted to keep $coefficients as an object
  const { cycleway_coefficient, road_coefficient, salmon_coefficient } =
    cd.get();

  // when the Routing start / end inputs are clicked,
  // queue them up to be changed on the next map click
  function onClick(event) {
    selected.set(event.target.id);
  }

  let start;
  let end;

  // start / end inputs should reflect the latlng of the markers
  startLatLng.subscribe(
    (s, _) =>
      (start = s?.lng
        ? `(${s.lng.toFixed(5)}, ${s.lat.toFixed(5)})`
        : 'Click to place start point')
  );
  endLatLng.subscribe(
    (e, _) =>
      (end = e?.lng
        ? `(${e.lng.toFixed(5)}, ${e.lat.toFixed(5)})`
        : 'Click to place end point')
  );
</script>

<div class="control">
  <table class="route-table">
    <tr>
      <td><label for={HtmlElementId.StartInput}>Start:</label></td>
      <td
        ><input
          type="text"
          id={HtmlElementId.StartInput}
          on:click={onClick}
          value={start}
        /></td
      >
    </tr>
    <tr>
      <td><label for={HtmlElementId.EndInput}>End:</label></td>
      <td
        ><input
          type="text"
          id={HtmlElementId.EndInput}
          on:click={onClick}
          value={end}
        /></td
      >
    </tr>
    <tr>
      <td><label for="with-traversal">Render Pathfinding?</label></td>
      <td
        ><input
          type="checkbox"
          id="with-traversal"
          name="with-traversal"
          bind:checked={$wt}
        /></td
      >
    </tr>
  </table>

  <hr />

  <details>
    <summary>Customize pathfinding algorithm</summary>
    <br />
    <div class="tooltip">
      Algorithm Greediness:
      <div class="tooltip-text">
        <i
          >A less greedy algorithm will explore more routes before making its
          decision <b>based on the given config<b></b></b></i
        >
      </div>
    </div>
    <input
      class="slider"
      id={HtmlElementId.HeuristicWeightRange}
      type="range"
      min="0.3"
      max="1.5"
      step="0.1"
      bind:value={$hw}
    />
    <br />

    <div class="tooltip">
      Prefer Bike Lanes:
      <span class="tooltip-text"
        ><i
          >Will prefer protected and dedicated bikes lanes over sharrows or
          shared roads</i
        ></span
      >
    </div>
    <input
      class="slider"
      id={HtmlElementId.CyclewayCoefficientRange}
      type="range"
      min="0"
      max="1"
      step="0.1"
      on:change={updateCycleway}
      value={cycleway_coefficient}
    />
    <br />

    <div class="tooltip">
      Prefer Quiet Streets:
      <span class="tooltip-text"
        ><i
          >Will prefer dedicated bike infra and residential roads over secondary
          and major roads</i
        ></span
      >
    </div>
    <input
      class="slider"
      id={HtmlElementId.RoadCoefficientRange}
      type="range"
      min="0"
      max="1"
      step="0.1"
      on:change={updateRoad}
      value={road_coefficient}
    />
    <br />

    <div class="tooltip">
      I follow rules:
      <span class="tooltip-text"
        ><i>Will greatly penalize going the opposite way on a one-way road</i
        ></span
      >
    </div>
    <input
      class="slider"
      id={HtmlElementId.SalmonCoefficientRange}
      type="range"
      min="1"
      max="2"
      step="0.1"
      on:change={updateSalmon}
      value={salmon_coefficient}
    />
  </details>
</div>

<style>
  .control {
    width: 250px;
    padding: 6px 8px;
    margin: 0.5rem;
    font:
      14px/16px 'Helvetica Neue',
      Arial,
      sans-serif;
    font-family: 'Helvetica Neue', Helvetica, Arial, sans-serif;
    background: white;
    box-shadow: 0 0 15px rgba(0, 0, 0, 0.2);
    border-radius: 10px;
  }

  .tooltip .tooltip-text {
    visibility: hidden;

    text-align: center;
    padding: 8px;
    background: white;
    box-shadow: 0 0 15px rgba(0, 0, 0, 0.2);
    border-radius: 10px;

    position: absolute;
    left: 50%;
    z-index: 1;
  }

  .tooltip:hover .tooltip-text {
    visibility: visible;
  }
</style>
