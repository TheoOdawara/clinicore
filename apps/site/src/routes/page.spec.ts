import { render, screen } from '@testing-library/svelte';
import { describe, expect, it } from 'vitest';

import Page from './+page.svelte';

describe('the landing page', () => {
  it('shows the product name as the page heading', () => {
    render(Page);

    expect(screen.getByRole('heading', { level: 1 }).textContent).toBe('Clinicore');
    expect(document.title).toBe('Clinicore');
  });
});
