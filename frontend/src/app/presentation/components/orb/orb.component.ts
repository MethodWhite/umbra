import { Component, ElementRef, OnDestroy, OnInit, ViewChild } from '@angular/core';
import * as THREE from 'three';

export type OrbState = 'idle' | 'listening' | 'thinking' | 'speaking';

@Component({
  selector: 'app-orb',
  standalone: true,
  template: `<canvas #orbCanvas></canvas>`,
  styles: [`
    canvas {
      position: fixed;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      display: block;
    }
  `],
})
export class OrbComponent implements OnInit, OnDestroy {
  @ViewChild('orbCanvas', { static: true }) canvas!: ElementRef<HTMLCanvasElement>;

  private renderer!: THREE.WebGLRenderer;
  private scene!: THREE.Scene;
  private camera!: THREE.PerspectiveCamera;
  private points!: THREE.Points;
  private lines!: THREE.LineSegments;
  private electrons!: THREE.Points;
  private pointMat!: THREE.PointsMaterial;
  private lineMat!: THREE.LineBasicMaterial;
  private pointGeo!: THREE.BufferGeometry;
  private lineGeo!: THREE.BufferGeometry;
  private electronGeo!: THREE.BufferGeometry;
  private vel!: Float32Array;
  private phase!: Float32Array;
  private clock = new THREE.Clock();
  private animId = 0;
  private destroyed = false;
  private analyser: AnalyserNode | null = null;
  private freqData = new Uint8Array(64);

  private N = 2000;
  private state: OrbState = 'idle';
  private targetRadius = 25;
  private currentRadius = 25;
  private targetSpeed = 0.3;
  private currentSpeed = 0.3;
  private targetBright = 0.6;
  private currentBright = 0.6;
  private targetSize = 0.4;
  private currentSize = 0.4;
  private lineAmount = 0;
  private targetLineAmount = 0;
  private lineDistance = 8;
  private spinX = 0;
  private spinY = 0;
  private spinZ = 0;
  private transitionEnergy = 0;
  private lastState: OrbState = 'idle';
  private cloudZ = 0;
  private cloudZVel = 0;
  private bass = 0;
  private mid = 0;

  private activeElectrons: {
    sx: number; sy: number; sz: number;
    ex: number; ey: number; ez: number;
    t: number; speed: number;
  }[] = [];
  private electronSpawnRate = 0;
  private targetElectronRate = 0;
  private lastElectronSpawn = 0;
  private activeConnections: { x1: number; y1: number; z1: number; x2: number; y2: number; z2: number }[] = [];

  private MAX_LINES = 8000;
  private MAX_ELECTRONS = 200;

  ngOnInit(): void {
    this.initScene();
    this.animate();
  }

  ngOnDestroy(): void {
    this.destroyed = true;
    cancelAnimationFrame(this.animId);
    this.renderer.dispose();
  }

  setState(s: OrbState): void {
    this.state = s;
  }

  setAnalyser(a: AnalyserNode | null): void {
    this.analyser = a;
    if (a) this.freqData = new Uint8Array(a.frequencyBinCount);
  }

