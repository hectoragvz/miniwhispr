//! The real Recorder: captures from whatever macOS's default input device is
//! when a Dictation starts, so plugging in headphones takes effect on the next
//! Dictation.

use std::sync::{Arc, Mutex};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{FromSample, SampleFormat, SizedSample, Stream, StreamConfig};
use rubato::audioadapter_buffers::direct::InterleavedSlice;
use rubato::{Fft, FixedSync, Resampler};

use crate::dictation::{Audio, Recorder, SAMPLE_RATE};

/// How often the input level is reported, for the Toast's voice waves.
const LEVELS_PER_SECOND: u32 = 30;
/// Input quieter than this shows as silence, louder than `LOUD_DB` as full.
const SILENCE_DB: f32 = -55.0;
const LOUD_DB: f32 = -15.0;

type OnLevel = Arc<dyn Fn(f32) + Send + Sync>;

pub struct Microphone {
    on_level: OnLevel,
    recording: Option<Recording>,
}

struct Recording {
    /// Capture stops when the stream is dropped.
    _stream: Stream,
    /// Mono samples at the device's own rate.
    samples: Arc<Mutex<Vec<f32>>>,
    sample_rate: u32,
}

impl Microphone {
    /// `on_level` gets the input level, from 0 to 1, while recording. It is
    /// called on the audio thread, so it must return quickly.
    pub fn new(on_level: impl Fn(f32) + Send + Sync + 'static) -> Self {
        Self {
            on_level: Arc::new(on_level),
            recording: None,
        }
    }
}

impl Recorder for Microphone {
    fn start(&mut self) -> Result<(), String> {
        let device = cpal::default_host()
            .default_input_device()
            .ok_or("there is no microphone")?;
        let config = device.default_input_config().map_err(|e| e.to_string())?;
        let samples = Arc::new(Mutex::new(Vec::new()));
        let capture = Capture {
            channels: usize::from(config.channels()),
            samples: samples.clone(),
            meter: LevelMeter::new(config.sample_rate(), self.on_level.clone()),
        };
        let stream_config = config.config();
        let stream = match config.sample_format() {
            SampleFormat::F32 => capture.into_stream::<f32>(&device, stream_config),
            SampleFormat::I16 => capture.into_stream::<i16>(&device, stream_config),
            SampleFormat::I32 => capture.into_stream::<i32>(&device, stream_config),
            other => return Err(format!("unsupported microphone sample format {other}")),
        }
        .map_err(|e| e.to_string())?;
        stream.play().map_err(|e| e.to_string())?;
        self.recording = Some(Recording {
            _stream: stream,
            samples,
            sample_rate: config.sample_rate(),
        });
        Ok(())
    }

    fn stop(&mut self) -> Audio {
        let Some(recording) = self.recording.take() else {
            return Audio { samples: vec![] };
        };
        let Recording {
            _stream,
            samples,
            sample_rate,
        } = recording;
        drop(_stream);
        let samples = std::mem::take(&mut *samples.lock().unwrap());
        Audio {
            samples: to_16_bit(&resample(&samples, sample_rate)),
        }
    }
}

/// What the audio callback works with.
struct Capture {
    channels: usize,
    samples: Arc<Mutex<Vec<f32>>>,
    meter: LevelMeter,
}

impl Capture {
    fn into_stream<T>(
        mut self,
        device: &cpal::Device,
        config: StreamConfig,
    ) -> Result<Stream, cpal::Error>
    where
        T: SizedSample,
        f32: FromSample<T>,
    {
        device.build_input_stream(
            config,
            move |data: &[T], _: &_| {
                let mut samples = self.samples.lock().unwrap();
                for frame in data.chunks(self.channels) {
                    let sum: f32 = frame.iter().map(|s| s.to_sample::<f32>()).sum();
                    let mono = sum / self.channels as f32;
                    samples.push(mono);
                    self.meter.add(mono);
                }
            },
            |e| log::error!("Microphone error: {e}"),
            None,
        )
    }
}

/// Turns samples into a level reported `LEVELS_PER_SECOND` times a second.
struct LevelMeter {
    on_level: OnLevel,
    window: usize,
    count: usize,
    sum_of_squares: f32,
}

impl LevelMeter {
    fn new(sample_rate: u32, on_level: OnLevel) -> Self {
        Self {
            on_level,
            window: (sample_rate / LEVELS_PER_SECOND) as usize,
            count: 0,
            sum_of_squares: 0.0,
        }
    }

    fn add(&mut self, sample: f32) {
        self.sum_of_squares += sample * sample;
        self.count += 1;
        if self.count == self.window {
            let rms = (self.sum_of_squares / self.count as f32).sqrt();
            let db = 20.0 * rms.max(f32::MIN_POSITIVE).log10();
            (self.on_level)(((db - SILENCE_DB) / (LOUD_DB - SILENCE_DB)).clamp(0.0, 1.0));
            self.count = 0;
            self.sum_of_squares = 0.0;
        }
    }
}

fn resample(samples: &[f32], from_rate: u32) -> Vec<f32> {
    if samples.is_empty() || from_rate == SAMPLE_RATE {
        return samples.to_vec();
    }
    let mut resampler = Fft::<f32>::new(
        from_rate as usize,
        SAMPLE_RATE as usize,
        1024,
        1,
        FixedSync::Both,
    )
    .expect("valid sample rates");
    let input = InterleavedSlice::new(samples, 1, samples.len()).expect("mono input");
    resampler
        .process_all(&input, samples.len(), None)
        .expect("output sized by rubato")
        .take_data()
}

fn to_16_bit(samples: &[f32]) -> Vec<i16> {
    samples
        .iter()
        .map(|s| (s.clamp(-1.0, 1.0) * f32::from(i16::MAX)) as i16)
        .collect()
}
