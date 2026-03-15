# Planning & Navigating Views Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the mobile client into two distinct views — a "Planning" view for route setup and a "Navigating" view for turn-by-turn GPS guidance — so the map behavior, header, and footer all adapt to the user's current activity.

**Architecture:** Add an `$appView` atom (`'planning' | 'navigating'`) to the settings store. App.svelte conditionally renders different header/footer components based on this atom. The map module gains a navigation camera mode with pitch, bearing-follow, and bottom-center user positioning. Planning view shows search bars + cost sliders with a "Go!" button; navigating view shows current instruction header + upcoming turns footer with an "Exit" button.

**Tech Stack:** Svelte 4, TypeScript, MapLibre GL JS, Nanostores

---

## File Structure

### New Files
| File | Responsibility |
|------|---------------|
| `client-mobile/src/components/NavigationHeader.svelte` | Nav-mode header: shows current instruction (arrow + street + distance) and route meta |
| `client-mobile/src/components/NavigationFooter.svelte` | Nav-mode footer: next 3 upcoming turns + red "Exit" button |

### Modified Files
| File | Changes |
|------|---------|
| `client-mobile/src/store/settings.ts` | Add `$appView` atom |
| `client-mobile/src/store/nav.ts` | Add `$upcomingInstructions` computed store (next 3 after current), add `getRouteStepBearing()` |
| `client-mobile/src/components/SettingsPanel.svelte` | Remove header/close button, add "Go!" button that transitions to navigating view |
| `client-mobile/src/modules/map.mts` | Add `followGPSNavMode()` with pitch + bottom-center offset, add `resetCamera()` for planning mode |
| `client-mobile/src/store/gps.ts` | Gate `processGPSUpdate` behind `$appView === 'navigating'` check |
| `client-mobile/src/components/MapView.svelte` | Branch GPS-follow behavior on `$appView`: planning = no follow, navigating = nav camera mode |
| `client-mobile/src/App.svelte` | Restructure: swap header/footer based on `$appView`, integrate OffRoutePrompt in nav mode, remove settings toggle button |

---

## Chunk 1: State & Store Foundation

### Task 1: Add view state and missing store atoms

**Files:**
- Modify: `client-mobile/src/store/settings.ts`
- Modify: `client-mobile/src/store/nav.ts`

- [ ] **Step 1: Add `$appView` and `$autoReroute` to settings store**

Replace the entire contents of `client-mobile/src/store/settings.ts` with:

```typescript
import { atom } from 'nanostores';

export type AppView = 'planning' | 'navigating';

export const $appView = atom<AppView>('planning');
export const $settingsOpen = atom<boolean>(false);
```

- [ ] **Step 2: Add `$upcomingInstructions` and `getRouteStepBearing` to nav store**

In `client-mobile/src/store/nav.ts`:

First, add these imports at the top of the file (turf is already a project dependency via `lib/navigation.ts`):

```typescript
import { bearing as turfBearing } from '@turf/bearing';
import { point } from '@turf/helpers';
```

Then add below the existing `$nextInstruction` computed store (after line 23):

```typescript
/** Next 3 instructions after the current one (for navigation footer) */
export const $upcomingInstructions = computed(
  [$instructions, $currentStepIndex],
  (instructions, idx) => instructions.slice(idx + 1, idx + 4),
);

/**
 * Returns bearing of the current route step's first segment.
 * Used as fallback camera bearing when user is stationary (no GPS heading).
 */
export function getRouteStepBearing(): number {
  const route = $route.get();
  const idx = $currentStepIndex.get();
  if (!route || idx >= route.features.length) return 0;

  const coords = route.features[idx].geometry.coordinates;
  if (coords.length < 2) return 0;

  return turfBearing(point(coords[0]), point(coords[1]));
}
```

Note: The existing off-route detection code in `processGPSUpdate` (`$isOnRoute`, `$distanceOffRoute`, `checkOffRoute`) is dead code for now — leave it in place but do not wire it to any UI. OffRoutePrompt will be revisited later.

- [ ] **Step 3: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds (warnings OK, no errors)

- [ ] **Step 4: Commit**

```bash
git add client-mobile/src/store/settings.ts client-mobile/src/store/nav.ts
git commit -m "feat(mobile): add appView state atom and missing nav/settings exports"
```

---

## Chunk 2: Map Navigation Camera

### Task 2: Add navigation camera mode to map module

**Files:**
- Modify: `client-mobile/src/modules/map.mts`

