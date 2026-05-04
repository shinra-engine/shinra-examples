use std::sync::Arc;

use eframe::egui;
use egui_dock::{DockArea, DockState, NodeIndex, Style};
use shinra_engine::engine::Engine;
use shinra_engine::mesh::Mesh;

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
    scene: scene::Scene,
    #[allow(dead_code)]
    selected_node: Option<usize>,
    dock: DockState<PanelKind>,
    engine: Engine,
    viewport_texture_id: egui::TextureId,
    tileset: scene::Tileset,
    tileset_path: String,
    brush_tile: Option<u32>,
    quad_mesh: Arc<Mesh>,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let render_state = cc
            .wgpu_render_state
            .as_ref()
            .expect("eframe must be configured with the wgpu backend");
        let engine = Engine::from_existing(
            render_state.device.clone(),
            render_state.queue.clone(),
            RENDER_W,
            RENDER_H,
        );

        let view = engine.color.create_view(&Default::default());
        let mut renderer = render_state.renderer.write();
        let viewport_texture_id = renderer.register_native_texture(
            &engine.device,
            &view,
            wgpu::FilterMode::Linear,
        );

        let mut dock = DockState::new(vec![PanelKind::Viewport]);
        let surface = dock.main_surface_mut();
        let [_viewport, _scene_tree] =
            surface.split_left(NodeIndex::root(), 0.2, vec![PanelKind::SceneTree]);
        let [_viewport, _inspector] =
            surface.split_right(NodeIndex::root(), 0.75, vec![PanelKind::Inspector]);
        let [_viewport, _palette] =
            surface.split_below(NodeIndex::root(), 0.7, vec![PanelKind::Palette]);

        let default_tileset_path = "assets/tilesets/town.tres.ron".to_string();
        let tileset: scene::Tileset = std::fs::read_to_string(&default_tileset_path)
            .ok()
            .and_then(|s| ron::from_str(&s).ok())
            .unwrap_or_default();

        let quad_mesh = Arc::new(
            Mesh::from_obj_file("assets/quad.obj").expect("assets/quad.obj missing"),
        );

        let mut scene = scene::Scene::default();
        ensure_ground(&mut scene);

        Self {
            scene,
            selected_node: None,
            dock,
            engine,
            viewport_texture_id,
            tileset,
            tileset_path: default_tileset_path,
            brush_tile: None,
            quad_mesh,
        }
    }
}

fn ensure_ground(sc: &mut scene::Scene) -> usize {
    if let Some(idx) = sc.nodes.iter().position(|n| n.tilemap.is_some()) {
        return idx;
    }
    sc.nodes.push(scene::Node {
        name: "ground".into(),
        transform: scene::Transform::default(),
        tilemap: Some(scene::Tilemap {
            tileset: "assets/tilesets/town.tres.ron".into(),
            tile_size: [1.0, 1.0],
            cells: vec![],
        }),
        ..Default::default()
    });
    sc.nodes.len() - 1
}

fn mouse_to_cell(
    p: egui::Pos2,
    rect: egui::Rect,
    half_h: f32,
    aspect: f32,
    tile_size: [f32; 2],
) -> (i32, i32) {
    let ndc_x = (p.x - rect.center().x) / (rect.width() * 0.5);
    let ndc_y = (p.y - rect.center().y) / (rect.height() * 0.5);
    let world_x = ndc_x * half_h * aspect;
    let world_z = ndc_y * half_h;
    let cell_x = (world_x / tile_size[0]).round() as i32;
    let cell_y = (world_z / tile_size[1]).round() as i32;
    (cell_x, cell_y)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Build engine scene from editor scene (one quad per painted cell).
        let mut sc = shinra_engine::scene::Scene::new(camera());
        for node in &self.scene.nodes {
            if let Some(tilemap) = &node.tilemap {
                for cell in &tilemap.cells {
                    let model = glam::Mat4::from_translation(glam::Vec3::new(
                        cell.x as f32 * tilemap.tile_size[0],
                        0.0,
                        cell.y as f32 * tilemap.tile_size[1],
                    ));
                    sc.spawn_mesh(Arc::clone(&self.quad_mesh), model);
                }
            }
        }
        self.engine.render(&sc);

        egui::TopBottomPanel::top("menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("New").clicked() {
                        self.scene = scene::Scene::default();
                        ensure_ground(&mut self.scene);
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
            scene: &mut self.scene,
            tileset: &self.tileset,
            tileset_path: &self.tileset_path,
            brush_tile: &mut self.brush_tile,
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
        up: glam::Vec3::Z,
        projection: Projection::Orthographic {
            half_height: 5.0,
            aspect: RENDER_W as f32 / RENDER_H as f32,
            znear: 0.1,
            zfar: 100.0,
        },
    }
}

