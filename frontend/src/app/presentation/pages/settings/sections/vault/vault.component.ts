import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { VaultRepository } from '../../../../../data/repositories/vault.repository';
import { ProviderRepository } from '../../../../../data/repositories/provider.repository';
import { UnlockVaultUseCase } from '../../../../../use-cases/vault/unlock-vault.use-case';
import { LockVaultUseCase } from '../../../../../use-cases/vault/lock-vault.use-case';
import { MigrateKeysUseCase } from '../../../../../use-cases/vault/migrate-keys.use-case';

@Component({
  selector: 'app-settings-vault',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './vault.component.html',
  styles: [`
    :host { display: block; }
    .vault-status-bar { display:flex; align-items:center; gap:12px; margin-bottom:12px; }
    .vault-icon { font-size:14px; color:rgba(0,212,255,0.8); letter-spacing:1px; }
    .vault-key-count { color:rgba(255,255,255,0.3); font-size:12px; }
    .vault-actions { display:flex; gap:8px; flex-wrap:wrap; margin-bottom:12px; }
    .vault-keys-list { margin-top:12px; }
    .vault-key-row {
      display:grid; grid-template-columns:2fr 1fr; gap:8px;
      padding:6px 4px; border-bottom:1px solid rgba(255,255,255,0.04);
      font-size:13px;
    }
    .key-status.valid { color:#22c55e; }
    .key-status.invalid { color:#ef4444; }
    .vault-keys-empty { color:rgba(255,255,255,0.2); padding:12px 0; font-size:13px; text-align:center; }
  `],
})
export class VaultComponent implements OnInit {
  status: { locked: boolean; key_count: number; auto_lock_remaining: number; providers_with_keys: { id: string }[] } | null = null;
  passphrase = '';
  autoLockMinutes = 15;
  testResults: Record<string, boolean> = {};

  constructor(
    private vaultRepo: VaultRepository,
    private providerRepo: ProviderRepository,
    private unlockVault: UnlockVaultUseCase,
    private lockVault: LockVaultUseCase,
    private migrateKeys: MigrateKeysUseCase,
  ) {}

  ngOnInit(): void {
    this.load();
  }

  async load(): Promise<void> {
    this.status = await this.vaultRepo.getStatus();
  }

  async unlock(): Promise<void> {
    const ok = await this.unlockVault.execute({ passphrase: this.passphrase || undefined });
    if (ok) { this.passphrase = ''; await this.load(); }
  }

  async lock(): Promise<void> {
    await this.lockVault.execute();
    this.testResults = {};
    await this.load();
  }

  async migrate(): Promise<void> {
    const result = await this.migrateKeys.execute();
    if (result.migrated.length) await this.load();
  }

  async testAll(): Promise<void> {
    this.testResults = await this.providerRepo.testAllKeys();
  }

  async setAutoLock(): Promise<void> {
    await this.vaultRepo.setAutoLock(this.autoLockMinutes);
  }
}
