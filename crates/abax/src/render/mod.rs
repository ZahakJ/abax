//! The soroban renderer: one `egui_wgpu` paint callback per soroban, one
//! pipeline, one instanced draw.
//!
//! Shared GPU state (the pipeline and its bind-group layout) is created once
//! by [`install`] and kept in egui-wgpu's `callback_resources`; a
//! [`GpuContext`] (device, queue, layout) goes into egui's context data so a
//! widget can create its own buffers the first time it draws. Each
//! [`SorobanRenderer`] owns an instance buffer, a uniform buffer and a bind
//! group: two sorobans painted in one frame never share a buffer.
//!
//! Instances are built on the CPU by `SorobanView` (see
//! `soroban_view::build_instances`) into a reused `Vec`, copied to the GPU in
//! `prepare` and drawn in `paint`. Text, digits and arrows stay in egui and
//! are painted after the callback.

use std::sync::{Arc, Mutex};

use eframe::egui_wgpu::{self, wgpu, CallbackResources, CallbackTrait, ScreenDescriptor};
use egui::{Color32, Rect};

/// Instance kinds, in the order they are drawn. Must match `soroban.wgsl`.
pub mod kind {
    pub const BOARD: u32 = 0;
    pub const WELL: u32 = 1;
    pub const ACTIVE_GLOW: u32 = 2;
    pub const ROD: u32 = 3;
    pub const BEAM: u32 = 4;
    pub const DOT: u32 = 5;
    pub const UNDERLINE: u32 = 6;
    pub const HINT_GLOW: u32 = 7;
    pub const SHADOW: u32 = 8;
    pub const BEAD: u32 = 9;
}

/// One quad. `rect` is (x, y, w, h) in points relative to the callback
/// origin; `kind` is (kind, variant, 0, 0); the rest is per-kind (see the
/// shader's `shade_*` functions). 96 bytes.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Instance {
    pub rect: [f32; 4],
    pub kind: [u32; 4],
    pub params: [f32; 4],
    pub color_a: [f32; 4],
    pub color_b: [f32; 4],
    pub color_c: [f32; 4],
}

impl Instance {
    pub fn new(kind: u32, variant: u32, rect: Rect) -> Self {
        Self {
            rect: [rect.min.x, rect.min.y, rect.width(), rect.height()],
            kind: [kind, variant, 0, 0],
            ..Default::default()
        }
    }

    pub fn params(mut self, p: [f32; 4]) -> Self {
        self.params = p;
        self
    }

    pub fn colors(mut self, a: [f32; 4], b: [f32; 4], c: [f32; 4]) -> Self {
        self.color_a = a;
        self.color_b = b;
        self.color_c = c;
        self
    }
}

/// Instances per soroban the buffers hold: board, well, glow, beam and
/// underline, then per rod a rod, up to one dot, five shadows, five beads
/// and up to five hint glows (16), for the 40-rod maximum.
pub const MAX_INSTANCES: usize = 40 * 16 + 32;

/// The uniform block. 48 bytes; layout matches `Globals` in the shader.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable)]
struct Globals {
    screen_px: [f32; 2],
    ppp: f32,
    time: f32,
    origin: [f32; 2],
    srgb_target: u32,
    _pad: u32,
    light: [f32; 4],
}

/// Key light: upper left, in front (y up, z toward the viewer).
const LIGHT: [f32; 4] = [-0.45, 0.62, 0.64, 0.0];

/// sRGB-encoded `Color32` to linear RGB with the given alpha (straight).
pub fn linear(c: Color32, alpha: f32) -> [f32; 4] {
    let f = |u: u8| egui::ecolor::linear_f32_from_gamma_u8(u);
    [f(c.r()), f(c.g()), f(c.b()), alpha]
}

/// Pipeline state shared by every soroban, kept in `callback_resources`.
struct SharedPipeline {
    pipeline: wgpu::RenderPipeline,
}

/// What a widget needs to create its own buffers; kept in egui's context
/// data under [`GPU_ID`].
#[derive(Clone)]
pub struct GpuContext {
    device: wgpu::Device,
    layout: wgpu::BindGroupLayout,
    srgb_target: bool,
}

fn gpu_id() -> egui::Id {
    egui::Id::new("abax-soroban-gpu")
}

