use glam::Vec3;
use serde::{Deserialize, Serialize};

/// A physical point mass along a flexible cable strand.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct VerletParticle {
    pub position: Vec3,
    pub prev_position: Vec3,
    pub acceleration: Vec3,
    pub is_pinned: bool,
}

impl VerletParticle {
    pub fn new(pos: Vec3) -> Self {
        Self {
            position: pos,
            prev_position: pos,
            acceleration: Vec3::ZERO,
            is_pinned: false,
        }
    }
}

/// Dynamic cable strand simulated with Verlet integration and distance constraints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerletCableStrand {
    pub particles: Vec<VerletParticle>,
    pub segment_rest_length: f32,
    pub damping: f32,        // Air resistance/velocity decay (e.g. 0.98)
    pub stiffness_iterations: usize, // Constraint relaxation iterations (e.g. 8)
    pub radius: f32,
}

impl VerletCableStrand {
    pub fn new(start: Vec3, end: Vec3, segments: usize, radius: f32) -> Self {
        let count = segments.max(2) + 1;
        let mut particles = Vec::with_capacity(count);
        let total_dist = start.distance(end);
        let rest_length = total_dist / (segments as f32);

        for i in 0..count {
            let t = i as f32 / (count - 1) as f32;
            let pos = start.lerp(end, t);
            particles.push(VerletParticle::new(pos));
        }

        Self {
            particles,
            segment_rest_length: rest_length,
            damping: 0.985,
            stiffness_iterations: 8,
            radius,
        }
    }

    /// Step the cable physics forward by dt.
    pub fn step(&mut self, dt: f32, gravity: Vec3, floor_y: f32) {
        let dt_sq = dt * dt;

        // 1. Verlet integration for all non-pinned particles
        for p in &mut self.particles {
            if p.is_pinned {
                continue;
            }

            let velocity = (p.position - p.prev_position) * self.damping;
            let next_pos = p.position + velocity + (gravity + p.acceleration) * dt_sq;

            p.prev_position = p.position;
            p.position = next_pos;
            p.acceleration = Vec3::ZERO;

            // Floor collision
            if p.position.y - self.radius < floor_y {
                p.position.y = floor_y + self.radius;
            }
        }

        // 2. Distance constraint relaxation iterations
        for _ in 0..self.stiffness_iterations {
            for i in 0..(self.particles.len() - 1) {
                let p1 = self.particles[i];
                let p2 = self.particles[i + 1];

                let delta = p2.position - p1.position;
                let current_dist = delta.length();
                if current_dist < 1e-6 {
                    continue;
                }

                let diff = (current_dist - self.segment_rest_length) / current_dist;
                let correction = delta * 0.5 * diff;

                match (p1.is_pinned, p2.is_pinned) {
                    (false, false) => {
                        self.particles[i].position += correction;
                        self.particles[i + 1].position -= correction;
                    }
                    (true, false) => {
                        self.particles[i + 1].position -= correction * 2.0;
                    }
                    (false, true) => {
                        self.particles[i].position += correction * 2.0;
                    }
                    (true, true) => {}
                }
            }
        }
    }

    /// Pin the start of the cable to a target world position.
    pub fn pin_start(&mut self, pos: Vec3) {
        if let Some(first) = self.particles.first_mut() {
            first.position = pos;
            first.prev_position = pos;
            first.is_pinned = true;
        }
    }

    /// Pin the end of the cable to a target world position.
    pub fn pin_end(&mut self, pos: Vec3) {
        if let Some(last) = self.particles.last_mut() {
            last.position = pos;
            last.prev_position = pos;
            last.is_pinned = true;
        }
    }

    /// Unpin start.
    pub fn unpin_start(&mut self) {
        if let Some(first) = self.particles.first_mut() {
            first.is_pinned = false;
        }
    }

    /// Unpin end.
    pub fn unpin_end(&mut self) {
        if let Some(last) = self.particles.last_mut() {
            last.is_pinned = false;
        }
    }
}
