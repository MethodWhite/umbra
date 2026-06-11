import { Injectable } from '@angular/core';
import { VaultRepository } from '../../data/repositories/vault.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

export interface MigrateKeysOutput {
  migrated: string[];
}

@Injectable({ providedIn: 'root' })
export class MigrateKeysUseCase implements UseCase<void, MigrateKeysOutput> {
  constructor(private vaultRepo: VaultRepository) {}

  async execute(): Promise<MigrateKeysOutput> {
    const migrated = await this.vaultRepo.migrateFromEnv();
    return { migrated };
  }
}
