import type { Routes } from '@angular/router';

import { Shell } from './shell/shell';

export const routes: Routes = [{ path: '', component: Shell, children: [] }];
