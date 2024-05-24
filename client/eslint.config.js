import globals from 'globals';
import stylistic from '@stylistic/eslint-plugin';
import prettierConfig from 'eslint-config-prettier';
import eslintPluginPrettierRecommended from 'eslint-plugin-prettier/recommended';

export default [
  {
    ignores: ['dist/'],
  },
  {
    plugins: {
      '@stylistic': stylistic,
    },
  },
  {
    languageOptions: { globals: globals.browser },
  },
  prettierConfig, // Turns off all ESLint rules that have the potential to interfere with Prettier rules.
  eslintPluginPrettierRecommended,
];
