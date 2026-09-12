use crate::mesh::CpuMesh;
use glam::Vec3;

pub struct RetroRoomMeshes {
    /// Static room geometry: walls, floor, rug, furniture, TV chassis, consoles, power strip
    pub room_static_mesh: CpuMesh,
    /// 3D CRT screen face with authentic 4:3 curved glass geometry
    pub crt_screen_3d_mesh: CpuMesh,
}

impl RetroRoomMeshes {
    pub fn build_all() -> Self {
        let mut room_mesh = CpuMesh::default();

        // 1. Room Architecture
        build_room_box(&mut room_mesh);

        // 2. Entertainment Center (TV Stand)
        build_entertainment_center(&mut room_mesh);

        // 3. 21-inch CRT Television Chassis (Casing, Bezel, Buttons, Rear Jacks)
        build_crt_chassis(&mut room_mesh);

        // 4. Consoles on Shelves: PS1, PS2, N64
        build_consoles(&mut room_mesh);

        // 5. Power Strip & Wall Outlet
        build_power_accessories(&mut room_mesh);

        // 6. CRT Screen 3D Mesh (separate mesh for CRT post-processing shader)
        let mut crt_screen_3d_mesh = CpuMesh::default();
        // TV is at Vec3::new(0.0, 0.74, -1.80)
        // Screen center is at Y = 0.77, Z = -1.56 (front of the bezel)
        crt_screen_3d_mesh.add_curved_screen(
            Vec3::new(0.0, 0.77, -1.562),
            0.43,   // width (meters)
            0.3225, // height (4:3 ratio)
            0.012,  // convex bulge
            16,
            16,
        );

        Self {
            room_static_mesh: room_mesh,
            crt_screen_3d_mesh,
        }
    }
}

