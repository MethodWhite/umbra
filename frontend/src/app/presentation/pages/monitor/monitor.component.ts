import { Component, OnInit, OnDestroy } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { HardwareRepository, MonitorModelInfo, MonitorSubAgent, MonitorDebuggerInfo } from '../../../data/repositories/hardware.repository';
import { TrainingRepository } from '../../../data/repositories/training.repository';
import { HardwareStatus } from '../../../domain/models/hardware.model';
import { TrainingStatus } from '../../../domain/models/training.model';

@Component({
  selector: 'app-monitor',
  standalone: true,
  imports: [CommonModule, RouterModule],
  templateUrl: './monitor.component.html',
  styleUrls: ['./monitor.component.scss'],
})
export class MonitorComponent implements OnInit, OnDestroy {
  hardware: HardwareStatus | null = null;
  models: MonitorModelInfo[] = [];
  agents: MonitorSubAgent[] = [];
  training: TrainingStatus | null = null;
  debugger: MonitorDebuggerInfo | null = null;
  private intervalId: any;

  constructor(
    private hardwareRepo: HardwareRepository,
    private trainingRepo: TrainingRepository,
  ) {}

  ngOnInit(): void {
    this.loadAll();
    this.intervalId = setInterval(() => this.loadAll(), 5000);
  }

  ngOnDestroy(): void {
    if (this.intervalId) clearInterval(this.intervalId);
  }

  async loadAll(): Promise<void> {
    await Promise.all([
      this.loadHardware(),
      this.loadModels(),
      this.loadAgents(),
      this.loadTraining(),
      this.loadDebugger(),
    ]);
  }

  async loadHardware(): Promise<void> {
    this.hardware = await this.hardwareRepo.getHardwareStatus();
  }

  async loadModels(): Promise<void> {
    this.models = await this.hardwareRepo.getModels();
  }

  async loadAgents(): Promise<void> {
    this.agents = await this.hardwareRepo.getSubAgents();
  }

  async loadTraining(): Promise<void> {
    this.training = await this.trainingRepo.getStatus();
  }

  async loadDebugger(): Promise<void> {
    this.debugger = await this.hardwareRepo.getDebuggerInfo();
  }

  async triggerTraining(): Promise<void> {
    await this.trainingRepo.trigger();
  }

  formatBytes(bytes: number): string {
    if (!bytes) return '--';
    const gb = bytes / (1024 * 1024 * 1024);
    return `${gb.toFixed(1)} GB`;
  }

  formatUptime(seconds: number): string {
    if (!seconds) return '--';
    if (seconds < 60) return `${Math.floor(seconds)}s`;
    if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
    const h = Math.floor(seconds / 3600);
    const m = Math.floor((seconds % 3600) / 60);
    return `${h}h ${m}m`;
  }
}