  private initScene(): void {
    const canvas = this.canvas.nativeElement;
    this.renderer = new THREE.WebGLRenderer({ canvas, antialias: true });
    this.renderer.setPixelRatio(window.devicePixelRatio);
    this.renderer.setSize(window.innerWidth, window.innerHeight);
    this.renderer.setClearColor(0x050508, 1);

    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(45, window.innerWidth / window.innerHeight, 1, 1000);
    this.camera.position.z = 80;

    const pos = new Float32Array(this.N * 3);
    this.vel = new Float32Array(this.N * 3);
    this.phase = new Float32Array(this.N);

    for (let i = 0; i < this.N; i++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const r = Math.pow(Math.random(), 0.5) * 25;
      pos[i * 3] = r * Math.sin(phi) * Math.cos(theta);
      pos[i * 3 + 1] = r * Math.sin(phi) * Math.sin(theta);
      pos[i * 3 + 2] = r * Math.cos(phi);
      this.phase[i] = Math.random() * 1000;
    }

    this.pointGeo = new THREE.BufferGeometry();
    this.pointGeo.setAttribute('position', new THREE.BufferAttribute(pos, 3));

    this.pointMat = new THREE.PointsMaterial({
      color: 0x4ca8e8, size: 0.4, transparent: true, opacity: 0.6,
      sizeAttenuation: true, blending: THREE.AdditiveBlending, depthWrite: false,
    });

    this.points = new THREE.Points(this.pointGeo, this.pointMat);
    this.scene.add(this.points);

    const linePos = new Float32Array(this.MAX_LINES * 6);
    this.lineGeo = new THREE.BufferGeometry();
    this.lineGeo.setAttribute('position', new THREE.BufferAttribute(linePos, 3));
    this.lineGeo.setDrawRange(0, 0);

    this.lineMat = new THREE.LineBasicMaterial({
      color: 0x4ca8e8, transparent: true, opacity: 0.0,
      blending: THREE.AdditiveBlending, depthWrite: false,
    });

    this.lines = new THREE.LineSegments(this.lineGeo, this.lineMat);
    this.scene.add(this.lines);

    const electronPos = new Float32Array(this.MAX_ELECTRONS * 3);
    this.electronGeo = new THREE.BufferGeometry();
    this.electronGeo.setAttribute('position', new THREE.BufferAttribute(electronPos, 3));
    this.electronGeo.setDrawRange(0, 0);

    const electronMat = new THREE.PointsMaterial({
      color: 0xffffff, size: 0.8, transparent: true, opacity: 1.0,
      sizeAttenuation: true, blending: THREE.AdditiveBlending, depthWrite: false,
    });

    this.electrons = new THREE.Points(this.electronGeo, electronMat);
    this.scene.add(this.electrons);

    window.addEventListener('resize', this.onResize);
  }

  private onResize = (): void => {
    this.camera.aspect = window.innerWidth / window.innerHeight;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(window.innerWidth, window.innerHeight);
  };

