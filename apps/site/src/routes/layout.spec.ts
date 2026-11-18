import { render, screen } from '@testing-library/svelte';
import { createRawSnippet } from 'svelte';
import { describe, expect, it } from 'vitest';

import Layout from './+layout.svelte';
import { prerender } from './+layout.ts';

describe('the root layout', () => {
  it('renders every page inside the main landmark', () => {
    const children = createRawSnippet(() => ({ render: () => '<p>conteúdo</p>' }));

    render(Layout, { children });

    expect(screen.getByRole('main').textContent).toBe('conteúdo');
  });

  it('prerenders every page', () => {
    expect(prerender).toBe(true);
  });
});
