import { Injectable } from '@angular/core';
import { ApiService } from '../../core/services/api.service';
import type { VaultStatusDto, VaultMigrateDto } from '../dto/vault.dto';

export interface VaultStatus {
  locked: boolean;
  key_count: number;
  auto_lock_remaining: number;
  providers_with_keys: { id: string }[];
}

@Injectable({ providedIn: 'root' })
export class VaultRepository {
  constructor(private api: ApiService) {}

  async getStatus(): Promise<VaultStatus | null> {
    try {
      return await this.api.get<VaultStatusDto>('/api/vault/status');
    } catch { return null; }
  }

  async unlock(passphrase?: string): Promise<boolean> {
    try {
      const res = await this.api.post<{ success: boolean }>('/api/vault/unlock', { passphrase });
      return res.success;
    } catch { return false; }
  }

  async lock(): Promise<void> {
    return this.api.post('/api/vault/lock');
  }

  async migrateFromEnv(): Promise<string[]> {
    try {
      const res = await this.api.post<VaultMigrateDto>('/api/vault/migrate');
      return res.migrated || [];
    } catch { return []; }
  }

  async setAutoLock(minutes: number): Promise<void> {
    return this.api.post('/api/vault/auto-lock', { minutes });
  }
}
