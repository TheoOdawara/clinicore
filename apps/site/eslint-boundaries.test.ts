import assert from 'node:assert/strict';
import { join } from 'node:path';
import { describe, it } from 'node:test';

import { ESLint } from 'eslint';
import tseslint from 'typescript-eslint';

import { architecture } from './eslint.config.ts';

const src = join(import.meta.dirname, 'fixtures/boundaries/src');

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
    filePath: join(src, from),
  });
  assert.ok(result, 'ESLint returns one result per linted text');
  return result.messages.map((message) => `${message.ruleId ?? 'fatal'}: ${message.message}`);
}

const route = 'routes/page.ts';
const hero = 'lib/sections/hero/hero.ts';
const button = 'lib/components/button/button.ts';
const content = 'lib/content/site.ts';

const allowed: [string, string, string][] = [
  ['a route composes a section', route, '../lib/sections/hero/hero'],
  ['a route reads the content', route, '../lib/content/site'],
  ['a route loads the styles', route, '../lib/styles/tokens'],
  ['a section reads its own content', hero, './hero-content'],
  ['a section uses a component', hero, '../../components/button/button'],
  ['a section reads the shared content', hero, '../../content/site'],
  ['a section shows an asset', hero, '../../assets/logo'],
  ['a component composes another component', button, '../accordion/accordion'],
  ['the content points to an asset', content, '../assets/logo'],
];

const forbidden: [string, string, string][] = [
  ['a section imports another section', hero, '../pricing/pricing'],
  ['a component imports a section', button, '../../sections/hero/hero'],
  ['a component reads the content', button, '../../content/site'],
  ['the content imports a component', content, '../components/button/button'],
  ['the lib imports a route', content, '../../routes/page'],
];

void describe('the architecture boundaries of apps/site', () => {
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
