<script lang="ts">
  import { PropKey } from '../consts.ts';
  export let properties;

  const OSMKeys: string[] = [PropKey.From, PropKey.To, PropKey.Way];

  const isNotOSMKey = (key: string) => !OSMKeys.includes(key);
  const isStartNode = properties[PropKey.From] === -1;
  const isEndNode = properties[PropKey.To] === -2;
</script>

<h4>Segment</h4>
<hr />
<table>
  <tr>
    <td><strong>from</strong></td>
    <td>
      {#if isStartNode}
        <span>Start</span>
      {:else}
        <a
          href="https://www.openstreetmap.org/node/{properties[PropKey.From]}"
          target="_blank"
          rel="noopener noreferrer"
        >
          {properties[PropKey.From]}
        </a>
      {/if}
    </td>
  </tr>

  <tr>
    <td><strong>to</strong></td>
    <td>
      {#if isEndNode}
        <span>End</span>
      {:else}
        <a
          href="https://www.openstreetmap.org/node/{properties[PropKey.To]}"
          target="_blank"
          rel="noopener noreferrer"
        >
          {properties[PropKey.To]}
        </a>
      {/if}
    </td></tr
  >

  <tr>
    <td><strong>way</strong></td>
    <td
      ><a
        href="https://www.openstreetmap.org/way/{Math.abs(
          properties[PropKey.Way]
        )}"
        target="_blank"
        rel="noopener noreferrer">{properties[PropKey.Way]}</a
      ></td
    >
  </tr>

  {#each Object.keys(properties) as key}
    {#if isNotOSMKey(key)}
      <tr>
        <td><strong>{key}</strong></td>
        <td>{properties[key]}</td>
      </tr>
    {/if}
  {/each}
</table>
