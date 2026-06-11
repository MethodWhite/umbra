import * as THREE from 'three';

export type HudState = 'idle' | 'listening' | 'agent-thinking' | 'sub-agent-working' | 'speaking';

export interface HudParticle {
  position: THREE.Vector3;
  velocity: THREE.Vector3;
  targetPosition: THREE.Vector3;
  baseColor: THREE.Color;
  currentColor: THREE.Color;
  phase: number;
  size: number;
}

export interface HudTheme {
  primaryColor: THREE.Color;
  particleCount: number;
  connectionDistance: number;
  ringCount: number;
}

export const DEFAULT_THEME: HudTheme = {
  primaryColor: new THREE.Color(0.3, 0.5, 0.8),
  particleCount: 4000,
  connectionDistance: 1.2,
  ringCount: 3,
};

export const STATE_COLORS: Record<HudState, THREE.Color> = {
  'idle': new THREE.Color(0.3, 0.5, 0.8),
  'listening': new THREE.Color(0.3, 1.0, 0.3),
  'agent-thinking': new THREE.Color(1.0, 0.7, 0.1),
  'sub-agent-working': new THREE.Color(0.1, 0.8, 1.0),
  'speaking': new THREE.Color(1.0, 0.3, 0.5),
};

export const STATE_SPEEDS: Record<HudState, number> = {
  'idle': 0.4,
  'listening': 0.8,
  'agent-thinking': 2.0,
  'sub-agent-working': 1.5,
  'speaking': 1.2,
};

export const STATE_PULSE_AMPLITUDES: Record<HudState, number> = {
  'idle': 0.05,
  'listening': 0.2,
  'agent-thinking': 0.3,
  'sub-agent-working': 0.15,
  'speaking': 0.4,
};
