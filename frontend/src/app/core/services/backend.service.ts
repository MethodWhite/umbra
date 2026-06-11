import { Injectable } from '@angular/core';
import { ApiService } from './api.service';

@Injectable({ providedIn: 'root' })
export class BackendService {
  constructor(private api: ApiService) {}

  async restartServer(): Promise<void> {
    return this.api.post('/api/restart');
  }

  async getHealth(): Promise<boolean> {
    try {
      await this.api.get('/api/health');
      return true;
    } catch {
      return false;
    }
  }
}
