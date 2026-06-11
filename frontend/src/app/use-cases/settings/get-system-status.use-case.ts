import { Injectable } from '@angular/core';
import { SettingsRepository, StatusResponse } from '../../data/repositories/settings.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class GetSystemStatusUseCase implements UseCase<void, StatusResponse | null> {
  constructor(private settingsRepo: SettingsRepository) {}

  async execute(): Promise<StatusResponse | null> {
    return this.settingsRepo.getSystemStatus();
  }
}
