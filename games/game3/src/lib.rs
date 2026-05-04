use gametok_abi::{Drawable, InputFrame};
use scene::{Node, Scene, Tileset};
use std::cell::RefCell;

const SCENE_PATH: &str = "assets/scenes/town.scn.ron";

struct LoadedTile { color: [f32; 3] }
struct LoadedTilemap {
    cells: Vec<scene::Cell>,
    tile_size: [f32; 2],
    tiles_by_id: std::collections::HashMap<u32, LoadedTile>,
}

struct DrawNode {
    transform: [f32; 16],
    mesh_id: u32,
}

thread_local! {
    static MESHES:   RefCell<Vec<String>>        = RefCell::new(Vec::new());
    static NODES:    RefCell<Vec<DrawNode>>       = RefCell::new(Vec::new());
    static TILEMAPS: RefCell<Vec<LoadedTilemap>>  = RefCell::new(Vec::new());
    static FRAME:    RefCell<Vec<Drawable>>       = RefCell::new(Vec::new());
    static INIT:     RefCell<bool>                = const { RefCell::new(false) };
}

fn ensure_init() {
    let already = INIT.with(|i| {
        if *i.borrow() { return true; }
        *i.borrow_mut() = true;
        false
    });
    if already { return; }

    let raw = match std::fs::read_to_string(SCENE_PATH) {
        Ok(s) => s,
        Err(e) => { eprintln!("[game3] scene read failed: {e}"); return; }
    };
    let scene: Scene = match ron::from_str(&raw) {
        Ok(s) => s,
        Err(e) => { eprintln!("[game3] ron parse failed: {e}"); return; }
    };

    fn walk_meshes(node: &Node, paths: &mut Vec<String>) {
        if let Some(mesh) = &node.mesh {
            if !paths.iter().any(|p| p == &mesh.path) {
                paths.push(mesh.path.clone());
            }
        }
        for child in &node.children { walk_meshes(child, paths); }
    }
    let mut paths: Vec<String> = Vec::new();
    for n in &scene.nodes { walk_meshes(n, &mut paths); }

    let quad_index = match paths.iter().position(|p| p == "assets/quad.obj") {
        Some(i) => i,
        None => { paths.push("assets/quad.obj".into()); paths.len() - 1 }
    };

    MESHES.with(|m| *m.borrow_mut() = paths.clone());

    fn bake_transform(t: &scene::Transform) -> [f32; 16] {
        let trans = glam::Vec3::from(t.translation);
        let rot = glam::Quat::from_array(t.rotation);
        let scale = glam::Vec3::from(t.scale);
        glam::Mat4::from_scale_rotation_translation(scale, rot, trans).to_cols_array()
    }

    fn walk(
        node: &Node,
        parent_xform: glam::Mat4,
        paths: &[String],
        quad_index: usize,
        nodes: &mut Vec<DrawNode>,
        tilemaps: &mut Vec<LoadedTilemap>,
    ) {
        let local = glam::Mat4::from_scale_rotation_translation(
            glam::Vec3::from(node.transform.scale),
            glam::Quat::from_array(node.transform.rotation),
            glam::Vec3::from(node.transform.translation),
        );
        let world = parent_xform * local;

        if let Some(mesh) = &node.mesh {
            if let Some(idx) = paths.iter().position(|p| p == &mesh.path) {
                nodes.push(DrawNode {
                    transform: world.to_cols_array(),
                    mesh_id: idx as u32,
                });
            }
        }

        if let Some(tm) = &node.tilemap {
            let tileset_raw = std::fs::read_to_string(&tm.tileset).unwrap_or_default();
            let tileset: Tileset = ron::from_str(&tileset_raw).unwrap_or_default();
            let mut by_id = std::collections::HashMap::new();
            for t in &tileset.tiles {
                by_id.insert(t.id, LoadedTile { color: t.color });
            }
            for c in &tm.cells {
                let cell_xform = world * glam::Mat4::from_translation(glam::Vec3::new(
                    c.x as f32 * tm.tile_size[0],
                    0.0,
                    c.y as f32 * tm.tile_size[1],
                ));
                nodes.push(DrawNode {
                    transform: cell_xform.to_cols_array(),
                    mesh_id: quad_index as u32,
                });
            }
            tilemaps.push(LoadedTilemap {
                cells: tm.cells.clone(),
                tile_size: tm.tile_size,
                tiles_by_id: by_id,
            });
        }

        for child in &node.children {
            walk(child, world, paths, quad_index, nodes, tilemaps);
        }
    }

    let mut nodes: Vec<DrawNode> = Vec::new();
    let mut tilemaps: Vec<LoadedTilemap> = Vec::new();
    for root in &scene.nodes {
        walk(root, glam::Mat4::IDENTITY, &paths, quad_index, &mut nodes, &mut tilemaps);
    }

    // bake_transform is available but walk builds matrices inline; suppress unused warning
    let _ = bake_transform;

    NODES.with(|n| *n.borrow_mut() = nodes);
    TILEMAPS.with(|t| *t.borrow_mut() = tilemaps);
    eprintln!("[game3] loaded scene from {SCENE_PATH}");
}

// ── FFI exports ───────────────────────────────────────────────────────────

#[no_mangle]
pub extern "C" fn meshes_count() -> u32 {
    ensure_init();
    MESHES.with(|m| m.borrow().len() as u32)
}

#[no_mangle]
pub extern "C" fn meshes_path(i: u32, out: *mut u8, cap: u32) -> u32 {
    ensure_init();
    MESHES.with(|m| {
        let m = m.borrow();
        let Some(path) = m.get(i as usize) else { return 0 };
        let bytes = path.as_bytes();
        let n = bytes.len().min(cap as usize);
        if !out.is_null() && n > 0 {
            unsafe { std::ptr::copy_nonoverlapping(bytes.as_ptr(), out, n); }
        }
        n as u32
    })
}

#[no_mangle]
pub extern "C" fn tick(_dt: f32, _input: *const InputFrame) {
    ensure_init();
    FRAME.with(|f| {
        let mut buf = f.borrow_mut();
        buf.clear();
        NODES.with(|n| {
            for dn in n.borrow().iter() {
                buf.push(Drawable {
                    mesh_id: dn.mesh_id,
                    _pad: 0,
                    model: dn.transform,
                });
            }
        });
    });
}

#[no_mangle]
pub extern "C" fn drawables_ptr() -> *const Drawable {
    FRAME.with(|f| f.borrow().as_ptr())
}

#[no_mangle]
pub extern "C" fn drawables_len() -> u32 {
    FRAME.with(|f| f.borrow().len() as u32)
}
