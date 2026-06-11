import { Component, OnInit, Input, Output, EventEmitter } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { Provider, ProviderConfigStatus, ProviderCategory } from '../../../../../domain/models/provider.model';
import { ProviderRepository } from '../../../../../data/repositories/provider.repository';
import { TestProviderUseCase } from '../../../../../use-cases/providers/test-provider.use-case';
import { ConfigureProviderUseCase } from '../../../../../use-cases/providers/configure-provider.use-case';

@Component({
  selector: 'app-settings-providers',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './providers.component.html',
  styles: [`
    :host { display: block; }
    .provider-tabs { display:flex; gap:4px; margin-bottom:12px; }
    .provider-tab {
      padding:6px 14px; border:1px solid transparent; border-radius:6px 6px 0 0;
      background:transparent; color:rgba(255,255,255,0.3); font-size:11px;
      letter-spacing:1px; cursor:pointer; transition:all 0.2s;
      text-transform:uppercase; font-family:inherit;
    }
    .provider-tab:hover { color:rgba(255,255,255,0.5); background:rgba(255,255,255,0.03); }
    .provider-tab.active { color:rgba(0,212,255,0.85); border-color:rgba(0,212,255,0.15) rgba(0,212,255,0.15) transparent; background:rgba(0,212,255,0.04); }
    .provider-list { display:flex; flex-direction:column; gap:10px; max-height:340px; overflow-y:auto; }
    .provider-card {
      border:1px solid rgba(255,255,255,0.06); border-radius:8px;
      background:rgba(255,255,255,0.02); overflow:hidden;
      transition:border-color 0.2s;
    }
    .provider-card:hover { border-color:rgba(0,212,255,0.12); }
    .provider-card-header {
      display:flex; justify-content:space-between; align-items:center;
      padding:10px 14px; background:rgba(255,255,255,0.02);
      border-bottom:1px solid rgba(255,255,255,0.04);
    }
    .provider-card-body { padding:10px 14px; }
    .provider-card-body .settings-field { margin-bottom:8px; }
    .provider-card-body .settings-field:last-of-type { margin-bottom:0; }
    .provider-card-actions { display:flex; gap:8px; margin-top:10px; justify-content:flex-end; }
    .provider-name { font-size:13px; color:rgba(255,255,255,0.7); font-weight:500; }
    .model-config-row { display:flex; gap:8px; align-items:center; margin-bottom:10px; }
    .model-config-row select { flex:1; }
    .empty-state { text-align:center; padding:24px 0; color:rgba(255,255,255,0.2); font-size:13px; }
    .status-dot { width:8px; height:8px; border-radius:50%; background:rgba(255,255,255,0.15); transition:background 0.3s; flex-shrink:0; }
    .status-green { background:#22c55e; box-shadow:0 0 6px rgba(34,197,94,0.4); }
    .status-yellow { background:#eab308; box-shadow:0 0 6px rgba(234,179,8,0.4); animation:pulse 1s ease-in-out infinite; }
    .status-red { background:#ef4444; box-shadow:0 0 6px rgba(239,68,68,0.4); }
    @keyframes pulse { 0%,100% { opacity:1; } 50% { opacity:0.4; } }
  `],
})
export class ProvidersComponent implements OnInit {
  @Input() providers: Provider[] = [];
  @Input() configStatus: ProviderConfigStatus | null = null;

  currentCategory: ProviderCategory = 'all';
  keys: Record<string, string> = {};
  urls: Record<string, string> = {};
  testing = new Set<string>();
  tested = new Map<string, boolean>();
  primaryProvider = '';
  primaryModel = '';
  secondaryProvider = '';
  secondaryModel = '';
  primaryModels: string[] = [];
  secondaryModels: string[] = [];

  tabs = [
    { id: 'all' as ProviderCategory, label: 'All' },
    { id: 'chinese' as ProviderCategory, label: 'Chinese' },
    { id: 'local' as ProviderCategory, label: 'Local' },
  ];

  constructor(
    private providerRepo: ProviderRepository,
    private testProviderUseCase: TestProviderUseCase,
    private configureProvider: ConfigureProviderUseCase,
  ) {}

  ngOnInit(): void {
    if (this.configStatus?.primary) {
      this.primaryProvider = this.configStatus.primary.provider_id;
      this.primaryModel = this.configStatus.primary.model;
      this.onPrimaryChange();
    }
    if (this.configStatus?.secondary) {
      this.secondaryProvider = this.configStatus.secondary.provider_id;
      this.secondaryModel = this.configStatus.secondary.model;
      this.onSecondaryChange();
    }
  }

  get filteredProviders(): Provider[] {
    if (this.currentCategory === 'all') return this.providers;
    return this.providers.filter(p => this.providerRepo.getProviderCategory(p) === this.currentCategory);
  }

  setCategory(cat: ProviderCategory): void {
    this.currentCategory = cat;
  }

  isConfigured(id: string): boolean {
    return this.configStatus?.configured_providers?.some(c => c.id === id && c.has_key) || false;
  }

  async testProvider(id: string): Promise<void> {
    this.testing.add(id);
    const apiKey = this.keys[id]?.trim();
    const baseUrl = this.urls[id]?.trim();
    try {
      const result = await this.testProviderUseCase.execute({ providerId: id, apiKey, baseUrl });
      this.tested.set(id, result.valid);
    } catch {
      this.tested.set(id, false);
    } finally {
      this.testing.delete(id);
    }
  }

  async saveProvider(id: string): Promise<void> {
    const apiKey = this.keys[id]?.trim();
    const baseUrl = this.urls[id]?.trim();
    await this.configureProvider.execute({ providerId: id, apiKey, baseUrl });
    this.tested.set(id, true);
  }

  onPrimaryChange(): void {
    const p = this.providers.find(x => x.id === this.primaryProvider);
    this.primaryModels = p?.models || [];
    this.primaryModel = '';
  }

  onSecondaryChange(): void {
    const p = this.providers.find(x => x.id === this.secondaryProvider);
    this.secondaryModels = p?.models || [];
    this.secondaryModel = '';
  }

  async saveModelConfig(): Promise<void> {
    if (this.primaryProvider && this.primaryModel) {
      await this.configureProvider.execute({ providerId: this.primaryProvider, isPrimary: true, model: this.primaryModel });
    }
    if (this.secondaryProvider && this.secondaryModel) {
      await this.configureProvider.execute({ providerId: this.secondaryProvider, isSecondary: true, model: this.secondaryModel });
    }
  }
}
