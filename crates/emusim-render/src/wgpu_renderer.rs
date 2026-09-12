use crate::crt_shader::CrtShaderUniforms;
use crate::mesh::{CpuMesh, Vertex};
use crate::room_mesh::RetroRoomMeshes;
use bytemuck::{cast_slice, Pod, Zeroable};
use emusim_libretro::framebuffer::VideoFrame;
use glam::{Mat4, Quat, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Clone, Copy, Pod, Zeroable)]
pub struct SceneUniforms {
    pub view_proj: [[f32; 4]; 4],
    pub camera_pos: [f32; 4],
    pub ceiling_light_pos: [f32; 4],
    pub ceiling_light_color: [f32; 4],
    pub crt_glow_pos: [f32; 4],
    pub crt_glow_color: [f32; 4],
    pub time_seconds: f32,
    pub is_direct_fullscreen: f32,
    pub _pad: [f32; 2],
}

impl Default for SceneUniforms {
    fn default() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
            camera_pos: [0.0, 1.35, 0.2, 1.0],
            ceiling_light_pos: [0.0, 2.55, -0.9, 1.0],
            ceiling_light_color: [1.0, 0.94, 0.82, 1.0],
            crt_glow_pos: [0.0, 0.77, -1.55, 1.0],
            crt_glow_color: [0.35, 0.45, 0.70, 1.0],
            time_seconds: 0.0,
            is_direct_fullscreen: 0.0,
            _pad: [0.0; 2],
        }
    }
}

pub struct Render3dContext<'a> {
    pub video_frame: &'a VideoFrame,
    pub crt_uniforms: &'a CrtShaderUniforms,
    pub camera_pos: Vec3,
    pub camera_rot: Quat,
    pub fov_y_degrees: f32,
    pub is_direct_fullscreen: bool,
    pub time_seconds: f32,
    pub crt_glow_color: [f32; 4],
    pub dynamic_cables: Option<&'a CpuMesh>,
}

pub struct WgpuCrtRenderer {
    pub surface: wgpu::Surface<'static>,
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub config: wgpu::SurfaceConfiguration,

    // Depth buffer
    pub depth_texture: wgpu::Texture,
    pub depth_view: wgpu::TextureView,

    // Uniforms & Bind Groups
    pub scene_uniform_buffer: wgpu::Buffer,
    pub scene_bind_group_layout: wgpu::BindGroupLayout,
    pub scene_bind_group: wgpu::BindGroup,

    pub crt_uniform_buffer: wgpu::Buffer,
    pub screen_texture: wgpu::Texture,
    pub screen_sampler: wgpu::Sampler,
    pub crt_bind_group_layout: wgpu::BindGroupLayout,
    pub crt_bind_group: wgpu::BindGroup,
    pub current_texture_size: (u32, u32),

    // Pipelines
    pub room_render_pipeline: wgpu::RenderPipeline,
    pub crt_render_pipeline: wgpu::RenderPipeline,

    // Static Room Meshes
    pub room_vertex_buffer: wgpu::Buffer,
    pub room_index_buffer: wgpu::Buffer,
    pub room_index_count: u32,

    // 3D CRT Screen Mesh
    pub crt_3d_vertex_buffer: wgpu::Buffer,
    pub crt_3d_index_buffer: wgpu::Buffer,
    pub crt_3d_index_count: u32,

    // 2D Fullscreen Quad Mesh (for direct fullscreen mode)
    pub fullscreen_vertex_buffer: wgpu::Buffer,
    pub fullscreen_index_buffer: wgpu::Buffer,
    pub fullscreen_index_count: u32,
}

