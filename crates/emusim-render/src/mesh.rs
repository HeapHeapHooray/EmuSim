use bytemuck::{Pod, Zeroable};
use emusim_physics::cable_verlet::VerletCableStrand;
use glam::Vec3;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    pub color: [f32; 4],
}

#[derive(Debug, Clone, Default)]
pub struct CpuMesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl CpuMesh {
    /// Generate a dynamic tubular 3D cylinder mesh following a Verlet cable strand.
    pub fn from_cable_strand(strand: &VerletCableStrand, radial_segments: usize, color_rgb: [u8; 3]) -> Self {
        let mut mesh = Self::default();
        if strand.particles.len() < 2 {
            return mesh;
        }

        let num_particles = strand.particles.len();
        let radial = radial_segments.max(4);
        let color = [
            color_rgb[0] as f32 / 255.0,
            color_rgb[1] as f32 / 255.0,
            color_rgb[2] as f32 / 255.0,
            1.0,
        ];

        for (i, p) in strand.particles.iter().enumerate() {
            // Compute forward tangent direction
            let forward = if i == 0 {
                (strand.particles[1].position - p.position).normalize_or_zero()
            } else if i == num_particles - 1 {
                (p.position - strand.particles[i - 1].position).normalize_or_zero()
            } else {
                (strand.particles[i + 1].position - strand.particles[i - 1].position).normalize_or_zero()
            };

            // Compute orthogonal plane
            let arbitrary = if forward.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
            let right = forward.cross(arbitrary).normalize_or_zero();
            let up = right.cross(forward).normalize_or_zero();

            let t = i as f32 / (num_particles - 1) as f32;

            for r in 0..radial {
                let angle = (r as f32 / radial as f32) * std::f32::consts::TAU;
                let normal = (right * angle.cos() + up * angle.sin()).normalize_or_zero();
                let pos = p.position + normal * strand.radius;

                mesh.vertices.push(Vertex {
                    position: pos.to_array(),
                    normal: normal.to_array(),
                    uv: [r as f32 / radial as f32, t],
                    color,
                });
            }
        }

        // Build index rings
        for i in 0..(num_particles - 1) {
            let ring_a = (i * radial) as u32;
            let ring_b = ((i + 1) * radial) as u32;

            for r in 0..radial {
                let next_r = (r + 1) % radial;

                let a0 = ring_a + r as u32;
                let a1 = ring_a + next_r as u32;
                let b0 = ring_b + r as u32;
                let b1 = ring_b + next_r as u32;

                mesh.indices.extend_from_slice(&[a0, b0, a1, a1, b0, b1]);
            }
        }

        mesh
    }

    /// Generate quad mesh for CRT screen face.
    pub fn crt_screen_quad(width: f32, height: f32) -> Self {
        let half_w = width * 0.5;
        let half_h = height * 0.5;

        let vertices = vec![
            Vertex {
                position: [-half_w, -half_h, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [half_w, -half_h, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 1.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [half_w, half_h, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [1.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
            Vertex {
                position: [-half_w, half_h, 0.0],
                normal: [0.0, 0.0, 1.0],
                uv: [0.0, 0.0],
                color: [1.0, 1.0, 1.0, 1.0],
            },
        ];

        let indices = vec![0, 1, 2, 0, 2, 3];

        Self { vertices, indices }
    }
}
