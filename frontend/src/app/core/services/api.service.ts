import { Injectable } from '@angular/core';
import { AuthService } from './auth.service';

interface TauriInvoke {
  (cmd: string, args?: Record<string, unknown>): Promise<unknown>;
}

@Injectable({ providedIn: 'root' })
export class ApiService {
  private backendUrl = 'http://127.0.0.1:8484';
  private frontendUrl = 'http://127.0.0.1:8340';
  private tauriInvoke: TauriInvoke | null = null;
  private isTauri = false;

  // Maps HTTP API paths to Tauri IPC commands
  private tauriRoutes: Record<string, string> = {
    'GET:/api/providers': 'get_providers',
    'GET:/api/providers/config/status': 'get_provider_config_status',
    'POST:/api/providers/test': 'test_provider',
    'POST:/api/providers/configure': 'configure_provider',
    'GET:/api/vault/status': 'get_vault_status',
    'POST:/api/vault/unlock': 'unlock_vault',
    'POST:/api/vault/lock': 'lock_vault',
    'POST:/api/vault/migrate': 'migrate_vault',
    'GET:/api/settings/status': 'get_system_status',
    'GET:/api/v1/hardware': 'get_hardware',
    'GET:/api/v1/models': 'get_models',
    'GET:/api/v1/sub-agents': 'get_sub_agents',
    'POST:/api/v1/training/train': 'trigger_training',
    'GET:/api/setup/status': 'get_setup_status',
    'GET:/api/v1/training': 'get_training_stats',
    'GET:/api/v1/debugger/snapshot': 'get_debugger_snapshot',
    'POST:/api/tts/synthesize': 'synthesize_speech',
  };

  constructor(private auth: AuthService) {
    this.detectEnvironment();
  }

  private tauriInvokeShim: TauriInvoke | null = null;

  private async detectEnvironment(): Promise<void> {
    // Check if running inside Tauri
    if (typeof window !== 'undefined' && (window as any).__TAURI__) {
      try {
        // Dynamic import - may fail at build time if package not installed, that's OK
        const tauriModule = await import('@tauri-apps/api/core');
        this.tauriInvoke = tauriModule.invoke as TauriInvoke;
        this.isTauri = true;
      } catch {
        this.isTauri = false;
      }
    }
  }

  private getRouteKey(method: string, path: string): string {
    const normalized = path.split('?')[0].replace(/\/+$/, '');
    return `${method}:${normalized}`;
  }

  async request<T>(path: string, options?: RequestInit): Promise<T> {
    if (this.isTauri && this.tauriInvoke) {
      const routeKey = this.getRouteKey(options?.method || 'GET', path);
      const command = this.tauriRoutes[routeKey];

      if (command) {
        // Extract args from path params (e.g., /api/vault/unlock -> { passphrase } from body)
        let args: Record<string, unknown> = {};
        if (options?.body) {
          try { args = { ...args, ...JSON.parse(options.body as string) }; } catch { /* ignore */ }
        }
        // Extract path params
        const pathMatch = path.match(/\/([^/]+)$/);
        if (pathMatch && command.includes('provider_id')) {
          args['providerId'] = pathMatch[1];
        }

        const result = await this.tauriInvoke(command, args);
        return result as T;
      }
    }

    // Fallback: HTTP
    const urls = this.makeUrl(path);
    const headers = { ...options?.headers, ...this.auth.authHeaders() };
    for (const url of urls) {
      try {
        const res = await fetch(url, { ...options, headers });
        if (res.ok) return res.json();
      } catch { continue; }
    }
    throw new Error(`Backend unreachable: ${path}`);
  }

  private makeUrl(path: string): string[] {
    const p = path.startsWith('/') ? path : `/${path}`;
    return [`${this.backendUrl}${p}`, `${this.frontendUrl}${p}`];
  }

  async get<T>(path: string): Promise<T> {
    return this.request<T>(path, { method: 'GET' });
  }

  async post<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>(path, {
      method: 'POST',
      body: body ? JSON.stringify(body) : undefined,
    });
  }

  async put<T>(path: string, body?: unknown): Promise<T> {
    return this.request<T>(path, {
      method: 'PUT',
      body: body ? JSON.stringify(body) : undefined,
    });
  }

  async del<T>(path: string): Promise<T> {
    return this.request<T>(path, { method: 'DELETE' });
  }
}
