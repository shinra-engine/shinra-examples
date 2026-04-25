pub struct Engine {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub color: wgpu::Texture,
    pub depth: wgpu::Texture,
    pub size: (u32, u32),
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

        Self {
            device,
            queue,
            color,
            depth,
            size: (width, height),
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
}
