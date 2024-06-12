<script lang="ts">
  import { HtmlElementId } from '../consts.ts';

  // TODO: move on click handlers here
</script>

<div id={HtmlElementId.RoutePanel}>
  <table class="route-table">
    <tr>
      <td><label for={HtmlElementId.StartInput}>Start:</label></td>
      <td><input type="text" id={HtmlElementId.StartInput} placeholder="Click to select start point"></td>
    </tr>
    <tr>
      <td><label for={HtmlElementId.EndInput}>End:</label></td>
      <td><input type="text" id={HtmlElementId.EndInput} placeholder="Click to select end point"></td>
    </tr>
  </table>
</div>