/// Constructs walls, floor, ceiling, rug, baseboards, and wall posters.
fn build_room_box(mesh: &mut CpuMesh) {
    let x_min = -2.6;
    let x_max = 2.6;
    let z_min = -2.35; // Back wall behind TV
    let z_max = 1.6;   // Front wall behind player
    let y_floor = 0.0;
    let y_ceil = 2.6;

    // A. Hardwood Floor with alternating plank tones
    let plank_rows = 14;
    let z_step = (z_max - z_min) / plank_rows as f32;
    for i in 0..plank_rows {
        let z0 = z_min + (i as f32) * z_step;
        let z1 = z0 + z_step;
        let tone = if i % 2 == 0 {
            [0.46, 0.31, 0.18, 1.0] // Warm oak
        } else {
            [0.41, 0.27, 0.15, 1.0] // Darker grain
        };
        mesh.add_quad(
            Vec3::new(x_min, y_floor, z1),
            Vec3::new(x_max, y_floor, z1),
            Vec3::new(x_max, y_floor, z0),
            Vec3::new(x_min, y_floor, z0),
            Vec3::Y,
            tone,
            [0.0, 1.0], [4.0, 1.0], [4.0, 0.0], [0.0, 0.0],
        );
    }

    // B. Cozy Retro Area Rug in front of the TV
    let rug_min = Vec3::new(-1.25, 0.003, -2.10);
    let rug_max = Vec3::new(1.25, 0.003, -0.30);
    // Dark teal field
    mesh.add_quad(
        Vec3::new(rug_min.x, rug_min.y, rug_max.z),
        Vec3::new(rug_max.x, rug_min.y, rug_max.z),
        Vec3::new(rug_max.x, rug_min.y, rug_min.z),
        Vec3::new(rug_min.x, rug_min.y, rug_min.z),
        Vec3::Y,
        [0.14, 0.26, 0.32, 1.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    // Burgundy geometric borders
    let border_w = 0.08;
    let border_color = [0.55, 0.16, 0.18, 1.0];
    // Top & Bottom rug border
    mesh.add_quad(
        Vec3::new(rug_min.x, 0.004, rug_min.z + border_w),
        Vec3::new(rug_max.x, 0.004, rug_min.z + border_w),
        Vec3::new(rug_max.x, 0.004, rug_min.z),
        Vec3::new(rug_min.x, 0.004, rug_min.z),
        Vec3::Y,
        border_color,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    mesh.add_quad(
        Vec3::new(rug_min.x, 0.004, rug_max.z),
        Vec3::new(rug_max.x, 0.004, rug_max.z),
        Vec3::new(rug_max.x, 0.004, rug_max.z - border_w),
        Vec3::new(rug_min.x, 0.004, rug_max.z - border_w),
        Vec3::Y,
        border_color,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    // Left & Right borders
    mesh.add_quad(
        Vec3::new(rug_min.x, 0.004, rug_max.z),
        Vec3::new(rug_min.x + border_w, 0.004, rug_max.z),
        Vec3::new(rug_min.x + border_w, 0.004, rug_min.z),
        Vec3::new(rug_min.x, 0.004, rug_min.z),
        Vec3::Y,
        border_color,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    mesh.add_quad(
        Vec3::new(rug_max.x - border_w, 0.004, rug_max.z),
        Vec3::new(rug_max.x, 0.004, rug_max.z),
        Vec3::new(rug_max.x, 0.004, rug_min.z),
        Vec3::new(rug_max.x - border_w, 0.004, rug_min.z),
        Vec3::Y,
        border_color,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );

    // C. Back Wall: Lower dark wood wainscoting (0 to 0.85m), upper vintage wallpaper (0.85 to 2.6m)
    let y_wainscot = 0.85;
    // Lower wood wainscoting
    mesh.add_quad(
        Vec3::new(x_min, y_floor, z_min),
        Vec3::new(x_max, y_floor, z_min),
        Vec3::new(x_max, y_wainscot, z_min),
        Vec3::new(x_min, y_wainscot, z_min),
        Vec3::Z,
        [0.26, 0.16, 0.10, 1.0], // Dark mahogany
        [0.0, 0.0], [4.0, 0.0], [4.0, 1.0], [0.0, 1.0],
    );
    // Chair rail trim
    mesh.add_box(
        Vec3::new(x_min, y_wainscot - 0.02, z_min),
        Vec3::new(x_max, y_wainscot + 0.03, z_min + 0.02),
        [0.32, 0.20, 0.12, 1.0],
    );
    // Upper vintage cream/taupe wallpaper
    mesh.add_quad(
        Vec3::new(x_min, y_wainscot, z_min),
        Vec3::new(x_max, y_wainscot, z_min),
        Vec3::new(x_max, y_ceil, z_min),
        Vec3::new(x_min, y_ceil, z_min),
        Vec3::Z,
        [0.68, 0.65, 0.60, 1.0],
        [0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0],
    );

    // D. Left Wall (X = x_min)
    mesh.add_quad(
        Vec3::new(x_min, y_floor, z_max),
        Vec3::new(x_min, y_floor, z_min),
        Vec3::new(x_min, y_ceil, z_min),
        Vec3::new(x_min, y_ceil, z_max),
        Vec3::X,
        [0.65, 0.62, 0.58, 1.0],
        [0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0],
    );

    // E. Right Wall (X = x_max)
    mesh.add_quad(
        Vec3::new(x_max, y_floor, z_min),
        Vec3::new(x_max, y_floor, z_max),
        Vec3::new(x_max, y_ceil, z_max),
        Vec3::new(x_max, y_ceil, z_min),
        -Vec3::X,
        [0.65, 0.62, 0.58, 1.0],
        [0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0],
    );

    // F. Front Wall behind camera (Z = z_max)
    mesh.add_quad(
        Vec3::new(x_max, y_floor, z_max),
        Vec3::new(x_min, y_floor, z_max),
        Vec3::new(x_min, y_ceil, z_max),
        Vec3::new(x_max, y_ceil, z_max),
        -Vec3::Z,
        [0.64, 0.61, 0.57, 1.0],
        [0.0, 0.0], [4.0, 0.0], [4.0, 3.0], [0.0, 3.0],
    );

    // G. Ceiling
    mesh.add_quad(
        Vec3::new(x_min, y_ceil, z_min),
        Vec3::new(x_max, y_ceil, z_min),
        Vec3::new(x_max, y_ceil, z_max),
        Vec3::new(x_min, y_ceil, z_max),
        -Vec3::Y,
        [0.82, 0.80, 0.76, 1.0],
        [0.0, 0.0], [2.0, 0.0], [2.0, 2.0], [0.0, 2.0],
    );

    // H. Baseboards along wall perimeters
    let base_color = [0.24, 0.15, 0.09, 1.0];
    let base_h = 0.10;
    let base_t = 0.02;
    // Back wall baseboard
    mesh.add_box(Vec3::new(x_min, 0.0, z_min), Vec3::new(x_max, base_h, z_min + base_t), base_color);
    // Left wall baseboard
    mesh.add_box(Vec3::new(x_min, 0.0, z_min), Vec3::new(x_min + base_t, base_h, z_max), base_color);
    // Right wall baseboard
    mesh.add_box(Vec3::new(x_max - base_t, 0.0, z_min), Vec3::new(x_max, base_h, z_max), base_color);

    // I. Ceiling Lamp Fixture
    let lamp_pos = Vec3::new(0.0, y_ceil - 0.04, -0.9);
    mesh.add_cylinder(lamp_pos, lamp_pos + Vec3::new(0.0, 0.04, 0.0), 0.16, 12, [0.75, 0.65, 0.35, 1.0]); // Brass rim
    mesh.add_cylinder(lamp_pos - Vec3::new(0.0, 0.06, 0.0), lamp_pos, 0.14, 12, [0.98, 0.96, 0.90, 1.0]); // Frosted glass globe

    // J. Retro Posters on Back Wall
    build_poster(mesh, Vec3::new(-1.45, 1.65, z_min + 0.005), 0.55, 0.75, [0.15, 0.12, 0.28, 1.0], [0.85, 0.25, 0.55, 1.0]);
    build_poster(mesh, Vec3::new(1.45, 1.65, z_min + 0.005), 0.55, 0.75, [0.10, 0.22, 0.18, 1.0], [0.25, 0.75, 0.85, 1.0]);
}

fn build_poster(mesh: &mut CpuMesh, center: Vec3, width: f32, height: f32, bg_col: [f32; 4], accent_col: [f32; 4]) {
    let hw = width * 0.5;
    let hh = height * 0.5;
    // Dark frame
    mesh.add_box(
        center - Vec3::new(hw + 0.02, hh + 0.02, 0.0),
        center + Vec3::new(hw + 0.02, hh + 0.02, 0.01),
        [0.10, 0.10, 0.10, 1.0],
    );
    // Poster face
    mesh.add_quad(
        center + Vec3::new(-hw, -hh, 0.011),
        center + Vec3::new(hw, -hh, 0.011),
        center + Vec3::new(hw, hh, 0.011),
        center + Vec3::new(-hw, hh, 0.011),
        Vec3::Z,
        bg_col,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    // Accent banner
    mesh.add_quad(
        center + Vec3::new(-hw * 0.8, -hh * 0.4, 0.012),
        center + Vec3::new(hw * 0.8, -hh * 0.4, 0.012),
        center + Vec3::new(hw * 0.8, hh * 0.4, 0.012),
        center + Vec3::new(-hw * 0.8, hh * 0.4, 0.012),
        Vec3::Z,
        accent_col,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
}

/// Constructs the retro wooden media cabinet / entertainment stand.
fn build_entertainment_center(mesh: &mut CpuMesh) {
    let wood_dark = [0.26, 0.16, 0.09, 1.0];
    let wood_med = [0.32, 0.20, 0.12, 1.0];

    let w = 1.48; // Width
    let d = 0.62; // Depth
    let z_center = -1.80;
    let z0 = z_center - d * 0.5; // -2.11
    let z1 = z_center + d * 0.5; // -1.49
    let hw = w * 0.5;           // 0.74

    let y_top = 0.46;
    let y_shelf = 0.22;
    let y_bottom = 0.05;

    // 1. Main Tabletop (holds the CRT TV)
    mesh.add_box(
        Vec3::new(-hw, y_top - 0.035, z0),
        Vec3::new(hw, y_top, z1),
        wood_med,
    );

    // 2. Middle Shelf (holds consoles)
    mesh.add_box(
        Vec3::new(-hw + 0.03, y_shelf - 0.02, z0 + 0.02),
        Vec3::new(hw - 0.03, y_shelf, z1 - 0.02),
        wood_dark,
    );

    // 3. Bottom Shelf
    mesh.add_box(
        Vec3::new(-hw + 0.03, y_bottom, z0 + 0.02),
        Vec3::new(hw - 0.03, y_bottom + 0.02, z1 - 0.02),
        wood_dark,
    );

    // 4. Four sturdy legs
    let leg_r = 0.035;
    let leg_h = y_top - 0.035;
    let leg_offsets = [
        (-hw + 0.05, z0 + 0.05),
        (hw - 0.05, z0 + 0.05),
        (-hw + 0.05, z1 - 0.05),
        (hw - 0.05, z1 - 0.05),
    ];
    for (lx, lz) in leg_offsets {
        mesh.add_box(
            Vec3::new(lx - leg_r, 0.0, lz - leg_r),
            Vec3::new(lx + leg_r, leg_h, lz + leg_r),
            wood_dark,
        );
    }

    // 5. Vertical dividers separating left, middle, right console bays
    for &dx in &[-0.24, 0.24] {
        mesh.add_box(
            Vec3::new(dx - 0.015, y_bottom + 0.02, z0 + 0.03),
            Vec3::new(dx + 0.015, y_top - 0.035, z1 - 0.03),
            wood_dark,
        );
    }

    // 6. Recessed Backing Board with cable holes
    mesh.add_box(
        Vec3::new(-hw + 0.03, y_bottom + 0.02, z0 + 0.02),
        Vec3::new(hw - 0.03, y_top - 0.035, z0 + 0.035),
        [0.18, 0.12, 0.07, 1.0],
    );
}

/// Constructs the 21-inch CRT Television housing, front bezel, buttons, speaker grilles, and rear RCA ports.
fn build_crt_chassis(mesh: &mut CpuMesh) {
    let tv_center = Vec3::new(0.0, 0.74, -1.80);
    let tv_dark = [0.12, 0.12, 0.14, 1.0]; // Sony Trinitron matte dark slate
    let tv_bezel_dark = [0.10, 0.10, 0.11, 1.0];
    let tv_accent = [0.22, 0.22, 0.24, 1.0];

    let w = 0.64;
    let h = 0.52;
    let d = 0.44;
    let hw = w * 0.5; // 0.32
    let hh = h * 0.5; // 0.26
    let hd = d * 0.5; // 0.22

    let z_front = tv_center.z + hd; // -1.58
    let z_back = tv_center.z - hd;  // -2.02

    // 1. Front Main Bezel Box
    mesh.add_box(
        Vec3::new(-hw, tv_center.y - hh, z_front - 0.08),
        Vec3::new(hw, tv_center.y + hh, z_front),
        tv_dark,
    );

    // 2. Bezel Border framing the screen (top, bottom, left, right borders)
    let screen_hw = 0.22;
    let screen_top = 0.94;
    let screen_bot = 0.60;
    let bezel_lip_color = [0.08, 0.08, 0.09, 1.0];

    // Left bezel bar
    mesh.add_box(
        Vec3::new(-hw, screen_bot, z_front - 0.01),
        Vec3::new(-screen_hw, screen_top, z_front + 0.005),
        bezel_lip_color,
    );
    // Right bezel bar
    mesh.add_box(
        Vec3::new(screen_hw, screen_bot, z_front - 0.01),
        Vec3::new(hw, screen_top, z_front + 0.005),
        bezel_lip_color,
    );
    // Top bezel bar
    mesh.add_box(
        Vec3::new(-hw, screen_top, z_front - 0.01),
        Vec3::new(hw, tv_center.y + hh, z_front + 0.005),
        bezel_lip_color,
    );

    // 3. Lower Front Chin (Control Bar & Speakers)
    let chin_y0 = tv_center.y - hh; // 0.48
    let chin_y1 = screen_bot;       // 0.60
    mesh.add_box(
        Vec3::new(-hw, chin_y0, z_front - 0.01),
        Vec3::new(hw, chin_y1, z_front + 0.008),
        tv_bezel_dark,
    );

    // A. Power button & Glowing Green LED
    mesh.add_box(
        Vec3::new(-0.25, 0.525, z_front + 0.009),
        Vec3::new(-0.21, 0.555, z_front + 0.015),
        tv_accent,
    );
    // Green Power LED indicator
    mesh.add_box(
        Vec3::new(-0.19, 0.535, z_front + 0.009),
        Vec3::new(-0.18, 0.545, z_front + 0.012),
        [0.10, 0.95, 0.20, 1.0], // Bright LED green
    );

    // B. Channel and Volume Dials / Buttons
    for i in 0..4 {
        let bx = -0.12 + (i as f32) * 0.035;
        mesh.add_box(
            Vec3::new(bx - 0.01, 0.53, z_front + 0.009),
            Vec3::new(bx + 0.01, 0.55, z_front + 0.014),
            tv_accent,
        );
    }

    // C. Trinitron / EmuSim Brand Emblem
    mesh.add_box(
        Vec3::new(-0.045, 0.575, z_front + 0.009),
        Vec3::new(0.045, 0.588, z_front + 0.012),
        [0.70, 0.70, 0.72, 1.0], // Metallic badge
    );

    // D. Stereo Speaker Grille Horizontal Slots
    for row in 0..4 {
        let sy = 0.51 + (row as f32) * 0.02;
        mesh.add_box(Vec3::new(0.12, sy, z_front + 0.009), Vec3::new(0.28, sy + 0.008, z_front + 0.012), [0.05, 0.05, 0.05, 1.0]);
    }

    // 4. Rear Tapered CRT Tube Housing
    mesh.add_box(
        Vec3::new(-0.24, tv_center.y - 0.18, z_back),
        Vec3::new(0.24, tv_center.y + 0.18, z_front - 0.08),
        tv_dark,
    );
    // Tapered end-cap
    mesh.add_box(
        Vec3::new(-0.16, tv_center.y - 0.12, z_back - 0.04),
        Vec3::new(0.16, tv_center.y + 0.12, z_back),
        [0.09, 0.09, 0.10, 1.0],
    );

    // 5. Rear Connection Panel with RCA Ports (Yellow Video, White Audio L, Red Audio R)
    let rca_z = z_back - 0.041;
    let rca_y = tv_center.y - 0.05;
    mesh.add_box(
        Vec3::new(-0.12, rca_y - 0.04, rca_z),
        Vec3::new(0.04, rca_y + 0.04, rca_z + 0.005),
        [0.05, 0.05, 0.05, 1.0],
    );
    mesh.add_cylinder(Vec3::new(-0.09, rca_y, rca_z), Vec3::new(-0.09, rca_y, rca_z - 0.015), 0.008, 8, [0.95, 0.85, 0.10, 1.0]);
    mesh.add_cylinder(Vec3::new(-0.05, rca_y, rca_z), Vec3::new(-0.05, rca_y, rca_z - 0.015), 0.008, 8, [0.90, 0.90, 0.90, 1.0]);
    mesh.add_cylinder(Vec3::new(-0.01, rca_y, rca_z), Vec3::new(-0.01, rca_y, rca_z - 0.015), 0.008, 8, [0.90, 0.15, 0.15, 1.0]);
    mesh.add_box(
        Vec3::new(0.07, rca_y - 0.02, rca_z),
        Vec3::new(0.11, rca_y + 0.02, rca_z + 0.008),
        [0.02, 0.02, 0.02, 1.0],
    );
}

/// Constructs 3D models of the Sony PlayStation 1, Sony PlayStation 2, and Nintendo 64 on the shelves.
fn build_consoles(mesh: &mut CpuMesh) {
    let shelf_y = 0.22;

    // 1. Sony PlayStation 1 (Left Bay)
    let ps1_pos = Vec3::new(-0.46, shelf_y, -1.72);
    let ps1_gray = [0.72, 0.72, 0.74, 1.0];
    let ps1_dark_gray = [0.55, 0.55, 0.57, 1.0];

    mesh.add_box(
        ps1_pos + Vec3::new(-0.13, 0.0, -0.095),
        ps1_pos + Vec3::new(0.13, 0.055, 0.095),
        ps1_gray,
    );
    mesh.add_cylinder(
        ps1_pos + Vec3::new(0.0, 0.055, -0.01),
        ps1_pos + Vec3::new(0.0, 0.060, -0.01),
        0.065,
        14,
        ps1_dark_gray,
    );
    mesh.add_cylinder(
        ps1_pos + Vec3::new(-0.09, 0.055, 0.05),
        ps1_pos + Vec3::new(-0.09, 0.062, 0.05),
        0.015,
        8,
        ps1_dark_gray,
    );
    mesh.add_cylinder(
        ps1_pos + Vec3::new(0.09, 0.055, 0.05),
        ps1_pos + Vec3::new(0.09, 0.062, 0.05),
        0.015,
        8,
        ps1_dark_gray,
    );
    mesh.add_box(
        ps1_pos + Vec3::new(-0.09, 0.015, 0.095),
        ps1_pos + Vec3::new(-0.03, 0.042, 0.098),
        [0.20, 0.20, 0.22, 1.0],
    );
    mesh.add_box(
        ps1_pos + Vec3::new(0.03, 0.015, 0.095),
        ps1_pos + Vec3::new(0.09, 0.042, 0.098),
        [0.20, 0.20, 0.22, 1.0],
    );

    // 2. Nintendo 64 (Center Bay)
    let n64_pos = Vec3::new(0.0, shelf_y, -1.72);
    let n64_charcoal = [0.18, 0.18, 0.20, 1.0];
    let n64_accent = [0.12, 0.12, 0.14, 1.0];

    mesh.add_box(
        n64_pos + Vec3::new(-0.13, 0.0, -0.095),
        n64_pos + Vec3::new(0.13, 0.065, 0.095),
        n64_charcoal,
    );
    mesh.add_box(
        n64_pos + Vec3::new(-0.11, 0.01, 0.095),
        n64_pos + Vec3::new(0.11, 0.045, 0.105),
        n64_accent,
    );
    for port in 0..4 {
        let px = -0.075 + (port as f32) * 0.05;
        mesh.add_cylinder(
            n64_pos + Vec3::new(px, 0.028, 0.10),
            n64_pos + Vec3::new(px, 0.028, 0.106),
            0.010,
            8,
            [0.05, 0.05, 0.05, 1.0],
        );
    }
    mesh.add_box(
        n64_pos + Vec3::new(-0.065, 0.065, -0.04),
        n64_pos + Vec3::new(0.065, 0.135, -0.015),
        [0.45, 0.45, 0.47, 1.0],
    );
    mesh.add_quad(
        n64_pos + Vec3::new(-0.055, 0.075, -0.014),
        n64_pos + Vec3::new(0.055, 0.075, -0.014),
        n64_pos + Vec3::new(0.055, 0.125, -0.014),
        n64_pos + Vec3::new(-0.055, 0.125, -0.014),
        Vec3::Z,
        [0.85, 0.20, 0.15, 1.0],
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    mesh.add_box(
        n64_pos + Vec3::new(-0.09, 0.065, 0.03),
        n64_pos + Vec3::new(-0.07, 0.075, 0.06),
        n64_accent,
    );
    mesh.add_box(
        n64_pos + Vec3::new(0.0, 0.045, 0.106),
        n64_pos + Vec3::new(0.008, 0.053, 0.108),
        [0.95, 0.15, 0.10, 1.0],
    );

    // 3. Sony PlayStation 2 (Right Bay)
    let ps2_pos = Vec3::new(0.46, shelf_y, -1.72);
    let ps2_black = [0.09, 0.09, 0.10, 1.0];
    let ps2_blue = [0.10, 0.45, 0.90, 1.0];

    mesh.add_box(
        ps2_pos + Vec3::new(-0.14, 0.0, -0.11),
        ps2_pos + Vec3::new(0.14, 0.075, 0.11),
        ps2_black,
    );
    for rib in 0..5 {
        let ry = 0.012 + (rib as f32) * 0.012;
        mesh.add_box(
            ps2_pos + Vec3::new(-0.138, ry, 0.11),
            ps2_pos + Vec3::new(0.138, ry + 0.004, 0.113),
            [0.05, 0.05, 0.05, 1.0],
        );
    }
    mesh.add_box(
        ps2_pos + Vec3::new(-0.12, 0.042, 0.111),
        ps2_pos + Vec3::new(-0.01, 0.068, 0.113),
        [0.06, 0.06, 0.07, 1.0],
    );
    mesh.add_quad(
        ps2_pos + Vec3::new(-0.10, 0.048, 0.114),
        ps2_pos + Vec3::new(-0.07, 0.048, 0.114),
        ps2_pos + Vec3::new(-0.07, 0.062, 0.114),
        ps2_pos + Vec3::new(-0.10, 0.062, 0.114),
        Vec3::Z,
        ps2_blue,
        [0.0, 1.0], [1.0, 1.0], [1.0, 0.0], [0.0, 0.0],
    );
    mesh.add_box(
        ps2_pos + Vec3::new(0.09, 0.055, 0.111),
        ps2_pos + Vec3::new(0.12, 0.068, 0.114),
        [0.20, 0.20, 0.22, 1.0],
    );
    mesh.add_box(
        ps2_pos + Vec3::new(0.122, 0.058, 0.111),
        ps2_pos + Vec3::new(0.128, 0.065, 0.114),
        [0.10, 0.95, 0.20, 1.0],
    );
}

/// Constructs the floor power strip and wall outlet.
fn build_power_accessories(mesh: &mut CpuMesh) {
    let ps_pos = Vec3::new(0.40, 0.0, -2.15);
    mesh.add_box(
        ps_pos + Vec3::new(-0.04, 0.0, -0.16),
        ps_pos + Vec3::new(0.04, 0.035, 0.16),
        [0.85, 0.84, 0.80, 1.0],
    );
    mesh.add_box(
        ps_pos + Vec3::new(-0.025, 0.035, -0.13),
        ps_pos + Vec3::new(0.025, 0.045, -0.09),
        [0.95, 0.20, 0.15, 1.0],
    );
    for i in 0..6 {
        let oz = -0.06 + (i as f32) * 0.038;
        mesh.add_box(
            ps_pos + Vec3::new(-0.015, 0.035, oz - 0.01),
            ps_pos + Vec3::new(0.015, 0.037, oz + 0.01),
            [0.15, 0.15, 0.15, 1.0],
        );
    }

    let outlet_pos = Vec3::new(0.52, 0.35, -2.35);
    mesh.add_box(
        outlet_pos + Vec3::new(-0.04, -0.06, 0.0),
        outlet_pos + Vec3::new(0.04, 0.06, 0.008),
        [0.88, 0.86, 0.82, 1.0],
    );
    mesh.add_box(
        outlet_pos + Vec3::new(-0.02, 0.005, 0.008),
        outlet_pos + Vec3::new(0.02, 0.045, 0.045),
        [0.12, 0.12, 0.12, 1.0],
    );
}
