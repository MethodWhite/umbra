import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SettingsRepository, StatusResponse } from '../../../../../data/repositories/settings.repository';

@Component({
  selector: 'app-settings-system',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './system.component.html',
  styles: [`
    :host { display: block; }
    .mode-selector { display:flex; gap:8px; }
    .mode-btn {
      flex:1; padding:12px 8px; border:1px solid rgba(255,255,255,0.08);
      border-radius:8px; background:rgba(255,255,255,0.02);
      cursor:pointer; transition:all 0.2s; text-align:center;
    }
    .mode-btn:hover { border-color:rgba(0,212,255,0.2); background:rgba(0,212,255,0.04); }
    .mode-btn.active { border-color:rgba(0,212,255,0.3); background:rgba(0,212,255,0.08); }
    .mode-icon { display:block; font-size:12px; color:rgba(0,212,255,0.7); letter-spacing:1px; text-transform:uppercase; margin-bottom:4px; }
    .mode-desc { display:block; font-size:10px; color:rgba(255,255,255,0.3); }
    .status-grid { display:flex; flex-direction:column; gap:10px; }
    .status-row { display:flex; align-items:center; gap:10px; font-size:13px; color:rgba(255,255,255,0.5); }
    .status-detail { margin-left:auto; font-size:11px; color:rgba(255,255,255,0.25); }
    .sysinfo-grid { display:flex; flex-direction:column; gap:8px; }
    .sysinfo-row { display:flex; justify-content:space-between; font-size:13px; }
    .sysinfo-label { color:rgba(255,255,255,0.35); }
    .sysinfo-row span:last-child { color:rgba(0,212,255,0.6); font-variant-numeric:tabular-nums; }
    .status-dot { width:8px; height:8px; border-radius:50%; background:rgba(255,255,255,0.15); flex-shrink:0; }
    .status-green { background:#22c55e; box-shadow:0 0 6px rgba(34,197,94,0.4); }
    .status-red { background:#ef4444; box-shadow:0 0 6px rgba(239,68,68,0.4); }
  `],
})
export class SystemComponent implements OnInit {
  status: StatusResponse | null = null;
  mode: 'secure' | 'balanced' | 'unrestricted' = 'balanced';

  constructor(private settingsRepo: SettingsRepository) {}

  ngOnInit(): void {
    this.load();
  }

  async load(): Promise<void> {
    this.status = await this.settingsRepo.getSystemStatus();
  }

  formatUptime(seconds: number): string {
    if (seconds < 60) return `${Math.floor(seconds)}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m`;
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return `${h}h ${m}m`;
  }
}
