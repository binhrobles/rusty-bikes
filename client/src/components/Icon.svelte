<script lang="ts">
  // the svg name in /public to render
  export let type: string;
</script>

<img class="scaled" src="./{type}.svg" alt="A {type} emoji" />

<style>
  .scaled {
    height: 2rem;
    width: 2rem;

    position: relative;
    top: 60%;
    transform: translateY(-50%);
  }
</style>
