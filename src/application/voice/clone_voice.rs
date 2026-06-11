// Zone 3 — Application
use crate::domain::ports::tts_port::TtsPort;

#[derive(Debug, Clone)]
pub struct VoiceProfile {
    pub name: String,
    pub pitch: f32,       // -1.0 to 1.0
    pub speed: f32,       // 0.5 to 2.0
    pub timbre: String,   // "bright", "warm", "neutral", "dark"
    pub sample_count: u32,
}

pub struct CloneVoiceUseCase;

impl CloneVoiceUseCase {
    pub fn new() -> Self {
        Self
    }

    /// Analyze audio sample and create a voice profile.
    pub async fn analyze(&self, audio_sample: &[u8], voice_name: &str) -> Result<VoiceProfile, String> {
        if audio_sample.is_empty() {
            return Err("Audio sample is empty".into());
        }
        // Extract basic audio characteristics from PCM data
        let (pitch, speed, timbre) = self.analyze_audio(audio_sample);
        Ok(VoiceProfile {
            name: voice_name.to_string(),
            pitch,
            speed,
            timbre,
            sample_count: 1,
        })
    }

    /// Synthesize speech with a given voice profile using a TTS adapter.
    pub async fn synthesize_with_profile(
        &self,
        text: &str,
        profile: &VoiceProfile,
        tts: &dyn TtsPort,
    ) -> Result<Vec<u8>, String> {
        let voice_param = format!("{}+pitch{}+speed{}",
            profile.timbre,
            (profile.pitch * 100.0) as i32,
            (profile.speed * 100.0) as i32,
        );
        tts.synthesize(text, &voice_param)
    }

    /// Merge new audio sample into an existing profile.
    pub fn update_profile(&self, profile: &mut VoiceProfile, audio_sample: &[u8]) {
        let (pitch, speed, timbre) = self.analyze_audio(audio_sample);
        let total = (profile.sample_count + 1) as f32;
        profile.pitch = (profile.pitch * profile.sample_count as f32 + pitch) / total;
        profile.speed = (profile.speed * profile.sample_count as f32 + speed) / total;
        profile.sample_count += 1;
        if timbre != "neutral" {
            profile.timbre = timbre;
        }
    }

    /// Basic audio analysis from raw PCM samples.
    /// Extracts normalized pitch, speed, and timbre characteristics.
    fn analyze_audio(&self, samples: &[u8]) -> (f32, f32, String) {
        if samples.len() < 44 {
            return (0.0, 1.0, "neutral".into());
        }

        // Skip WAV header (44 bytes), extract PCM data
        let pcm = &samples[44.min(samples.len())..];
        if pcm.len() < 4 {
            return (0.0, 1.0, "neutral".into());
        }

        // Count zero crossings as pitch estimator
        let samples_i16: Vec<i16> = pcm.chunks(2)
            .filter_map(|c| if c.len() == 2 {
                Some(i16::from_ne_bytes([c[0], c[1]]))
            } else { None })
            .collect();

        if samples_i16.is_empty() {
            return (0.0, 1.0, "neutral".into());
        }

        let zero_crossings = samples_i16.windows(2)
            .filter(|w| (w[0] > 0 && w[1] < 0) || (w[0] < 0 && w[1] > 0))
            .count();

        let avg_amplitude = samples_i16.iter().map(|s| s.abs() as f32).sum::<f32>() / samples_i16.len() as f32;

        // Normalize pitch: zero_crossings / len → 0.0-1.0
        let pitch = ((zero_crossings as f32 / samples_i16.len() as f32) * 4.0 - 0.5).clamp(-1.0, 1.0);
        // Speed: avg_amplitude / max → 0.5-2.0
        let speed = ((avg_amplitude / 32768.0) * 2.0 + 0.5).clamp(0.5, 2.0);
        // Timbre based on high-frequency content (simple proxy: amplitude variance)
        let variance = samples_i16.iter().map(|s| (s.abs() as f32 - avg_amplitude).powi(2)).sum::<f32>() / samples_i16.len() as f32;
        let timbre = if variance > 1000.0 { "bright" }
                     else if variance > 500.0 { "warm" }
                     else { "neutral" };

        (pitch, speed, timbre.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_analyze_empty() {
        let uc = CloneVoiceUseCase::new();
        let (p, s, t) = uc.analyze_audio(&[]);
        assert_eq!(p, 0.0);
        assert_eq!(s, 1.0);
        assert_eq!(t, "neutral");
    }

    #[test]
    fn test_analyze_wav_header_only() {
        let uc = CloneVoiceUseCase::new();
        let header = vec![0u8; 44];
        let (p, s, t) = uc.analyze_audio(&header);
        assert!((p - 0.0).abs() < 0.01);
        assert_eq!(t, "neutral");
    }

    #[test]
    fn test_analyze_sine_wave() {
        let uc = CloneVoiceUseCase::new();
        // Generate a simple sine wave as 16-bit PCM
        let mut wav = vec![0u8; 44];
        for i in 0..200 {
            let sample = (i as f32 * 0.1).sin() * 16000.0;
            let bytes = (sample as i16).to_ne_bytes();
            wav.extend_from_slice(&bytes);
        }
        let (pitch, speed, timbre) = uc.analyze_audio(&wav);
        assert!(pitch.abs() <= 1.0, "pitch out of range: {}", pitch);
        assert!((speed - 0.5..=2.0).contains(&speed), "speed out of range: {}", speed);
        assert!(!timbre.is_empty());
    }

    #[tokio::test]
    async fn test_analyze_empty_rejected() {
        let uc = CloneVoiceUseCase::new();
        let result = uc.analyze(&[], "test").await;
        assert!(result.is_err());
    }
}
