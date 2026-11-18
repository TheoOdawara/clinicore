import { formatDate } from '@angular/common';
import { LOCALE_ID } from '@angular/core';
import { TestBed } from '@angular/core/testing';
import { RouterTestingHarness } from '@angular/router/testing';

import { appConfig } from './app.config';

describe('App', () => {
  beforeEach(() => {
    TestBed.configureTestingModule({ providers: appConfig.providers });
  });

  it('opens the shell at the root route', async () => {
    const harness = await RouterTestingHarness.create('/');

    expect(harness.routeNativeElement?.querySelector('header')?.textContent).toContain('Clinicore');
  });

  it('formats dates in Brazilian Portuguese', () => {
    const locale = TestBed.inject(LOCALE_ID);

    expect(locale).toBe('pt-BR');
    expect(formatDate(new Date(2026, 10, 18), 'longDate', locale)).toBe('18 de novembro de 2026');
  });
});
