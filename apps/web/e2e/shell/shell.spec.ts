import { expect, test } from '@playwright/test';

import {
  expectNoAccessibilityViolations,
  expectNoHorizontalScroll,
} from '../support/accessibility';

test.describe('shell', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/');
  });

  test('opens in Brazilian Portuguese with the product name', async ({ page }) => {
    await expect(page.locator('html')).toHaveAttribute('lang', 'pt-BR');
    await expect(page).toHaveTitle('Clinicore');
    await expect(page.getByRole('banner')).toHaveText('Clinicore');
    await expect(page.getByRole('main')).toBeVisible();
  });

  test('fits the viewport without horizontal scroll', async ({ page }) => {
    await expectNoHorizontalScroll(page);
  });

  test('meets WCAG 2.2 AA', async ({ page }) => {
    await expectNoAccessibilityViolations(page);
  });
});
