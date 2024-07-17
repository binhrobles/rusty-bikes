<script lang="ts">
  import Icon from './Icon.svelte';
  import {
    $isLoading as isLoading,
    $isSuccess as isSuccess,
  } from '../store/fetch.ts';
</script>

<div>
  {#if $isLoading}
    <Icon type="hourglass" />
  {:else if $isSuccess}
    <Icon type="check" />
  {:else}
    <Icon type="cross" />
    <Icon type="shrug" />
  {/if}
</div>

<style>
  div {
    height: 1.75rem;
  }
</style>