  private animate = (): void => {
    if (this.destroyed) return;
    this.animId = requestAnimationFrame(this.animate);
    const t = this.clock.getElapsedTime();

    switch (this.state) {
      case 'idle':
        this.targetRadius = 28; this.targetSpeed = 0.2; this.targetBright = 0.5; this.targetSize = 0.35;
        this.targetLineAmount = 0.15; this.targetElectronRate = 0; break;
      case 'listening':
        this.targetRadius = 22; this.targetSpeed = 0.3; this.targetBright = 0.65; this.targetSize = 0.4;
        this.targetLineAmount = 0.4; this.targetElectronRate = 0; break;
      case 'thinking':
        this.targetRadius = 16; this.targetSpeed = 0.5; this.targetBright = 0.7; this.targetSize = 0.3;
        this.targetLineAmount = 1.0; this.targetElectronRate = 0.015; break;
      case 'speaking':
        this.targetRadius = 18; this.targetSpeed = 0.2; this.targetBright = 0.7; this.targetSize = 0.4;
        this.targetLineAmount = 0.8; this.targetElectronRate = 0; break;
    }

    this.currentRadius += (this.targetRadius - this.currentRadius) * 0.02;
    this.currentSpeed += (this.targetSpeed - this.currentSpeed) * 0.02;
    this.currentBright += (this.targetBright - this.currentBright) * 0.02;
    this.currentSize += (this.targetSize - this.currentSize) * 0.02;
    this.lineAmount += (this.targetLineAmount - this.lineAmount) * 0.02;
    this.electronSpawnRate += (this.targetElectronRate - this.electronSpawnRate) * 0.02;

    if (this.state !== this.lastState) { this.transitionEnergy = 1.0; this.lastState = this.state; }
    this.transitionEnergy *= 0.985;
    if (this.transitionEnergy > 0.05) {
      this.spinX += this.transitionEnergy * 0.012 * Math.sin(t * 1.7);
      this.spinY += this.transitionEnergy * 0.015;
      this.spinZ += this.transitionEnergy * 0.008 * Math.cos(t * 1.3);
    }

    this.bass = 0; this.mid = 0;
    if (this.analyser) {
      this.analyser.getByteFrequencyData(this.freqData);
      let bSum = 0, mSum = 0;
      for (let i = 0; i < 8; i++) bSum += this.freqData[i];
      for (let i = 8; i < 24; i++) mSum += this.freqData[i];
      this.bass = bSum / (8 * 255);
      this.mid = mSum / (16 * 255);
    }

    let zTarget = Math.sin(t * 0.12) * 8;
    if (this.state === 'thinking') zTarget = Math.sin(t * 0.3) * 15 + Math.sin(t * 0.9) * 6;
    else if (this.state === 'speaking') zTarget = Math.sin(t * 0.15) * 6 - this.bass * 10;
    this.cloudZVel += (zTarget - this.cloudZ) * 0.008;
    this.cloudZVel *= 0.94;
    this.cloudZ += this.cloudZVel;

    this.points.rotation.x = this.spinX; this.points.rotation.y = this.spinY; this.points.rotation.z = this.spinZ;
    this.points.position.z = this.cloudZ;
    this.lines.rotation.x = this.spinX; this.lines.rotation.y = this.spinY; this.lines.rotation.z = this.spinZ;
    this.lines.position.z = this.cloudZ;

    const p = this.pointGeo.getAttribute('position') as THREE.BufferAttribute;
    const a = p.array as Float32Array;

    for (let i = 0; i < this.N; i++) {
      const i3 = i * 3;
      let x = a[i3], y = a[i3 + 1], z = a[i3 + 2];
      const px = this.phase[i];

      this.vel[i3] += Math.sin(t * 0.05 + px) * 0.001 * this.currentSpeed;
      this.vel[i3 + 1] += Math.cos(t * 0.06 + px * 1.3) * 0.001 * this.currentSpeed;
      this.vel[i3 + 2] += Math.sin(t * 0.055 + px * 0.7) * 0.001 * this.currentSpeed;
      this.vel[i3] += Math.sin(t * 0.02 + px * 2.1 + y * 0.1) * 0.0008 * this.currentSpeed;
      this.vel[i3 + 1] += Math.cos(t * 0.025 + px * 1.7 + z * 0.1) * 0.0008 * this.currentSpeed;
      this.vel[i3 + 2] += Math.sin(t * 0.022 + px * 0.9 + x * 0.1) * 0.0008 * this.currentSpeed;

      const dist = Math.sqrt(x * x + y * y + z * z) || 0.01;
      const pull = Math.max(0, dist - this.currentRadius) * 0.002 + 0.0003;
      this.vel[i3] -= (x / dist) * pull;
      this.vel[i3 + 1] -= (y / dist) * pull;
      this.vel[i3 + 2] -= (z / dist) * pull;

      if (this.bass > 0.05) {
        this.vel[i3] += (x / dist) * this.bass * 0.02;
        this.vel[i3 + 1] += (y / dist) * this.bass * 0.02;
        this.vel[i3 + 2] += (z / dist) * this.bass * 0.02;
      }
      if (this.state === 'speaking' && this.mid > 0.1) {
        const pulse = Math.sin(t * 8 + px);
        this.vel[i3] += (x / dist) * this.mid * 0.012 * pulse;
        this.vel[i3 + 1] += (y / dist) * this.mid * 0.012 * pulse;
      }

      this.vel[i3] *= 0.992; this.vel[i3 + 1] *= 0.992; this.vel[i3 + 2] *= 0.992;
      a[i3] += this.vel[i3]; a[i3 + 1] += this.vel[i3 + 1]; a[i3 + 2] += this.vel[i3 + 2];
    }
    p.needsUpdate = true;

    if (this.lineAmount > 0.01) {
      const lp = this.lineGeo.getAttribute('position') as THREE.BufferAttribute;
      const la = lp.array as Float32Array;
      let lineCount = 0;
      const maxDist = this.lineDistance * (1 + this.bass * 0.5);
      const maxDistSq = maxDist * maxDist;
      const step = Math.max(1, Math.floor(this.N / 600));

      for (let i = 0; i < this.N && lineCount < this.MAX_LINES; i += step) {
        const i3 = i * 3;
        const x1 = a[i3], y1 = a[i3 + 1], z1 = a[i3 + 2];
        for (let j = i + step; j < this.N && lineCount < this.MAX_LINES; j += step) {
          const j3 = j * 3;
          const dx = a[j3] - x1, dy = a[j3 + 1] - y1, dz = a[j3 + 2] - z1;
          if (dx * dx + dy * dy + dz * dz < maxDistSq) {
            const idx = lineCount * 6;
            la[idx] = x1; la[idx + 1] = y1; la[idx + 2] = z1;
            la[idx + 3] = a[j3]; la[idx + 4] = a[j3 + 1]; la[idx + 5] = a[j3 + 2];
            lineCount++;
          }
        }
      }
      this.lineGeo.setDrawRange(0, lineCount * 2);
      lp.needsUpdate = true;
      this.lineMat.opacity = this.lineAmount * 0.12;

      this.activeConnections = [];
      for (let c = 0; c < Math.min(lineCount, 500); c++) {
        const ci = c * 6;
        this.activeConnections.push({
          x1: la[ci], y1: la[ci + 1], z1: la[ci + 2],
          x2: la[ci + 3], y2: la[ci + 4], z2: la[ci + 5],
        });
      }
    } else {
      this.lineGeo.setDrawRange(0, 0);
      this.activeConnections = [];
    }

    if (this.activeConnections.length > 0 && this.electronSpawnRate > 0.005) {
      if (this.activeElectrons.length < 3 && (t - this.lastElectronSpawn) > 1.0) {
        const conn = this.activeConnections[Math.floor(Math.random() * this.activeConnections.length)];
        this.activeElectrons.push({
          sx: conn.x1, sy: conn.y1, sz: conn.z1,
          ex: conn.x2, ey: conn.y2, ez: conn.z2,
          t: 0,
          speed: 0.003 + Math.random() * 0.003,
        });
        this.lastElectronSpawn = t;
      }
    }

    const ep = this.electronGeo.getAttribute('position') as THREE.BufferAttribute;
    const ea = ep.array as Float32Array;
    let aliveCount = 0;

    for (let e = this.activeElectrons.length - 1; e >= 0; e--) {
      const el = this.activeElectrons[e];
      el.t += el.speed;
      if (el.t >= 1) {
        this.activeElectrons.splice(e, 1);
        continue;
      }
      const ei = aliveCount * 3;
      ea[ei] = el.sx + (el.ex - el.sx) * el.t;
      ea[ei + 1] = el.sy + (el.ey - el.sy) * el.t;
      ea[ei + 2] = el.sz + (el.ez - el.sz) * el.t;
      aliveCount++;
    }

    this.electronGeo.setDrawRange(0, aliveCount);
    ep.needsUpdate = true;

    this.electrons.rotation.x = this.spinX;
    this.electrons.rotation.y = this.spinY;
    this.electrons.rotation.z = this.spinZ;
    this.electrons.position.z = this.cloudZ;

    this.pointMat.opacity = this.currentBright + this.bass * 0.08;
    this.pointMat.size = this.currentSize + this.bass * 0.05;

    if (this.state === 'thinking') {
      this.pointMat.color.lerp(new THREE.Color(0x6ec4ff), 0.015);
      this.lineMat.color.lerp(new THREE.Color(0x6ec4ff), 0.015);
    } else if (this.state === 'speaking') {
      this.pointMat.color.lerp(new THREE.Color(0x5ab8f0), 0.015);
      this.lineMat.color.lerp(new THREE.Color(0x5ab8f0), 0.015);
    } else {
      this.pointMat.color.lerp(new THREE.Color(0x4ca8e8), 0.015);
      this.lineMat.color.lerp(new THREE.Color(0x4ca8e8), 0.015);
    }

    this.camera.position.x = Math.sin(t * 0.02) * 5;
    this.camera.position.y = Math.cos(t * 0.03) * 3;
    this.camera.lookAt(0, 0, this.cloudZ * 0.2);

    this.renderer.render(this.scene, this.camera);
  };
}
