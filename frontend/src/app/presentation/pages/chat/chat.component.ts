import { Component, OnInit, OnDestroy, NgZone, Output, EventEmitter, ViewChild, ElementRef } from '@angular/core';
import { CommonModule } from '@angular/common';
import { RouterModule } from '@angular/router';
import { HudComponent } from '../../components/hud/hud.component';
import { HudState } from '../../components/hud/hud.types';
import { WebSocketService } from '../../../core/services/websocket.service';
import { AuthService } from '../../../core/services/auth.service';
import { ApiService } from '../../../core/services/api.service';
import { Subject, takeUntil } from 'rxjs';

interface Message {
  text: string;
  from: 'user' | 'umbra';
  timestamp: number;
}

@Component({
  selector: 'app-chat',
  standalone: true,
  imports: [CommonModule, RouterModule, HudComponent],
  templateUrl: './chat.component.html',
  styleUrls: ['./chat.component.scss'],
})
export class ChatComponent implements OnInit, OnDestroy {
  @Output() hudStateChange = new EventEmitter<HudState>();
  @ViewChild('messageLog') messageLog!: ElementRef<HTMLDivElement>;

  hudState: HudState = 'idle';
  messages: Message[] = [];
  isMuted = false;
  statusText = '';
  menuOpen = false;
  audioData: Uint8Array | null = null;

  primaryModel = '';
  secondaryModel = '';
  activeAgents: Array<{ name: string; active: boolean }> = [];
  agentState: HudState | 'idle' = 'idle';
  transcript = '';
  currentTime = new Date();

  private audioCtx: AudioContext | null = null;
  private analyser: AnalyserNode | null = null;
  private source: MediaStreamAudioSourceNode | null = null;
  private recognition: any = null;
  private shouldListen = false;
  private paused = false;
  private destroy$ = new Subject<void>();
  private animFrameId = 0;
  private statusMap: Record<string, string> = {
    idle: 'STANDBY',
    listening: 'LISTENING',
    'agent-thinking': 'PROCESSING',
    'sub-agent-working': 'EXECUTING',
    speaking: 'SPEAKING',
  };
  private clockInterval: ReturnType<typeof setInterval> | null = null;

  get isListening(): boolean {
    return this.shouldListen && !this.paused;
  }

  constructor(
    private ws: WebSocketService,
    private auth: AuthService,
    private api: ApiService,
    private ngZone: NgZone,
  ) {}

  async ngOnInit(): Promise<void> {
    await this.loadModels();
    await this.loadAgents();
    this.initAudio();
    this.ws.connect();

    this.ws.message$.pipe(takeUntil(this.destroy$)).subscribe(msg => {
      this.ngZone.run(() => this.handleMessage(msg));
    });

    this.clockInterval = setInterval(() => {
      this.currentTime = new Date();
    }, 1000);

    setTimeout(() => {
      this.startVoice();
      this.transition('listening');
    }, 1000);
  }

  ngOnDestroy(): void {
    this.destroy$.next();
    this.destroy$.complete();
    this.ws.close();
    this.stopVoice();
    cancelAnimationFrame(this.animFrameId);
    if (this.clockInterval) clearInterval(this.clockInterval);
  }

  private async loadModels(): Promise<void> {
    try {
      const resp: any = await this.api.get('/api/v1/models');
      this.primaryModel = resp.primary?.name || resp.primary?.id || 'NONE';
      this.secondaryModel = resp.secondary?.name || resp.secondary?.id || 'NONE';
    } catch {
      this.primaryModel = 'NONE';
      this.secondaryModel = 'NONE';
    }
  }

  private async loadAgents(): Promise<void> {
    try {
      const resp: any = await this.api.get('/api/v1/sub-agents');
      this.activeAgents = (resp.sub_agents || resp.agents || []).map((a: any) => ({
        name: a.name,
        active: false,
      }));
    } catch {
      this.activeAgents = [];
    }
  }

  private initAudio(): void {
    this.audioCtx = new AudioContext();
    this.analyser = this.audioCtx.createAnalyser();
    this.analyser.fftSize = 256;
    this.analyser.smoothingTimeConstant = 0.8;

    const SR = (window as any).SpeechRecognition || (window as any).webkitSpeechRecognition;
    if (SR) {
      this.recognition = new SR();
      this.recognition.continuous = true;
      this.recognition.interimResults = true;
      this.recognition.lang = 'en-US';

      this.recognition.onresult = (event: any) => {
        let finalTranscript = '';
        for (let i = event.resultIndex; i < event.results.length; i++) {
          const text = event.results[i][0].transcript;
          if (event.results[i].isFinal) {
            finalTranscript = text.trim();
            if (finalTranscript) {
              this.messages.push({ text: finalTranscript, from: 'user', timestamp: Date.now() });
              this.ws.send({ type: 'transcript', text: finalTranscript, isFinal: true });
              this.transition('agent-thinking');
            }
            this.transcript = '';
          } else {
            this.transcript = text;
          }
        }
      };

      this.recognition.onend = () => {
        if (this.shouldListen && !this.paused) {
          try { this.recognition.start(); } catch { }
        }
      };

      this.recognition.onerror = (event: any) => {
        if (event.error === 'not-allowed') {
          this.statusText = 'MIC ACCESS DENIED';
          this.shouldListen = false;
        }
      };
    }
  }

