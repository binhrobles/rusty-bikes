# Mobile Navigation Client Plan for Rusty Bikes

## Context

The user wants to build a **mobile-centric PWA client** for turn-by-turn bike navigation on top of the existing Rust A* routing backend. The desktop client already does route planning; the mobile client will focus on **runtime navigation experience**: GPS tracking, auto-scrolling turn instructions, real-time re-routing, and a heading-up map.

The key insight from research: no single off-the-shelf library solves turn-by-turn navigation for a custom routing backend in 2025. Instead, we compose proven open-source primitives: `@turf.js` for geometry/bearing math, MapLibre GL JS for heading-up navigation rendering, native `Geolocation` API for GPS, and a small backend endpoint enhancement.

---

## Recommended Approach

### 1. Code Organization: Separate Svelte App in `client-mobile/`

- New directory: `/client-mobile/` — standalone Svelte 4 project with its own `vite.config.ts`
- Shares: package.json workspace setup, some shared types (CostModel, RouteMetadata)
- Deployed to: GitHub Pages at `/navigate` path (via GitHub Actions, new workflow)
- **Why**: Full architectural separation, independent build, avoids bloating the existing desktop app

### 2. Backend: New `POST /navigate` Endpoint (Mobile-Optimized)

**Scope**: New dedicated endpoint for mobile turn-by-turn navigation.

**Files to create/modify**:

- `services/src/bin/lambda-handler.rs` — add new `/navigate` route dispatcher (similar to `/route` but different response shape)
- `services/src/api/` — new module `navigation.rs` for lean response serialization
- `services/src/db/` — if needed, add `name` column to `WayLabels` (extend ETL to capture OSM `name` tag)

**Request shape** (same as `/route`):
```json
{
  "start": { "lat": 40.68, "lon": -73.96 },
  "end": { "lat": 40.69, "lon": -73.95 },
  "cost_model": { ... },
  "heuristic_weight": 0.75
}
```

**Response shape** (lean, mobile-optimized):
```json
{
  "route": {
    "type": "FeatureCollection",
    "features": [
      {
        "type": "Feature",
        "geometry": { "type": "LineString", "coordinates": [...] },
        "properties": {
          "distance": 143,
          "way_name": "Atlantic Ave",
          "labels": [2, 3, false]
        }
      }
    ]
  },
  "meta": { "total_distance": 3421, "total_time_estimate": 450 }
}
```

**Design decisions**:
- Drop `from`, `to`, `way` IDs (not needed for navigation; saves bytes)
- Include `way_name` for turn instructions
- Reuse `labels` for road type / salmon styling
- Include `total_distance` + `total_time_estimate` for display
- Desktop `/route` endpoint unchanged; both coexist

**Alternative routes** (deferred to v2):
- For now, single primary route per request
- Future: POST body with `?alternatives=2` param to return 3 options

### 3. Mobile Client: Architecture

#### 3.1 Stores (Nanostores)

Reuse the pattern from `client/src/store/` but streamlined for navigation:

| Store | Atoms | Purpose |
|-------|-------|---------|
| `store/nav.ts` | `$currentStepIndex`, `$routeSteps`, `$isOnRoute`, `$distanceOffRoute`, `$offRoutePromptVisible` | Navigation state (step progression, off-route status, re-route prompt) |
| `store/gps.ts` | `$userPosition`, `$userBearing`, `$gpsBearing` | GPS tracking (Geolocation watchPosition) |
| `store/cost.ts` | `$comfortSlider` (0-1), `$salmonToggle` (bool) | Mobile-optimized cost model (Comfort ↔ Speed slider + Traffic rules toggle) |
| `store/route.ts` | `$route`, `$routeMetadata`, `$cachedRoute` | Fetched route and metadata; cached to LocalStorage for offline access |
| `store/fetch.ts` | `$isLoading`, `$error` | Route fetch state |
| `store/settings.ts` | `$autoReroute` (bool) | User preferences (auto-reroute vs. prompt) |

**Cost model mapping** (mobile preset):

- `$comfortSlider` (0-1) maps to cost preference: `roadPreference = comfortSlider * 10`, `cyclewayPreference = comfortSlider * 10`
  - 0 = speed-optimized (prefer fast roads, penalize slow bike infrastructure)
  - 1 = comfort-optimized (prefer bike lanes, protected tracks, quiet streets)
