import { defineConfig } from 'vite';

import { resolve } from 'path';
import handlebars from 'vite-plugin-handlebars';
import eslint from 'vite-plugin-eslint';

export default defineConfig({
  plugins: [
    eslint(),
    handlebars({
      partialDirectory: resolve(__dirname, 'src/templates'),
    }),
  ],
});


