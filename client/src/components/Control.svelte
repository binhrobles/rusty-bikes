<script lang="ts">
  import { HtmlElementId } from '../consts.ts';
  import {
    $startMarkerLatLng as startLatLng,
    $endMarkerLatLng as endLatLng,
    $selectedInput as selected,
    $withTraversal as withTraversal,
  } from '../store/route.ts';
  import {
    $heuristicWeight as heuristicWeight,
    $cyclewayPreference as cyclewayPreference,
    $roadPreference as roadPreference,
    $salmonCoefficient as salmonCoefficient,
  } from '../store/cost.ts';
  import type { WritableAtom } from 'nanostores';
  import type { ChangeEventHandler } from 'svelte/elements';

  // when the Routing start / end inputs are clicked,
  // queue them up to be changed on the next map click
  const createOnClickHandler =
    (elementId: HtmlElementId.StartInput | HtmlElementId.EndInput) => () => {
      selected.set(elementId);
    };

  // creates a debouncer function for the provided store
  const createRangeUpdateHandler = (
    store: WritableAtom<number>
  ): ChangeEventHandler<HTMLInputElement> => {
    let timer: number;

    return (event) => {
      clearTimeout(timer);
      timer = setTimeout(() => {
        store.set(Number((event.target as HTMLInputElement)?.value));
      }, 500);
    };
  };

  let start: string;
  let end: string;

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
          on:click={createOnClickHandler(HtmlElementId.StartInput)}
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
          on:click={createOnClickHandler(HtmlElementId.EndInput)}
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
          bind:checked={$withTraversal}
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
      id={HtmlElementId.HeuristicRange}
      type="range"
      min="0.3"
      max="1.5"
      step="0.1"
      on:change={createRangeUpdateHandler(heuristicWeight)}
      value={heuristicWeight.get()}
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
      id={HtmlElementId.CyclewayRange}
      type="range"
      min="0"
      max="10"
      step="0.5"
      on:change={createRangeUpdateHandler(cyclewayPreference)}
      value={cyclewayPreference.get()}
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
      id={HtmlElementId.RoadRange}
      type="range"
      min="0"
      max="10"
      step="0.5"
      on:change={createRangeUpdateHandler(roadPreference)}
      value={roadPreference.get()}
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
      id={HtmlElementId.SalmonRange}
      type="range"
      min="1"
      max="2"
      step="0.1"
      on:change={createRangeUpdateHandler(salmonCoefficient)}
      value={salmonCoefficient.get()}
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