- `$salmonToggle` (bool) determines `salmonCoefficient`:
  - `false` = no salmoning at all; ignores traffic direction (fastest possible)
  - `true` = salmon penalty enabled (1.3x cost multiplier); respects traffic rules
- When `salmonToggle = false`, faster routes that go against traffic are accepted
- Computed `$costModel` expands to full backend CostModel object (reuse type from `client/src/store/cost.ts`)

#### 3.2 Navigation Logic (Turf.js Integration)

File: `src/lib/navigation.ts`

```typescript
// Exported functions:

// Turn instruction generation (Turf-based)
export function computeTurnDirection(
  prevStepGeometry: LineString,
  nextStepGeometry: LineString
): 'left' | 'right' | 'straight' | 'uturn';
// Uses @turf/bearing to compute incoming/outgoing headings, classifies delta

export function generateInstruction(
  stepIndex: number,
  step: RouteStep,
  nextStep?: RouteStep,
  wayName?: string
): NavigationInstruction;
// Returns { action: 'turn' | 'continue', direction: 'left' | ..., distance: meters, name: wayName }

// Off-route detection
export function checkOffRoute(
  userPosition: [lat, lon],
  currentStepGeometry: LineString,
  thresholdMeters: number = 30
): { offRoute: boolean; distanceOff: number; snappedPoint: Point };
// Uses @turf/nearest-point-on-line, @turf/distance

// Step advancement
export function getStepBoundaryDistance(
  currentStepGeometry: LineString,
  userPosition: [lat, lon]
): number;
// Returns distance along line from start (0 to step.distance)
```

#### 3.3 GPS Tracking & Update Loop

File: `src/lib/gps.ts`

```typescript
export function startGPSTracking(
  onPositionUpdate: (pos: Position, bearing: number) => void,
  onError: (error: GeolocationPositionError) => void
): () => void; // returns unsubscribe function

// Uses navigator.geolocation.watchPosition({ enableHighAccuracy: true })
// Fires onPositionUpdate on each fix (automatically uses device GPS on mobile PWA over HTTPS)
```

#### 3.4 Components

| Component | Purpose |
|-----------|---------|
| `App.svelte` | Main layout (header, map, instruction panel, settings) |
| `MapView.svelte` | Radar/MapLibre map with route overlay, GPS marker, heading-up rotation |
| `InstructionPanel.svelte` | Auto-scrolling turn-by-turn list + next instruction HUD |
| `OffRoutePrompt.svelte` | Modal/side panel for re-route confirmation (separate from instructions) |
| `CostSlider.svelte` | Comfort ↔ Speed slider + Traffic rules toggle |
| `SearchInput.svelte` | Start/end address input (leverage existing Radar geocoding) |
| `SettingsPanel.svelte` | Auto-reroute toggle + other preferences |

#### 3.5 Map: Radar Maps (MapLibre)

File: `src/modules/map.mts`

**Why Radar Maps (MapLibre)**:
- Native heading-up rotation via `map.easeTo({ bearing: heading })` with smooth GPU animation
- WebGL rendering enables 60fps camera following
- Radar SDK provides vector tiles + geocoding; reuses existing API key
- No need for separate tile source configuration

**Setup**:
- Use Radar SDK for tiles and map
- Route layer: GeoJSON source updated reactively from `$route` store
- GPS marker: Centered with `map.easeTo({ center: [lng, lat], bearing: $userBearing })`

### 4. Route Request & Re-routing

**Initial route**: `POST /navigate` to Rust backend, using mobile cost preset (from `$comfortSlider` + `$salmonToggle`).

**Re-routing on off-route**:

- Detected by turf.js `checkOffRoute(userPos, currentStepGeometry)` returning `distanceOff > 30m`
- Set `$offRoutePromptVisible = true` (modal/panel separate from instruction list)
- User can dismiss (continue with existing directions) or confirm
- On confirm: new `POST /navigate` from user's current GPS location to original destination
- Update `$route` atom; reset `$currentStepIndex = 0`; cache new route to LocalStorage
- Later: check `$autoReroute` setting to toggle between prompt vs. silent auto-routing with toast

**Offline routing** (deferred to v2):
- Current: all re-routing requires API call
- Future: cache routing graph + use client-side lite A* for fast offline adjustments

### 5. TypeScript Types (Reused & New)

**Reused from desktop client** (`client/src/`):
- `CostModel` from `store/cost.ts`
- `RouteMetadata` from `store/fetch.ts`

