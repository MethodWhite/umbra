import { Injectable } from '@angular/core';
import { AuthService } from './auth.service';
import { Subject, Observable } from 'rxjs';

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'reconnecting';

@Injectable({ providedIn: 'root' })
export class WebSocketService {
  private ws: WebSocket | null = null;
  private closed = false;
  private reconnectDelay = 1000;
  private messageSubject = new Subject<any>();
  private stateSubject = new Subject<ConnectionState>();
  private _state: ConnectionState = 'disconnected';

  message$: Observable<any> = this.messageSubject.asObservable();
  state$: Observable<ConnectionState> = this.stateSubject.asObservable();

  get state(): ConnectionState {
    return this._state;
  }

  constructor(private auth: AuthService) {}

  connect(url?: string): void {
    if (this.ws) this.close();
    this.closed = false;

    const wsProto = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const defaultUrl = `${wsProto}//${window.location.host}/ws/voice`;
    const wsUrl = url || defaultUrl;

    this.setConnectionState('connecting');
    this.ws = new WebSocket(wsUrl);

    this.ws.onopen = () => {
      this.setConnectionState('connected');
      this.reconnectDelay = 1000;
      const token = this.auth.getToken();
      if (token) {
        this.ws!.send(JSON.stringify({ type: 'auth', token }));
      }
    };

    this.ws.onmessage = (event) => {
      try {
        const msg = JSON.parse(event.data);
        this.messageSubject.next(msg);
      } catch {
        console.warn('[ws] bad message', event.data);
      }
    };

    this.ws.onclose = () => {
      this.setConnectionState('disconnected');
      if (!this.closed) {
        this.setConnectionState('reconnecting');
        setTimeout(() => this.connect(wsUrl), this.reconnectDelay);
        this.reconnectDelay = Math.min(this.reconnectDelay * 2, 30000);
      }
    };

    this.ws.onerror = (err) => {
      console.error('[ws] error', err);
      this.ws?.close();
    };
  }

  send(data: Record<string, unknown>): void {
    if (this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data));
    }
  }

  close(): void {
    this.closed = true;
    this.ws?.close();
    this.ws = null;
    this.setConnectionState('disconnected');
  }

  isConnected(): boolean {
    return this.ws?.readyState === WebSocket.OPEN;
  }

  private setConnectionState(state: ConnectionState): void {
    this._state = state;
    this.stateSubject.next(state);
  }
}
