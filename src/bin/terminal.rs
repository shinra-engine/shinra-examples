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
    scene::{orbit_eye, Camera, Projection, Scene},
};
use std::{
    io::stdout,
    sync::Arc,
    time::{Duration, Instant},
};

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

    let mesh = Arc::new(Mesh::from_obj_file("assets/bunny.obj")?);
    let mut scene = Scene::new(Camera {
        eye: orbit_eye(0.0, 3.0, 1.5),
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
    let start = Instant::now();

    loop {
        let frame_start = Instant::now();

        let t = start.elapsed().as_secs_f32();
        scene.camera.eye = orbit_eye(t * 0.6, 3.0, 1.5);

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
                _ => {}
            }
        }

        let elapsed = frame_start.elapsed();
        if elapsed < frame_duration {
            std::thread::sleep(frame_duration - elapsed);
        }
    }
}
