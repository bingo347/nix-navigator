import { defineConfig } from "eslint/config";
import js from "@eslint/js";
import ts from 'typescript-eslint';
import svelte from "eslint-plugin-svelte";

export default defineConfig({
  files: ["packages/**/*.svelte", "packages/**/*.ts"],
  extends: [
    js.configs.recommended,
    ts.configs.recommended,
    svelte.configs.recommended,
  ],
  rules: {
    "quotes": ["error", "double"],
  },
});