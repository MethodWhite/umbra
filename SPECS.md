# Umbra Project Specifications

## Overview
- Name: UMBRA
- Version: 0.3.0
- Language: Rust (edition 2021)
- GUI: egui/eframe
- License: MIT

## Architecture
- Clean Architecture with domain/application/infrastructure/api layers
- 47 modular components
- Event-driven agent orchestration

## Modules

### Desktop GUI (`src/desktop/`)
- HUD with 3D Fibonacci sphere (500 particles)
- 5 emotional states mapped to sphere colors
- Voice interaction with TTS/STT
- Trading panel with MT5 integration
- Cognitive therapy system for AI agents

### Domain Layer (`src/domain/`)
- `models/`: agent, emotion, voice, trading, security
- `ports/`: TTS, STT, VoiceID, Language, Cybersecurity, Execution

### Application Layer (`src/application/`)
- 12 use cases for voice, security, execution, agent processing

### Infrastructure Layer (`src/infrastructure/`)
- TTS: espeak, Fish Audio, Piper adapters
- STT: whisper, local adapters
- Voice ID: speaker recognition
- Security: IronClaw, Thoth

## Features
[Full feature list...]
