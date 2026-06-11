export interface StatusResponseDto {
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

export interface PreferencesResponseDto {
  user_name: string;
  honorific: string;
  calendar_accounts: string[];
}

export interface VoiceSettingsResponseDto {
  stt_language: string;
  tts_engine: string;
  wake_word: string;
  voice_feedback: boolean;
}

export interface CustomizationResponseDto {
  name: string;
  greeting: string;
  response_style: string;
  persona: string;
}

export interface HardwareResponseDto {
  cpu: { name: string; usage: number; temperature: number; cores: number };
  gpu: { name: string; usage: number; temperature: number; memory_total: number; memory_used: number };
  ram: { total: number; used: number; available: number };
  vram: { total: number; used: number };
}

export interface TrainingResponseDto {
  active: boolean;
  current_epoch: number;
  total_epochs: number;
  loss: number;
  accuracy: number;
  learning_rate: number;
  dataset_size: number;
  started_at: string;
  estimated_completion: string;
}
