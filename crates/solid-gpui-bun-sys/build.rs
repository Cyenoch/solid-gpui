use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

#[derive(serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct BuildConfig {
    revision: String,
    toolchain: String,
    repository: String,
    ninja_version: String,
}

fn build_config() -> &'static BuildConfig {
    static CONFIG: std::sync::LazyLock<BuildConfig> = std::sync::LazyLock::new(|| {
        serde_json::from_str(include_str!("bun-build.json")).expect("invalid pinned Bun build config")
    });
    &CONFIG
}

fn run(command: &mut Command, label: &str) {
    command
        .env_remove("RUSTC")
        // Outer cargo clippy/check wrappers belong to the host toolchain;
        // Bun's native graph must use its separately pinned nightly compiler.
        .env_remove("RUSTC_WRAPPER")
        .env_remove("RUSTC_WORKSPACE_WRAPPER")
        .env_remove("CLIPPY_ARGS")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_MAKEFLAGS")
        .env("RUSTUP_TOOLCHAIN", &build_config().toolchain)
        .env("CARGO", "cargo");
    let status = command
        .stdin(Stdio::null())
        .status()
        .unwrap_or_else(|error| panic!("{label} could not start: {error}"));
    if !status.success() {
        panic!("{label} failed with {status}");
    }
}

fn output(command: &mut Command, label: &str) -> String {
    let output = command
        .stdin(Stdio::null())
        .output()
        .unwrap_or_else(|error| panic!("{label} could not start: {error}"));
    if !output.status.success() {
        panic!("{label} failed with {}", output.status);
    }
    String::from_utf8(output.stdout)
        .unwrap_or_else(|error| panic!("{label} returned non-UTF-8 output: {error}"))
        .trim()
        .to_owned()
}

fn try_output(command: &mut Command) -> Option<String> {
    let output = command.output().ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn find_ninja(out_dir: &Path) -> PathBuf {
    if let Some(found) = try_output(Command::new("sh").args(["-c", "command -v ninja"]))
        && !found.is_empty()
    {
        return PathBuf::from(found);
    }

    let install_root = out_dir.join("ninja");
    let marker = install_root.join(".installed");
    if !marker.is_file() {
        fs::create_dir_all(&install_root)
            .unwrap_or_else(|error| panic!("creating Ninja cache failed: {error}"));
        run(
            Command::new("python3").args([
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "--no-input",
                "--target",
                install_root.to_str().unwrap_or(""),
                &format!("ninja=={}", build_config().ninja_version),
            ]),
            "installing pinned Ninja",
        );
        fs::write(&marker, &build_config().ninja_version)
            .unwrap_or_else(|error| panic!("recording Ninja install failed: {error}"));
    }
    let path = install_root.join("bin/ninja");
    if !path.is_file() {
        panic!("pinned Ninja was not installed at {}", path.display());
    }
    path
}

fn checkout_bun(out_dir: &Path) -> PathBuf {
    let source = out_dir.join("bun-source");
    if !source.join(".git").is_dir() {
        run(
            Command::new("git").args([
                "clone",
                "--filter=blob:none",
                "--no-checkout",
                &build_config().repository,
                source.to_str().unwrap_or(""),
            ]),
            "cloning pinned Bun source",
        );
    }
    if try_output(Command::new("git").args([
        "-C",
        source.to_str().unwrap_or(""),
        "cat-file",
        "-t",
        &build_config().revision,
    ]))
    .as_deref()
        != Some("commit")
    {
        run(
            Command::new("git").args([
                "-C",
                source.to_str().unwrap_or(""),
                "fetch",
                "--depth=1",
                "origin",
                &build_config().revision,
            ]),
            "fetching pinned Bun revision",
        );
    }
    run(
        Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "checkout",
            "--detach",
            &build_config().revision,
        ]),
        "checking out pinned Bun revision",
    );
    run(
        Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "reset",
            "--hard",
            &build_config().revision,
        ]),
        "resetting pinned Bun source",
    );
    run(
        Command::new("git").args(["-C", source.to_str().unwrap_or(""), "clean", "-fd"]),
        "cleaning generated Bun source",
    );
    let actual = output(
        Command::new("git").args(["-C", source.to_str().unwrap_or(""), "rev-parse", "HEAD"]),
        "verifying Bun revision",
    );
    assert_eq!(actual, build_config().revision, "Bun source revision drifted");
    source
}

