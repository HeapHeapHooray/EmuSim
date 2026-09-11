use glam::{Quat, Vec3};

/// Configuration for 3D positional audio attenuation.
#[derive(Debug, Clone)]
pub struct SpatialAudioConfig {
    pub min_distance: f32, // Distance before attenuation starts (e.g. 0.5m)
    pub max_distance: f32, // Maximum audible distance (e.g. 15.0m)
    pub rolloff_factor: f32,
    pub head_radius: f32, // Head radius for Interaural Time Difference (approx 0.0875m)
}

impl Default for SpatialAudioConfig {
    fn default() -> Self {
        Self {
            min_distance: 0.6,
            max_distance: 12.0,
            rolloff_factor: 1.0,
            head_radius: 0.0875,
        }
    }
}

/// Listener orientation and position in VR world space.
#[derive(Debug, Clone, Copy)]
pub struct VrListener {
    pub position: Vec3,
    pub rotation: Quat,
}

/// 3D Spatial panner for retro console sound emitters.
#[derive(Debug, Clone)]
pub struct SpatialSource {
    pub position: Vec3,
    pub gain: f32,
    config: SpatialAudioConfig,
}

impl SpatialSource {
    pub fn new(position: Vec3) -> Self {
        Self {
            position,
            gain: 1.0,
            config: SpatialAudioConfig::default(),
        }
    }

    /// Process a stereo buffer of interleaved `i16` samples [L, R, L, R, ...]
    /// and spatializes it relative to listener position and head rotation.
    pub fn process_spatial(
        &self,
        samples: &mut [i16],
        listener: &VrListener,
    ) {
        let to_source = self.position - listener.position;
        let distance = to_source.length();

        // Inverse distance attenuation
        let dist_clamped = distance.clamp(self.config.min_distance, self.config.max_distance);
        let attenuation = (self.config.min_distance / dist_clamped).powf(self.config.rolloff_factor);
        let total_gain = (self.gain * attenuation).clamp(0.0, 1.0);

        // Transform relative vector into listener local space
        let local_dir = listener.rotation.inverse() * to_source.normalize_or_zero();

        // local_dir.x: -1.0 (full left), +1.0 (full right)
        let pan = local_dir.x.clamp(-1.0, 1.0);

        // Constant power pan law
        let angle = (pan + 1.0) * (std::f32::consts::PI / 4.0);
        let left_gain = angle.cos() * total_gain;
        let right_gain = angle.sin() * total_gain;

        for chunk in samples.chunks_exact_mut(2) {
            let l = chunk[0] as f32;
            let r = chunk[1] as f32;

            // Combine console stereo channels and apply spatial gains
            let out_l = (l * left_gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;
            let out_r = (r * right_gain).clamp(i16::MIN as f32, i16::MAX as f32) as i16;

            chunk[0] = out_l;
            chunk[1] = out_r;
        }
    }
}
