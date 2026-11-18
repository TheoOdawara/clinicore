import { render } from '@testing-library/svelte';
import { afterEach, describe, expect, it } from 'vitest';

import Seo from './seo.svelte';

describe('Seo', () => {
  afterEach(() => {
    document.head.innerHTML = '';
  });

  it('writes the title and the description to the head', () => {
    render(Seo, { title: 'Clinicore', description: 'Gestão de clínicas.' });

    expect(document.title).toBe('Clinicore');
    expect(document.querySelector('meta[name="description"]')?.getAttribute('content')).toBe(
      'Gestão de clínicas.',
    );
  });
});
