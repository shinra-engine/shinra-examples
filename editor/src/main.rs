use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use scene::Scene;
use shinra_engine::engine::Engine;

const RENDER_W: u32 = 512;
const RENDER_H: u32 = 384;

#[derive(PartialEq, Eq, Clone, Debug)]
enum PanelKind {
    Viewport,
    SceneTree,
    Inspector,
    Palette,
}

struct App {
    #[allow(dead_code)]
    scene: Scene,
    #[allow(dead_code)]
    selected_node: Option<usize>,
    dock: DockState<PanelKind>,
    engine: Engine,
    viewport_texture_id: egui::TextureId,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("eframe must be configured with the wgpu backend");
        // Reuse eframe's wgpu device — Engine renders to its own offscreen
        // texture, then we register that texture with egui_wgpu's renderer.
        let engine = Engine::from_existing(
            render_state.device.clone(),
            render_state.queue.clone(),
            RENDER_W,
            RENDER_H,
        );

        // Register engine.color as a native egui texture.
        let view = engine.color.create_view(&Default::default());
        let mut renderer = render_state.renderer.write();
        let viewport_texture_id = renderer.register_native_texture(
            &engine.device,
            &view,
            wgpu::FilterMode::Linear,
        );

        // Default dock layout: viewport center, scene-tree left, inspector right, palette bottom.
        let mut dock = DockState::new(vec![PanelKind::Viewport]);
        let surface = dock.main_surface_mut();
        let [_viewport, _scene_tree] =
            surface.split_left(NodeIndex::root(), 0.2, vec![PanelKind::SceneTree]);
        let [_viewport, _inspector] =
            surface.split_right(NodeIndex::root(), 0.75, vec![PanelKind::Inspector]);
        let [_viewport, _palette] =
            surface.split_below(NodeIndex::root(), 0.7, vec![PanelKind::Palette]);

        Self {
            scene: Scene::default(),
            selected_node: None,
            dock,
            engine,
            viewport_texture_id,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Render the (currently empty) scene into our offscreen texture.
        // Slice 4+ will populate with tilemap quads, slice 5 with mesh nodes.
        let sc = shinra_engine::scene::Scene::new(camera());
        let _ = &self.scene;
        self.engine.render(&sc);

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.scene = Scene::default();
                        ui.close_menu();
                    }
                    if ui.button("Open…").clicked() {
                        ui.close_menu();
                    } // wired in slice 6
                    if ui.button("Save").clicked() {
                        ui.close_menu();
                    } // wired in slice 6
                    ui.separator();
                    if ui.button("Quit").clicked() {
                        std::process::exit(0);
                    }
                });
                ui.label("shinra editor");
            });
        });

        let mut tab_viewer = TabViewer {
            engine_texture: self.viewport_texture_id,
        };
        DockArea::new(&mut self.dock)
            .style(Style::from_egui(ctx.style().as_ref()))
            .show(ctx, &mut tab_viewer);
    }
}

fn camera() -> shinra_engine::scene::Camera {
    use shinra_engine::scene::{Camera, Projection};
    Camera {
        eye: glam::Vec3::new(0.0, 10.0, 0.0),
        target: glam::Vec3::ZERO,
        up: glam::Vec3::Z, // top-down: looking down -Y
        projection: Projection::Orthographic {
            half_height: 5.0,
            aspect: RENDER_W as f32 / RENDER_H as f32,
            znear: 0.1,
            zfar: 100.0,
        },
    }
}

struct TabViewer {
    engine_texture: egui::TextureId,
}

impl egui_dock::TabViewer for TabViewer {
    type Tab = PanelKind;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        match tab {
            PanelKind::Viewport => "Viewport",
            PanelKind::SceneTree => "Scene Tree",
            PanelKind::Inspector => "Inspector",
            PanelKind::Palette => "Palette",
        }
        .into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        match tab {
            PanelKind::Viewport => {
                let avail = ui.available_size();
                ui.image((self.engine_texture, avail));
            }
            PanelKind::SceneTree => {
                ui.label("(scene tree — wired in slice 5)");
            }
            PanelKind::Inspector => {
                ui.label("(inspector — wired in slice 5)");
            }
            PanelKind::Palette => {
                ui.label("(palette — wired in slice 4)");
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let opts = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "shinra editor",
        opts,
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}
