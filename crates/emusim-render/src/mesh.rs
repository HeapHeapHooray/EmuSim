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

    pub fn append(&mut self, other: &CpuMesh) {
        let base_index = self.vertices.len() as u32;
        self.vertices.extend_from_slice(&other.vertices);
        self.indices.extend(other.indices.iter().map(|&i| i + base_index));
    }

    pub fn add_quad(
        &mut self,
        v0: Vec3,
        v1: Vec3,
        v2: Vec3,
        v3: Vec3,
        normal: Vec3,
        color: [f32; 4],
        uv0: [f32; 2],
        uv1: [f32; 2],
        uv2: [f32; 2],
        uv3: [f32; 2],
    ) {
        let base = self.vertices.len() as u32;
        let n = normal.to_array();
        self.vertices.push(Vertex { position: v0.to_array(), normal: n, uv: uv0, color });
        self.vertices.push(Vertex { position: v1.to_array(), normal: n, uv: uv1, color });
        self.vertices.push(Vertex { position: v2.to_array(), normal: n, uv: uv2, color });
        self.vertices.push(Vertex { position: v3.to_array(), normal: n, uv: uv3, color });

        self.indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }

    pub fn add_box(&mut self, min: Vec3, max: Vec3, color: [f32; 4]) {
        // Front (+Z)
        self.add_quad(
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::Z,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
        // Back (-Z)
        self.add_quad(
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            -Vec3::Z,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
        // Top (+Y)
        self.add_quad(
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(min.x, max.y, min.z),
            Vec3::Y,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
        // Bottom (-Y)
        self.add_quad(
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(min.x, min.y, max.z),
            -Vec3::Y,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
        // Left (-X)
        self.add_quad(
            Vec3::new(min.x, min.y, min.z),
            Vec3::new(min.x, min.y, max.z),
            Vec3::new(min.x, max.y, max.z),
            Vec3::new(min.x, max.y, min.z),
            -Vec3::X,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
        // Right (+X)
        self.add_quad(
            Vec3::new(max.x, min.y, max.z),
            Vec3::new(max.x, min.y, min.z),
            Vec3::new(max.x, max.y, min.z),
            Vec3::new(max.x, max.y, max.z),
            Vec3::X,
            color,
            [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
        );
    }

    pub fn add_cylinder(
        &mut self,
        p0: Vec3,
        p1: Vec3,
        radius: f32,
        radial_segments: usize,
        color: [f32; 4],
    ) {
        let forward = (p1 - p0).normalize_or_zero();
        if forward.length_squared() < 1e-4 {
            return;
        }
        let arbitrary = if forward.y.abs() < 0.99 { Vec3::Y } else { Vec3::X };
        let right = forward.cross(arbitrary).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();

        let radial = radial_segments.max(6);
        let base_start = self.vertices.len() as u32;

        for r in 0..radial {
            let angle = (r as f32 / radial as f32) * std::f32::consts::TAU;
            let normal = right * angle.cos() + up * angle.sin();
            let offset = normal * radius;

            self.vertices.push(Vertex {
                position: (p0 + offset).to_array(),
                normal: normal.to_array(),
                uv: [r as f32 / radial as f32, 0.0],
                color,
            });
            self.vertices.push(Vertex {
                position: (p1 + offset).to_array(),
                normal: normal.to_array(),
                uv: [r as f32 / radial as f32, 1.0],
                color,
            });
        }

        for r in 0..radial {
            let next_r = (r + 1) % radial;
            let i0 = base_start + (r * 2) as u32;
            let i1 = base_start + (r * 2 + 1) as u32;
            let i2 = base_start + (next_r * 2) as u32;
            let i3 = base_start + (next_r * 2 + 1) as u32;

            self.indices.extend_from_slice(&[i0, i1, i2, i2, i1, i3]);
        }
    }

    pub fn add_curved_screen(
        &mut self,
        center: Vec3,
        width: f32,
        height: f32,
        curvature_depth: f32,
        grid_x: usize,
        grid_y: usize,
    ) {
        let base = self.vertices.len() as u32;
        let gx = grid_x.max(4);
        let gy = grid_y.max(4);

        for y in 0..=gy {
            let ty = y as f32 / gy as f32;
            let py = (ty - 0.5) * height;

            for x in 0..=gx {
                let tx = x as f32 / gx as f32;
                let px = (tx - 0.5) * width;

                let nx = (tx - 0.5) * 2.0;
                let ny = (ty - 0.5) * 2.0;
                let dist_sq = (nx * nx + ny * ny).min(1.0);
                let pz = (1.0 - dist_sq) * curvature_depth;

                let pos = center + Vec3::new(px, py, pz);
                let normal = Vec3::new(nx * 0.2, ny * 0.2, 1.0).normalize_or_zero();

                self.vertices.push(Vertex {
                    position: pos.to_array(),
                    normal: normal.to_array(),
                    uv: [tx, 1.0 - ty],
                    color: [1.0, 1.0, 1.0, 1.0],
                });
            }
        }

        let stride = (gx + 1) as u32;
        for y in 0..gy {
            for x in 0..gx {
                let row1 = base + (y as u32) * stride + (x as u32);
                let row2 = row1 + stride;

                self.indices.extend_from_slice(&[
                    row1,
                    row1 + 1,
                    row2,
                    row1 + 1,
                    row2 + 1,
                    row2,
                ]);
            }
        }
    }
}
