import assert from 'node:assert/strict';
import { join } from 'node:path';
import { describe, it } from 'node:test';

import { ESLint } from 'eslint';
import tseslint from 'typescript-eslint';

import { architecture } from './eslint.config.ts';

const app = join(import.meta.dirname, 'fixtures/boundaries/src/app');

const eslint = new ESLint({
  cwd: import.meta.dirname,
  overrideConfigFile: true,
  overrideConfig: [
    { files: ['**/*.ts'], languageOptions: { parser: tseslint.parser } },
    architecture,
  ],
});

async function violations(from: string, importPath: string): Promise<string[]> {
  const [result] = await eslint.lintText(`import '${importPath}';\n`, {
    filePath: join(app, from),
  });
  assert.ok(result, 'ESLint returns one result per linted text');
  return result.messages.map((message) => `${message.ruleId ?? 'fatal'}: ${message.message}`);
}

const page = 'features/patients/pages/patient-list/patient-list.ts';
const ui = 'features/patients/ui/patient-card/patient-card.ts';
const data = 'features/patients/data/patients-api.ts';
const routes = 'features/patients/patients.routes.ts';

const allowed: [string, string, string][] = [
  ['a page reads its own store', page, '../../data/patients-store'],
  ['a page renders its own ui', page, '../../ui/patient-card/patient-card'],
  ['a ui uses shared', ui, '../../../../shared/button/button'],
  ['a ui composes another ui of its feature', ui, '../patient-avatar/patient-avatar'],
  ['data calls core/http', data, '../../../core/http/api-error'],
  ['data reads core/config', data, '../../../core/config/app-config'],
  ['data uses its own data', data, './patients-store'],
  ['the feature routes load its pages', routes, './pages/patient-list/patient-list'],
  ['the feature routes guard with core', routes, '../../core/session/session-store'],
  ['core uses core', 'core/session/session-store.ts', '../http/api-error'],
  ['shared uses shared', 'shared/button/button.ts', './button'],
  ['the shell joins the features', 'shell/shell.ts', '../features/patients/patients.routes'],
];

const forbidden: [string, string, string][] = [
  ['a page calls core/http directly', page, '../../../../core/http/api-error'],
  ['a page reads another feature', page, '../../../billing/data/billing-api'],
  ['a ui reads data', ui, '../../data/patients-api'],
  ['data reads core/session', data, '../../../core/session/session-store'],
  ['data reads another feature', data, '../../billing/data/billing-api'],
  ['a feature imports another feature', routes, '../billing/billing.routes'],
  ['a feature imports the shell', routes, '../../shell/shell'],
  ['shared imports core', 'shared/button/button.ts', '../../core/http/api-error'],
  ['core imports shared', 'core/http/api-error.ts', '../../shared/button/button'],
];

void describe('the architecture boundaries of apps/web', () => {
  for (const [name, from, target] of allowed) {
    void it(`allows: ${name}`, async () => {
      assert.deepEqual(await violations(from, target), []);
    });
  }

  for (const [name, from, target] of forbidden) {
    void it(`rejects: ${name}`, async () => {
      const messages = await violations(from, target);
      assert.equal(messages.length, 1, messages.join('\n'));
      assert.match(messages[0] ?? '', /^boundaries\/dependencies: /);
    });
  }
});
