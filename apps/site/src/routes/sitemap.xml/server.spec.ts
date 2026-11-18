import { describe, expect, it } from 'vitest';

import { GET } from './+server.ts';

describe('sitemap.xml', () => {
  it('lists the landing as XML', async () => {
    const response = GET();

    expect(response.headers.get('Content-Type')).toBe('application/xml');
    expect(await response.text()).toContain('<loc>https://clinicore.com.br/</loc>');
  });
});
