import { Component, ElementRef, Input, OnDestroy, OnInit, NgZone, ViewChild } from '@angular/core';
import { HudEngine } from './hud.service';
import { HudState } from './hud.types';

@Component({
  selector: 'app-hud',
  standalone: true,
  template: '<div #container class="hud-container"></div>',
  styles: [`
    .hud-container {
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      z-index: 0;
      overflow: hidden;
    }
  `]
})
export class HudComponent implements OnInit, OnDestroy {
  @Input() set state(value: HudState) {
    this.hudEngine?.setState(value);
  }

  @Input() set audioData(value: Uint8Array | null) {
    this.hudEngine?.setAudioData(value);
  }

  @ViewChild('container', { static: true }) private container!: ElementRef<HTMLDivElement>;

  private hudEngine: HudEngine | null = null;

  constructor(private ngZone: NgZone) {}

  ngOnInit(): void {
    this.ngZone.runOutsideAngular(() => {
      this.hudEngine = new HudEngine();
      this.hudEngine.initialize(this.container.nativeElement);
    });
  }

  ngOnDestroy(): void {
    this.hudEngine?.destroy();
    this.hudEngine = null;
  }
}