impl WgpuCrtRenderer {
    pub async fn new(window: Arc<Window>) -> Result<Self, String> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window)
            .map_err(|e| format!("Failed to create wgpu surface: {e}"))?;

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| "No compatible GPU adapter found".to_string())?;

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("EmuSim Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .map_err(|e| format!("Failed to create GPU device: {e}"))?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // 1. Create Depth Texture
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // 2. Scene Uniforms Buffer & Bind Group Layout
        let scene_uniforms = SceneUniforms::default();
        let scene_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Scene Uniform Buffer"),
            contents: cast_slice(&[scene_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let scene_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("Scene Bind Group Layout"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
            });

        let scene_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Scene Bind Group"),
            layout: &scene_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: scene_uniform_buffer.as_entire_binding(),
            }],
        });

        // 3. CRT Texture & Uniforms
        let initial_tex_size = (320u32, 240u32);
        let screen_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Console Screen Texture"),
            size: wgpu::Extent3d {
                width: initial_tex_size.0,
                height: initial_tex_size.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let screen_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let initial_crt_uniforms = CrtShaderUniforms::default();
        let crt_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CRT Uniform Buffer"),
            contents: cast_slice(&[initial_crt_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let crt_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("CRT Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let texture_view = screen_texture.create_view(&wgpu::TextureViewDescriptor::default());
        let crt_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("CRT Bind Group"),
            layout: &crt_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&texture_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&screen_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: crt_uniform_buffer.as_entire_binding(),
                },
            ],
        });

        // 4. Room Shading Shader Module & Pipeline
        let room_shader_src = r#"
struct SceneUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    ceiling_light_pos: vec4<f32>,
    ceiling_light_color: vec4<f32>,
    crt_glow_pos: vec4<f32>,
    crt_glow_color: vec4<f32>,
    time_seconds: f32,
    is_direct_fullscreen: f32,
};

@group(0) @binding(0) var<uniform> scene: SceneUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) world_normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

@vertex
fn vs_room(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.world_pos = model.position;
    out.world_normal = normalize(model.normal);
    out.uv = model.uv;
    out.color = model.color;
    out.clip_position = scene.view_proj * vec4<f32>(model.position, 1.0);
    return out;
}

