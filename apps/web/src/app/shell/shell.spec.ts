import { render, screen } from '@testing-library/angular';

import { Shell } from './shell';

describe('Shell', () => {
  it('shows the product name in the banner and an empty main region', async () => {
    await render(Shell);

    expect(screen.getByRole('banner').textContent).toContain('Clinicore');
    expect(screen.getByRole('main')).toBeTruthy();
  });
});
