import { Injectable } from '@angular/core';
import { ApiService } from '../../core/services/api.service';
import { HardwareStatus } from '../../domain/models/hardware.model';
import type { HardwareResponseDto } from '../dto/settings.dto';

export interface MonitorModelInfo {
  id: string;
  name: string;
  type: 'primary' | 'secondary' | 'local';
  status: 'loaded' | 'loading' | 'unloaded';
  provider: string;
}

export interface MonitorSubAgent {
  id: string;
  name: string;
  status: 'running' | 'idle' | 'error';
  task: string;
  uptime: number;
}

export interface MonitorDebuggerInfo {
  health: 'healthy' | 'degraded' | 'down';
  latency_ms: number;
  error_rate: number;
  uptime: number;
}

@Injectable({ providedIn: 'root' })
export class HardwareRepository {
  constructor(private api: ApiService) {}

  async getHardwareStatus(): Promise<HardwareStatus | null> {
    try {
      return await this.api.get<HardwareResponseDto>('/api/monitor/hardware');
    } catch { return null; }
  }

  async getModels(): Promise<MonitorModelInfo[]> {
    try {
      return await this.api.get<MonitorModelInfo[]>('/api/monitor/models');
    } catch { return []; }
  }

  async getSubAgents(): Promise<MonitorSubAgent[]> {
    try {
      return await this.api.get<MonitorSubAgent[]>('/api/monitor/agents');
    } catch { return []; }
  }

  async getDebuggerInfo(): Promise<MonitorDebuggerInfo | null> {
    try {
      return await this.api.get<MonitorDebuggerInfo>('/api/monitor/debugger');
    } catch { return null; }
  }

  async getTrainingStatus(): Promise<{ active: boolean; current_epoch: number; total_epochs: number; loss: number; accuracy: number; dataset_size: number } | null> {
    try {
      return await this.api.get('/api/monitor/training');
    } catch { return null; }
  }

  async triggerTraining(): Promise<void> {
    try { await this.api.post('/api/monitor/training/trigger'); } catch {}
  }
}
