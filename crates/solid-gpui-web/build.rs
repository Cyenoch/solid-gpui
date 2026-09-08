use std::{env, fs, path::PathBuf};
fn main() {
    let directory = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap())
        .join("../../vendor/gpui-component/crates/assets/assets/icons");
    println!("cargo:rerun-if-changed={}", directory.display());
    let mut paths = fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "svg"))
        .collect::<Vec<_>>();
    paths.sort();
    let mut code = String::from("pub const COMPONENT_ICONS: &[(&str, &[u8])] = &[\n");
    for path in paths {
        code.push_str(&format!(
            "({:?}, include_bytes!({:?})),\n",
            format!("icons/{}", path.file_name().unwrap().to_str().unwrap()),
            path
        ));
    }
    code.push_str("];\n");
    fs::write(
        PathBuf::from(env::var("OUT_DIR").unwrap()).join("icons.rs"),
        code,
    )
    .unwrap();
}
