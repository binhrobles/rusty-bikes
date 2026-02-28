# Frontend Conventions

## Stack

- **Svelte 4** (not SvelteKit — plain Svelte + Vite)
- **TypeScript 5.2**
- **Vite 5** for dev server and builds
- **Leaflet 1.9** for map rendering
- **Nanostores 0.10** for state management
- **rainbowvis.js** for cost gradient coloring on route segments

## Project Structure

```
client/src/
├── components/
│   ├── App.svelte              # Root component
│   ├── Control.svelte          # UI controls (cost sliders, routing options)
│   ├── Icon.svelte             # Icon rendering
│   ├── LoadingIndicator.svelte # Loading state
│   └── DebugPopup.svelte       # Debug overlays
├── store/
│   ├── map.ts                  # Map state, click events
│   ├── route.ts                # Route parameters (start/end, traversal toggle)
│   ├── cost.ts                 # Cost model configuration (weights, coefficients)
│   ├── fetch.ts                # API calls: batched task, auto-fires when inputs ready
│   ├── render.ts               # Visualization state, polyline rendering
│   └── marker.ts               # Start/end marker positions
├── modules/
│   ├── map.mts                 # Leaflet map initialization and manipulation
│   └── control.mts             # Control interaction logic
├── config.ts                   # API URL: prod (API Gateway) vs local (localhost:9000)
├── consts.ts                   # PropKey enum, HtmlElementId enum, defaults
├── main.ts                     # Vite entry point
└── style.css                   # Global styles
```

## Nanostores Pattern

State flows through a reactive chain:

```
atoms (user input)
  → computed stores (derived state)
    → batched task (side effects / API calls)
```

Key example in `store/fetch.ts`:
- `$startMarkerLatLng`, `$endMarkerLatLng`, `$costModel`, `$heuristicWeight` are atom inputs
- `$raw` is a `batched()` store that auto-fires `fetchRoute()` when all inputs are set
- Response flows into route/render stores for map display

## Leaflet Integration

- Map setup in `modules/map.mts` — creates Leaflet map, handles click events
- Control logic in `modules/control.mts` — wires UI controls to stores
- Route rendering uses GeoJSON layers with cost-based color gradients (rainbowvis.js)
- CSS keyframe animations for route segment drawing

## Styling

- Vanilla CSS with scoped Svelte `<style>` blocks
- Global styles in `style.css`
- No CSS framework or preprocessor
- Animations via CSS keyframes

## Config

- `config.ts` — API base URL, switches between prod and local based on `import.meta.env.PROD`
- `consts.ts` — Property key enums (matching backend serialization), HTML element IDs, default values for cost model and traversal

## Tooling

- **ESLint** with `@stylistic` rules (via `eslint.config.js`)
- **Prettier** for formatting (`.prettierrc`)
- **svelte-check** for type checking
- **dependency-cruiser** for dependency analysis (`.dependency-cruiser.cjs`)
- **No test framework** — no unit or integration tests for the client
- Package manager: **Yarn** (with `yarn.lock`)