/// Create the shared pipeline. Call once at startup with eframe's render
/// state; without it the widget keeps the classic egui painter path.
pub fn install(rs: &egui_wgpu::RenderState, ctx: &egui::Context) {
    let device = &rs.device;
    let srgb_target = rs.target_format.is_srgb();
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("abax-soroban"),
        source: wgpu::ShaderSource::Wgsl(include_str!("soroban.wgsl").into()),
    });
    let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("abax-soroban-globals"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: std::num::NonZeroU64::new(std::mem::size_of::<Globals>() as u64),
            },
            count: None,
        }],
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("abax-soroban"),
        bind_group_layouts: &[Some(&layout)],
        immediate_size: 0,
    });
    let attributes = wgpu::vertex_attr_array![
        0 => Float32x4,
        1 => Uint32x4,
        2 => Float32x4,
        3 => Float32x4,
        4 => Float32x4,
        5 => Float32x4,
    ];
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("abax-soroban"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[Some(wgpu::VertexBufferLayout {
                array_stride: std::mem::size_of::<Instance>() as u64,
                step_mode: wgpu::VertexStepMode::Instance,
                attributes: &attributes,
            })],
            compilation_options: Default::default(),
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            cull_mode: None,
            ..Default::default()
        },
        // eframe's default: no depth buffer, no MSAA (the shader does its own
        // anti-aliasing).
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            // Premultiplied alpha, the same blend egui uses.
            targets: &[Some(wgpu::ColorTargetState {
                format: rs.target_format,
                blend: Some(wgpu::BlendState {
                    color: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::One,
                        dst_factor: wgpu::BlendFactor::OneMinusSrcAlpha,
                        operation: wgpu::BlendOperation::Add,
                    },
                    alpha: wgpu::BlendComponent {
                        src_factor: wgpu::BlendFactor::OneMinusDstAlpha,
                        dst_factor: wgpu::BlendFactor::One,
                        operation: wgpu::BlendOperation::Add,
                    },
                }),
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        multiview_mask: None,
        cache: None,
    });
    rs.renderer
        .write()
        .callback_resources
        .insert(SharedPipeline { pipeline });
    let gpu = GpuContext {
        device: device.clone(),
        layout,
        srgb_target,
    };
    ctx.data_mut(|d| d.insert_temp(gpu_id(), Arc::new(gpu)));
}

/// The GPU context, if [`install`] ran (i.e. eframe runs on wgpu).
pub fn gpu_context(ctx: &egui::Context) -> Option<Arc<GpuContext>> {
    ctx.data(|d| d.get_temp::<Arc<GpuContext>>(gpu_id()))
}

/// Per-widget GPU buffers.
struct ViewBuffers {
    instances: wgpu::Buffer,
    globals: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    srgb_target: bool,
}

/// A soroban's handle on the renderer: its own buffers plus the instance
/// list it fills each frame (reused, never reallocated after warm-up).
#[derive(Clone)]
pub struct SorobanRenderer {
    buffers: Arc<ViewBuffers>,
    frame: Arc<Mutex<Vec<Instance>>>,
}

impl SorobanRenderer {
    pub fn new(gpu: &GpuContext) -> Self {
        let instances = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("abax-soroban-instances"),
            size: (MAX_INSTANCES * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let globals = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("abax-soroban-globals"),
            size: std::mem::size_of::<Globals>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("abax-soroban-globals"),
            layout: &gpu.layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: globals.as_entire_binding(),
            }],
        });
        Self {
            buffers: Arc::new(ViewBuffers {
                instances,
                globals,
                bind_group,
                srgb_target: gpu.srgb_target,
            }),
            frame: Arc::new(Mutex::new(Vec::with_capacity(MAX_INSTANCES))),
        }
    }

    /// Fill this frame's instances with `build`, then add the paint
    /// callback to `painter`. `origin` is the point instance rects are
    /// relative to; `clip` bounds the callback.
    pub fn paint(
        &self,
        painter: &egui::Painter,
        origin: egui::Pos2,
        clip: Rect,
        time: f32,
        build: impl FnOnce(&mut Vec<Instance>),
    ) {
        let count = {
            let mut v = self.frame.lock().unwrap_or_else(|e| e.into_inner());
            v.clear();
            build(&mut v);
            v.truncate(MAX_INSTANCES);
            v.len() as u32
        };
        painter.add(egui_wgpu::Callback::new_paint_callback(
            clip,
            PaintCallback {
                buffers: self.buffers.clone(),
                frame: self.frame.clone(),
                count,
                origin: [origin.x, origin.y],
                time,
            },
        ));
    }
}

struct PaintCallback {
    buffers: Arc<ViewBuffers>,
    frame: Arc<Mutex<Vec<Instance>>>,
    count: u32,
    origin: [f32; 2],
    time: f32,
}

