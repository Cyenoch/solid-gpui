fn main() {
    println!("cargo:rerun-if-changed=app.rc");
    println!("cargo:rerun-if-changed=../../../assets/branding/solid-gpui.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        embed_resource::compile("app.rc", embed_resource::NONE)
            .manifest_required()
            .expect("compile the Solid GPUI application icon");
    }
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_WEBSITE_BUNDLE");
    if std::env::var_os("CARGO_FEATURE_DISTRIBUTION").is_none() {
        return;
    }
    let source = std::env::var_os("SOLID_GPUI_WEBSITE_BUNDLE")
        .expect("distribution requires a bundle; run bun run task website-package");
    let source = std::path::PathBuf::from(source);
    assert!(source.is_absolute(), "Website bundle path must be absolute");
    let contents = std::fs::read_to_string(&source).expect("read Website JavaScript bundle");
    assert!(
        !contents.trim().is_empty(),
        "Website bundle must not be empty"
    );
    println!("cargo:rerun-if-changed={}", source.display());
    let output = std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("website.js");
    std::fs::write(output, contents).expect("copy Website bundle into build output");
}
