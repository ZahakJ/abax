//! A synthesised bead click. Desktop only (cpal); on Android the module is a
//! silent stub.

#[cfg(not(target_os = "android"))]
mod desktop {
    use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
    use std::sync::{Arc, Mutex};

    struct Voice {
        t: u32,
        freq: f32,
        gain: f32,
        seed: u32,
    }

    #[derive(Default)]
    struct Mixer {
        voices: Vec<Voice>,
        sample_rate: f32,
    }

    impl Mixer {
        fn render(&mut self, out: &mut [f32], channels: usize) {
            let sr = self.sample_rate.max(8000.0);
            let len = (sr * 0.035) as u32;
            for frame in out.chunks_mut(channels) {
                let mut s = 0.0f32;
                for v in &mut self.voices {
                    let t = v.t as f32 / sr;
                    // 2 ms noise burst into a decaying resonance.
                    v.seed = v.seed.wrapping_mul(1664525).wrapping_add(1013904223);
                    let noise = ((v.seed >> 9) as f32 / (1u32 << 23) as f32) - 1.0;
                    let burst = if t < 0.002 { noise * 0.6 } else { 0.0 };
                    let env = (-t * 110.0).exp();
                    let tone = (t * v.freq * std::f32::consts::TAU).sin() * env;
                    s += (burst + tone * 0.8) * v.gain;
                    v.t += 1;
                }
                self.voices.retain(|v| v.t < len);
                let s = s.clamp(-1.0, 1.0);
                for ch in frame.iter_mut() {
                    *ch = s;
                }
            }
        }
    }

    pub struct Audio {
        _stream: Option<cpal::Stream>,
        mixer: Arc<Mutex<Mixer>>,
        pub volume: f32,
        pub enabled: bool,
    }

    impl Default for Audio {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Audio {
        pub fn new() -> Self {
            let mixer = Arc::new(Mutex::new(Mixer::default()));
            let stream = Self::open(mixer.clone());
            Self {
                _stream: stream,
                mixer,
                volume: 0.6,
                enabled: false,
            }
        }

        fn open(mixer: Arc<Mutex<Mixer>>) -> Option<cpal::Stream> {
            let host = cpal::default_host();
            let device = host.default_output_device()?;
            let config = device.default_output_config().ok()?;
            let channels = config.channels() as usize;
            if let Ok(mut m) = mixer.lock() {
                m.sample_rate = config.sample_rate() as f32;
            }
            if config.sample_format() != cpal::SampleFormat::F32 {
                log::info!("audio: output is not f32; sound disabled");
                return None;
            }
            let m2 = mixer.clone();
            let stream = device
                .build_output_stream(
                    config.into(),
                    move |data: &mut [f32], _| {
                        if let Ok(mut m) = m2.lock() {
                            m.render(data, channels);
                        } else {
                            data.fill(0.0);
                        }
                    },
                    |e| log::warn!("audio error: {e}"),
                    None,
                )
                .ok()?;
            stream.play().ok()?;
            Some(stream)
        }

        pub fn available(&self) -> bool {
            self._stream.is_some()
        }

        /// Play one click. Heaven beads sound a little lower.
        pub fn click(&self, heaven: bool) {
            if !self.enabled || self._stream.is_none() {
                return;
            }
            if let Ok(mut m) = self.mixer.lock() {
                if m.voices.len() < 16 {
                    let n = m.voices.len() as u32;
                    m.voices.push(Voice {
                        t: 0,
                        freq: if heaven { 1400.0 } else { 1800.0 },
                        gain: self.volume.clamp(0.0, 1.0) * 0.5,
                        seed: 0x2545_F491 ^ n.wrapping_mul(2654435761),
                    });
                }
            }
        }
    }
}

#[cfg(target_os = "android")]
mod stub {
    pub struct Audio {
        pub volume: f32,
        pub enabled: bool,
    }
    impl Default for Audio {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Audio {
        pub fn new() -> Self {
            Self {
                volume: 0.6,
                enabled: false,
            }
        }
        pub fn available(&self) -> bool {
            false
        }
        pub fn click(&self, _heaven: bool) {}
    }
}

#[cfg(not(target_os = "android"))]
pub use desktop::Audio;
#[cfg(target_os = "android")]
pub use stub::Audio;
