import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SettingsRepository } from '../../../../../data/repositories/settings.repository';

@Component({
  selector: 'app-settings-preferences',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './preferences.component.html',
  styles: [`
    :host { display: block; }
  `],
})
export class PreferencesComponent implements OnInit {
  userName = '';
  honorific = 'sir';
  calendarAccounts = '';

  constructor(private settingsRepo: SettingsRepository) {}

  ngOnInit(): void {
    this.load();
  }

  async load(): Promise<void> {
    const p = await this.settingsRepo.getPreferences();
    if (p) {
      this.userName = p.user_name || '';
      this.honorific = p.honorific || 'sir';
      const accounts = Array.isArray(p.calendar_accounts) ? p.calendar_accounts.join(', ') : 'auto';
      this.calendarAccounts = accounts;
    }
  }

  async save(): Promise<void> {
    const raw = this.calendarAccounts.trim();
    const calendar_accounts = raw === '' || raw === 'auto' ? [] : raw.split(',').map(s => s.trim());
    await this.settingsRepo.savePreferences({
      user_name: this.userName,
      honorific: this.honorific,
      calendar_accounts,
    });
  }
}
