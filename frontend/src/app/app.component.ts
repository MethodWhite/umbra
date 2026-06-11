import { Component } from '@angular/core';
import { RouterOutlet } from '@angular/router';
import { StatusBarComponent } from './presentation/components/status-bar/status-bar.component';
import { WindowControlsComponent } from './presentation/components/window-controls/window-controls.component';

@Component({
  selector: 'app-root',
  standalone: true,
  imports: [RouterOutlet, StatusBarComponent, WindowControlsComponent],
  template: `
    <app-window-controls></app-window-controls>
    <router-outlet></router-outlet>
    <app-status-bar></app-status-bar>
  `,
  styles: [`
    :host { display: contents; }
  `],
})
export class AppComponent {}
