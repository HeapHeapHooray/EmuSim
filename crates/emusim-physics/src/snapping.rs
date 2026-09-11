use emusim_core::sockets::{SocketKind, SocketPort};
use glam::Vec3;

pub struct SnapZoneConfig {
    pub guide_radius: f32,       // e.g. 0.05m (5 cm)
    pub max_angle_degrees: f32,  // e.g. 40 degrees
    pub insertion_depth: f32,    // e.g. 0.015m (1.5 cm)
    pub pull_out_force: f32,     // threshold to unplug
}

impl Default for SnapZoneConfig {
    fn default() -> Self {
        Self {
            guide_radius: 0.04,
            max_angle_degrees: 35.0,
            insertion_depth: 0.018,
            pull_out_force: 8.0,
        }
    }
}

pub enum SnapState {
    Free,
    Guided {
        guided_pos: Vec3,
        insertion_progress: f32, // 0.0 = mouth, 1.0 = fully seated
    },
    Mated {
        seated_pos: Vec3,
    },
}

/// Evaluates snap alignment between a held plug and a fixed socket.
pub fn evaluate_socket_snap(
    socket: &SocketPort,
    socket_world_pos: Vec3,
    socket_world_normal: Vec3,
    plug_kind: &SocketKind,
    plug_world_pos: Vec3,
    plug_forward_dir: Vec3,
    config: &SnapZoneConfig,
) -> SnapState {
    if !socket.kind.is_compatible_with(plug_kind) {
        return SnapState::Free;
    }

    let to_plug = plug_world_pos - socket_world_pos;
    let distance = to_plug.length();

    if distance > config.guide_radius {
        return SnapState::Free;
    }

    // Alignment: plug forward dir should oppose socket normal (facing into socket)
    let dot = (-plug_forward_dir).dot(socket_world_normal);
    let min_cos = (config.max_angle_degrees.to_radians()).cos();
    if dot < min_cos {
        return SnapState::Free;
    }

    // Distance along the normal axis
    let depth_along_normal = to_plug.dot(socket_world_normal);

    // If pushed inside socket past threshold, it is fully mated
    if depth_along_normal <= 0.005 {
        return SnapState::Mated {
            seated_pos: socket_world_pos,
        };
    }

    // Magnetic guiding toward socket center axis
    let projected_on_axis = socket_world_pos + socket_world_normal * depth_along_normal;
    let t = 1.0 - (depth_along_normal / config.guide_radius).clamp(0.0, 1.0);

    SnapState::Guided {
        guided_pos: projected_on_axis,
        insertion_progress: t,
    }
}
