<script lang="ts">
  import { onMount } from 'svelte';
  import Radar from 'radar-sdk-js';
  import { Marker, LatLng } from 'leaflet';
  import { HtmlElementId } from '../consts.ts';
  import { NYC_CENTER } from '../config.ts';
  import {
    $startMarker as startMarker,
    $endMarker as endMarker,
    $startMarkerLatLng as startLatLng,
    $endMarkerLatLng as endLatLng,
    $startAddress as startAddress,
    $endAddress as endAddress,
    $selectedInput as selected,
    $withTraversal as withTraversal,
  } from '../store/route.ts';
  import {
    $heuristicWeight as heuristicWeight,
    $cyclewayPreference as cyclewayPreference,
    $roadPreference as roadPreference,
    $salmonCoefficient as salmonCoefficient,
  } from '../store/cost.ts';
  import { fitMarkers } from '../modules/map.mts';
  import type { WritableAtom } from 'nanostores';
  import type { ChangeEventHandler } from 'svelte/elements';
  import type { RadarAutocompleteAddress } from 'radar-sdk-js';

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

  // Handle autocomplete selection for a given marker
  const createOnSelection = (
    markerStore: typeof startMarker,
    addressStore: typeof startAddress,
    otherLatLng: typeof startLatLng
  ) => {
    return (address: RadarAutocompleteAddress) => {
      const latLng = new LatLng(address.latitude, address.longitude);
      const marker = new Marker(latLng, { draggable: true });
      markerStore.set(marker);
      addressStore.set(address.formattedAddress || `(${address.longitude.toFixed(5)}, ${address.latitude.toFixed(5)})`);
      fitMarkers(latLng, otherLatLng.get());
    };
  };

  onMount(() => {
    const startAutocomplete = Radar.ui.autocomplete({
      container: 'radar-start-autocomplete',
      near: NYC_CENTER,
      countryCode: 'US',
      placeholder: 'Search for start address...',
      limit: 5,
      minCharacters: 3,
      responsive: true,
      showMarkers: false,
      layers: ['address', 'place', 'coarse'],
      onSelection: createOnSelection(startMarker, startAddress, endLatLng),
    });

    const endAutocomplete = Radar.ui.autocomplete({
      container: 'radar-end-autocomplete',
      near: NYC_CENTER,
      countryCode: 'US',
      placeholder: 'Search for end address...',
      limit: 5,
      minCharacters: 3,
      responsive: true,
      showMarkers: false,
      layers: ['address', 'place', 'coarse'],
      onSelection: createOnSelection(endMarker, endAddress, startLatLng),
    });

    // Set $selectedInput on focus so map clicks go to the focused input
    startAutocomplete.inputField?.addEventListener('focus', () => {
      selected.set(HtmlElementId.StartInput);
    });
    endAutocomplete.inputField?.addEventListener('focus', () => {
      selected.set(HtmlElementId.EndInput);
    });

    // When address atoms change (from map click/drag reverse geocode),
    // update the widget input text
    startAddress.subscribe((addr) => {
      if (startAutocomplete.inputField && addr) {
        startAutocomplete.inputField.value = addr;
      }
    });
    endAddress.subscribe((addr) => {
      if (endAutocomplete.inputField && addr) {
        endAutocomplete.inputField.value = addr;
      }
    });
  });
</script>

<div class="control">
  <table class="route-table">
    <tr>
      <td><label>Start:</label></td>
      <td><div id="radar-start-autocomplete"></div></td>
    </tr>
    <tr>
      <td><label>End:</label></td>
      <td><div id="radar-end-autocomplete"></div></td>
    </tr>
  </table>

  <hr />

  <details>
    <summary>Customize pathfinding</summary>
    <br />
    <label for="with-traversal">Render Pathfinding?</label>
    <input
      type="checkbox"
      id="with-traversal"
      name="with-traversal"
      bind:checked={$withTraversal}
    />
    <br />
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
      max="3"
      step="0.1"
      on:change={createRangeUpdateHandler(salmonCoefficient)}
      value={salmonCoefficient.get()}
    />
  </details>
</div>

<style>
  .control {
    width: 450px;
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

  .route-table {
    width: 100%;
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

  :global(#radar-start-autocomplete),
  :global(#radar-end-autocomplete) {
    width: 100%;
  }

  :global(.radar-autocomplete-results-list) {
    z-index: 1000;
  }
</style>
