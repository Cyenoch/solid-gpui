#[path = "../build/provenance.rs"]
mod provenance;

#[test]
fn consumer_resolution_and_embedded_runtime_inputs_lock_native_builds() {
    let directory = tempfile::tempdir().unwrap();
    let sdk = directory.path().join("sdk");
    for path in [
        "crates/solid-gpui/src/lib.rs",
        "crates/solid-gpui/build.rs",
        "crates/solid-gpui/build/provenance.rs",
        "crates/solid-gpui/Cargo.toml",
        "crates/solid-gpui-macros/src/lib.rs",
        "crates/solid-gpui-macros/Cargo.toml",
        "crates/gpui-iconify/src/lib.rs",
        "crates/gpui-iconify/Cargo.toml",
        "crates/solid-gpui-bun-sys/embedded/runtime.rs",
        "crates/solid-gpui-bun-sys/embedded/build/embed-native.ts",
        "crates/solid-gpui-bun-sys/bun-build.json",
        "crates/solid-gpui-bun-sys/bun_embed.patch",
        "Cargo.toml",
        "Cargo.lock",
    ] {
        let path = sdk.join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "pinned source\n").unwrap();
    }
    std::fs::create_dir(sdk.join("vendor")).unwrap();
    let lock = directory.path().join("Cargo.lock");
    std::fs::write(&lock, "dependency resolution A\n").unwrap();
    let original = provenance::digest(&sdk, &lock);
    std::fs::write(&lock, "dependency resolution B\n").unwrap();
    assert_ne!(provenance::digest(&sdk, &lock), original);
    std::fs::write(&lock, "dependency resolution A\r\n").unwrap();
    assert_eq!(provenance::digest(&sdk, &lock), original);
    for path in [
        "crates/solid-gpui-bun-sys/embedded/runtime.rs",
        "crates/solid-gpui-bun-sys/embedded/build/embed-native.ts",
        "crates/solid-gpui-bun-sys/bun-build.json",
        "crates/solid-gpui-bun-sys/bun_embed.patch",
    ] {
        let path = sdk.join(path);
        std::fs::write(&path, "changed runtime input\n").unwrap();
        assert_ne!(provenance::digest(&sdk, &lock), original);
        std::fs::write(&path, "pinned source\r\n").unwrap();
        assert_eq!(provenance::digest(&sdk, &lock), original);
    }
}