struct TabViewer<'a> {
    engine_texture: egui::TextureId,
    scene: &'a mut scene::Scene,
    tileset: &'a scene::Tileset,
    tileset_path: &'a str,
    brush_tile: &'a mut Option<u32>,
}

impl<'a> egui_dock::TabViewer for TabViewer<'a> {
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
                let (rect, response) =
                    ui.allocate_exact_size(avail, egui::Sense::click_and_drag());
                ui.painter().image(
                    self.engine_texture,
                    rect,
                    egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0)),
                    egui::Color32::WHITE,
                );

                if let Some(brush) = *self.brush_tile {
                    let painting = (response.is_pointer_button_down_on() && response.dragged())
                        || response.clicked();
                    let erasing = response.secondary_clicked()
                        || (response.is_pointer_button_down_on()
                            && response.dragged_by(egui::PointerButton::Secondary));

                    if painting || erasing {
                        if let Some(p) = response.interact_pointer_pos() {
                            let cell = mouse_to_cell(
                                p,
                                rect,
                                5.0,
                                RENDER_W as f32 / RENDER_H as f32,
                                [1.0, 1.0],
                            );
                            let idx = ensure_ground(self.scene);
                            let tm = self.scene.nodes[idx].tilemap.as_mut().unwrap();
                            if erasing {
                                tm.cells.retain(|c| !(c.x == cell.0 && c.y == cell.1));
                            } else if let Some(c) =
                                tm.cells.iter_mut().find(|c| c.x == cell.0 && c.y == cell.1)
                            {
                                c.tile_id = brush;
                            } else {
                                tm.cells.push(scene::Cell {
                                    x: cell.0,
                                    y: cell.1,
                                    tile_id: brush,
                                });
                            }
                        }
                    }
                }
            }
            PanelKind::SceneTree => {
                ui.label("(scene tree — wired in slice 5)");
            }
            PanelKind::Inspector => {
                ui.label("(inspector — wired in slice 5)");
            }
            PanelKind::Palette => {
                ui.heading("Tileset");
                ui.label(self.tileset_path);
                ui.separator();
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.horizontal_wrapped(|ui| {
                        let mut new_brush: Option<u32> = None;
                        for tile in self.tileset.tiles.iter() {
                            let color = egui::Color32::from_rgb(
                                (tile.color[0] * 255.0) as u8,
                                (tile.color[1] * 255.0) as u8,
                                (tile.color[2] * 255.0) as u8,
                            );
                            let selected = *self.brush_tile == Some(tile.id);
                            let size = egui::vec2(40.0, 40.0);
                            let (rect, resp) =
                                ui.allocate_exact_size(size, egui::Sense::click());
                            ui.painter().rect_filled(rect, 4.0, color);
                            if selected {
                                ui.painter().rect_stroke(
                                    rect,
                                    4.0,
                                    egui::Stroke::new(2.0, egui::Color32::WHITE),
                                    egui::StrokeKind::Inside,
                                );
                            }
                            if resp.on_hover_text(tile.name.as_str()).clicked() {
                                new_brush = Some(tile.id);
                            }
                        }
                        if let Some(id) = new_brush {
                            *self.brush_tile = Some(id);
                        }
                    });
                });
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
