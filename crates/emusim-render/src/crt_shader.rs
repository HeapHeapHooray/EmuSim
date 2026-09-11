use bytemuck::{Pod, Zeroable};


/// Uniform parameters sent to the CRT screen post-processing shader.
#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct CrtShaderUniforms {
    pub resolution: [f32; 2],
    pub curvature_barrel: f32, // typically 0.08 to 0.15 for CRT curvature
    pub scanline_intensity: f32, // 0.25 to 0.50
    pub phosphor_mask_strength: f32, // 0.3
    pub bloom_glow: f32,
    pub static_noise_intensity: f32, // 1.0 when unconnected, 0.0 when playing
    pub time_seconds: f32,
    pub degauss_active: f32, // > 0.0 during degauss wobble
    pub power_fade: f32, // 0.0 = off, 1.0 = fully on, animates vertical collapse
    pub _padding: [f32; 2],
}

impl Default for CrtShaderUniforms {
    fn default() -> Self {
        Self {
            resolution: [320.0, 240.0],
            curvature_barrel: 0.12,
            scanline_intensity: 0.35,
            phosphor_mask_strength: 0.25,
            bloom_glow: 0.15,
            static_noise_intensity: 0.0,
            time_seconds: 0.0,
            degauss_active: 0.0,
            power_fade: 1.0,
            _padding: [0.0; 2],
        }
    }
}

/// CRT Screen WGSL / SPIR-V compatible fragment shader code.
pub const CRT_FRAGMENT_WGSL: &str = r#"
struct CrtUniforms {
    resolution: vec2<f32>,
    curvature_barrel: f32,
    scanline_intensity: f32,
    phosphor_mask_strength: f32,
    bloom_glow: f32,
    static_noise_intensity: f32,
    time_seconds: f32,
    degauss_active: f32,
    power_fade: f32,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var screen_sampler: sampler;
@group(0) @binding(2) var<uniform> crt: CrtUniforms;

fn curve_uv(uv: vec2<f32>, curvature: f32) -> vec2<f32> {
    var p = uv * 2.0 - 1.0;
    var offset = p.yx * p.yx;
    p = p + p * offset * curvature;
    return p * 0.5 + 0.5;
}

fn hash21(p: vec2<f32>) -> f32 {
    return fract(sin(dot(p, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@fragment
fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    if crt.power_fade <= 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    // Curvature distortion
    var curved_uv = curve_uv(uv, crt.curvature_barrel);

    // Degauss magnetic wobble
    if crt.degauss_active > 0.0 {
        let wobble = sin(curved_uv.y * 50.0 + crt.time_seconds * 30.0) * 0.015 * crt.degauss_active;
        curved_uv.x += wobble;
    }

    // Screen bezel vignette / black border
    if curved_uv.x < 0.0 || curved_uv.x > 1.0 || curved_uv.y < 0.0 || curved_uv.y > 1.0 {
        return vec4<f32>(0.02, 0.02, 0.02, 1.0);
    }

    var color: vec3<f32>;

    // TV static noise when disconnected
    if crt.static_noise_intensity > 0.5 {
        let noise = hash21(curved_uv * 1000.0 + vec2<f32>(crt.time_seconds * 133.0, crt.time_seconds * 97.0));
        color = vec3<f32>(noise);
    } else {
        color = textureSample(screen_texture, screen_sampler, curved_uv).rgb;
    }

    // Horizontal Scanlines
    let scanline = sin(curved_uv.y * crt.resolution.y * 3.14159) * 0.5 + 0.5;
    color *= (1.0 - crt.scanline_intensity * (1.0 - scanline));

    // RGB Phosphor aperture grille mask (Sony Trinitron / PVM style)
    let pixel_x = curved_uv.x * crt.resolution.x * 3.0;
    let triad_index = i32(pixel_x) % 3;
    var mask = vec3<f32>(1.0, 1.0, 1.0);
    if triad_index == 0 {
        mask = vec3<f32>(1.0 + crt.phosphor_mask_strength, 1.0 - crt.phosphor_mask_strength, 1.0 - crt.phosphor_mask_strength);
    } else if triad_index == 1 {
        mask = vec3<f32>(1.0 - crt.phosphor_mask_strength, 1.0 + crt.phosphor_mask_strength, 1.0 - crt.phosphor_mask_strength);
    } else {
        mask = vec3<f32>(1.0 - crt.phosphor_mask_strength, 1.0 - crt.phosphor_mask_strength, 1.0 + crt.phosphor_mask_strength);
    }
    color *= mask;

    // Glass bulb reflection & vignette
    let vig = 16.0 * curved_uv.x * curved_uv.y * (1.0 - curved_uv.x) * (1.0 - curved_uv.y);
    color *= clamp(pow(vig, 0.15), 0.0, 1.0);

    // Apply power fade / collapse
    color *= crt.power_fade;

    return vec4<f32>(color, 1.0);
}
"#;
