/// <reference types="svelte" />
/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly VITE_RADAR_API_KEY: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
