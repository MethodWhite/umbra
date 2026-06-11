import { Injectable } from '@angular/core';
import { ProviderRepository } from '../../data/repositories/provider.repository';
import { ProviderTestResult } from '../../domain/models/provider.model';
import { UseCase } from '../../domain/interfaces/use-case.interface';

export interface TestProviderInput {
  providerId: string;
  apiKey?: string;
  baseUrl?: string;
}

@Injectable({ providedIn: 'root' })
export class TestProviderUseCase implements UseCase<TestProviderInput, ProviderTestResult> {
  constructor(private providerRepo: ProviderRepository) {}

  async execute(input: TestProviderInput): Promise<ProviderTestResult> {
    return this.providerRepo.testProvider(input.providerId, input.apiKey, input.baseUrl);
  }
}
