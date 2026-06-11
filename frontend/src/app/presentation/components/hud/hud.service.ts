import * as THREE from 'three';
import { HudParticle, HudState, HudTheme, STATE_COLORS, STATE_SPEEDS, STATE_PULSE_AMPLITUDES, DEFAULT_THEME } from './hud.types';

const MAX_CONNECTION_EDGES = 800;
const PARTICLE_UPDATE_STRIDE = 8;
const CONNECTION_SEARCH_STRIDE = 4;

export class HudEngine {
  private renderer: THREE.WebGLRenderer | null = null;
  private scene: THREE.Scene | null = null;
  private camera: THREE.PerspectiveCamera | null = null;
  private particles: THREE.Points | null = null;
  private particleData: HudParticle[] = [];
  private positions: Float32Array | null = null;
  private colors: Float32Array | null = null;
  private rings: THREE.Mesh[] = [];
  private coreSprite: THREE.Sprite | null = null;
  private glowSprite: THREE.Sprite | null = null;
  private connectionLines: THREE.LineSegments | null = null;
  private connectionPositions: Float32Array | null = null;
  private animationFrameId: number = 0;
  private clock = new THREE.Clock();
  private currentColor = new THREE.Color(0.3, 0.5, 0.8);
  private container: HTMLElement | null = null;
  private resizeObserver: ResizeObserver | null = null;
  private theme: HudTheme;
  private state: HudState = 'idle';

  constructor(theme: Partial<HudTheme> = {}) {
    this.theme = { ...DEFAULT_THEME, ...theme };
  }

