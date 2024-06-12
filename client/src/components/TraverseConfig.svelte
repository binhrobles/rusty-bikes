<script lang="ts">
  import { TraversalDefaults, HtmlElementId, PaintOptions } from '../consts.ts';
  import { $depth as d, $paint as p } from '../store/traverse.ts';
</script>

<div id={HtmlElementId.TraversePanel}>
  <span>Traversal Depth:</span>
  <span id={HtmlElementId.DepthValue}>{$d}</span>
  <br />

  <input
    class="slider"
    id={HtmlElementId.DepthRange}
    type="range"
    min="0"
    max="100"
    bind:value={$d}>
  <br />

  <label for={HtmlElementId.PaintSelect}>Paint with:</label>
  <select name={HtmlElementId.PaintSelect} id={HtmlElementId.PaintSelect} bind:value={$p}>
    {#each Object.keys(PaintOptions) as option}
    <option id={option} value={option}>{option}</option>
    {/each}
  </select>
</div>
