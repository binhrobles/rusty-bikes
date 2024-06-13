<script lang="ts">
  import { $isLoading as isLoading, $isSuccess as isSuccess } from '../store/fetch.ts';
</script>

<style>
  span {
    font-size: x-large;
  }

</style>

{#if $isLoading}
  <span>⏳</span>
{:else}
  {#if $isSuccess}
  <span>✅</span>
  {:else}
  <span>⚠️</span>
  {/if}
{/if}
