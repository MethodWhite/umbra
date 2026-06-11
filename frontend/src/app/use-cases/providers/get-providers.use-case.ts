import { Injectable } from '@angular/core';
import { ProviderRepository } from '../../data/repositories/provider.repository';
import { Provider, ProviderConfigStatus, ProviderCategory } from '../../domain/models/provider.model';
import { UseCase } from '../../domain/interfaces/use-case.interface';

export interface GetProvidersOutput {
  providers: Provider[];
  configStatus: ProviderConfigStatus | null;
}

@Injectable({ providedIn: 'root' })
export class GetProvidersUseCase implements UseCase<void, GetProvidersOutput> {
  constructor(private providerRepo: ProviderRepository) {}

  async execute(): Promise<GetProvidersOutput> {
    const [providers, configStatus] = await Promise.all([
      this.providerRepo.getAll(),
      this.providerRepo.getConfigStatus(),
    ]);
    return { providers, configStatus };
  }

  getCategory(provider: Provider): 'chinese' | 'local' | 'other' {
    return this.providerRepo.getProviderCategory(provider);
  }

  filterByCategory(providers: Provider[], category: ProviderCategory): Provider[] {
    if (category === 'all') return providers;
    return providers.filter(p => this.getCategory(p) === category);
  }
}