The navigation camera needs to:
1. Pitch the map (45 degrees) to give a "behind the rider" perspective
2. Rotate the map to face the user's travel direction (or the route's next bearing)
3. Position the user's location in the bottom-center of the viewport (using MapLibre padding)

- [ ] **Step 1: Add `followGPSNavMode()` function**

Add at the end of `client-mobile/src/modules/map.mts`, before the closing:

```typescript
/**
 * Navigation-mode camera: pitched, bearing-rotated, user at bottom-center.
 * Padding pushes the logical center upward so the user dot sits in the lower third.
 */
export function followGPSNavMode(lat: number, lon: number, bearing: number): void {
  if (!map) return;
  map.easeTo({
    center: [lon, lat],
    bearing,
    pitch: 45,
    padding: { top: Math.round(map.getContainer().clientHeight * 0.55), bottom: 0, left: 0, right: 0 },
    duration: 500,
  });
}

/**
 * Reset camera to top-down planning mode: no pitch, north-up.
 */
export function resetCamera(): void {
  if (!map) return;
  map.easeTo({
    pitch: 0,
    bearing: 0,
    padding: { top: 0, bottom: 0, left: 0, right: 0 },
    duration: 400,
  });
}
```

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/modules/map.mts
git commit -m "feat(mobile): add navigation camera mode and resetCamera to map module"
```

---

## Chunk 3: Navigation Header Component

### Task 3: Create NavigationHeader component

**Files:**
- Create: `client-mobile/src/components/NavigationHeader.svelte`

This replaces the search bars when in navigating mode. Shows the current instruction prominently (arrow + street name + distance) and route meta (total distance/time) on the right side.

- [ ] **Step 1: Create the component**

Create `client-mobile/src/components/NavigationHeader.svelte`:

```svelte
<script lang="ts">
  import { $currentInstruction as currentInstruction } from '../store/nav.ts';
  import { $routeMeta as routeMeta } from '../store/route.ts';

  const ARROW: Record<string, string> = {
    left: '↰', right: '↱', straight: '↑', uturn: '↶',
  };

  function formatDist(m: number) {
    return m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`;
  }

  function formatTime(s: number) {
    const mins = Math.round(s / 60);
    return mins < 60 ? `${mins} min` : `${Math.floor(mins / 60)}h ${mins % 60}m`;
  }
</script>

<div class="nav-header">
  {#if $currentInstruction}
    <span class="arrow">{ARROW[$currentInstruction.direction ?? 'straight'] ?? '↑'}</span>
    <div class="info">
      <div class="street">{$currentInstruction.wayName || 'Continue'}</div>
      <div class="dist">{formatDist($currentInstruction.distance)}</div>
    </div>
    {#if $routeMeta}
      <div class="meta">
        <span>{formatDist($routeMeta.total_distance)}</span>
        <span>{formatTime($routeMeta.total_time_estimate)}</span>
      </div>
    {/if}
  {/if}
</div>

<style>
  .nav-header {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 1.25rem;
    background: #1e293b;
    color: #f8fafc;
    min-height: 4rem;
    border-bottom: 1px solid #334155;
  }

  .arrow {
    font-size: 2.5rem;
    line-height: 1;
    min-width: 2.5rem;
    text-align: center;
  }

  .info { flex: 1; min-width: 0; }

  .street {
    font-size: 1.1rem;
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .dist { font-size: 0.85rem; color: #94a3b8; margin-top: 0.1rem; }

  .meta {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    font-size: 0.8rem;
    color: #94a3b8;
    gap: 0.1rem;
    white-space: nowrap;
  }
</style>
```

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/components/NavigationHeader.svelte
git commit -m "feat(mobile): add NavigationHeader component for nav-mode header"
```

---

## Chunk 4: Navigation Footer Component

### Task 4: Create NavigationFooter component

**Files:**
- Create: `client-mobile/src/components/NavigationFooter.svelte`

Shows the next 3 upcoming turns after the current instruction, plus a red "Exit" button that returns to planning view.

- [ ] **Step 1: Create the component**

Create `client-mobile/src/components/NavigationFooter.svelte`:

```svelte
<script lang="ts">
  import { $upcomingInstructions as upcomingInstructions } from '../store/nav.ts';
  import { $appView } from '../store/settings.ts';
  import { resetCamera } from '../modules/map.mts';

  const ARROW: Record<string, string> = {
    left: '↰', right: '↱', straight: '↑', uturn: '↶',
  };

  function formatDist(m: number) {
    return m >= 1000 ? `${(m / 1000).toFixed(1)} km` : `${Math.round(m)} m`;
  }

  function exitNavigation() {
    $appView.set('planning');
    resetCamera();
    // App.svelte's appView.listen() handles resizeMap + fitRoute
  }
</script>

<div class="nav-footer">
  <div class="upcoming">
    {#each $upcomingInstructions as instr, i}
      <div class="turn-row" class:first={i === 0}>
        <span class="turn-arrow">{ARROW[instr.direction ?? 'straight'] ?? '↑'}</span>
        <span class="turn-street">{instr.wayName || 'Continue'}</span>
        <span class="turn-dist">{formatDist(instr.distance)}</span>
      </div>
    {:else}
      <div class="turn-row empty">Arriving soon</div>
    {/each}
  </div>
  <button class="exit-btn" on:click={exitNavigation}>Exit</button>
</div>

<style>
  .nav-footer {
    background: #1e293b;
    border-top: 1px solid #334155;
    padding: 0.5rem 0;
  }

  .upcoming {
    padding: 0 1.25rem;
  }

  .turn-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.6rem 0;
    color: #cbd5e1;
    font-size: 0.9rem;
    border-bottom: 1px solid #334155;
  }

  .turn-row.first {
    color: #f8fafc;
    font-weight: 500;
  }

  .turn-row.empty {
    color: #94a3b8;
    justify-content: center;
    font-style: italic;
  }

  .turn-row:last-child { border-bottom: none; }

  .turn-arrow {
    font-size: 1.3rem;
    min-width: 1.5rem;
    text-align: center;
  }

  .turn-street {
    flex: 1;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .turn-dist {
    font-size: 0.8rem;
    color: #94a3b8;
    white-space: nowrap;
  }

  .exit-btn {
    display: block;
    width: calc(100% - 2.5rem);
    margin: 0.75rem 1.25rem;
    padding: 0.7rem;
    background: #dc2626;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-size: 1rem;
    font-weight: 600;
    cursor: pointer;
  }
</style>
```

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/components/NavigationFooter.svelte
git commit -m "feat(mobile): add NavigationFooter component with upcoming turns and exit button"
```

---

## Chunk 5: Update SettingsPanel with "Go!" Button

### Task 5: Modify SettingsPanel for planning view

**Files:**
- Modify: `client-mobile/src/components/SettingsPanel.svelte`

The SettingsPanel becomes the always-visible footer in planning mode. Remove the header/close button. Add a "Go!" button below the cost sliders that transitions to navigating view. The button is only enabled when a route exists.

- [ ] **Step 1: Update SettingsPanel**

In `client-mobile/src/components/SettingsPanel.svelte`:

Remove the `createEventDispatcher` import and `dispatch` usage. Remove the `.header` div with close button and "Settings" title. Add imports for `$appView`, `$route`, and `resetNav`. Add a "Go!" button at the bottom.

Replace the `<script>` block with:

```svelte
<script lang="ts">
  import {
    $comfortSlider as comfortSlider,
    $speedSlider as speedSlider,
    $hillSlider as hillSlider,
    $salmonSlider as salmonSlider,
  } from '../store/cost.ts';
  import { $route as route } from '../store/route.ts';
  import { $appView } from '../store/settings.ts';
  import { resetNav } from '../store/nav.ts';
  import type { WritableAtom } from 'nanostores';

  const debounce = (store: WritableAtom<number>, parse: (v: string) => number, ms = 400) => {
    let timer: ReturnType<typeof setTimeout>;
    return (e: Event) => {
      clearTimeout(timer);
      timer = setTimeout(() => store.set(parse((e.target as HTMLInputElement).value)), ms);
    };
  };

  function startNavigation() {
    resetNav();
    $appView.set('navigating');
  }
</script>
```

Replace the template (from `<div class="panel">` onward, before `<style>`) with:

```svelte
<div class="panel">
  <div class="section-label">Routing</div>
  <label class="slider-row">
    <span class="slider-label">Comfort</span>
    <input type="range" min="0" max="1" step="0.05" value={$comfortSlider}
      on:input={debounce(comfortSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Speed</span>
    <input type="range" min="0" max="1" step="0.05" value={$speedSlider}
      on:input={debounce(speedSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Avoid Hills</span>
    <input type="range" min="0" max="1" step="0.05" value={$hillSlider}
      on:input={debounce(hillSlider, parseFloat)} />
  </label>
  <label class="slider-row">
    <span class="slider-label">Rules</span>
    <input type="range" min="0" max="3" step="1" value={$salmonSlider}
      on:input={debounce(salmonSlider, parseInt)} />
  </label>
  <div class="slider-ticks">
    <span>Ignore</span>
    <span>Sometimes</span>
    <span>Mostly</span>
    <span>Always</span>
  </div>

  <button class="go-btn" disabled={!$route} on:click={startNavigation}>
    Go!
  </button>
</div>
```

Add the `.go-btn` styles to the `<style>` block. Also remove the `.header`, `.title`, `.close-btn` styles since they're no longer used:

```css
  .go-btn {
    display: block;
    width: 100%;
    margin-top: 1rem;
    padding: 0.75rem;
    background: #2563eb;
    color: #fff;
    border: none;
    border-radius: 0.5rem;
    font-size: 1.1rem;
    font-weight: 700;
    cursor: pointer;
    letter-spacing: 0.02em;
  }

  .go-btn:disabled {
    background: #334155;
    color: #64748b;
    cursor: not-allowed;
  }
```

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/components/SettingsPanel.svelte
git commit -m "feat(mobile): add Go button to SettingsPanel, remove close/header for planning view"
```

---

## Chunk 6: Update MapView and GPS Store for Dual-Mode Behavior

### Task 6a: Gate processGPSUpdate behind navigating mode

**Files:**
- Modify: `client-mobile/src/store/gps.ts`

In planning mode, `processGPSUpdate` (step advancement, off-route detection) should not run — it could trigger OffRoutePrompt or advance steps while the user is just planning.

- [ ] **Step 1: Add appView import and gate the call**

In `client-mobile/src/store/gps.ts`, add the import at the top:

```typescript
import { $appView } from './settings.ts';
```

Then wrap the `processGPSUpdate` call (line 23) with the view check:

```typescript
    (pos, bearing) => {
      $userPosition.set(pos);
      $userBearing.set(bearing);
      if ($appView.get() === 'navigating') {
        processGPSUpdate([pos.coords.latitude, pos.coords.longitude]);
      }
    },
```

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/store/gps.ts
git commit -m "feat(mobile): only run nav GPS processing in navigating mode"
```

---

### Task 6b: Branch MapView behavior on app view

**Files:**
- Modify: `client-mobile/src/components/MapView.svelte`

In planning mode: GPS marker updates but no camera follow. In navigating mode: use `followGPSNavMode()` for pitched, bearing-rotated, bottom-center follow. Uses route step bearing as fallback when user is stationary.

- [ ] **Step 1: Update MapView subscriptions**

Replace the `<script>` block in `client-mobile/src/components/MapView.svelte` with:

```svelte
<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import 'maplibre-gl/dist/maplibre-gl.css';
  import { createMap, updateRoute, updateCorridor, updateGPSMarker, followGPSNavMode, fitRoute, updateEndMarker, setEndMarkerDragHandler } from '../modules/map.mts';
  import Radar from 'radar-sdk-js';
  import { $route as route, $corridor as corridor, $endLatLng as endLatLng, $endAddress as endAddress } from '../store/route.ts';
  import { $userPosition as userPosition, $userBearing as userBearing } from '../store/gps.ts';
  import { $appView as appView } from '../store/settings.ts';
  import { getRouteStepBearing } from '../store/nav.ts';

  let container: HTMLDivElement;
  const unsubs: Array<() => void> = [];

  onMount(() => {
    const map = createMap(container.id);

    unsubs.push(
      corridor.subscribe((c) => updateCorridor(c)),
    );

    unsubs.push(
      route.subscribe((r) => {
        updateRoute(r);
        // Only auto-fit in planning mode
        if (r && appView.get() === 'planning') fitRoute(r);
      }),
    );

    // Draggable destination marker
    setEndMarkerDragHandler((lat, lon) => {
      endLatLng.set([lat, lon]);
      endAddress.set('Dropped pin');
      Radar.reverseGeocode({ latitude: lat, longitude: lon })
        .then((res) => {
          const addr = res.addresses?.[0];
          if (addr) {
            endAddress.set(addr.formattedAddress ?? addr.street ?? 'Dropped pin');
          }
        })
        .catch(() => {});
    });

    unsubs.push(
      endLatLng.subscribe((coords) => {
        if (coords) updateEndMarker(coords[0], coords[1]);
      }),
    );

    unsubs.push(
      userPosition.subscribe((pos) => {
        if (!pos) return;
        const { latitude: lat, longitude: lon } = pos.coords;
        updateGPSMarker(lat, lon);
        // Only auto-follow camera in navigating mode
        if (appView.get() === 'navigating') {
          // Use GPS bearing when moving, fall back to route step bearing when stationary
          const bearing = userBearing.get() || getRouteStepBearing();
          followGPSNavMode(lat, lon, bearing);
        }
      }),
    );

    return () => map.remove();
  });

  onDestroy(() => unsubs.forEach((u) => u()));
</script>
```

Key changes from original:
- Import `followGPSNavMode` instead of `followGPS`
- Import `$appView` instead of `$settingsOpen`
- Import `getRouteStepBearing` for stationary bearing fallback
- Route subscription: only `fitRoute` in planning mode
- GPS subscription: only `followGPSNavMode` in navigating mode (no follow at all in planning)
- Bearing uses `userBearing || getRouteStepBearing()` so map faces route direction when stationary

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Commit**

```bash
git add client-mobile/src/components/MapView.svelte
git commit -m "feat(mobile): branch MapView camera behavior on planning vs navigating view"
```

---

## Chunk 7: Wire Up App.svelte

### Task 7: Restructure App.svelte for dual views

**Files:**
- Modify: `client-mobile/src/App.svelte`

This is the main integration point. Planning mode: SearchInput header + SettingsPanel footer. Navigating mode: NavigationHeader + NavigationFooter + OffRoutePrompt.

- [ ] **Step 1: Rewrite App.svelte**

Replace the entire contents of `client-mobile/src/App.svelte` with:

```svelte
<script lang="ts">
  import { onMount } from 'svelte';
  import MapView from './components/MapView.svelte';
  import SearchInput from './components/SearchInput.svelte';
  import SettingsPanel from './components/SettingsPanel.svelte';
  import NavigationHeader from './components/NavigationHeader.svelte';
  import NavigationFooter from './components/NavigationFooter.svelte';
  import { startGPS } from './store/gps.ts';
  import { $appView as appView } from './store/settings.ts';
  import { loadRoute, loadEndpoints } from './lib/cache.ts';
  import {
    $route as route,
    $routeMeta as routeMeta,
    $startLatLng as startLatLng,
    $endLatLng as endLatLng,
    $startAddress as startAddress,
    $endAddress as endAddress,
  } from './store/route.ts';
  import { fitRoute, resizeMap } from './modules/map.mts';
  // Side-effect import: activates the batched fetch watcher
  import './store/fetch.ts';
  import { tick } from 'svelte';
  import { RUSTY_BASE_URL } from './lib/config.ts';

  let lambdaReady = false;

  onMount(async () => {
    startGPS();

    // Restore last session from localStorage
    const cached = loadRoute();
    if (cached) {
      route.set(cached.route);
      routeMeta.set(cached.meta);
    }

    const endpoints = loadEndpoints();
    if (endpoints) {
      if (endpoints.startLatLng) startLatLng.set(endpoints.startLatLng);
      if (endpoints.endLatLng) endLatLng.set(endpoints.endLatLng);
      if (endpoints.startAddress) startAddress.set(endpoints.startAddress);
      if (endpoints.endAddress) endAddress.set(endpoints.endAddress);
    }

    // Ping lambda
    let retries = 0;
    while (!lambdaReady && retries < 10) {
      try {
        await fetch(`${RUSTY_BASE_URL}/ping`);
        lambdaReady = true;
      } catch (e) {
        retries++;
        console.error(`received ${e} from /ping`);
        await new Promise((resolve) => setTimeout(resolve, 2000));
      }
    }
  });

  // When switching back to planning, resize map and fit route
  appView.listen(async (view) => {
    if (view === 'planning') {
      await tick();
      resizeMap();
      const r = route.get();
      if (r) fitRoute(r);
    }
  });
</script>

<div class="app">
  <header>
    {#if $appView === 'planning'}
      {#if lambdaReady}
        <SearchInput />
      {:else}
        <div class="connecting">Connecting...</div>
      {/if}
    {:else}
      <NavigationHeader />
    {/if}
  </header>

  <main class="map-area">
    <MapView />
  </main>

  <footer>
    {#if $appView === 'planning'}
      <SettingsPanel />
    {:else}
      <NavigationFooter />
    {/if}
  </footer>
</div>

<style>
  .connecting {
    padding: 0.75rem 1rem;
    color: #94a3b8;
    font-size: 0.9rem;
  }

  :global(*) { box-sizing: border-box; margin: 0; padding: 0; }
  :global(body) {
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
    height: 100dvh;
    background: #0f172a;
  }

  .app {
    display: flex;
    flex-direction: column;
    height: 100dvh;
    width: 100vw;
  }

  header {
    flex: 0 0 auto;
    z-index: 10;
    display: flex;
    align-items: flex-start;
  }

  .map-area {
    flex: 1 1 auto;
    position: relative;
    overflow: hidden;
  }

  footer {
    flex: 0 0 auto;
    z-index: 10;
  }
</style>
```

Key changes from original:
- Removed `$settingsOpen` toggle logic and settings gear button
- Removed `hasCachedRoute` (unused after removing conditional logic)
- `$appView` controls header/footer swap — but MapView stays outside the conditional (single persistent instance, avoids map destroy/recreate on view switch)
- SettingsPanel is always visible in planning footer (no toggle)
- NavigationHeader replaces SearchInput in navigating mode
- NavigationFooter replaces SettingsPanel in navigating mode
- OffRoutePrompt NOT included (off-route feature deferred — dead code left in nav store for now)
- Added `appView.listen()` to resize map when returning to planning

- [ ] **Step 2: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 3: Verify in browser — planning mode**

Run: `cd client-mobile && npx vite dev`

Open in mobile viewport. Verify:
- Search bars visible in header
- Cost sliders visible in footer (no settings toggle needed)
- "Go!" button visible below sliders, disabled (grayed out) when no route
- Map shows top-down (no pitch, north-up)
- Setting a route: "Go!" button becomes enabled (blue)
- Map does NOT follow GPS position (GPS dot visible but map stays put)

- [ ] **Step 4: Verify in browser — navigating mode**

With a route set, tap "Go!". Verify:
- Header switches to current instruction (arrow + street + distance + meta)
- Map pitches and rotates to face travel direction
- GPS dot positioned in lower portion of screen
- Footer shows next 3 upcoming turns
- Red "Exit" button visible at bottom of footer
- Tapping "Exit" returns to planning view (map resets to top-down, search bars return)

- [ ] **Step 5: Commit**

```bash
git add client-mobile/src/App.svelte
git commit -m "feat(mobile): wire up planning/navigating view switching in App.svelte"
```

---

## Chunk 8: Cleanup

### Task 8: Remove unused code

**Files:**
- Modify: `client-mobile/src/components/InstructionPanel.svelte` (delete or keep as reference)
- Modify: `client-mobile/src/modules/map.mts` (remove unused `followGPS`)

- [ ] **Step 1: Remove old `followGPS` from map module**

In `client-mobile/src/modules/map.mts`, remove the `followGPS` function (lines 174-177):

```typescript
// DELETE this function:
export function followGPS(lat: number, lon: number, bearing: number): void {
  if (!map) return;
  map.easeTo({ center: [lon, lat], bearing, duration: 500 });
}
```

- [ ] **Step 2: Delete InstructionPanel.svelte**

`InstructionPanel.svelte` is fully replaced by `NavigationHeader` (current instruction) and `NavigationFooter` (upcoming turns). Delete it:

```bash
rm client-mobile/src/components/InstructionPanel.svelte
```

- [ ] **Step 3: Verify no remaining imports of deleted code**

Search for any remaining references:

```bash
grep -r "InstructionPanel\|followGPS" client-mobile/src/
```

Expected: No matches. If any found, update those imports.

- [ ] **Step 4: Verify the app compiles**

Run: `cd client-mobile && npx vite build --mode development 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 5: Commit**

```bash
git add -u client-mobile/src/
git commit -m "chore(mobile): remove InstructionPanel and unused followGPS function"
```

---

## Summary of View Behavior

| Aspect | Planning View | Navigating View |
|--------|--------------|-----------------|
| **Header** | SearchInput (start/destination) | NavigationHeader (current turn arrow + street + distance) |
| **Footer** | SettingsPanel (cost sliders + "Go!" button) | NavigationFooter (next 3 turns + red "Exit") |
| **Map pitch** | 0 (top-down) | 45 degrees |
| **Map bearing** | 0 (north-up) | User's travel direction |
| **GPS follow** | No (dot visible, camera stays) | Yes (user at bottom-center, camera follows) |
| **Route fit** | Auto-fits bounds on route load | No auto-fit |
| **Off-route** | N/A | Deferred (dead code in nav store) |
| **Transition in** | Default / "Exit" button | "Go!" button |
