import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import eslint from 'vite-plugin-eslint';

export default defineConfig({
  base: '/rusty-bikes/navigate',
  plugins: [eslint(), svelte()],
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    esbuildOptions: {
      target: 'esnext',
    },
  },
});
