fn main() {
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_GALLERY_BUNDLE");
    if std::env::var_os("CARGO_FEATURE_DISTRIBUTION").is_none() {
        return;
    }
    let source = std::env::var_os("SOLID_GPUI_GALLERY_BUNDLE")
        .expect("distribution requires a bundle; run bun run task gallery-package");
    let source = std::path::PathBuf::from(source);
    assert!(source.is_absolute(), "Gallery bundle path must be absolute");
    let contents = std::fs::read_to_string(&source).expect("read Gallery JavaScript bundle");
    assert!(
        !contents.trim().is_empty(),
        "Gallery bundle must not be empty"
    );
    println!("cargo:rerun-if-changed={}", source.display());
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("gallery.js");
    std::fs::write(output, contents).expect("copy Gallery bundle into build output");
}