**New for mobile client**:

```typescript
// src/types/index.ts

// Lean RouteStep for /navigate endpoint (mobile-optimized)
export interface MobileRouteStep extends GeoJSON.Feature {
  properties: {
    distance: number;        // meters for this step
    way_name: string;        // street name
    labels: [number, number, boolean]; // [cycleway, road, salmon]
  };
  geometry: GeoJSON.LineString;
}

// Computed instruction from route step (for UI)
export interface NavigationInstruction {
  action: 'turn' | 'continue' | 'arrive';
  direction: 'left' | 'right' | 'straight' | 'uturn' | null;
  distance: number; // meters
  wayName: string;
  stepIndex: number;
}

export interface Position {
  coords: {
    latitude: number;
    longitude: number;
    accuracy: number;
    heading: number | null;
    speed: number | null;
  };
  timestamp: number;
}
```

### 6. Package Dependencies

**New npm packages for mobile client**:

- `radar-sdk-js` v4.5+ (map rendering + geocoding; wraps maplibre-gl)
- `@turf/bearing`, `@turf/nearest-point-on-line`, `@turf/distance`, `@turf/helpers` (navigation math)
- `nanostores` (state management; reuse from desktop client)

**Keep existing**:

- `svelte` 4
- `vite`
- `typescript`
- Radar API key (configure in env; geocoding + vector tiles)

---

## Implementation Order

**Phase A: Mobile Client Foundation** (start here)

1. **Mobile client skeleton** (1 session):
   - Create `client-mobile/` directory structure
   - Set up Vite + Svelte 4 + TypeScript boilerplate
   - Add `radar-sdk-js`, turf, nanostores dependencies
   - Configure GitHub Pages deployment to `/navigate` path

2. **Stores & core logic** (1 session):
   - Implement `store/gps.ts` (position + bearing atoms)
   - Implement `store/nav.ts` (step index, off-route state)
   - Implement `store/cost.ts` (comfort slider + salmon toggle)
   - Implement `lib/navigation.ts` (bearing, turn direction, off-route detection)
   - Implement `lib/gps.ts` (watchPosition wrapper)

3. **Components & map** (1-2 sessions):
   - `MapView.svelte` with Radar/MapLibre integration
   - `InstructionPanel.svelte` with auto-scrolling list
   - `OffRoutePrompt.svelte` (modal for re-route confirmation)
   - `App.svelte` main layout
   - Search input + settings panel

**Phase B: Backend Endpoint**

4. **New `/navigate` endpoint** (1 session):
   - Add `POST /navigate` dispatcher in lambda-handler.rs
   - Create `services/src/api/navigation.rs` for lean response serialization
   - Add `name` column to WayLabels schema + ETL
   - Return mobile-optimized lean response (no from/to/way IDs)
   - Test with existing `/route` cost model

**Phase C: Integration & Polish**

5. **Route fetch + re-routing** (1 session):
   - Implement route request to `/navigate` endpoint
   - Implement off-route detection → re-route prompt flow
   - LocalStorage caching of route + turn metadata
   - Error handling (GPS unavailable, no route, timeout)

6. **Deployment** (1 session):
   - GitHub Actions workflow: build `client-mobile`, deploy to `/navigate`
   - Test end-to-end from `/navigate` path
   - Document Radar API key setup

---

## Critical Design Decisions

### Why compose Turf.js instead of using a turn-by-turn SDK?

- No single SDK supports custom backends. `leaflet-routing-machine` is abandoned; `osrm-text-instructions` requires adapter code.
- Turf.js is smaller, actively maintained, and flexible. Custom turn logic is ~50 lines of TS.

### Why Radar Maps (MapLibre) instead of Leaflet?

- Native heading-up rotation via `map.easeTo({ bearing: heading })` with smooth GPU animation
- WebGL rendering enables 60fps camera following
- Radar SDK provides vector tiles + geocoding; reuses existing API key
- No need for separate tile source configuration

### Why a separate `/navigate` endpoint instead of enhancing `/route`?

- Desktop `/route` carries full metadata (from/to/way IDs) for visualization; mobile doesn't need these
- Lean response saves bandwidth on mobile networks
- Separate contracts allow independent evolution (e.g., add alternatives to `/navigate` later)
- Clearer semantics: `/route` = planning UI, `/navigate` = runtime navigation

### Street names in backend response?

