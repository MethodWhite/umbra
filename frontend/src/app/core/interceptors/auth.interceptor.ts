import { Injectable } from '@angular/core';
import { AuthService } from '../services/auth.service';

@Injectable({ providedIn: 'root' })
export class AuthInterceptor {
  constructor(private auth: AuthService) {}

  intercept(url: string, options?: RequestInit): { url: string; options: RequestInit } {
    const headers = { ...options?.headers, ...this.auth.authHeaders() };
    return { url, options: { ...options, headers } };
  }
}
