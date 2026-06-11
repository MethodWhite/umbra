use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::mpsc::{self, Receiver};
use std::sync::{Arc, Mutex};

/// Captures audio from microphone and detects speech
#[allow(dead_code)]
pub struct VoiceInput {
    stream: Option<cpal::Stream>,
    audio_rx: Receiver<Vec<f32>>,
    is_recording: Arc<Mutex<bool>>,
    vad: Option<crate::audio::vad::VoiceActivityDetector>,
}

impl VoiceInput {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host.default_input_device().ok_or("No microphone found")?;
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let (tx, audio_rx) = mpsc::channel::<Vec<f32>>();
        let is_recording = Arc::new(Mutex::new(false));

        // Build stream
        let is_rec = is_recording.clone();
        let stream = device.build_input_stream(
            &config.into(),
            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                let _ = tx.send(data.to_vec());
            },
            move |err| eprintln!("Audio input error: {}", err),
            None
        ).map_err(|e| e.to_string())?;

        Ok(VoiceInput {
            stream: Some(stream),
            audio_rx,
            is_recording: is_rec,
            vad: Some(crate::audio::vad::VoiceActivityDetector::new()),
        })
    }

    pub fn start_recording(&mut self) {
        if let Some(stream) = &self.stream {
            let _ = stream.play();
        }
        *self.is_recording.lock().unwrap() = true;
    }

    pub fn stop_recording(&mut self) {
        *self.is_recording.lock().unwrap() = false;
    }

    /// Get the latest audio samples (non-blocking)
    pub fn poll_audio(&self) -> Option<Vec<f32>> {
        self.audio_rx.try_recv().ok()
    }

    pub fn is_recording(&self) -> bool {
        *self.is_recording.lock().unwrap()
    }
}
