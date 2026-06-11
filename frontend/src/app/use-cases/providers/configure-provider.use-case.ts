import { Injectable } from '@angular/core';
import { ProviderRepository } from '../../data/repositories/provider.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

export interface ConfigureProviderInput {
  providerId: string;
  apiKey?: string;
  baseUrl?: string;
  isPrimary?: boolean;
  isSecondary?: boolean;
  model?: string;
}

@Injectable({ providedIn: 'root' })
export class ConfigureProviderUseCase implements UseCase<ConfigureProviderInput, void> {
  constructor(private providerRepo: ProviderRepository) {}

  async execute(input: ConfigureProviderInput): Promise<void> {
    await this.providerRepo.configureProvider(input.providerId, {
      api_key: input.apiKey,
      base_url: input.baseUrl,
      is_primary: input.isPrimary,
      is_secondary: input.isSecondary,
      model: input.model,
    });
  }
}