impl CallbackTrait for PaintCallback {
    fn prepare(
        &self,
        _device: &wgpu::Device,
        queue: &wgpu::Queue,
        screen: &ScreenDescriptor,
        _encoder: &mut wgpu::CommandEncoder,
        _resources: &mut CallbackResources,
    ) -> Vec<wgpu::CommandBuffer> {
        let globals = Globals {
            screen_px: [
                screen.size_in_pixels[0] as f32,
                screen.size_in_pixels[1] as f32,
            ],
            ppp: screen.pixels_per_point,
            time: self.time,
            origin: self.origin,
            srgb_target: self.buffers.srgb_target as u32,
            _pad: 0,
            light: LIGHT,
        };
        queue.write_buffer(&self.buffers.globals, 0, bytemuck::bytes_of(&globals));
        let v = self.frame.lock().unwrap_or_else(|e| e.into_inner());
        let n = (self.count as usize).min(v.len());
        if n > 0 {
            queue.write_buffer(&self.buffers.instances, 0, bytemuck::cast_slice(&v[..n]));
        }
        Vec::new()
    }

    fn paint(
        &self,
        info: egui::PaintCallbackInfo,
        pass: &mut wgpu::RenderPass<'static>,
        resources: &CallbackResources,
    ) {
        let Some(shared) = resources.get::<SharedPipeline>() else {
            return;
        };
        if self.count == 0 {
            return;
        }
        // Positions are computed against the whole screen (the shader adds
        // the origin), so the viewport never depends on how egui clamps the
        // callback rect at the window edge. The scissor (clip) still holds.
        let [w, h] = info.screen_size_px;
        pass.set_viewport(0.0, 0.0, w as f32, h as f32, 0.0, 1.0);
        pass.set_pipeline(&shared.pipeline);
        pass.set_bind_group(0, &self.buffers.bind_group, &[]);
        let bytes = self.count as u64 * std::mem::size_of::<Instance>() as u64;
        pass.set_vertex_buffer(0, self.buffers.instances.slice(0..bytes));
        pass.draw(0..6, 0..self.count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn instance_layout_matches_the_shader() {
        assert_eq!(std::mem::size_of::<Instance>(), 96);
        assert_eq!(std::mem::size_of::<Globals>(), 48);
    }

    fn validated() -> (
        eframe::wgpu::naga::Module,
        eframe::wgpu::naga::valid::ModuleInfo,
    ) {
        use eframe::wgpu::naga;
        let src = include_str!("soroban.wgsl");
        let module = naga::front::wgsl::parse_str(src)
            .unwrap_or_else(|e| panic!("{}", e.emit_to_string(src)));
        let info = naga::valid::Validator::new(
            naga::valid::ValidationFlags::all(),
            naga::valid::Capabilities::empty(),
        )
        .validate(&module)
        .expect("shader validates");
        (module, info)
    }

    #[test]
    fn shader_parses_and_validates() {
        validated();
    }

    /// Android may run on the GLES backend: the shader must translate to
    /// GLSL ES 3.00 (both stages) as well as to SPIR-V (Vulkan).
    #[test]
    fn shader_translates_for_gles_and_vulkan() {
        use eframe::wgpu::naga;
        let (module, info) = validated();
        for (stage, entry) in [
            (naga::ShaderStage::Vertex, "vs_main"),
            (naga::ShaderStage::Fragment, "fs_main"),
        ] {
            let options = naga::back::glsl::Options {
                version: naga::back::glsl::Version::Embedded {
                    version: 300,
                    is_webgl: false,
                },
                ..Default::default()
            };
            let pipeline = naga::back::glsl::PipelineOptions {
                shader_stage: stage,
                entry_point: entry.to_string(),
                multiview: None,
            };
            let mut out = String::new();
            naga::back::glsl::Writer::new(
                &mut out,
                &module,
                &info,
                &options,
                &pipeline,
                naga::proc::BoundsCheckPolicies::default(),
            )
            .and_then(|mut w| w.write())
            .unwrap_or_else(|e| panic!("GLSL ES {entry}: {e}"));
            assert!(out.contains("#version 300 es"));
        }
        naga::back::spv::write_vec(&module, &info, &Default::default(), None).expect("SPIR-V");
    }

    #[test]
    fn linear_conversion() {
        assert_eq!(linear(Color32::WHITE, 1.0), [1.0, 1.0, 1.0, 1.0]);
        let mid = linear(Color32::from_gray(128), 0.5);
        assert!((mid[0] - 0.2158).abs() < 0.01);
        assert_eq!(mid[3], 0.5);
    }
}