fn build_embedded_library(out_dir: &Path) -> PathBuf {
    if env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("macos") {
        panic!("direct embedded Bun builds currently support macOS only; use the static application packager for other targets");
    }

    let source = checkout_bun(out_dir);
    let patch = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("bun_embed.patch");
    run(
        Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "apply",
            "--check",
            patch.to_str().unwrap_or(""),
        ]),
        "checking Bun embedding patch",
    );
    run(
        Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "apply",
            patch.to_str().unwrap_or(""),
        ]),
        "applying Bun embedding patch",
    );
    let overlays = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("embedded");
    fs::create_dir_all(source.join("src/runtime/embedded"))
        .expect("creating embedded module directory");
    fs::copy(
        overlays.join("runtime.rs"),
        source.join("src/runtime/embedded.rs"),
    )
    .expect("copying embedded runtime");
    fs::copy(
        overlays.join("lifecycle.rs"),
        source.join("src/runtime/embedded/lifecycle.rs"),
    )
    .expect("copying embedded lifecycle");
    fs::copy(
        overlays.join("build/embed-native.ts"),
        source.join("scripts/build/embed-native.ts"),
    )
    .expect("copying embedded native build product");

    let build_dir = out_dir.join("bun-build");
    let bun = output(
        Command::new("sh").args(["-c", "command -v bun"]),
        "locating Bun build driver",
    );
    if bun.is_empty() {
        panic!("Bun 1.4 is required in PATH to build the pinned embedding library");
    }
    run(
        Command::new(&bun).current_dir(&source).args([
            "scripts/build.ts",
            "--profile=debug-no-asan",
            "--configure-only",
            "--webkit=prebuilt",
            "--build-dir",
            build_dir.to_str().unwrap_or(""),
        ]),
        "configuring pinned Bun native graph",
    );

    let ninja = find_ninja(out_dir);
    run(
        Command::new(&ninja).args([
            "-d",
            "keeprsp",
            "-C",
            build_dir.to_str().unwrap_or(""),
            "bun-debug",
        ]),
        "building pinned Bun native graph",
    );

    let target = env::var("TARGET").expect("Cargo did not provide TARGET");
    let embed_target = build_dir.join("embed-target");
    let codegen = build_dir.join("codegen");
    let mut cargo_command = Command::new("rustup");
    cargo_command.args(["run", &build_config().toolchain, "cargo"]);
    cargo_command
        .current_dir(&source)
        .args([
            "build",
            "-p",
            "bun_bin",
            "--features",
            "solid-gpui-embed",
            "--lib",
            "--target-dir",
            embed_target.to_str().unwrap_or(""),
            "--target",
            &target,
            "--profile",
            "dev",
            "--locked",
        ])
        .env("BUN_CODEGEN_DIR", &codegen);
    run(&mut cargo_command, "building Bun embedding staticlib");

    let rsp = build_dir.join("bun-debug.rsp");
    let rsp_text = fs::read_to_string(&rsp)
        .unwrap_or_else(|error| panic!("reading Bun link response file failed: {error}"));
    let old_staticlib = format!("rust-target/{target}/debug/libbun_rust.a");
    let new_staticlib = format!("embed-target/{target}/debug/libbun_rust.a");
    let rsp_text = rsp_text.replace(&old_staticlib, &new_staticlib);
    let embed_rsp = build_dir.join("bun-embed.rsp");
    fs::write(&embed_rsp, rsp_text)
        .unwrap_or_else(|error| panic!("writing Bun embedding response file failed: {error}"));

    let sdk = output(
        Command::new("xcrun").args(["--show-sdk-path"]),
        "locating macOS SDK",
    );
    let cxx = output(
        Command::new("sh").args(["-c", "command -v clang++"]),
        "locating C++ linker",
    );
    let library = build_dir.join("libbun_embed.dylib");
    run(
        Command::new(cxx).current_dir(&build_dir).args([
            "-dynamiclib",
            &format!("@{}", embed_rsp.display()),
            "-fsanitize=null",
            "-Wl,-no_compact_unwind",
            "-fno-keep-static-consts",
            "-Wl,-ld_new",
            "-mmacosx-version-min=26",
            "-isysroot",
            &sdk,
            "-Wl,-w",
            "-Wl,-exported_symbol,_bun_embedded_create",
            "-Wl,-exported_symbol,_bun_embedded_run",
            "-Wl,-exported_symbol,_bun_embedded_wake",
            "-Wl,-exported_symbol,_bun_embedded_terminate",
            "-Wl,-exported_symbol,_bun_embedded_destroy",
            "-licucore",
            "-lresolv",
            "-o",
            library.to_str().unwrap_or(""),
        ]),
        "linking libbun_embed.dylib",
    );
    library
}

fn main() {
    println!("cargo:rerun-if-changed=bun_embed.patch");
    println!("cargo:rerun-if-changed=bun-build.json");
    println!("cargo:rerun-if-changed=embedded");
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_BUN_CACHE");
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_BUN_CHECK_ONLY");
    println!("cargo:rerun-if-env-changed=SOLID_GPUI_BUN_LINK_MANIFEST");
    if env::var_os("CARGO_FEATURE_EMBEDDED_BUN").is_none() {
        return;
    }
    // Cargo check and Clippy need the Rust ABI, but never link the native VM.
    // No substitute symbols are provided: executables still require a real build.
    match env::var("SOLID_GPUI_BUN_CHECK_ONLY").as_deref() {
        Ok("1") => return,
        Err(env::VarError::NotPresent) => {}
        _ => panic!("SOLID_GPUI_BUN_CHECK_ONLY must be unset or 1"),
    }
    if let Some(path) = env::var_os("SOLID_GPUI_BUN_LINK_MANIFEST") {
        let path = PathBuf::from(path);
        let manifest: serde_json::Value = serde_json::from_slice(
            &fs::read(&path).expect("reading static Bun link manifest"),
        )
        .expect("parsing static Bun link manifest");
        assert_eq!(manifest["schemaVersion"].as_u64(), Some(1));
        assert_eq!(
            manifest["target"].as_str(),
            Some(env::var("TARGET").expect("Cargo target").as_str()),
            "static Bun link manifest target does not match Cargo"
        );
        println!("cargo:rerun-if-changed={}", path.display());
        // The application root links the native manifest and Bun's rlibs in
        // one rustc invocation. No separate Rust staticlib or dylib is linked.
        return;
    }
    let out_dir = PathBuf::from(
        env::var_os("SOLID_GPUI_BUN_CACHE")
            .or_else(|| env::var_os("OUT_DIR"))
            .expect("Cargo did not provide OUT_DIR"),
    );
    let library = build_embedded_library(&out_dir);
    println!(
        "cargo:rustc-link-search=native={}",
        library.parent().unwrap().display()
    );
    println!("cargo:rustc-link-lib=dylib=bun_embed");
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{}",
        library.parent().unwrap().display()
    );
}
