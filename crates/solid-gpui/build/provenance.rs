use sha2::{Digest, Sha256};
use std::{
    fs,
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
        Some("rs" | "toml" | "lock" | "ts" | "json" | "patch")
    ) {
        output.push(path.to_owned());
    }
}

pub fn digest(root: &Path, lockfile: &Path) -> String {
    let mut inputs = Vec::new();
    for path in [
        "crates/solid-gpui/src",
        "crates/solid-gpui/build.rs",
        "crates/solid-gpui/build",
        "crates/solid-gpui/Cargo.toml",
        "crates/solid-gpui-macros/src",
        "crates/solid-gpui-macros/Cargo.toml",
        "crates/gpui-iconify/src",
        "crates/gpui-iconify/Cargo.toml",
        "crates/solid-gpui-bun-sys",
        "vendor",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        files(&root.join(path), &mut inputs);
    }
    inputs.sort();
    let mut hash = Sha256::new();
    let lock = fs::read_to_string(lockfile).unwrap().replace("\r\n", "\n");
    let name = "consuming-workspace/Cargo.lock";
    hash.update((name.len() as u64).to_le_bytes());
    hash.update(name.as_bytes());
    hash.update((lock.len() as u64).to_le_bytes());
    hash.update(lock.as_bytes());
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
    hash.finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