  initialize(container: HTMLElement): void {
    this.container = container;
    const width = container.clientWidth;
    const height = container.clientHeight;

    this.scene = new THREE.Scene();
    this.camera = new THREE.PerspectiveCamera(65, width / height, 0.1, 50);
    this.camera.position.set(0, 0.5, 5);
    this.camera.lookAt(0, 0, 0);

    this.renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true });
    this.renderer.setSize(width, height);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 2));
    this.renderer.setClearColor(0x000000, 0);
    container.appendChild(this.renderer.domElement);

    this.createCoreGlow();
    this.createRings();
    this.createParticles();
    this.createConnectionLines();
    this.createStarfield();
    this.startAnimationLoop();

    this.resizeObserver = new ResizeObserver(() => this.handleResize());
    this.resizeObserver.observe(container);
  }

  setState(newState: HudState): void {
    this.state = newState;
  }

  setAudioData(data: Uint8Array | null): void {
  }

  destroy(): void {
    cancelAnimationFrame(this.animationFrameId);
    this.resizeObserver?.disconnect();
    this.renderer?.dispose();
    this.scene?.clear();
    this.particleData = [];
    if (this.container && this.renderer) {
      this.container.removeChild(this.renderer.domElement);
    }
  }

  private handleResize(): void {
    if (!this.container || !this.camera || !this.renderer) return;
    const width = this.container.clientWidth;
    const height = this.container.clientHeight;
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }

  private createCoreGlow(): void {
    const innerGlow = this.createRadialGradientCanvas(256, [
      { offset: 0, color: 'rgba(180, 220, 255, 1)' },
      { offset: 0.15, color: 'rgba(100, 180, 255, 0.9)' },
      { offset: 0.4, color: 'rgba(40, 100, 255, 0.5)' },
      { offset: 0.7, color: 'rgba(10, 40, 200, 0.15)' },
      { offset: 1, color: 'rgba(0, 0, 0, 0)' },
    ]);

    this.coreSprite = new THREE.Sprite(
      new THREE.SpriteMaterial({ map: innerGlow, blending: THREE.AdditiveBlending, transparent: true, depthWrite: false })
    );
    this.coreSprite.scale.set(3, 3, 1);
    this.scene?.add(this.coreSprite);

    const outerGlow = this.createRadialGradientCanvas(256, [
      { offset: 0, color: 'rgba(60, 120, 255, 0.3)' },
      { offset: 0.5, color: 'rgba(20, 60, 200, 0.1)' },
      { offset: 1, color: 'rgba(0, 0, 0, 0)' },
    ]);

    this.glowSprite = new THREE.Sprite(
      new THREE.SpriteMaterial({ map: outerGlow, blending: THREE.AdditiveBlending, transparent: true, opacity: 0.5, depthWrite: false })
    );
    this.glowSprite.scale.set(6, 6, 1);
    this.scene?.add(this.glowSprite);
  }

  private createRadialGradientCanvas(size: number, stops: Array<{ offset: number; color: string }>): THREE.CanvasTexture {
    const canvas = document.createElement('canvas');
    canvas.width = size;
    canvas.height = size;
    const context = canvas.getContext('2d')!;
    const gradient = context.createRadialGradient(size / 2, size / 2, 0, size / 2, size / 2, size / 2);
    stops.forEach(stop => gradient.addColorStop(stop.offset, stop.color));
    context.fillStyle = gradient;
    context.fillRect(0, 0, size, size);
    return new THREE.CanvasTexture(canvas);
  }

  private createRings(): void {
    const ringConfigs = [
      { radius: 1.8, opacity: 0.5, color: 0x4488ff, rotationX: Math.PI / 3, rotationY: 0 },
      { radius: 2.2, opacity: 0.3, color: 0x66aaff, rotationX: Math.PI / 2, rotationY: Math.PI / 4 },
      { radius: 1.4, opacity: 0.2, color: 0x88ccff, rotationX: Math.PI / 4, rotationY: Math.PI / 3 },
    ];

    this.rings = ringConfigs.map(config => {
      const geometry = new THREE.TorusGeometry(config.radius, 0.015, 24, 96);
      const material = new THREE.MeshBasicMaterial({
        color: config.color,
        transparent: true,
        opacity: config.opacity,
        blending: THREE.AdditiveBlending,
        depthWrite: false,
      });
      const mesh = new THREE.Mesh(geometry, material);
      mesh.rotation.x = config.rotationX;
      mesh.rotation.y = config.rotationY;
      this.scene?.add(mesh);
      return mesh;
    });
  }

  private createParticles(): void {
    const count = this.theme.particleCount;
    const geometry = new THREE.BufferGeometry();
    this.positions = new Float32Array(count * 3);
    this.colors = new Float32Array(count * 3);
    const sizes = new Float32Array(count);

    for (let index = 0; index < count; index++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const radius = 1.5 + Math.random() * 3.5;
      const x = radius * Math.sin(phi) * Math.cos(theta);
      const y = radius * Math.cos(phi) * 0.6;
      const z = radius * Math.sin(phi) * Math.sin(theta);
      const positionIndex = index * 3;

      this.positions[positionIndex] = x;
      this.positions[positionIndex + 1] = y;
      this.positions[positionIndex + 2] = z;

      const brightness = 0.3 + Math.random() * 0.7;
      this.colors[positionIndex] = brightness * 0.4;
      this.colors[positionIndex + 1] = brightness * 0.6;
      this.colors[positionIndex + 2] = brightness;
      sizes[index] = 0.015 + Math.random() * 0.035;

      this.particleData.push({
        position: new THREE.Vector3(x, y, z),
        velocity: new THREE.Vector3(0, 0, 0),
        targetPosition: new THREE.Vector3(0, 0, 0),
        baseColor: new THREE.Color(this.colors[positionIndex], this.colors[positionIndex + 1], this.colors[positionIndex + 2]),
        currentColor: new THREE.Color(0.3, 0.5, 0.8),
        phase: Math.random() * Math.PI * 2,
        size: sizes[index],
      });
    }

    geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3));
    geometry.setAttribute('color', new THREE.BufferAttribute(this.colors, 3));
    geometry.setAttribute('size', new THREE.BufferAttribute(sizes, 1));

    const material = new THREE.PointsMaterial({
      size: 0.03,
      vertexColors: true,
      transparent: true,
      opacity: 0.9,
      blending: THREE.AdditiveBlending,
      sizeAttenuation: true,
      depthWrite: false,
    });

    this.particles = new THREE.Points(geometry, material);
    this.scene?.add(this.particles);
  }

  private createConnectionLines(): void {
    this.connectionPositions = new Float32Array(MAX_CONNECTION_EDGES * 6);
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(this.connectionPositions, 3));
    const material = new THREE.LineBasicMaterial({
      color: 0x4488ff,
      transparent: true,
      opacity: 0.08,
      blending: THREE.AdditiveBlending,
    });
    this.connectionLines = new THREE.LineSegments(geometry, material);
    this.connectionLines.frustumCulled = false;
    this.scene?.add(this.connectionLines);
  }

  private createStarfield(): void {
    const starCount = 800;
    const positions = new Float32Array(starCount * 3);

    for (let index = 0; index < starCount; index++) {
      const theta = Math.random() * Math.PI * 2;
      const phi = Math.acos(2 * Math.random() - 1);
      const radius = 15 + Math.random() * 25;
      const positionIndex = index * 3;
      positions[positionIndex] = radius * Math.sin(phi) * Math.cos(theta);
      positions[positionIndex + 1] = radius * Math.cos(phi);
      positions[positionIndex + 2] = radius * Math.sin(phi) * Math.sin(theta);
    }

    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(positions, 3));
    const material = new THREE.PointsMaterial({
      color: 0x446688,
      size: 0.05,
      transparent: true,
      opacity: 0.4,
      blending: THREE.AdditiveBlending,
      depthWrite: false,
    });
    this.scene?.add(new THREE.Points(geometry, material));
  }

  private startAnimationLoop(): void {
    const animate = () => {
      this.animationFrameId = requestAnimationFrame(animate);
      const deltaTime = Math.min(this.clock.getDelta(), 0.05);
      const elapsedTime = this.clock.getElapsedTime();
      this.updateAnimation(deltaTime, elapsedTime);
      this.renderer?.render(this.scene!, this.camera!);
    };
    animate();
  }

  private updateAnimation(deltaTime: number, elapsedTime: number): void {
    const targetColor = STATE_COLORS[this.state];
    const animationSpeed = STATE_SPEEDS[this.state];
    const pulseAmplitude = STATE_PULSE_AMPLITUDES[this.state];

    this.currentColor.lerp(targetColor, deltaTime * 1.5);
    this.updateCoreGlow(elapsedTime, pulseAmplitude);
    this.updateRings(deltaTime, animationSpeed);
    this.updateParticles(deltaTime, elapsedTime, animationSpeed);
    this.updateConnections();
    this.updateCamera(elapsedTime);
  }

  private updateCoreGlow(elapsedTime: number, pulseAmplitude: number): void {
    const pulse = 1 + Math.sin(elapsedTime * 2.5) * (0.08 + pulseAmplitude * 0.15);
    if (this.coreSprite) {
      this.coreSprite.scale.set(3 * pulse, 3 * pulse, 1);
      (this.coreSprite.material as THREE.SpriteMaterial).color.lerp(this.currentColor, 0.03);
    }
    if (this.glowSprite) {
      this.glowSprite.scale.set(
        6 * (1 + Math.sin(elapsedTime * 1.5) * 0.05),
        6 * (1 + Math.sin(elapsedTime * 1.5) * 0.05),
        1
      );
    }
  }

  private updateRings(deltaTime: number, animationSpeed: number): void {
    const rotationSpeeds = [
      { y: 0.8, x: 0.1 },
      { y: 0.5, z: 0.15 },
      { x: 0.6, z: 0.2 },
    ];

    this.rings.forEach((ring, index) => {
      const speeds = rotationSpeeds[index] || { y: 0.3 };
      if (speeds.y) ring.rotation.y += deltaTime * animationSpeed * speeds.y;
      if (speeds.x) ring.rotation.x += deltaTime * animationSpeed * speeds.x;
      if (speeds.z) ring.rotation.z += deltaTime * animationSpeed * speeds.z;
      (ring.material as THREE.MeshBasicMaterial).color.lerp(this.currentColor, deltaTime * 1.5);
    });
  }

  private updateParticles(deltaTime: number, elapsedTime: number, animationSpeed: number): void {
    const count = this.particleData.length;
    if (!this.positions || !this.colors) return;

    for (let index = 0; index < count; index++) {
      const particle = this.particleData[index];
      const positionIndex = index * 3;
      const phase = particle.phase;
      const time = elapsedTime;

      switch (this.state) {
        case 'agent-thinking':
          this.updateParticleAgentThinking(particle, time, phase, deltaTime);
          break;
        case 'sub-agent-working':
          this.updateParticleSubAgentWorking(particle, index, time, deltaTime);
          break;
        default:
          this.updateParticleAmbient(particle, time, phase, animationSpeed);
          break;
      }

      this.positions[positionIndex] = particle.position.x;
      this.positions[positionIndex + 1] = particle.position.y;
      this.positions[positionIndex + 2] = particle.position.z;

      particle.currentColor.copy(this.currentColor);
      particle.currentColor.multiplyScalar(0.3 + 0.7 * (0.5 + 0.5 * Math.sin(time + particle.phase)));
      this.colors[positionIndex] = particle.currentColor.r;
      this.colors[positionIndex + 1] = particle.currentColor.g;
      this.colors[positionIndex + 2] = particle.currentColor.b;
    }

    const particlePosAttr = this.particles!.geometry.getAttribute('position');
    const particleColorAttr = this.particles!.geometry.getAttribute('color');
    if (particlePosAttr) particlePosAttr.needsUpdate = true;
    if (particleColorAttr) particleColorAttr.needsUpdate = true;
  }

  private updateParticleAgentThinking(particle: HudParticle, time: number, phase: number, deltaTime: number): void {
    const angle = time * 0.5 + phase;
    const radius = 1.8 + Math.sin(time * 0.3 + phase) * 0.5;
    particle.targetPosition.set(
      Math.cos(angle) * radius,
      Math.sin(angle * 0.7) * radius * 0.5,
      Math.sin(angle * 0.5) * radius * 0.4
    );
    particle.position.lerp(particle.targetPosition, deltaTime * 2.0);
    particle.position.x += Math.sin(time + phase) * 0.005;
    particle.position.y += Math.cos(time * 0.7 + phase) * 0.005;
  }

  private updateParticleSubAgentWorking(particle: HudParticle, index: number, time: number, deltaTime: number): void {
    const angle = time * 0.4 + index * 0.002;
    const radius = 1.5 + Math.sin(index * 0.05 + time * 0.2) * 1.0;
    particle.targetPosition.set(
      Math.cos(angle) * radius,
      Math.sin(angle * 2) * radius * 0.4,
      Math.sin(angle * 1.3) * radius * 0.3
    );
    particle.position.lerp(particle.targetPosition, deltaTime * 1.2);
  }

  private updateParticleAmbient(particle: HudParticle, time: number, phase: number, speed: number): void {
    particle.position.x += Math.sin(time + phase) * 0.002 * speed;
    particle.position.y += Math.cos(time * 0.7 + phase) * 0.002 * speed;
    particle.position.z += Math.sin(time * 0.5 + phase * 1.3) * 0.002 * speed;
    const distance = particle.position.length();
    if (distance > 4.5) particle.position.multiplyScalar(0.995);
    if (distance < 1.2) particle.position.multiplyScalar(1.005);
  }

  private updateConnections(): void {
    if (!this.connectionPositions || !this.connectionLines) return;
    const count = this.particleData.length;
    let edgeCount = 0;

    for (let i = 0; i < count && edgeCount < MAX_CONNECTION_EDGES; i += PARTICLE_UPDATE_STRIDE) {
      for (let j = i + CONNECTION_SEARCH_STRIDE; j < count && edgeCount < MAX_CONNECTION_EDGES; j += CONNECTION_SEARCH_STRIDE) {
        const dx = this.particleData[i].position.x - this.particleData[j].position.x;
        const dy = this.particleData[i].position.y - this.particleData[j].position.y;
        const dz = this.particleData[i].position.z - this.particleData[j].position.z;
        const distance = Math.sqrt(dx * dx + dy * dy + dz * dz);

        if (distance < this.theme.connectionDistance) {
          const edgeIndex = edgeCount * 6;
          this.connectionPositions[edgeIndex] = this.particleData[i].position.x;
          this.connectionPositions[edgeIndex + 1] = this.particleData[i].position.y;
          this.connectionPositions[edgeIndex + 2] = this.particleData[i].position.z;
          this.connectionPositions[edgeIndex + 3] = this.particleData[j].position.x;
          this.connectionPositions[edgeIndex + 4] = this.particleData[j].position.y;
          this.connectionPositions[edgeIndex + 5] = this.particleData[j].position.z;
          edgeCount++;
        }
      }
    }

    const positionAttribute = (this.connectionLines.geometry as THREE.BufferGeometry).getAttribute('position');
    if (positionAttribute) positionAttribute.needsUpdate = true;
    (this.connectionLines.geometry as THREE.BufferGeometry).setDrawRange(0, edgeCount * 2);
    (this.connectionLines.material as THREE.LineBasicMaterial).color.lerp(this.currentColor, 0.03);
    (this.connectionLines.material as THREE.LineBasicMaterial).opacity = 0.04 + 0.06 * (edgeCount / MAX_CONNECTION_EDGES);
  }

  private updateCamera(elapsedTime: number): void {
    if (!this.camera) return;
    this.camera.position.x = Math.sin(elapsedTime * 0.04) * 0.3;
    this.camera.position.y = 0.5 + Math.cos(elapsedTime * 0.06) * 0.15;
    this.camera.lookAt(0, 0, 0);
  }
}
