import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { RouterModule } from '@angular/router';
import { ApiService } from '../../../core/services/api.service';
import { I18nService, Language } from '../../../core/services/i18n.service';
import { ProviderRepository } from '../../../data/repositories/provider.repository';
import { VaultRepository } from '../../../data/repositories/vault.repository';
import { Provider, ProviderConfigStatus } from '../../../domain/models/provider.model';
import { HardwareStatus } from '../../../domain/models/hardware.model';

interface SetupStatus {
  ollama_detected: boolean;
  ollama_models: string[];
  llamacpp_detected: boolean;
  llamacpp_models: string[];
  other_models: string[];
}

interface SubAgentData {
  id: string;
  name: string;
  status: string;
  enabled: boolean;
  task?: string;
  uptime?: number;
}

interface VaultStatus {
  locked: boolean;
  key_count: number;
  providers_with_keys: { id: string }[];
  auto_lock_remaining: number;
}

type TabId = 'vault' | 'providers' | 'models' | 'agents' | 'hardware';

@Component({
  selector: 'app-settings',
  standalone: true,
  imports: [CommonModule, FormsModule, RouterModule],
  templateUrl: './settings.component.html',
  styleUrls: ['./settings.component.scss'],
})
export class SettingsComponent implements OnInit, OnDestroy {
  currentTab: TabId = 'vault';
  currentLang: Language = 'en';
  currentProviderCategory: 'all' | 'chinese' | 'local' | 'other' = 'all';

  vaultStatus: VaultStatus | null = null;
  passphrase = '';
  vaultTestResults: Record<string, boolean> = {};

  providers: Provider[] = [];
  configStatus: ProviderConfigStatus | null = null;
  providerKeys: Record<string, string> = {};
  providerTesting = new Set<string>();
  providerTested = new Map<string, boolean>();

  setupStatus: SetupStatus | null = null;
  primaryModel = '';
  secondaryModel = '';
  allDetectedModels: string[] = [];
  modelStatus: string = '';

  agents: SubAgentData[] = [];
  agentTesting = new Set<string>();

  hardware: HardwareStatus | null = null;

  migrateMessage = '';
  migrateSuccess = false;

  private pollTimer: ReturnType<typeof setInterval> | null = null;

  readonly tabs: { id: TabId; labelKey: string }[] = [
    { id: 'vault', labelKey: 'settings.tab.vault' },
    { id: 'providers', labelKey: 'settings.tab.providers' },
    { id: 'models', labelKey: 'settings.tab.models' },
    { id: 'agents', labelKey: 'settings.tab.agents' },
    { id: 'hardware', labelKey: 'settings.tab.hardware' },
  ];

  readonly providerCategoryTabs: { id: 'all' | 'chinese' | 'local' | 'other'; label: string }[] = [
    { id: 'all', label: 'All' },
    { id: 'chinese', label: 'CHINESE' },
    { id: 'local', label: 'LOCAL' },
    { id: 'other', label: 'WESTERN' },
  ];

  constructor(
    public i18n: I18nService,
    private api: ApiService,
    private providerRepo: ProviderRepository,
    private vaultRepo: VaultRepository,
  ) {}

  ngOnInit(): void {
    this.i18n.currentLang$.subscribe(l => this.currentLang = l);
    this.loadAll();
    this.pollTimer = setInterval(() => this.poll(), 10000);
  }

  ngOnDestroy(): void {
    if (this.pollTimer) clearInterval(this.pollTimer);
  }

  private async loadAll(): Promise<void> {
    await Promise.all([
      this.loadVaultStatus(),
      this.loadProviders(),
      this.loadSetupStatus(),
      this.loadAgents(),
      this.loadHardware(),
    ]);
  }

  private async poll(): Promise<void> {
    if (this.currentTab === 'hardware') await this.loadHardware();
    if (this.currentTab === 'agents') await this.loadAgents();
  }

  switchTab(tab: TabId): void {
    this.currentTab = tab;
  }

  setProviderCategory(cat: 'all' | 'chinese' | 'local' | 'other'): void {
    this.currentProviderCategory = cat;
  }

  get filteredProviders(): Provider[] {
    if (this.currentProviderCategory === 'all') return this.providers;
    return this.providers.filter(p =>
      this.providerRepo.getProviderCategory(p) === this.currentProviderCategory
    );
  }

  getCategoryBadge(provider: Provider): string {
    const cat = this.providerRepo.getProviderCategory(provider);
    if (cat === 'chinese') return 'CHINESE';
    if (cat === 'local') return 'LOCAL';
    return 'WESTERN';
  }

  isProviderConfigured(id: string): boolean {
    return this.configStatus?.configured_providers?.some(c => c.id === id && c.has_key) || false;
  }

  isProviderTesting(id: string): boolean {
    return this.providerTesting.has(id);
  }

  getProviderTestResult(id: string): boolean | undefined {
    return this.providerTested.get(id);
  }

