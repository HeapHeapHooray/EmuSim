use std::f32::consts::TAU;

/// Procedural CRT television static noise generator.
/// Synthesizes authentic analog RF static by combining:
/// 1. Warm pink noise (filtered thermal electron noise)
/// 2. 60 Hz AC mains transformer hum
/// 3. 15.734 kHz CRT flyback line frequency whistle
#[derive(Debug, Clone)]
pub struct CrtStaticAudioGenerator {
    prng_state: u64,
    // Paul Kellet pink noise filter states
    b0: f32,
    b1: f32,
    b2: f32,
    b3: f32,
    b4: f32,
    b5: f32,
    b6: f32,
    time_phase: f32,
    sample_rate: f32,
}

impl Default for CrtStaticAudioGenerator {
    fn default() -> Self {
        Self::new(48000.0)
    }
}

impl CrtStaticAudioGenerator {
    pub fn new(sample_rate: f32) -> Self {
        Self {
            prng_state: 0x853c49e6748fea9b,
            b0: 0.0,
            b1: 0.0,
            b2: 0.0,
            b3: 0.0,
            b4: 0.0,
            b5: 0.0,
            b6: 0.0,
            time_phase: 0.0,
            sample_rate,
        }
    }

    /// Linear congruential PRNG returning float in [-1.0, 1.0].
    #[inline]
    fn next_white_noise(&mut self) -> f32 {
        self.prng_state = self
            .prng_state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        let val = (self.prng_state >> 32) as u32;
        (val as f32 / u32::MAX as f32) * 2.0 - 1.0
    }

    /// Generate next mono sample.
    #[inline]
    pub fn next_sample(&mut self) -> f32 {
        let white = self.next_white_noise();

        // Paul Kellet's filter for pink noise (-3dB/octave slope)
        self.b0 = 0.99886 * self.b0 + white * 0.0555179;
        self.b1 = 0.99332 * self.b1 + white * 0.0750759;
        self.b2 = 0.96900 * self.b2 + white * 0.1538520;
        self.b3 = 0.86650 * self.b3 + white * 0.3104856;
        self.b4 = 0.55000 * self.b4 + white * 0.5329522;
        self.b5 = -0.7616 * self.b5 - white * 0.0168980;
        let pink = (self.b0 + self.b1 + self.b2 + self.b3 + self.b4 + self.b5 + self.b6
            + white * 0.5362)
            * 0.12;
        self.b6 = white * 0.115926;

        // Subtle 60Hz and 120Hz AC power mains ground hum
        let hum = (self.time_phase * 60.0 * TAU).sin() * 0.04
            + (self.time_phase * 120.0 * TAU).sin() * 0.015;

        // Iconic 15.734 kHz horizontal deflection flyback transformer coil whistle
        let flyback = (self.time_phase * 15734.0 * TAU).sin() * 0.012;

        self.time_phase += 1.0 / self.sample_rate;
        if self.time_phase > 1000.0 {
            self.time_phase -= 1000.0;
        }

        (pink + hum + flyback).clamp(-1.0, 1.0)
    }

    /// Generate interleaved stereo i16 samples for a given duration.
    pub fn generate_stereo_batch(&mut self, frames: usize, volume: f32) -> Vec<i16> {
        let mut buffer = Vec::with_capacity(frames * 2);
        let vol = volume.clamp(0.0, 1.0);

        for _ in 0..frames {
            // Slight decorrelation between Left and Right channels for realistic spatial width
            let sample_l = self.next_sample() * vol;
            let sample_r = (self.next_sample() * 0.8 + sample_l * 0.2) * vol;

            let s_l = (sample_l * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            let s_r = (sample_r * i16::MAX as f32).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

            buffer.push(s_l);
            buffer.push(s_r);
        }

        buffer
    }
}
