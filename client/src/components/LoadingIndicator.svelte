<script lang="ts">
  import {
    $isLoading as isLoading,
    $isSuccess as isSuccess,
  } from '../store/fetch.ts';
</script>

{#if $isLoading}
  <span>⏳</span>
{:else if $isSuccess}
  <span>✅</span>
{:else}
  <span>❌ 🤷</span>
{/if}

<style>
  span {
    font-size: x-large;
  }
</style>
