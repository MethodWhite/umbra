# Umbra Modular Architecture v2

## Module Tree

```
src/
├── domain/                    # Domain layer (entities, ports)
│   ├── models/
│   │   ├── agent.rs
│   │   ├── emotion.rs
│   │   ├── voice.rs
│   │   ├── trading.rs
│   │   └── security.rs
│   └── ports/
│       ├── tts_port.rs
│       ├── stt_port.rs
│       ├── voice_id_port.rs
│       ├── language_port.rs
│       ├── cybersecurity_port.rs
│       └── execution_port.rs
│
├── application/               # Use cases
│   ├── voice/
│   │   ├── synthesize.rs      # TTS use case
│   │   ├── transcribe.rs      # STT use case
│   │   ├── clone_voice.rs     # Voice cloning
│   │   ├── detect_language.rs # Language detection
│   │   └── detect_emotion.rs  # User emotion from voice
│   ├── security/
│   │   ├── verify_identity.rs # Voice ID verification
│   │   ├── authorize_action.rs
│   │   └── audit_log.rs
│   ├── execution/
│   │   ├── execute_command.rs
│   │   └── validate_command.rs
│   └── agent/
│       ├── generate_tone.rs   # AI voice tone generation
│       └── process_request.rs # Request recognition
│
├── infrastructure/            # Implementations
│   ├── tts/
│   │   ├── espeak_tts.rs
│   │   ├── fish_tts.rs
│   │   └── piper_tts.rs
│   ├── stt/
│   │   ├── whisper_stt.rs
│   │   └── local_stt.rs
│   ├── voice_id/
│   │   └── speaker_recognition.rs
│   ├── language/
│   │   └── language_detector.rs
│   └── security/
│       ├── ironclaw.rs
│       └── thoth.rs
│
├── api/
│   ├── routes/
│   │   ├── voice_routes.rs
│   │   ├── security_routes.rs
│   │   └── execution_routes.rs
│   └── middleware/
│       └── auth.rs
│
├── desktop/                   # GUI (existing, simplified)
│   ├── mod.rs
│   ├── sphere.rs
│   └── panels.rs
│
├── audio/
│   ├── mod.rs
│   ├── playback.rs
│   ├── input.rs
│   └── vad.rs
│
└── bin/
    └── umbra-gui.rs
```

## Layer Responsibilities

### Domain Layer (`domain/`)
- **Models**: Pure data structures with no behavior. Define entities like Agent, Emotion, Voice, Trading, Security.
- **Ports**: Trait interfaces that define contracts for external adapters. Each port is an abstraction over an external service (TTS, STT, Voice ID, Language Detection, Cybersecurity, Execution).

### Application Layer (`application/`)
- **Use Cases**: Each use case is a single struct with an `execute()` method. Orchestrates domain logic by calling port interfaces. Contains zero infrastructure code.
- Organized by capability: `voice/`, `security/`, `execution/`, `agent/`.

### Infrastructure Layer (`infrastructure/`)
- **Implementations**: Concrete implementations of domain port traits. Each sub-module maps to a port.
- Example: `tts/espeak_tts.rs` implements `TtsPort`, `stt/whisper_stt.rs` implements `SttPort`.

### API Layer (`api/`)
- **Routes**: HTTP handlers that accept requests, call application use cases, return responses.
- **Middleware**: Cross-cutting concerns like authentication, rate limiting.

### Desktop Layer (`desktop/`)
- **GUI**: egui-based desktop application with sphere visualization, panels, and HUD.

### Audio Layer (`audio/`)
- **Core Audio**: Low-level audio playback, input capture, and voice activity detection.

## Dependency Rules

1. **Domain** has zero dependencies on other layers.
2. **Application** depends only on `domain` (ports and models).
3. **Infrastructure** depends on `domain` (implements ports).
4. **API** depends on `application` (calls use cases).
5. **Desktop** depends on `application` (calls use cases).
6. **Audio** is independent, used by infrastructure.

No layer may import from a layer above it.