  private async loadVaultStatus(): Promise<void> {
    const status = await this.vaultRepo.getStatus();
    if (status) {
      this.vaultStatus = {
        locked: status.locked,
        key_count: status.key_count,
        providers_with_keys: status.providers_with_keys || [],
        auto_lock_remaining: status.auto_lock_remaining,
      };
    }
  }

  async unlockVault(): Promise<void> {
    if (!this.vaultStatus?.locked) return;
    const ok = await this.vaultRepo.unlock(this.passphrase || undefined);
    if (ok) {
      this.passphrase = '';
      await this.loadVaultStatus();
    }
  }

  async lockVault(): Promise<void> {
    if (this.vaultStatus?.locked) return;
    await this.vaultRepo.lock();
    this.vaultTestResults = {};
    await this.loadVaultStatus();
  }

  async migrateVault(): Promise<void> {
    const migrated = await this.vaultRepo.migrateFromEnv();
    if (migrated.length > 0) {
      this.migrateMessage = `Migrated: ${migrated.join(', ')}`;
      this.migrateSuccess = true;
      await this.loadVaultStatus();
    } else {
      this.migrateMessage = 'No keys to migrate';
      this.migrateSuccess = false;
    }
    setTimeout(() => this.migrateMessage = '', 5000);
  }

  async testVaultProvider(id: string): Promise<void> {
    const result = await this.providerRepo.testProvider(id);
    this.vaultTestResults[id] = result.valid;
  }

  private async loadProviders(): Promise<void> {
    const [providers, configStatus] = await Promise.all([
      this.providerRepo.getAll(),
      this.providerRepo.getConfigStatus(),
    ]);
    this.providers = providers;
    this.configStatus = configStatus;
  }

  async testProvider(id: string): Promise<void> {
    this.providerTesting.add(id);
    const apiKey = this.providerKeys[id]?.trim();
    try {
      const result = await this.providerRepo.testProvider(id, apiKey || undefined);
      this.providerTested.set(id, result.valid);
    } catch {
      this.providerTested.set(id, false);
    } finally {
      this.providerTesting.delete(id);
    }
  }

  async saveProvider(id: string): Promise<void> {
    const apiKey = this.providerKeys[id]?.trim();
    await this.providerRepo.configureProvider(id, {
      api_key: apiKey || undefined,
    });
    this.providerTested.set(id, true);
    await this.loadProviders();
  }

  private async loadSetupStatus(): Promise<void> {
    try {
      this.setupStatus = await this.api.get<SetupStatus>('/api/setup/status');
      if (this.setupStatus) {
        this.allDetectedModels = [
          ...(this.setupStatus.ollama_models || []),
          ...(this.setupStatus.llamacpp_models || []),
          ...(this.setupStatus.other_models || []),
        ];
      }
    } catch {
      this.setupStatus = null;
    }
  }

  async selectPrimaryModel(model: string): Promise<void> {
    this.primaryModel = model;
    this.modelStatus = `Primary model set to ${model}`;
    setTimeout(() => this.modelStatus = '', 3000);
  }

  async selectSecondaryModel(model: string): Promise<void> {
    this.secondaryModel = model;
    this.modelStatus = `Secondary model set to ${model}`;
    setTimeout(() => this.modelStatus = '', 3000);
  }

  private async loadAgents(): Promise<void> {
    try {
      this.agents = await this.api.get<SubAgentData[]>('/api/v1/sub-agents');
    } catch {
      this.agents = [];
    }
  }

  async toggleAgent(agent: SubAgentData): Promise<void> {
    this.agentTesting.add(agent.id);
    try {
      await this.api.post(`/api/v1/sub-agents/${agent.id}/toggle`, { enabled: !agent.enabled });
      agent.enabled = !agent.enabled;
    } catch {
    } finally {
      this.agentTesting.delete(agent.id);
    }
  }

  getActiveAgent(): SubAgentData | undefined {
    return this.agents.find(a => a.status === 'running' || a.enabled);
  }

  private async loadHardware(): Promise<void> {
    try {
      this.hardware = await this.api.get<HardwareStatus>('/api/v1/hardware');
    } catch {
      this.hardware = null;
    }
  }

  getRamPercent(): number {
    if (!this.hardware?.ram?.total) return 0;
    return Math.round((this.hardware.ram.used / this.hardware.ram.total) * 100);
  }

  getVramPercent(): number {
    if (!this.hardware?.vram?.total) return 0;
    return Math.round((this.hardware.vram.used / this.hardware.vram.total) * 100);
  }

  getRecommendedQuantization(): string {
    if (!this.hardware) return '--';
    const totalVramGB = (this.hardware.vram?.total || 0) / (1024 * 1024 * 1024);
    if (totalVramGB >= 24) return 'Q4_K_M / Q5_K_M';
    if (totalVramGB >= 12) return 'Q4_K_M';
    if (totalVramGB >= 8) return 'Q3_K_M';
    if (totalVramGB >= 4) return 'Q2_K';
    return 'IQ2_XXS / Q2_K';
  }

  formatBytes(bytes: number): string {
    if (!bytes) return '--';
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  setLanguage(lang: Language): void {
    this.i18n.setLanguage(lang);
  }
}
