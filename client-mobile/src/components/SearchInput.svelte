<script lang="ts">
  import { onMount } from 'svelte';
  import Radar from 'radar-sdk-js';
  import { NYC_CENTER } from '../lib/config.ts';
  import {
    $startLatLng as startLatLng,
    $endLatLng as endLatLng,
    $startAddress as startAddress,
    $endAddress as endAddress,
  } from '../store/route.ts';
  import { $userPosition as userPosition, startGPS } from '../store/gps.ts';

  type Suggestion = { label: string; lat: number; lon: number };

  let startQuery = startAddress.get();
  let endQuery = endAddress.get();
  let startSuggestions: Suggestion[] = [];
  let endSuggestions: Suggestion[] = [];
  let startFocused = false;
  let endFocused = false;

  // Default start to current location with reverse geocode
  onMount(() => {
    // Don't override if start was already restored from cache
    if (startLatLng.get()) return;

    const unsub = userPosition.subscribe(async (pos) => {
      if (!pos) return;
      unsub();

      const lat = pos.coords.latitude;
      const lon = pos.coords.longitude;
      startLatLng.set([lat, lon]);
      startQuery = 'My location';

      try {
        const res = await Radar.reverseGeocode({ latitude: lat, longitude: lon });
        const addr = res.addresses?.[0];
        if (addr) {
          const label = addr.formattedAddress ?? addr.street ?? 'My location';
          startAddress.set(label);
          startQuery = label;
        }
      } catch {
        startAddress.set('My location');
      }
    });
  });

  async function suggest(query: string): Promise<Suggestion[]> {
    if (query.length < 2) return [];
    try {
      const res = await Radar.autocomplete({ query, near: NYC_CENTER, limit: 5 });
      return (res.addresses ?? []).map((a) => ({
        label: a.formattedAddress ?? a.street ?? query,
        lat: a.latitude,
        lon: a.longitude,
      }));
    } catch {
      return [];
    }
  }

  async function onStartInput(e: Event) {
    startQuery = (e.target as HTMLInputElement).value;
    startSuggestions = await suggest(startQuery);
  }

  async function onEndInput(e: Event) {
    endQuery = (e.target as HTMLInputElement).value;
    endSuggestions = await suggest(endQuery);
  }

  function pickStart(s: Suggestion) {
    startLatLng.set([s.lat, s.lon]);
    startAddress.set(s.label);
    startQuery = s.label;
    startSuggestions = [];
    startFocused = false;
  }

  function pickEnd(s: Suggestion) {
    endLatLng.set([s.lat, s.lon]);
    endAddress.set(s.label);
    endQuery = s.label;
    endSuggestions = [];
    endFocused = false;
  }

  function useCurrentLocation() {
    startGPS();
    const pos = userPosition.get();
    if (pos) {
      startLatLng.set([pos.coords.latitude, pos.coords.longitude]);
      startAddress.set('My location');
      startQuery = 'My location';
    } else {
      startQuery = 'Getting location…';
      const unsub = userPosition.subscribe((p) => {
        if (!p) return;
        startLatLng.set([p.coords.latitude, p.coords.longitude]);
        startAddress.set('My location');
        startQuery = 'My location';
        unsub();
      });
    }
  }
</script>

<div class="search-bar">
  <div class="input-row">
    <button class="icon-btn" title="Use my location" on:click={useCurrentLocation}>📍</button>
    <div class="input-wrap">
      <input
        type="search"
        placeholder="Start"
        bind:value={startQuery}
        on:input={onStartInput}
        on:focus={() => (startFocused = true)}
        on:blur={() => setTimeout(() => (startFocused = false), 150)}
      />
      {#if startFocused && startSuggestions.length}
        <ul class="suggestions">
          {#each startSuggestions as s}
            <li><button on:click={() => pickStart(s)}>{s.label}</button></li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>

  <div class="input-row">
    <span class="icon-btn icon-btn--static">🏁</span>
    <div class="input-wrap">
      <input
        type="search"
        placeholder="Destination"
        bind:value={endQuery}
        on:input={onEndInput}
        on:focus={() => (endFocused = true)}
        on:blur={() => setTimeout(() => (endFocused = false), 150)}
      />
      {#if endFocused && endSuggestions.length}
        <ul class="suggestions">
          {#each endSuggestions as s}
            <li><button on:click={() => pickEnd(s)}>{s.label}</button></li>
          {/each}
        </ul>
      {/if}
    </div>
  </div>
</div>

<style>
  .search-bar {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    padding: 0.5rem;
    background: #1e293b;
    border-bottom: 1px solid #334155;
    flex: 1;
  }

  .input-row {
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .icon-btn {
    font-size: 1.1rem;
    background: none;
    border: none;
    cursor: pointer;
    padding: 0.2rem;
    min-width: 1.8rem;
    text-align: center;
  }

  .icon-btn--static { cursor: default; }

  .input-wrap { flex: 1; position: relative; }

  input {
    width: 100%;
    padding: 0.45rem 0.7rem;
    border: 1px solid #334155;
    border-radius: 0.5rem;
    background: #0f172a;
    color: #f8fafc;
    font-size: 0.9rem;
    outline: none;
  }

  input:focus { border-color: #2563eb; }

  .suggestions {
    position: absolute;
    top: 100%;
    left: 0;
    right: 0;
    z-index: 50;
    background: #1e293b;
    border: 1px solid #334155;
    border-radius: 0 0 0.5rem 0.5rem;
    margin: 0;
    padding: 0;
    list-style: none;
    max-height: 11rem;
    overflow-y: auto;
  }

  .suggestions li button {
    width: 100%;
    text-align: left;
    padding: 0.55rem 0.7rem;
    background: none;
    border: none;
    border-bottom: 1px solid #334155;
    color: #f8fafc;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .suggestions li:last-child button { border-bottom: none; }
  .suggestions li button:hover { background: #334155; }
</style>
