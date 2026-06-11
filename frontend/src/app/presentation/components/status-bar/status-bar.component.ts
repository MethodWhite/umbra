import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { WebSocketService, ConnectionState } from '../../../core/services/websocket.service';
import { Subject, takeUntil } from 'rxjs';

@Component({
  selector: 'app-status-bar',
  standalone: true,
  imports: [CommonModule],
  templateUrl: './status-bar.component.html',
  styles: [`
    :host { display: contents; }
    .status-bar {
      position: fixed; bottom: 0; left: 0; right: 0; height: 32px;
      display: flex; align-items: center; justify-content: space-between;
      padding: 0 16px; background: rgba(5, 5, 8, 0.85);
      border-top: 1px solid rgba(14, 165, 233, 0.1); z-index: 1000;
      -webkit-app-region: drag; backdrop-filter: blur(8px);
    }
    .brand { font-size: 10px; letter-spacing: 4px; color: rgba(14, 165, 233, 0.35); text-transform: uppercase; font-weight: 300; }
    .status-center { position: absolute; left: 50%; transform: translateX(-50%); }
    .status-label { font-size: 11px; color: rgba(14, 165, 233, 0.4); letter-spacing: 2px; font-weight: 300; }
    .status-right { display: flex; align-items: center; gap: 6px; }
    .connection-dot { width: 6px; height: 6px; border-radius: 50%; background: rgba(255,255,255,0.15); transition: background 0.3s; }
    .connection-dot.connected { background: #22c55e; box-shadow: 0 0 6px rgba(34,197,94,0.4); }
    .connection-dot.reconnecting { background: #eab308; box-shadow: 0 0 6px rgba(234,179,8,0.4); animation: pulse 1s ease-in-out infinite; }
    .connection-dot.disconnected { background: #ef4444; box-shadow: 0 0 6px rgba(239,68,68,0.4); }
    .connection-text { font-size: 10px; color: rgba(255,255,255,0.25); letter-spacing: 1px; }
    @keyframes pulse { 0%,100% { opacity: 1; } 50% { opacity: 0.4; } }
  `],
})
export class StatusBarComponent implements OnInit, OnDestroy {
  wsState: ConnectionState = 'disconnected';
  wsLabel = '';
  statusText = '';
  private destroy$ = new Subject<void>();

  constructor(private ws: WebSocketService) {}

  ngOnInit(): void {
    this.ws.state$.pipe(takeUntil(this.destroy$)).subscribe(state => {
      this.wsState = state;
      const labels: Record<ConnectionState, string> = {
        connecting: 'connecting...',
        connected: 'connected',
        disconnected: 'offline',
        reconnecting: 'reconnecting...',
      };
      this.wsLabel = labels[state];
    });
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
  }
}
