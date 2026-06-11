import { Injectable } from '@angular/core';
import { SettingsRepository, PreferencesResponse } from '../../data/repositories/settings.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class SavePreferencesUseCase implements UseCase<Partial<PreferencesResponse>, void> {
  constructor(private settingsRepo: SettingsRepository) {}

  async execute(input: Partial<PreferencesResponse>): Promise<void> {
    await this.settingsRepo.savePreferences(input);
  }
}
