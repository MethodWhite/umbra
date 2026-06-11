import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SettingsRepository } from '../../../../../data/repositories/settings.repository';

@Component({
  selector: 'app-settings-voice',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './voice.component.html',
  styles: [`
    :host { display: block; }
  `],
})
export class VoiceComponent implements OnInit {
  sttLanguage = 'en-US';
  ttsEngine = 'fish';
  wakeWord = 'umbra';
  voiceFeedback = true;

  constructor(private settingsRepo: SettingsRepository) {}

  ngOnInit(): void {
    this.load();
  }

  async load(): Promise<void> {
    const v = await this.settingsRepo.getVoiceSettings();
    if (v) {
      this.sttLanguage = v.stt_language || 'en-US';
      this.ttsEngine = v.tts_engine || 'fish';
      this.wakeWord = v.wake_word || 'umbra';
      this.voiceFeedback = v.voice_feedback !== false;
    }
  }

  async save(): Promise<void> {
    await this.settingsRepo.saveVoiceSettings({
      stt_language: this.sttLanguage,
      tts_engine: this.ttsEngine,
      wake_word: this.wakeWord,
      voice_feedback: this.voiceFeedback,
    });
  }
}
