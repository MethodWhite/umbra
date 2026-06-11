import { Injectable } from '@angular/core';
import { SettingsRepository, VoiceSettingsResponse } from '../../data/repositories/settings.repository';
import { UseCase } from '../../domain/interfaces/use-case.interface';

@Injectable({ providedIn: 'root' })
export class SaveVoiceSettingsUseCase implements UseCase<Partial<VoiceSettingsResponse>, void> {
  constructor(private settingsRepo: SettingsRepository) {}

  async execute(input: Partial<VoiceSettingsResponse>): Promise<void> {
    await this.settingsRepo.saveVoiceSettings(input);
  }
}
