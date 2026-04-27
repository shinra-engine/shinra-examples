use shinra::{
    engine::Engine,
    mesh::Mesh,
    presenter::{window::WindowPresenter, FrameCtx, Presenter},
    scene::{BunnyTag, Camera, MeshHandle, Model, PlayerControlled, Projection, Scene, TeapotTag},
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

const SCALE_MIN: f32 = 0.5;
const SCALE_MAX: f32 = 50.0;
const SCALE_FACTOR: f32 = 1.10;

const MOVE_STEP: f32 = 0.10;

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

struct TeapotCtrl {
    translation: glam::Vec3,
}

struct Initialized {
    window: Arc<Window>,
    engine: Engine,
    presenter: WindowPresenter,
    scene: Scene,
    ctrl: CameraCtrl,
    tctrl: TeapotCtrl,
    bunny_entity: hecs::Entity,
    teapot_entity: hecs::Entity,
    bunny_scale: f32,
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
            radius: 5.0,
            target: glam::Vec3::new(1.5, 0.5, 0.0),
        };

        let tctrl = TeapotCtrl {
            translation: glam::Vec3::new(3.0, 0.0, 0.0),
        };

        let bunny_mesh = Arc::new(Mesh::from_obj_file("assets/bunny.obj").unwrap());
        let teapot_mesh = Arc::new(Mesh::from_obj_file("assets/teapot.obj").unwrap());

        let mut scene = Scene::new(Camera {
            eye: ctrl.eye(),
            target: ctrl.target,
            up: glam::Vec3::Y,
            projection: Projection::Perspective {
                fov_y_radians: 45f32.to_radians(),
                aspect: self.render_w as f32 / self.render_h as f32,
                znear: 0.1,
                zfar: 100.0,
            },
        });

        let bunny_entity = scene.world.spawn((
            MeshHandle(bunny_mesh),
            Model(glam::Mat4::from_scale(glam::Vec3::splat(10.0))),
            BunnyTag,
        ));
        let teapot_entity = scene.world.spawn((
            MeshHandle(teapot_mesh),
            Model(glam::Mat4::from_translation(tctrl.translation)),
            TeapotTag,
            PlayerControlled,
        ));

        self.state = Some(Initialized {
            window,
            engine,
            presenter,
            scene,
            ctrl,
            tctrl,
            bunny_entity,
            teapot_entity,
            bunny_scale: 10.0,
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
                Key::Character(ref c) => match c.as_str() {
                    "w" => s.tctrl.translation.z -= MOVE_STEP,
                    "s" => s.tctrl.translation.z += MOVE_STEP,
                    "a" => s.tctrl.translation.x -= MOVE_STEP,
                    "d" => s.tctrl.translation.x += MOVE_STEP,
                    "j" => s.bunny_scale = (s.bunny_scale * SCALE_FACTOR).min(SCALE_MAX),
                    "k" => s.bunny_scale = (s.bunny_scale / SCALE_FACTOR).max(SCALE_MIN),
                    _ => {}
                },
                _ => {}
            },
            WindowEvent::Resized(sz) => {
                s.presenter.resize(&s.engine.device, sz.width, sz.height);
            }
            WindowEvent::RedrawRequested => {
                s.scene.camera.eye = s.ctrl.eye();
                s.scene.camera.target = s.ctrl.target;
                s.scene.set_model(
                    s.bunny_entity,
                    glam::Mat4::from_scale(glam::Vec3::splat(s.bunny_scale)),
                );
                s.scene.set_model(
                    s.teapot_entity,
                    glam::Mat4::from_translation(s.tctrl.translation),
                );
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
