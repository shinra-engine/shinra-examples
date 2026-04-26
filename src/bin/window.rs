use shinra::{
    engine::Engine,
    mesh::Mesh,
    presenter::{window::WindowPresenter, FrameCtx, Presenter},
    scene::{Camera, Projection, Scene},
};
use std::sync::Arc;
use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{Key, NamedKey},
    window::{Window, WindowId},
};

const YAW_STEP: f32 = 0.10;
const PITCH_STEP: f32 = 0.10;
const PITCH_MIN: f32 = -1.4;
const PITCH_MAX: f32 = 1.4;

struct CameraCtrl {
    yaw: f32,
    pitch: f32,
    radius: f32,
    target: glam::Vec3,
}

impl CameraCtrl {
    fn eye(&self) -> glam::Vec3 {
        glam::Vec3::new(
            self.target.x + self.radius * self.pitch.cos() * self.yaw.sin(),
            self.target.y + self.radius * self.pitch.sin(),
            self.target.z + self.radius * self.pitch.cos() * self.yaw.cos(),
        )
    }
}

struct Initialized {
    window: Arc<Window>,
    engine: Engine,
    presenter: WindowPresenter,
    scene: Scene,
    ctrl: CameraCtrl,
}

struct App {
    instance: wgpu::Instance,
    state: Option<Initialized>,
    render_w: u32,
    render_h: u32,
}

impl App {
    fn new() -> Self {
        Self {
            instance: wgpu::Instance::default(),
            state: None,
            render_w: 256,
            render_h: 144,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.state.is_some() {
            return;
        }

        let window_attrs = Window::default_attributes()
            .with_title("shinra — window")
            .with_inner_size(winit::dpi::LogicalSize::new(800u32, 600u32));
        let window = Arc::new(event_loop.create_window(window_attrs).unwrap());

        let surface = self.instance.create_surface(window.clone()).unwrap();
        let adapter =
            pollster::block_on(self.instance.request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                power_preference: wgpu::PowerPreference::HighPerformance,
                ..Default::default()
            }))
            .expect("no adapter");

        let (device, queue) =
            pollster::block_on(adapter.request_device(&Default::default(), None)).unwrap();

        let engine = Engine::from_existing(device, queue, self.render_w, self.render_h);

        let win_size = window.inner_size();
        let presenter = WindowPresenter::new(
            &engine.device,
            &adapter,
            surface,
            win_size.width.max(1),
            win_size.height.max(1),
        );

        let ctrl = CameraCtrl {
            yaw: 0.0,
            pitch: 0.4,
            radius: 3.0,
            target: glam::Vec3::ZERO,
        };

        let mesh = Arc::new(Mesh::from_obj_file("assets/teapot.obj").unwrap());
        let mut scene = Scene::new(Camera {
            eye: ctrl.eye(),
            target: glam::Vec3::ZERO,
            up: glam::Vec3::Y,
            projection: Projection::Perspective {
                fov_y_radians: 45f32.to_radians(),
                aspect: self.render_w as f32 / self.render_h as f32,
                znear: 0.1,
                zfar: 100.0,
            },
        });
        scene.add(mesh);

        self.state = Some(Initialized {
            window,
            engine,
            presenter,
            scene,
            ctrl,
        });
        self.state.as_ref().unwrap().window.request_redraw();
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        if let WindowEvent::CloseRequested = event {
            event_loop.exit();
            return;
        }

        let Some(s) = self.state.as_mut() else {
            return;
        };

        match event {
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key,
                        ..
                    },
                ..
            } => match logical_key {
                Key::Named(NamedKey::ArrowLeft) => s.ctrl.yaw -= YAW_STEP,
                Key::Named(NamedKey::ArrowRight) => s.ctrl.yaw += YAW_STEP,
                Key::Named(NamedKey::ArrowUp) => {
                    s.ctrl.pitch = (s.ctrl.pitch + PITCH_STEP).min(PITCH_MAX)
                }
                Key::Named(NamedKey::ArrowDown) => {
                    s.ctrl.pitch = (s.ctrl.pitch - PITCH_STEP).max(PITCH_MIN)
                }
                Key::Named(NamedKey::Escape) => event_loop.exit(),
                _ => {}
            },
            WindowEvent::Resized(sz) => {
                s.presenter.resize(&s.engine.device, sz.width, sz.height);
            }
            WindowEvent::RedrawRequested => {
                s.scene.camera.eye = s.ctrl.eye();
                s.engine.render(&s.scene);
                let mut ctx = FrameCtx {
                    device: &s.engine.device,
                    queue: &s.engine.queue,
                    texture: &s.engine.color,
                    width: s.engine.size.0,
                    height: s.engine.size.1,
                };
                s.presenter.present(&mut ctx);
                s.window.request_redraw();
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new()?;
    let mut app = App::new();
    event_loop.run_app(&mut app)?;
    Ok(())
}
