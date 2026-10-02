use sha2::{Digest, Sha256};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

fn files(path: &Path, output: &mut Vec<PathBuf>) {
    if path.is_dir() {
        println!("cargo:rerun-if-changed={}", path.display());
        for entry in fs::read_dir(path).unwrap() {
            let path = entry.unwrap().path();
            if !matches!(
                path.file_name().and_then(|name| name.to_str()),
                Some("target" | ".git")
            ) {
                files(&path, output);
            }
        }
    } else if matches!(
        path.extension().and_then(|ext| ext.to_str()),
        Some("rs" | "toml" | "lock")
    ) {
        output.push(path.to_owned());
    }
}

fn main() {
    let crate_root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = crate_root.parent().unwrap().parent().unwrap();
    assert!(
        root.join("crates/solid-gpui").is_dir() && root.join("Cargo.lock").is_file(),
        "native SDK build provenance requires the complete pinned SDK source checkout and Cargo.lock"
    );
    let mut inputs = Vec::new();
    for path in [
        "crates/solid-gpui/src",
        "crates/solid-gpui/build.rs",
        "crates/solid-gpui/Cargo.toml",
        "crates/solid-gpui-macros/src",
        "crates/solid-gpui-macros/Cargo.toml",
        "crates/gpui-iconify/src",
        "crates/gpui-iconify/Cargo.toml",
        "vendor",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        files(&root.join(path), &mut inputs);
    }
    inputs.sort();
    let mut hash = Sha256::new();
    for path in inputs {
        println!("cargo:rerun-if-changed={}", path.display());
        let name = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&path).unwrap().replace("\r\n", "\n");
        hash.update((name.len() as u64).to_le_bytes());
        hash.update(name.as_bytes());
        hash.update((source.len() as u64).to_le_bytes());
        hash.update(source.as_bytes());
    }
    let digest: String = hash
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    println!("cargo:rustc-env=SOLID_GPUI_SDK_SOURCE_DIGEST={digest}");
}