  private handleMessage(msg: any): void {
    const type = msg.type as string;

    if (type === 'audio') {
      if (msg.text) {
        this.messages.push({ text: msg.text, from: 'umbra', timestamp: Date.now() });
      }
      if (msg.data) {
        if (this.hudState !== 'speaking') this.transition('speaking');
        this.playAudio(msg.data);
      } else {
        this.transition('idle');
      }
    } else if (type === 'status') {
      const s = msg.state as string;
      if (s === 'thinking' && this.hudState !== 'agent-thinking') this.transition('agent-thinking');
      else if (s === 'working') { this.transition('sub-agent-working'); this.statusText = 'WORKING...'; }
      else if (s === 'idle') this.transition('idle');
    } else if (type === 'text') {
      console.log('[UMBRA]', msg.text);
    } else if (type === 'agent_state') {
      this.agentState = msg.agentState || this.agentState;
      if (msg.agentName && this.activeAgents.length) {
        const agent = this.activeAgents.find(a => a.name === msg.agentName);
        if (agent) agent.active = msg.active ?? true;
      }
    }
  }

  private transition(newState: HudState): void {
    if (newState === this.hudState) return;
    const prevState = this.hudState;
    this.hudState = newState;
    this.agentState = newState;
    this.statusText = this.statusMap[newState] || newState.toUpperCase();
    this.hudStateChange.emit(newState);
    this.ws.send({ type: 'hud_state', state: newState });
    if (newState !== 'speaking' && newState !== 'agent-thinking' && newState !== 'sub-agent-working') {
      if (!this.isMuted) this.resumeVoice();
    } else {
      this.pauseVoice();
    }
    if (newState === 'idle' && prevState !== 'idle') {
      this.transcript = '';
    }
  }

  startVoice(): void {
    this.shouldListen = true;
    this.paused = false;
    try { this.recognition?.start(); } catch { }
  }

  stopVoice(): void {
    this.shouldListen = false;
    this.paused = false;
    this.recognition?.stop();
  }

  pauseVoice(): void {
    this.paused = true;
    this.recognition?.stop();
  }

  resumeVoice(): void {
    this.paused = false;
    if (this.shouldListen) {
      try { this.recognition?.start(); } catch { }
    }
  }

  toggleMute(): void {
    this.isMuted = !this.isMuted;
    if (this.isMuted) {
      this.pauseVoice();
      this.transition('idle');
    } else {
      this.resumeVoice();
      this.transition('listening');
    }
  }

  toggleMenu(): void {
    this.menuOpen = !this.menuOpen;
  }

  openSettings(): void {
    this.menuOpen = false;
  }

  navigateTo(path: string): void {
    this.menuOpen = false;
  }

  private async playAudio(base64: string): Promise<void> {
    if (!this.audioCtx) return;
    if (this.audioCtx.state === 'suspended') await this.audioCtx.resume();

    try {
      const binary = atob(base64);
      const bytes = new Uint8Array(binary.length);
      for (let i = 0; i < binary.length; i++) bytes[i] = binary.charCodeAt(i);
      const audioBuffer = await this.audioCtx.decodeAudioData(bytes.buffer.slice(0));

      const source = this.audioCtx.createBufferSource();
      source.buffer = audioBuffer;
      source.connect(this.analyser!);
      this.analyser!.connect(this.audioCtx.destination);

      this.startAudioAnalysis();

      source.onended = () => {
        this.transition('idle');
        cancelAnimationFrame(this.animFrameId);
      };
      source.start();
    } catch (err) {
      console.error('[audio] decode error:', err);
      this.transition('idle');
    }
  }

  private startAudioAnalysis(): void {
    const read = () => {
      if (!this.analyser) return;
      this.audioData = new Uint8Array(this.analyser.frequencyBinCount);
      this.analyser.getByteFrequencyData(this.audioData);
      this.animFrameId = requestAnimationFrame(read);
    };
    read();
  }

  restartServer(): void {
    this.statusText = 'RESTARTING...';
    this.menuOpen = false;
    this.auth.init().then(() => {
      fetch('/api/restart', {
        method: 'POST',
        headers: this.auth.authHeaders(),
      }).then(() => {
        setTimeout(() => window.location.reload(), 4000);
      }).catch(() => {
        this.statusText = 'RESTART FAILED';
      });
    });
  }

  fixSelf(): void {
    this.ws.send({ type: 'fix_self' });
    this.statusText = 'ENTERING WORK MODE...';
    this.menuOpen = false;
    this.transition('agent-thinking');
  }
}
