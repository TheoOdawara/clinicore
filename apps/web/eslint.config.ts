import eslint from '@eslint/js';
import angular from 'angular-eslint';
import boundaries from 'eslint-plugin-boundaries';
import type { Linter } from 'eslint';
import { defineConfig } from 'eslint/config';
import tseslint from 'typescript-eslint';

const sameFeature = { feature: '{{ from.element.captured.feature }}' };

export const architecture: Linter.Config = {
  files: ['**/src/app/**/*.ts'],
  plugins: { boundaries },
  settings: {
    'import/resolver': { node: { extensions: ['.ts'] } },
    'boundaries/elements': [
      { type: 'feature-page', pattern: 'app/features/*/pages/*', capture: ['feature', 'page'] },
      { type: 'feature-ui', pattern: 'app/features/*/ui/*', capture: ['feature', 'component'] },
      { type: 'feature-data', pattern: 'app/features/*/data', capture: ['feature'] },
      { type: 'feature', pattern: 'app/features/*', capture: ['feature'] },
      { type: 'core', pattern: 'app/core/*', capture: ['area'] },
      { type: 'shared', pattern: 'app/shared/*', capture: ['component'] },
      { type: 'shell', pattern: 'app/shell' },
    ],
  },
  rules: {
    'boundaries/dependencies': [
      'error',
      {
        default: 'disallow',
        policies: [
          { from: { element: { type: 'shell' } }, allow: { to: { element: { type: '*' } } } },
          {
            from: { element: { type: 'feature' } },
            allow: [
              { to: { element: { types: ['core', 'shared'] } } },
              {
                to: {
                  element: {
                    types: ['feature', 'feature-page', 'feature-ui', 'feature-data'],
                    captured: sameFeature,
                  },
                },
              },
            ],
          },
          {
            from: { element: { type: 'feature-page' } },
            allow: {
              to: { element: { types: ['feature-ui', 'feature-data'], captured: sameFeature } },
            },
          },
          {
            from: { element: { type: 'feature-ui' } },
            allow: [
              { to: { element: { type: 'shared' } } },
              { to: { element: { type: 'feature-ui', captured: sameFeature } } },
            ],
          },
          {
            from: { element: { type: 'feature-data' } },
            allow: [
              {
                to: { element: { type: 'core', captured: [{ area: 'http' }, { area: 'config' }] } },
              },
              { to: { element: { type: 'feature-data', captured: sameFeature } } },
            ],
          },
          { from: { element: { type: 'shared' } }, allow: { to: { element: { type: 'shared' } } } },
          { from: { element: { type: 'core' } }, allow: { to: { element: { type: 'core' } } } },
        ],
      },
    ],
  },
};

export default defineConfig(
  { ignores: ['dist/', 'coverage/', '.angular/', 'test-results/', 'playwright-report/'] },
  {
    files: ['**/*.ts'],
    extends: [
      eslint.configs.recommended,
      tseslint.configs.strictTypeChecked,
      tseslint.configs.stylisticTypeChecked,
    ],
    languageOptions: { parserOptions: { projectService: true } },
  },
  {
    files: ['src/**/*.ts'],
    extends: [angular.configs.tsRecommended],
    rules: {
      '@angular-eslint/component-selector': [
        'error',
        { type: 'element', prefix: 'app', style: 'kebab-case' },
      ],
      '@angular-eslint/directive-selector': [
        'error',
        { type: 'attribute', prefix: 'app', style: 'camelCase' },
      ],
      '@angular-eslint/prefer-on-push-component-change-detection': 'error',
      '@typescript-eslint/no-extraneous-class': ['error', { allowWithDecorator: true }],
    },
  },
  {
    files: ['src/**/*.html'],
    extends: [angular.configs.templateRecommended, angular.configs.templateAccessibility],
  },
  architecture,
);
