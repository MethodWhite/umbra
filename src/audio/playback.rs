// Zone 1 — Desktop/UI
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};

pub struct AudioPacket {
    pub data: Vec<f32>,
    pub sample_rate: u32,
    pub channels: u16,
}

pub struct AudioSink {
    volume: Arc<Mutex<f32>>,
    running: Arc<AtomicBool>,
    tx: Sender<AudioPacket>,
    _join: std::thread::JoinHandle<()>,
}

impl AudioSink {
    pub fn new() -> Result<Self, anyhow::Error> {
        let volume = Arc::new(Mutex::new(1.0));
        let running = Arc::new(AtomicBool::new(true));
        let (tx, rx) = mpsc::channel::<AudioPacket>();

        let vol = Arc::clone(&volume);
        let run = Arc::clone(&running);
        let _join = std::thread::spawn(move || {
            if let Err(e) = Self::run(rx, vol, run) {
                eprintln!("Audio playback thread exited: {}", e);
            }
        });

        Ok(Self { volume, running, tx, _join })
    }

    pub fn enqueue(&self, packet: AudioPacket) -> Result<(), anyhow::Error> {
        self.tx.send(packet)?;
        Ok(())
    }

    pub fn set_volume(&self, vol: f32) {
        if let Ok(mut v) = self.volume.lock() {
            *v = vol.clamp(0.0, 1.0);
        }
    }

    pub fn volume(&self) -> f32 {
        self.volume.lock().map(|v| *v).unwrap_or(1.0)
    }

    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }

    fn run(rx: Receiver<AudioPacket>, volume: Arc<Mutex<f32>>, running: Arc<AtomicBool>) -> Result<(), anyhow::Error> {
        let host = cpal::default_host();
        let device = host.default_output_device()
            .ok_or_else(|| anyhow::anyhow!("No audio output device found"))?;
        let config = device.default_output_config()?;

        loop {
            let packet = rx.recv()?;
            if !running.load(Ordering::Relaxed) {
                break;
            }
            let vol = *volume.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
            if vol <= 0.0 {
                continue;
            }
            Self::play_chunk(&device, &config, &packet, vol)?;
        }
        Ok(())
    }

    fn play_chunk(
        device: &cpal::Device,
        config: &cpal::SupportedStreamConfig,
        packet: &AudioPacket,
        volume: f32,
    ) -> Result<(), anyhow::Error> {
        let data = packet.data.clone();
        let data_len = data.len();
        let ch = packet.channels as usize;
        let sample_rate = packet.sample_rate;
        let err_fn = |e: cpal::StreamError| eprintln!("Stream error: {}", e);

        let stream = device.build_output_stream(
            &config.config(),
            move |out: &mut [f32], _: &cpal::OutputCallbackInfo| {
                for (frame, sample) in out.chunks_mut(ch).zip(data.chunks(ch)) {
                    let val = sample.first().copied().unwrap_or(0.0) * volume;
                    for o in frame.iter_mut() {
                        *o = val;
                    }
                }
            },
            err_fn,
            None,
        )?;

        stream.play()?;

        let duration_ms = (data_len as u64 * 1000) / (sample_rate as u64 * ch as u64);
        std::thread::sleep(std::time::Duration::from_millis(duration_ms));

        drop(stream);
        Ok(())
    }
}

impl Drop for AudioSink {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
    }
}
