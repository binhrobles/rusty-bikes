import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import eslint from 'vite-plugin-eslint';

export default defineConfig({
  base: '/navigate/',
  plugins: [eslint(), svelte()],
});
