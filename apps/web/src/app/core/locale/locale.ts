import { registerLocaleData } from '@angular/common';
import localePt from '@angular/common/locales/pt';
import { LOCALE_ID, type Provider } from '@angular/core';

export const APP_LOCALE = 'pt-BR';

export function provideAppLocale(): Provider[] {
  registerLocaleData(localePt, APP_LOCALE);
  return [{ provide: LOCALE_ID, useValue: APP_LOCALE }];
}
