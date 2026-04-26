use crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use shinra::{
    engine::Engine,
    mesh::Mesh,
    presenter::{terminal::TerminalPresenter, FrameCtx, Presenter},
    scene::{Camera, Projection, Scene},
};
use std::{
    io::stdout,
    sync::Arc,
    time::{Duration, Instant},
};

const YAW_STEP: f32 = 0.10;
const PITCH_STEP: f32 = 0.10;
const PITCH_MIN: f32 = -1.4;
const PITCH_MAX: f32 = 1.4;

const SCALE_MIN: f32 = 0.5;
const SCALE_MAX: f32 = 50.0;
const SCALE_FACTOR: f32 = 1.10;

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

struct ModelCtrl {
    scale: f32,
}

impl ModelCtrl {
    fn matrix(&self) -> glam::Mat4 {
        glam::Mat4::from_scale(glam::Vec3::splat(self.scale))
    }
}

struct RawModeGuard;

impl RawModeGuard {
    fn enter() -> Self {
        enable_raw_mode().expect("enable raw mode");
        execute!(stdout(), Hide).expect("hide cursor");
        Self
    }
}

impl Drop for RawModeGuard {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(stdout(), Show);
    }
}

fn main() -> anyhow::Result<()> {
    let (cols, rows) = size()?;
    let cols = cols.max(20) as u32;
    let rows = rows.max(20) as u32;

    let width = (cols * 2).min(320);
    let height = (rows * 4).min(180);

    let mut engine = Engine::new(width, height);
    let mut presenter = TerminalPresenter::new(&engine.device, width, height);

    let mut ctrl = CameraCtrl {
        yaw: 0.0,
        pitch: 0.4,
        radius: 3.0,
        target: glam::Vec3::ZERO,
    };

    let mut mctrl = ModelCtrl { scale: 10.0 };

    let mesh = Arc::new(Mesh::from_obj_file("assets/bunny.obj")?);
    let mut scene = Scene::new(Camera {
        eye: ctrl.eye(),
        target: glam::Vec3::ZERO,
        up: glam::Vec3::Y,
        projection: Projection::Perspective {
            fov_y_radians: 45f32.to_radians(),
            aspect: width as f32 / height as f32,
            znear: 0.1,
            zfar: 100.0,
        },
    });
    scene.add(mesh);

    let _guard = RawModeGuard::enter();

    let frame_duration = Duration::from_millis(33);

    loop {
        let frame_start = Instant::now();

        scene.camera.eye = ctrl.eye();
        scene.set_drawable_model(0, mctrl.matrix());

        engine.render(&scene);

        let mut ctx = FrameCtx {
            device: &engine.device,
            queue: &engine.queue,
            texture: &engine.color,
            width: engine.size.0,
            height: engine.size.1,
        };
        presenter.present(&mut ctx);

        while event::poll(Duration::from_millis(0))? {
            match event::read()? {
                Event::Key(KeyEvent {
                    code: KeyCode::Esc, ..
                })
                | Event::Key(KeyEvent {
                    code: KeyCode::Char('q'),
                    ..
                }) => {
                    return Ok(());
                }
                Event::Key(KeyEvent {
                    code: KeyCode::Left,
                    ..
                }) => ctrl.yaw -= YAW_STEP,
                Event::Key(KeyEvent {
                    code: KeyCode::Right,
                    ..
                }) => ctrl.yaw += YAW_STEP,
                Event::Key(KeyEvent {
                    code: KeyCode::Up, ..
                }) => ctrl.pitch = (ctrl.pitch + PITCH_STEP).min(PITCH_MAX),
                Event::Key(KeyEvent {
                    code: KeyCode::Down,
                    ..
                }) => ctrl.pitch = (ctrl.pitch - PITCH_STEP).max(PITCH_MIN),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('j'),
                    ..
                }) => mctrl.scale = (mctrl.scale * SCALE_FACTOR).min(SCALE_MAX),
                Event::Key(KeyEvent {
                    code: KeyCode::Char('k'),
                    ..
                }) => mctrl.scale = (mctrl.scale / SCALE_FACTOR).max(SCALE_MIN),
                _ => {}
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
