import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';

@Component({
  selector: 'app-window-controls',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './window-controls.component.html',
  styles: [`
    .electron-titlebar {
      display: flex; align-items: center; justify-content: space-between;
      height: 38px; background: rgba(5,5,8,0.9); border-bottom: 1px solid rgba(14,165,233,0.08);
      -webkit-app-region: drag; user-select: none; position: fixed; top: 0; left: 0; right: 0; z-index: 9999;
    }
    .logo-area { font-size: 11px; letter-spacing: 4px; color: rgba(14,165,233,0.35); font-weight: 300; margin-left: 16px; text-transform: uppercase; }
    .window-buttons { display: flex; -webkit-app-region: no-drag; }
    .win-btn {
      width: 46px; height: 38px; border: none; background: transparent;
      color: rgba(255,255,255,0.35); font-size: 12px; cursor: pointer;
      display: flex; align-items: center; justify-content: center;
      transition: all 0.15s; font-family: inherit;
    }
    .win-btn:hover { background: rgba(255,255,255,0.06); color: rgba(255,255,255,0.7); }
    .win-btn.btn-close:hover { background: #e81123; color: white; }
    :host-context(body:not(.electron)) { display: none; }
  `],
})
export class WindowControlsComponent implements OnInit {
  isElectron = false;

  ngOnInit(): void {
    this.isElectron = !!(window as any).electronAPI?.isElectron;
  }

  minimize(): void { (window as any).electronAPI?.minimize(); }
  maximize(): void { (window as any).electronAPI?.maximize(); }
  close(): void { (window as any).electronAPI?.close(); }
}
