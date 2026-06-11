pub struct VoiceActivityDetector {
    energy_threshold: f32,
    min_speech_frames: u32,
    min_silence_frames: u32,
    frame_size: usize,
    sample_rate: u32,

    speech_frames: u32,
    silence_frames: u32,
    is_speaking: bool,

    silero_vad: Option<SileroVadEngine>,
}

struct SileroVadEngine;

impl SileroVadEngine {
    fn new(_model_bytes: &[u8]) -> Result<Self, anyhow::Error> {
        Err(anyhow::anyhow!("Silero VAD not yet implemented"))
    }

    fn is_speech(&self, _samples: &[f32]) -> Result<bool, anyhow::Error> {
        Err(anyhow::anyhow!("Silero VAD not yet implemented"))
    }
}

impl VoiceActivityDetector {
    pub fn new() -> Self {
        Self {
            energy_threshold: 0.02,
            min_speech_frames: 3,
            min_silence_frames: 10,
            frame_size: 480,
            sample_rate: 16000,
            speech_frames: 0,
            silence_frames: 0,
            is_speaking: false,
            silero_vad: None,
        }
    }

    pub fn with_energy_threshold(mut self, threshold: f32) -> Self {
        self.energy_threshold = threshold;
        self
    }

    pub fn with_frame_size(mut self, size: usize) -> Self {
        self.frame_size = size;
        self
    }

    pub fn with_sample_rate(mut self, rate: u32) -> Self {
        self.sample_rate = rate;
        self
    }

    pub fn set_energy_threshold(&mut self, threshold: f32) {
        self.energy_threshold = threshold;
    }

    pub fn energy_threshold(&self) -> f32 {
        self.energy_threshold
    }

    pub fn is_speaking(&self) -> bool {
        self.is_speaking
    }

    pub fn reset(&mut self) {
        self.speech_frames = 0;
        self.silence_frames = 0;
        self.is_speaking = false;
    }

    pub fn process_frame(&mut self, samples: &[f32]) -> bool {
        let energy = samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32;
        let above_threshold = energy > self.energy_threshold;

        if above_threshold {
            self.speech_frames += 1;
            self.silence_frames = 0;
        } else {
            self.silence_frames += 1;
            self.speech_frames = 0;
        }

        if !self.is_speaking && self.speech_frames >= self.min_speech_frames {
            self.is_speaking = true;
        } else if self.is_speaking && self.silence_frames >= self.min_silence_frames {
            self.is_speaking = false;
        }

        self.is_speaking
    }

    pub fn load_silero_model(&mut self, model_bytes: &[u8]) -> Result<(), anyhow::Error> {
        let engine = SileroVadEngine::new(model_bytes)?;
        self.silero_vad = Some(engine);
        Ok(())
    }

    pub fn process_frame_silero(&self, _samples: &[f32]) -> Result<bool, anyhow::Error> {
        match &self.silero_vad {
            Some(engine) => engine.is_speech(_samples),
            None => Err(anyhow::anyhow!("Silero VAD model not loaded")),
        }
    }
}
