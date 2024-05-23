import globals from "globals";
import pluginJs from "@eslint/js";
import tseslint from "typescript-eslint";
import airbnb_base from 'airbnb-base';
import airbnb_typescript from 'airbnb_typescript/base';

export default [
  airbnb_base,
  airbnb_typescript,
  {
    ignores: ["dist/"],
  },
  {
    languageOptions: { globals: globals.browser },
  },
  pluginJs.configs.recommended,
  ...tseslint.configs.recommended,
];
