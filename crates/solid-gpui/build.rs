use std::{env, path::PathBuf};
#[path = "build/provenance.rs"]
mod provenance;

fn main() {
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_BUILD_LOCKFILE");
    let lockfile = PathBuf::from(env::var_os("SOLID_GPUI_BUILD_LOCKFILE").expect(
        "native SDK provenance requires SOLID_GPUI_BUILD_LOCKFILE pointing to the consuming workspace Cargo.lock; Vite sets it automatically, or configure it in the consuming .cargo/config.toml",
    ));
    assert!(
        lockfile.is_absolute() && lockfile.is_file(),
        "SOLID_GPUI_BUILD_LOCKFILE must identify the resolved consuming workspace Cargo.lock"
    );
    println!("cargo:rerun-if-changed={}", lockfile.display());
    let crate_root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = crate_root.parent().unwrap().parent().unwrap();
    assert!(
        root.join("crates/solid-gpui").is_dir() && root.join("Cargo.lock").is_file(),
        "native SDK build provenance requires the complete pinned SDK source checkout and Cargo.lock"
    );
    let digest = provenance::digest(root, &lockfile);
    println!("cargo:rustc-env=SOLID_GPUI_SDK_SOURCE_DIGEST={digest}");
}
