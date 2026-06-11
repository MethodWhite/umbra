import { Injectable } from '@angular/core';
import { ApiService } from '../../core/services/api.service';
import { Provider, ProviderConfigStatus, ProviderTestResult } from '../../domain/models/provider.model';
import type { IRepository } from '../../domain/interfaces/repository.interface';
import type {
  ProviderListDto,
  ProviderConfigStatusDto,
  ProviderTestResultDto,
  TestAllKeysResultDto,
} from '../dto/provider.dto';

@Injectable({ providedIn: 'root' })
export class ProviderRepository implements IRepository<Provider> {
  constructor(private api: ApiService) {}

  async getAll(): Promise<Provider[]> {
    try {
      const data = await this.api.get<ProviderListDto>('/api/providers');
      return data.providers || [];
    } catch { return []; }
  }

  async getById(id: string): Promise<Provider | null> {
    const providers = await this.getAll();
    return providers.find(p => p.id === id) || null;
  }

  async create(item: Partial<Provider>): Promise<Provider> {
    throw new Error('Not implemented');
  }

  async update(id: string, item: Partial<Provider>): Promise<Provider> {
    throw new Error('Not implemented');
  }

  async delete(id: string): Promise<void> {
    throw new Error('Not implemented');
  }

  async getConfigStatus(): Promise<ProviderConfigStatus | null> {
    try {
      return await this.api.get<ProviderConfigStatusDto>('/api/providers/config/status');
    } catch { return null; }
  }

  async testProvider(providerId: string, apiKey?: string, baseUrl?: string): Promise<ProviderTestResult> {
    return this.api.post<ProviderTestResultDto>('/api/providers/test', {
      provider_id: providerId,
      api_key: apiKey || undefined,
      base_url: baseUrl || undefined,
    });
  }

  async configureProvider(providerId: string, config: {
    api_key?: string; base_url?: string;
    is_primary?: boolean; is_secondary?: boolean; model?: string;
  }): Promise<void> {
    return this.api.post('/api/providers/configure', {
      provider_id: providerId,
      ...config,
    });
  }

  async testAllKeys(): Promise<Record<string, boolean>> {
    try {
      const res = await this.api.post<TestAllKeysResultDto>('/api/providers/test-all');
      const results: Record<string, boolean> = {};
      if (res.results) {
        for (const [pid, r] of Object.entries(res.results)) {
          results[pid] = r.valid === true;
        }
      }
      return results;
    } catch { return {}; }
  }

  getProviderCategory(provider: Provider): 'chinese' | 'local' | 'other' {
    const knownChinese = [
      'deepseek', 'moonshot', 'qwen', 'baidu', 'zhipu', 'glm',
      'minimax', 'stepfun', 'hunyuan', 'doubao', 'spark', 'wenxin',
      'ernie', 'yi', '01ai', 'lingyi', 'baichuan', 'internlm',
      'xverse', 'kimi', 'chatglm',
    ];
    const id = provider.id.toLowerCase();
    const name = provider.name.toLowerCase();
    const url = (provider.base_url || '').toLowerCase();

    if (knownChinese.some(k => id.includes(k) || name.includes(k)) || url.includes('.cn')) return 'chinese';
    if (url.includes('localhost') || url.includes('127.0.0.1') || url.includes('0.0.0.0') ||
        provider.api_type === 'ollama' || provider.api_type === 'local' || provider.api_type === 'llama.cpp') return 'local';
    return 'other';
  }
}
