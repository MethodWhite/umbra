import { Routes } from '@angular/router';
import { ChatComponent } from './presentation/pages/chat/chat.component';
import { SettingsComponent } from './presentation/pages/settings/settings.component';
import { MonitorComponent } from './presentation/pages/monitor/monitor.component';

export const routes: Routes = [
  { path: '', redirectTo: '/chat', pathMatch: 'full' },
  { path: 'chat', component: ChatComponent },
  { path: 'settings', component: SettingsComponent },
  { path: 'monitor', component: MonitorComponent },
];
