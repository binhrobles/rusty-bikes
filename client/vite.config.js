import { defineConfig } from 'vite';

import { resolve } from 'path';
import { svelte } from '@sveltejs/vite-plugin-svelte'
import handlebars from 'vite-plugin-handlebars';
import eslint from 'vite-plugin-eslint';

export default defineConfig({
  base: '/rusty-bikes/',
  plugins: [
    eslint(),
    svelte(),
    handlebars({
      partialDirectory: resolve(__dirname, 'src/templates'),
    }),
  ],
});
