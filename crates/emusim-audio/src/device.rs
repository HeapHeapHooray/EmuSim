use crate::spatial::{SpatialSource, VrListener};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use cpal::{SampleFormat, Stream, StreamConfig};
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::sync::Arc;
use tracing::{error, info};

pub struct AudioOutputEngine {
    _stream: Stream,
    sample_queue: Arc<Mutex<VecDeque<f32>>>,
}

impl AudioOutputEngine {
    pub fn new() -> Result<Self, String> {
        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .ok_or_else(|| "No default audio output device found".to_string())?;

        let device_name = device.name().unwrap_or_else(|_| "Unknown".into());
        info!("Selected audio output device: {}", device_name);

        let default_config = device
            .default_output_config()
            .map_err(|e| format!("Failed to get default audio config: {e}"))?;

        let sample_rate = default_config.sample_rate().0;
        let channels = default_config.channels() as usize;
        let sample_format = default_config.sample_format();

        info!(
            "Audio configuration: sample_rate={}, channels={}, format={:?}",
            sample_rate, channels, sample_format
        );

        let sample_queue = Arc::new(Mutex::new(VecDeque::<f32>::with_capacity(8192)));
        let queue_clone = sample_queue.clone();

        let err_fn = |err| error!("Audio output stream error: {}", err);

        let config: StreamConfig = default_config.into();

        let stream = match sample_format {
            SampleFormat::F32 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [f32], _| {
                        let mut q = queue_clone.lock();
                        for sample in data.iter_mut() {
                            *sample = q.pop_front().unwrap_or(0.0);
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Failed to build f32 audio stream: {e}"))?,
            SampleFormat::I16 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [i16], _| {
                        let mut q = queue_clone.lock();
                        for sample in data.iter_mut() {
                            let s = q.pop_front().unwrap_or(0.0);
                            *sample = (s * i16::MAX as f32) as i16;
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Failed to build i16 audio stream: {e}"))?,
            SampleFormat::U16 => device
                .build_output_stream(
                    &config,
                    move |data: &mut [u16], _| {
                        let mut q = queue_clone.lock();
                        for sample in data.iter_mut() {
                            let s = q.pop_front().unwrap_or(0.0);
                            *sample = ((s * 0.5 + 0.5) * u16::MAX as f32) as u16;
                        }
                    },
                    err_fn,
                    None,
                )
                .map_err(|e| format!("Failed to build u16 audio stream: {e}"))?,
            _ => return Err("Unsupported audio sample format".into()),
        };

        stream
            .play()
            .map_err(|e| format!("Failed to start audio playback stream: {e}"))?;

        Ok(Self {
            _stream: stream,
            sample_queue,
        })
    }

    /// Push stereo i16 samples from console emulator with 3D spatial positioning.
    pub fn push_spatial_samples(
        &self,
        samples: &mut [i16],
        spatial_source: &SpatialSource,
        listener: &VrListener,
    ) {
        spatial_source.process_spatial(samples, listener);

        let mut q = self.sample_queue.lock();
        // Prevent queue from growing unbounded if audio runs ahead of output
        if q.len() > 8192 {
            q.drain(0..4096);
        }

        for &s in samples.iter() {
            q.push_back(s as f32 / i16::MAX as f32);
        }
    }
}
