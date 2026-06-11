import { Injectable } from '@angular/core';
import { ApiService } from '../../core/services/api.service';
import { TrainingStatus } from '../../domain/models/training.model';
import type { TrainingResponseDto } from '../dto/settings.dto';

@Injectable({ providedIn: 'root' })
export class TrainingRepository {
  constructor(private api: ApiService) {}

  async getStatus(): Promise<TrainingStatus | null> {
    try {
      return await this.api.get<TrainingResponseDto>('/api/monitor/training');
    } catch { return null; }
  }

  async trigger(): Promise<void> {
    await this.api.post('/api/monitor/training/trigger');
  }
}
