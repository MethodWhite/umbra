import { Injectable } from '@angular/core';
import { VaultRepository } from '../../data/repositories/vault.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class LockVaultUseCase implements UseCase<void, void> {
  constructor(private vaultRepo: VaultRepository) {}

  async execute(): Promise<void> {
    await this.vaultRepo.lock();
  }
}
