import eslint from '@eslint/js';
import boundaries from 'eslint-plugin-boundaries';
import svelte from 'eslint-plugin-svelte';
import type { Linter } from 'eslint';
import { defineConfig } from 'eslint/config';
import globals from 'globals';
import tseslint from 'typescript-eslint';

const sameSection = { section: '{{ from.element.captured.section }}' };

export const architecture: Linter.Config = {
  files: ['**/src/**/*.{ts,svelte}'],
  plugins: { boundaries },
  settings: {
    'import/resolver': {
      typescript: { project: './tsconfig.json', extensions: ['.ts', '.svelte', '.css'] },
    },
    'boundaries/elements': [
      { type: 'route', pattern: 'src/routes' },
      { type: 'section', pattern: 'lib/sections/*', capture: ['section'] },
      { type: 'component', pattern: 'lib/components/*', capture: ['component'] },
      { type: 'content', pattern: 'lib/content' },
      { type: 'assets', pattern: 'lib/assets' },
      { type: 'styles', pattern: 'lib/styles' },
    ],
  },
  rules: {
    'boundaries/dependencies': [
      'error',
      {
        default: 'disallow',
        policies: [
          {
            from: { element: { type: 'route' } },
            allow: {
              to: {
                element: {
                  types: ['route', 'section', 'component', 'content', 'assets', 'styles'],
                },
              },
            },
          },
          {
            from: { element: { type: 'section' } },
            allow: [
              { to: { element: { types: ['component', 'assets', 'content', 'styles'] } } },
              { to: { element: { type: 'section', captured: sameSection } } },
            ],
          },
          {
            from: { element: { type: 'component' } },
            allow: { to: { element: { type: 'component' } } },
          },
          {
            from: { element: { type: 'content' } },
            allow: { to: { element: { type: 'assets' } } },
          },
        ],
      },
    ],
  },
};

export default defineConfig(
  { ignores: ['.svelte-kit/', 'build/', 'coverage/', 'test-results/', 'playwright-report/'] },
  eslint.configs.recommended,
  tseslint.configs.strictTypeChecked,
  tseslint.configs.stylisticTypeChecked,
  svelte.configs.recommended,
  {
    languageOptions: {
      globals: { ...globals.browser, ...globals.node },
      parserOptions: { projectService: true, extraFileExtensions: ['.svelte'] },
    },
  },
  {
    files: ['**/*.svelte', '**/*.svelte.ts'],
    languageOptions: { parserOptions: { parser: tseslint.parser } },
  },
  architecture,
);
