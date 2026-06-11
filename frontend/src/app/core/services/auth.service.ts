import { Injectable } from '@angular/core';

@Injectable({ providedIn: 'root' })
export class AuthService {
  private token = '';
  private backendUrl = 'http://127.0.0.1:8484';
  private frontendUrl = 'http://127.0.0.1:8340';

  async init(): Promise<void> {
    const urls = [
      `${this.backendUrl}/api/auth/session`,
      `${this.frontendUrl}/api/auth/session`,
    ];
    for (const url of urls) {
      try {
        const res = await fetch(url, { credentials: 'include' });
        if (res.ok) {
          const data = await res.json();
          this.token = data.token || '';
          return;
        }
      } catch { continue; }
    }
  }

  getToken(): string {
    return this.token;
  }

  setToken(token: string): void {
    this.token = token;
  }

  authHeaders(): Record<string, string> {
    const h: Record<string, string> = { 'Content-Type': 'application/json' };
    if (this.token) {
      h['X-UMBRA-Key'] = this.token;
    }
    return h;
  }
}
