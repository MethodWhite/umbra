import { Injectable } from '@angular/core';
import { ApiService } from '../../core/services/api.service';
import type {
  StatusResponseDto,
  PreferencesResponseDto,
  VoiceSettingsResponseDto,
  CustomizationResponseDto,
} from '../dto/settings.dto';

export interface StatusResponse {
  claude_code_installed: boolean;
  calendar_accessible: boolean;
  mail_accessible: boolean;
  notes_accessible: boolean;
  memory_count: number;
  task_count: number;
  server_port: number;
  uptime_seconds: number;
  keys_configured: boolean;
}

export interface PreferencesResponse {
  user_name: string;
  honorific: string;
  calendar_accounts: string[];
}

export interface VoiceSettingsResponse {
  stt_language: string;
  tts_engine: string;
  wake_word: string;
  voice_feedback: boolean;
}

export interface CustomizationResponse {
  name: string;
  greeting: string;
  response_style: string;
  persona: string;
}

@Injectable({ providedIn: 'root' })
export class SettingsRepository {
  constructor(private api: ApiService) {}

  async getSystemStatus(): Promise<StatusResponse | null> {
    try {
      return await this.api.get<StatusResponseDto>('/api/settings/status');
    } catch { return null; }
  }

  async getPreferences(): Promise<PreferencesResponse | null> {
    try {
      return await this.api.get<PreferencesResponseDto>('/api/settings/preferences');
    } catch { return null; }
  }

  async savePreferences(prefs: Partial<PreferencesResponse>): Promise<void> {
    return this.api.post('/api/settings/preferences', prefs);
  }

  async getVoiceSettings(): Promise<VoiceSettingsResponse | null> {
    try {
      return await this.api.get<VoiceSettingsResponseDto>('/api/settings/voice');
    } catch { return null; }
  }

  async saveVoiceSettings(voice: Partial<VoiceSettingsResponse>): Promise<void> {
    return this.api.post('/api/settings/voice', voice);
  }

  async getCustomization(): Promise<CustomizationResponse | null> {
    try {
      return await this.api.get<CustomizationResponseDto>('/api/customization');
    } catch { return null; }
  }

  async saveCustomization(custom: Partial<CustomizationResponse>): Promise<void> {
    return this.api.post('/api/customization', custom);
  }
}