@fragment
fn fs_room(in: VertexOutput) -> @location(0) vec4<f32> {
    let N = normalize(in.world_normal);
    let V = normalize(scene.camera_pos.xyz - in.world_pos);

    // Warm ambient illumination
    let ambient = vec3<f32>(0.22, 0.20, 0.24) * in.color.rgb;

    // Overhead ceiling fixture lighting
    let L_ceil = scene.ceiling_light_pos.xyz - in.world_pos;
    let dist_ceil = length(L_ceil);
    let l_ceil_dir = normalize(L_ceil);
    let diff_ceil = max(dot(N, l_ceil_dir), 0.0);
    let atten_ceil = 1.0 / (1.0 + 0.15 * dist_ceil + 0.06 * dist_ceil * dist_ceil);
    let H_ceil = normalize(l_ceil_dir + V);
    let spec_ceil = pow(max(dot(N, H_ceil), 0.0), 24.0) * 0.15;
    let ceil_light = scene.ceiling_light_color.rgb * (diff_ceil * in.color.rgb + vec3<f32>(spec_ceil)) * atten_ceil;

    // Dynamic CRT Screen Glow (forward light cone from TV face)
    let L_crt = scene.crt_glow_pos.xyz - in.world_pos;
    let dist_crt = length(L_crt);
    let l_crt_dir = normalize(L_crt);
    let diff_crt = max(dot(N, l_crt_dir), 0.0);
    let atten_crt = 1.0 / (1.0 + 0.6 * dist_crt + 1.2 * dist_crt * dist_crt);
    let crt_forward = vec3<f32>(0.0, 0.0, 1.0);
    let crt_cone = max(dot(-l_crt_dir, crt_forward), 0.0);
    let crt_light = scene.crt_glow_color.rgb * in.color.rgb * diff_crt * atten_crt * crt_cone * 3.2;

    let final_color = ambient + ceil_light + crt_light;
    return vec4<f32>(final_color, in.color.a);
}
"#;

        let room_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Room 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(room_shader_src.into()),
        });

        let room_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Room Pipeline Layout"),
            bind_group_layouts: &[&scene_bind_group_layout],
            push_constant_ranges: &[],
        });

        let room_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Room Render Pipeline"),
            layout: Some(&room_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &room_shader,
                entry_point: "vs_room",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Self::vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &room_shader,
                entry_point: "fs_room",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 5. CRT Screen 3D Shader Module & Pipeline
        let crt_shader_src = r#"
struct SceneUniforms {
    view_proj: mat4x4<f32>,
    camera_pos: vec4<f32>,
    ceiling_light_pos: vec4<f32>,
    ceiling_light_color: vec4<f32>,
    crt_glow_pos: vec4<f32>,
    crt_glow_color: vec4<f32>,
    time_seconds: f32,
    is_direct_fullscreen: f32,
};

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

@group(0) @binding(0) var<uniform> scene: SceneUniforms;
@group(1) @binding(0) var screen_texture: texture_2d<f32>;
@group(1) @binding(1) var screen_sampler: sampler;
@group(1) @binding(2) var<uniform> crt: CrtUniforms;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_crt(model: VertexInput) -> VertexOutput {
    var out: VertexOutput;
    out.uv = model.uv;
    if scene.is_direct_fullscreen > 0.5 {
        out.clip_position = vec4<f32>(model.position.xy * 2.0, 0.0, 1.0);
    } else {
        out.clip_position = scene.view_proj * vec4<f32>(model.position, 1.0);
    }
    return out;
}

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
fn fs_crt(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {
    if crt.power_fade <= 0.001 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }

    var curved_uv = curve_uv(uv, crt.curvature_barrel);

    if crt.degauss_active > 0.0 {
        let wobble = sin(curved_uv.y * 50.0 + crt.time_seconds * 30.0) * 0.015 * crt.degauss_active;
        curved_uv.x += wobble;
    }

    if curved_uv.x < 0.0 || curved_uv.x > 1.0 || curved_uv.y < 0.0 || curved_uv.y > 1.0 {
        return vec4<f32>(0.02, 0.02, 0.02, 1.0);
    }

    var color: vec3<f32>;

    if crt.static_noise_intensity > 0.5 {
        let noise = hash21(curved_uv * 1000.0 + vec2<f32>(crt.time_seconds * 133.0, crt.time_seconds * 97.0));
        color = vec3<f32>(noise);
    } else {
        color = textureSample(screen_texture, screen_sampler, curved_uv).rgb;
    }

    // Horizontal Scanlines
    let scanline = sin(curved_uv.y * crt.resolution.y * 3.14159) * 0.5 + 0.5;
    color *= (1.0 - crt.scanline_intensity * (1.0 - scanline));

    // RGB Phosphor aperture grille mask (Sony Trinitron style)
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

    // Glass bulb vignette
    let vig = 16.0 * curved_uv.x * curved_uv.y * (1.0 - curved_uv.x) * (1.0 - curved_uv.y);
    color *= clamp(pow(vig, 0.15), 0.0, 1.0);

    color *= crt.power_fade;

    return vec4<f32>(color, 1.0);
}
"#;

        let crt_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("CRT 3D Shader"),
            source: wgpu::ShaderSource::Wgsl(crt_shader_src.into()),
        });

        let crt_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("CRT Pipeline Layout"),
            bind_group_layouts: &[&scene_bind_group_layout, &crt_bind_group_layout],
            push_constant_ranges: &[],
        });

        let crt_render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("CRT Render Pipeline"),
            layout: Some(&crt_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &crt_shader,
                entry_point: "vs_crt",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                buffers: &[Self::vertex_buffer_layout()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &crt_shader,
                entry_point: "fs_crt",
                compilation_options: wgpu::PipelineCompilationOptions::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::LessEqual,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        // 6. Build procedural 3D meshes for room and CRT screen
        let room_meshes = RetroRoomMeshes::build_all();

        let room_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Room Static Vertices"),
            contents: cast_slice(&room_meshes.room_static_mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let room_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Room Static Indices"),
            contents: cast_slice(&room_meshes.room_static_mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let room_index_count = room_meshes.room_static_mesh.indices.len() as u32;

        let crt_3d_vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CRT 3D Screen Vertices"),
            contents: cast_slice(&room_meshes.crt_screen_3d_mesh.vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let crt_3d_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("CRT 3D Screen Indices"),
            contents: cast_slice(&room_meshes.crt_screen_3d_mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let crt_3d_index_count = room_meshes.crt_screen_3d_mesh.indices.len() as u32;

        // 2D Fullscreen quad (fallback for direct 2D fullscreen mode)
        let quad_mesh = CpuMesh::crt_screen_quad(1.0, 1.0);
        let fullscreen_vertex_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Fullscreen Screen Quad Vertices"),
                contents: cast_slice(&quad_mesh.vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let fullscreen_index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Fullscreen Screen Quad Indices"),
            contents: cast_slice(&quad_mesh.indices),
            usage: wgpu::BufferUsages::INDEX,
        });
        let fullscreen_index_count = quad_mesh.indices.len() as u32;

        Ok(Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            scene_uniform_buffer,
            scene_bind_group_layout,
            scene_bind_group,
            crt_uniform_buffer,
            screen_texture,
            screen_sampler,
            crt_bind_group_layout,
            crt_bind_group,
            current_texture_size: initial_tex_size,
            room_render_pipeline,
            crt_render_pipeline,
            room_vertex_buffer,
            room_index_buffer,
            room_index_count,
            crt_3d_vertex_buffer,
            crt_3d_index_buffer,
            crt_3d_index_count,
            fullscreen_vertex_buffer,
            fullscreen_index_buffer,
            fullscreen_index_count,
        })
    }

    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: width.max(1),
                height: height.max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    fn vertex_buffer_layout() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 32,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x4,
                },
            ],
        }
    }

    pub fn resize(&mut self, new_width: u32, new_height: u32) {
        if new_width > 0 && new_height > 0 {
            self.config.width = new_width;
            self.config.height = new_height;
            self.surface.configure(&self.device, &self.config);
            let (tex, view) = Self::create_depth_texture(&self.device, new_width, new_height);
            self.depth_texture = tex;
            self.depth_view = view;
        }
    }

    /// Upload new frame from emulator to GPU texture.
    pub fn update_video_texture(&mut self, frame: &VideoFrame) {
        if frame.width == 0 || frame.height == 0 || frame.pixels.is_empty() {
            return;
        }

        if self.current_texture_size != (frame.width, frame.height) {
            self.screen_texture = self.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("Console Screen Texture"),
                size: wgpu::Extent3d {
                    width: frame.width,
                    height: frame.height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });

            let texture_view = self
                .screen_texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            self.crt_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("CRT Bind Group Recreated"),
                layout: &self.crt_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&texture_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&self.screen_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.crt_uniform_buffer.as_entire_binding(),
                    },
                ],
            });

            self.current_texture_size = (frame.width, frame.height);
        }

        self.queue.write_texture(
            wgpu::ImageCopyTexture {
                texture: &self.screen_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &frame.pixels,
            wgpu::ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * frame.width),
                rows_per_image: Some(frame.height),
            },
            wgpu::Extent3d {
                width: frame.width,
                height: frame.height,
                depth_or_array_layers: 1,
            },
        );
    }

    /// Primary 3D render function: renders retro room, furniture, TV chassis, consoles, dynamic cables, and CRT screen.
    pub fn render_3d(&mut self, ctx: &Render3dContext) -> Result<(), wgpu::SurfaceError> {
        self.update_video_texture(ctx.video_frame);

        // 1. Upload CRT shader uniforms
        self.queue
            .write_buffer(&self.crt_uniform_buffer, 0, cast_slice(&[*ctx.crt_uniforms]));

        // 2. Compute View-Projection matrix
        let aspect = (self.config.width as f32) / (self.config.height as f32).max(1.0);
        let fov_rad = ctx.fov_y_degrees.to_radians();
        let proj = Mat4::perspective_rh(fov_rad, aspect, 0.05, 50.0);
        let view = Mat4::from_rotation_translation(ctx.camera_rot, ctx.camera_pos).inverse();
        let view_proj = proj * view;

        // 3. Upload Scene uniforms
        let scene_uniforms = SceneUniforms {
            view_proj: view_proj.to_cols_array_2d(),
            camera_pos: [ctx.camera_pos.x, ctx.camera_pos.y, ctx.camera_pos.z, 1.0],
            ceiling_light_pos: [0.0, 2.55, -0.9, 1.0],
            ceiling_light_color: [1.0, 0.94, 0.82, 1.0],
            crt_glow_pos: [0.0, 0.77, -1.55, 1.0],
            crt_glow_color: ctx.crt_glow_color,
            time_seconds: ctx.time_seconds,
            is_direct_fullscreen: if ctx.is_direct_fullscreen { 1.0 } else { 0.0 },
            _pad: [0.0; 2],
        };
        self.queue
            .write_buffer(&self.scene_uniform_buffer, 0, cast_slice(&[scene_uniforms]));

        // 4. Create Dynamic Cable Buffers (if provided)
        let cable_buffers = ctx.dynamic_cables.and_then(|cables| {
            if cables.vertices.is_empty() || cables.indices.is_empty() {
                None
            } else {
                let vbuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Dynamic Cables Vertices"),
                    contents: cast_slice(&cables.vertices),
                    usage: wgpu::BufferUsages::VERTEX,
                });
                let ibuf = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Dynamic Cables Indices"),
                    contents: cast_slice(&cables.indices),
                    usage: wgpu::BufferUsages::INDEX,
                });
                Some((vbuf, ibuf, cables.indices.len() as u32))
            }
        });

        // 5. Render Pass
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("EmuSim Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("EmuSim Scene Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.04,
                            b: 0.05,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            if ctx.is_direct_fullscreen {
                // Direct Fullscreen 2D Mode: Draw screen quad directly
                render_pass.set_pipeline(&self.crt_render_pipeline);
                render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
                render_pass.set_bind_group(1, &self.crt_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.fullscreen_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.fullscreen_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.fullscreen_index_count, 0, 0..1);
            } else {
                // Full 3D Retro Room Mode:
                // Step A: Draw static room, TV stand, TV casing, and consoles
                render_pass.set_pipeline(&self.room_render_pipeline);
                render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.room_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.room_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.room_index_count, 0, 0..1);

                // Step B: Draw dynamic Verlet cables (if any)
                if let Some((ref vbuf, ref ibuf, count)) = cable_buffers {
                    render_pass.set_vertex_buffer(0, vbuf.slice(..));
                    render_pass.set_index_buffer(ibuf.slice(..), wgpu::IndexFormat::Uint32);
                    render_pass.draw_indexed(0..count, 0, 0..1);
                }

                // Step C: Draw 3D curved CRT screen face with CRT shader
                render_pass.set_pipeline(&self.crt_render_pipeline);
                render_pass.set_bind_group(0, &self.scene_bind_group, &[]);
                render_pass.set_bind_group(1, &self.crt_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.crt_3d_vertex_buffer.slice(..));
                render_pass.set_index_buffer(
                    self.crt_3d_index_buffer.slice(..),
                    wgpu::IndexFormat::Uint32,
                );
                render_pass.draw_indexed(0..self.crt_3d_index_count, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }

    /// Legacy render wrapper for backwards compatibility with tests.
    pub fn render(
        &mut self,
        video_frame: &VideoFrame,
        uniforms: &CrtShaderUniforms,
    ) -> Result<(), wgpu::SurfaceError> {
        self.render_3d(&Render3dContext {
            video_frame,
            crt_uniforms: uniforms,
            camera_pos: Vec3::new(0.0, 0.77, -0.96),
            camera_rot: Quat::IDENTITY,
            fov_y_degrees: 60.0,
            is_direct_fullscreen: false,
            time_seconds: uniforms.time_seconds,
            crt_glow_color: if uniforms.power_fade > 0.0 {
                [0.35, 0.45, 0.65, 1.0]
            } else {
                [0.0; 4]
            },
            dynamic_cables: None,
        })
    }
}