- Server-side: single SQL column lookup from existing OSM data, zero client latency
- Client-side: would require Overpass API batch queries per route, adds network round-trip
- Decision: include `way_name` in `/navigate` response (cleanest, fastest)

---

## File Structure (New)

```
rusty-bikes/
├── client-mobile/                  # NEW: separate Svelte PWA
│   ├── src/
│   │   ├── App.svelte             # Main layout
│   │   ├── components/
│   │   │   ├── MapView.svelte
│   │   │   ├── InstructionPanel.svelte
│   │   │   ├── OffRoutePrompt.svelte
│   │   │   ├── CostSlider.svelte
│   │   │   ├── SearchInput.svelte
│   │   │   └── SettingsPanel.svelte
│   │   ├── store/
│   │   │   ├── gps.ts            # GPS tracking atoms
│   │   │   ├── nav.ts            # Navigation state (step index, off-route, prompt)
│   │   │   ├── cost.ts           # Cost model atoms
│   │   │   ├── route.ts          # Route + metadata atoms + $cachedRoute
│   │   │   ├── fetch.ts          # Route fetch state
│   │   │   └── settings.ts       # $autoReroute, other preferences
│   │   ├── lib/
│   │   │   ├── navigation.ts     # Turf-based turn logic, off-route detection
│   │   │   ├── gps.ts            # watchPosition wrapper
│   │   │   ├── cache.ts          # LocalStorage helpers for route caching
│   │   │   └── config.ts         # API URL, Radar key
│   │   ├── modules/
│   │   │   └── map.mts           # MapLibre integration
│   │   └── types/
│   │       └── index.ts          # RouteStep, NavigationInstruction, etc.
│   ├── vite.config.ts
│   ├── tsconfig.json
│   └── package.json (workspace)
│
├── services/                       # MODIFIED: backend
│   ├── src/
│   │   ├── db/
│   │   │   ├── core.rs           # Add `name` column to WayLabels
│   │   │   └── etl.rs            # Capture OSM name tag
│   │   ├── graph/
│   │   │   └── traversal.rs      # Carry way_name through TraversalSegment
│   │   └── api/
│   │       └── geojson.rs        # Serialize way_name in features
│   └── tests/                     # Update integration tests for new field
│
├── .github/workflows/
│   └── deploy-mobile.yml          # NEW: build + deploy client-mobile to /navigate
│
└── package.json (root workspace)
```

---

## Verification & Testing

**Backend changes**:
- Run existing tests: `make service-test` (should pass with new nullable `way_name` field)
- Manually test `/navigate` endpoint with curl; confirm `way_name` present in response

**Mobile client**:

1. **Local dev**: `cd client-mobile && npm run dev` — Vite dev server
2. **Route fetch**: Load app, search start/end, confirm route renders on map
3. **GPS simulation** (dev tools browser console):
   ```javascript
   Object.defineProperty(navigator, 'geolocation', {
     value: {
       watchPosition: (cb) =>
         setInterval(() => cb({
           coords: {
             latitude: 40.7,
             longitude: -73.98,
             heading: 45,
           },
         }), 1000),
     },
   });
   ```
4. **Off-route detection**: Manually trigger re-route prompt by changing simulated position far from route
5. **Instruction panel**: Confirm turns advance correctly, wayName displays
6. **Map rotation**: Confirm map bearing changes as simulated heading changes

**Deployment**:
- GitHub Actions workflow builds `client-mobile`, outputs to `/navigate` folder
- Manual test: `https://YOUR_GITHUB_PAGES_URL/navigate` loads map + can request route

---

## Design Decisions (Finalized)

1. **Basemap tiles**: Use Radar's tile hosting (you already have the API key)
   - Reuse existing Radar setup for geocoding + tiles
   - Mobile-optimized rendering

2. **Turn instruction text**: Start with simple English templates
   - Keep initial bundle lean; easy to add more later

3. **Offline caching**: Cache full route GeoJSON + turn metadata
   - Store `$route` atom state to LocalStorage on successful route fetch
   - Allows map redraw if app reloads or is backgrounded

4. **Re-route UX**: Configurable, but start with prompt modal
   - Initial: Show "You're off route. Recalculate?" prompt separate from instruction panel
   - Prompt displays as a modal or side panel (not overlaying active directions)
   - User can dismiss to continue with old route or confirm to re-route
   - Later: Add setting to toggle auto-re-route with toast notification
   - Implementation: `$autoReroute` boolean atom in settings store
