import { Component, OnInit } from '@angular/core';
import { CommonModule } from '@angular/common';
import { FormsModule } from '@angular/forms';
import { SettingsRepository } from '../../../../../data/repositories/settings.repository';

@Component({
  selector: 'app-settings-customization',
  standalone: true,
  imports: [CommonModule, FormsModule],
  templateUrl: './customization.component.html',
  styles: [`
    :host { display: block; }
  `],
})
export class CustomizationComponent implements OnInit {
  name = 'UMBRA';
  responseStyle = 'concise';
  persona = 'professional';

  constructor(private settingsRepo: SettingsRepository) {}

  ngOnInit(): void {
    this.load();
  }

  async load(): Promise<void> {
    const c = await this.settingsRepo.getCustomization();
    if (c) {
      this.name = c.name || 'UMBRA';
      this.responseStyle = c.response_style || 'concise';
      this.persona = c.persona || 'professional';
    }
  }

  async save(): Promise<void> {
    await this.settingsRepo.saveCustomization({
      name: this.name,
      response_style: this.responseStyle,
      persona: this.persona,
    });
  }
}
