import { Injectable } from '@angular/core';
import { VaultRepository } from '../../data/repositories/vault.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

export interface UnlockVaultInput {
  passphrase?: string;
}

@Injectable({ providedIn: 'root' })
export class UnlockVaultUseCase implements UseCase<UnlockVaultInput, boolean> {
  constructor(private vaultRepo: VaultRepository) {}

  async execute(input: UnlockVaultInput): Promise<boolean> {
    return this.vaultRepo.unlock(input.passphrase);
  }
}
