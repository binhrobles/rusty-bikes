<script lang="ts">
  import { HtmlElementId } from '../consts.ts';
  import {
    $startMarkerLatLng as startLatLng,
    $endMarkerLatLng as endLatLng,
    $selectedInput as selected
  } from '../store/route.ts';

  // when the Routing start / end inputs are clicked,
  // queue them up to be changed on the next map click
  function onClick(event) {
    selected.set(event.target.id);
  }

  let start;
  let end;

  startLatLng.subscribe((s, _) => start = s?.lng ? `(${s.lng.toFixed(5)}, ${s.lat.toFixed(5)})` : 'Click to place start point');
  endLatLng.subscribe((e, _) => end = e?.lng ? `(${e.lng.toFixed(5)}, ${e.lat.toFixed(5)})` : 'Click to place end point');
</script>

<div id={HtmlElementId.RoutePanel}>
  <table class="route-table">
    <tr>
      <td><label for={HtmlElementId.StartInput}>Start:</label></td>
      <td><input type="text" id={HtmlElementId.StartInput} on:click={onClick} value={start}></td>
    </tr>
    <tr>
      <td><label for={HtmlElementId.EndInput}>End:</label></td>
      <td><input type="text" id={HtmlElementId.EndInput} on:click={onClick} value={end}></td>
    </tr>
  </table>
</div>
