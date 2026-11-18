import assert from 'node:assert/strict';
import { readFile } from 'node:fs/promises';
import { join } from 'node:path';
import { describe, it } from 'node:test';

const build = join(import.meta.dirname, 'build');

async function built(file: string): Promise<string> {
  return readFile(join(build, file), 'utf8');
}

void describe('the prerendered landing', () => {
  void it('is in Brazilian Portuguese', async () => {
    assert.match(await built('index.html'), /<html lang="pt-BR">/);
  });

  void it('carries the title and the description for search engines', async () => {
    const html = await built('index.html');
    assert.match(html, /<title>Clinicore<\/title>/);
    assert.match(html, /<meta name="description" content="[^"]+"/);
  });

  void it('renders the heading without JavaScript', async () => {
    assert.match(await built('index.html'), /<h1[^>]*>Clinicore<\/h1>/);
  });

  void it('publishes the sitemap and robots.txt', async () => {
    assert.match(await built('sitemap.xml'), /<loc>https:\/\/clinicore\.com\.br\/<\/loc>/);
    assert.match(await built('robots.txt'), /Sitemap: https:\/\/clinicore\.com\.br\/sitemap\.xml/);
  });
});
