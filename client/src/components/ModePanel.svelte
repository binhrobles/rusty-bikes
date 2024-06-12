<script lang="ts">
  import { Mode, ModeMeta, HtmlElementId } from '../consts.ts';
  import TraverseConfig from './TraverseConfig.svelte';
  import RouteConfig from './RouteConfig.svelte';
  import { $mode as m } from '../store/mode.ts';
</script>

<h4>Rusty Bikes</h4>
<br />

<label for="mode-select">Mode:</label>
<select name="mode-select" id="mode-select" bind:value={$m}>
  {#each Object.keys(ModeMeta) as key }
    <option id={key} value={key}>{ModeMeta[key].label}</option>
  {/each}
</select>

<hr>

{#if $m === Mode.Traverse}
  <TraverseConfig />
{/if}
{#if $m === Mode.Route || $m === Mode.RouteViz}
  <RouteConfig />
{/if}
