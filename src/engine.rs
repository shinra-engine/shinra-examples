use wgpu::util::DeviceExt;

pub struct Engine {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub color: wgpu::Texture,
    pub depth: wgpu::Texture,
    pub size: (u32, u32),
    pipeline: wgpu::RenderPipeline,
    camera_buf: wgpu::Buffer,
    camera_bgl: wgpu::BindGroupLayout,
    camera_bg: wgpu::BindGroup,
}

impl Engine {
    /// Build a headless engine (no window/surface) at the given render size.
    pub fn new(width: u32, height: u32) -> Self {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::HighPerformance,
            compatible_surface: None,
            force_fallback_adapter: false,
        }))
        .expect("failed to find a suitable wgpu adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor::default(), None))
                .expect("failed to create wgpu device");

        let color = Self::make_color(&device, width, height);
        let depth = Self::make_depth(&device, width, height);

        let camera_bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("camera_bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            }],
        });

        let identity: [f32; 16] = glam::Mat4::IDENTITY.to_cols_array();
        let camera_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("camera"),
            contents: bytemuck::bytes_of(&identity),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let camera_bg = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("camera_bg"),
            layout: &camera_bgl,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&camera_bgl],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[crate::mesh::Vertex::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Rgba8UnormSrgb,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Self {
            device,
            queue,
            color,
            depth,
            size: (width, height),
            pipeline,
            camera_buf,
            camera_bgl,
            camera_bg,
        }
    }

    /// Reallocate color + depth textures at a new size.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.color = Self::make_color(&self.device, width, height);
        self.depth = Self::make_depth(&self.device, width, height);
        self.size = (width, height);
    }

    fn make_color(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("color"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                | wgpu::TextureUsages::TEXTURE_BINDING
                | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        })
    }

    fn make_depth(device: &wgpu::Device, width: u32, height: u32) -> wgpu::Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_init() {
        let engine = Engine::new(64, 64);
        assert_eq!(engine.size, (64, 64));

        let color_size = engine.color.size();
        assert_eq!(color_size.width, 64);
        assert_eq!(color_size.height, 64);
        assert_eq!(engine.color.format(), wgpu::TextureFormat::Rgba8UnormSrgb);

        let depth_size = engine.depth.size();
        assert_eq!(depth_size.width, 64);
        assert_eq!(depth_size.height, 64);
        assert_eq!(engine.depth.format(), wgpu::TextureFormat::Depth32Float);
    }

    #[test]
    fn engine_resize() {
        let mut engine = Engine::new(64, 64);
        engine.resize(128, 96);
        assert_eq!(engine.size, (128, 96));

        let color_size = engine.color.size();
        assert_eq!(color_size.width, 128);
        assert_eq!(color_size.height, 96);

        let depth_size = engine.depth.size();
        assert_eq!(depth_size.width, 128);
        assert_eq!(depth_size.height, 96);
    }

    #[test]
    fn engine_pipeline() {
        let engine = Engine::new(64, 64);
        assert_eq!(engine.camera_buf.size(), 64);
    }
}
