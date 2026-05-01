use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let out_dir  = PathBuf::from(env::var("OUT_DIR").unwrap());
    let hom      = manifest.join("src/main.hom");
    let out_rs   = out_dir.join("main.rs");

    println!("cargo:rerun-if-changed=src/main.hom");
    println!("cargo:rerun-if-changed=src/hom_hecs/mod.rs");

    let workspace = manifest.join("../..");
    let homunc = ["target/release/homunc", "target/debug/homunc", ".tmp/homunc"]
        .iter()
        .map(|p| workspace.join(p))
        .find_map(|p| p.canonicalize().ok())
        .expect("homunc not found — drop binary at .tmp/homunc");

    let status = Command::new(&homunc)
        .args([hom.to_str().unwrap(), "-o", out_rs.to_str().unwrap()])
        .status().expect("homunc spawn failed");
    if !status.success() { panic!("homunc failed for src/main.hom"); }

    let src = fs::read_to_string(&out_rs).unwrap();
    let patched: String = src.lines()
        .map(|l| if l.starts_with("#![") { l.replacen("#![", "#[", 1) } else { l.to_string() })
        .collect::<Vec<_>>().join("\n");
    fs::write(&out_rs, patched).unwrap();
}
