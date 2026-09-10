use std::{env, fs, path::PathBuf};
fn main() {
    let assets = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../vendor/gpui-kit/crates/assets");
    let manifest = assets.join("default-icons.txt");
    println!("cargo:rerun-if-changed={}", manifest.display());
    let source = fs::read_to_string(manifest).expect("Kit's default control icon manifest");
    let mut paths: Vec<_> = source
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect();
    paths.sort_unstable();
    paths.dedup();
    let mut code = String::from("pub const COMPONENT_ICONS: &[(&str, &[u8])] = &[\n");
    for key in paths {
        assert!(
            key.starts_with("icons/") && key.ends_with(".svg") && !key.contains(".."),
            "invalid Kit asset path"
        );
        let path = assets.join("assets").join(key);
        assert!(path.is_file(), "missing Kit default icon: {key}");
        println!("cargo:rerun-if-changed={}", path.display());
        code.push_str(&format!("({key:?}, include_bytes!({path:?})),\n"));
    }
    code.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("icons.rs"),
        code,
    )
    .unwrap();
}
