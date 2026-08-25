use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

const BUN_REVISION: &str = "34cbb9a40b4bd1bd767d134a7065e66c2432a676";
const BUN_TOOLCHAIN: &str = "nightly-2026-07-20";
const BUN_REPOSITORY: &str = "https://github.com/oven-sh/bun.git";
const NINJA_VERSION: &str = "1.13.0";

fn run(command: &mut Command, label: &str) {
    command
        .env_remove("RUSTC")
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env_remove("CARGO_MAKEFLAGS")
        .env("RUSTUP_TOOLCHAIN", BUN_TOOLCHAIN)
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
    if let Some(found) = try_output(&mut Command::new("sh").args(["-c", "command -v ninja"])) {
        if !found.is_empty() {
            return PathBuf::from(found);
        }
    }

    let install_root = out_dir.join("ninja");
    let marker = install_root.join(".installed");
    if !marker.is_file() {
        fs::create_dir_all(&install_root)
            .unwrap_or_else(|error| panic!("creating Ninja cache failed: {error}"));
        run(
            &mut Command::new("python3").args([
                "-m",
                "pip",
                "install",
                "--disable-pip-version-check",
                "--no-input",
                "--target",
                install_root.to_str().unwrap_or(""),
                &format!("ninja=={NINJA_VERSION}"),
            ]),
            "installing pinned Ninja",
        );
        fs::write(&marker, NINJA_VERSION)
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
            &mut Command::new("git").args([
                "clone",
                "--filter=blob:none",
                "--no-checkout",
                BUN_REPOSITORY,
                source.to_str().unwrap_or(""),
            ]),
            "cloning pinned Bun source",
        );
    }
    run(
        &mut Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "fetch",
            "--depth=1",
            "origin",
            BUN_REVISION,
        ]),
        "fetching pinned Bun revision",
    );
    run(
        &mut Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "checkout",
            "--detach",
            BUN_REVISION,
        ]),
        "checking out pinned Bun revision",
    );
    run(
        &mut Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "reset",
            "--hard",
            BUN_REVISION,
        ]),
        "resetting pinned Bun source",
    );
    run(
        &mut Command::new("git").args(["-C", source.to_str().unwrap_or(""), "clean", "-fd"]),
        "cleaning generated Bun source",
    );
    let actual = output(
        &mut Command::new("git").args(["-C", source.to_str().unwrap_or(""), "rev-parse", "HEAD"]),
        "verifying Bun revision",
    );
    if actual != BUN_REVISION {
        panic!("Bun source revision drifted: expected {BUN_REVISION}, got {actual}");
    }
    source
}

fn build_embedded_library(out_dir: &Path) -> PathBuf {
    if env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("macos") {
        panic!("embedded Bun library build currently requires macOS JavaScriptCore");
    }

    let source = checkout_bun(out_dir);
    let patch = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("bun_embed.patch");
    run(
        &mut Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "apply",
            "--check",
            patch.to_str().unwrap_or(""),
        ]),
        "checking Bun embedding patch",
    );
    run(
        &mut Command::new("git").args([
            "-C",
            source.to_str().unwrap_or(""),
            "apply",
            patch.to_str().unwrap_or(""),
        ]),
        "applying Bun embedding patch",
    );

    let build_dir = out_dir.join("bun-build");
    let bun = output(
        &mut Command::new("sh").args(["-c", "command -v bun"]),
        "locating Bun build driver",
    );
    if bun.is_empty() {
        panic!("Bun 1.4 is required in PATH to build the pinned embedding library");
    }
    run(
        &mut Command::new(&bun).current_dir(&source).args([
            "scripts/build.ts",
            "--profile=debug-no-asan",
            "--configure-only",
            "--build-dir",
            build_dir.to_str().unwrap_or(""),
        ]),
        "configuring pinned Bun native graph",
    );

    let ninja = find_ninja(out_dir);
    run(
        &mut Command::new(&ninja).args([
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
    cargo_command.args(["run", BUN_TOOLCHAIN, "cargo"]);
    cargo_command
        .current_dir(&source)
        .args([
            "build",
            "-p",
            "bun_bin",
            "--features",
            "react-gpui-embed",
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
        &mut Command::new("xcrun").args(["--show-sdk-path"]),
        "locating macOS SDK",
    );
    let cxx = output(
        &mut Command::new("sh").args(["-c", "command -v clang++"]),
        "locating C++ linker",
    );
    let library = build_dir.join("libbun_embed.dylib");
    run(
        &mut Command::new(cxx).current_dir(&build_dir).args([
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
            "-Wl,-exported_symbol,_bun_embedded_eval",
            "-Wl,-exported_symbol,_bun_embedded_eval_with_io",
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
    if env::var_os("CARGO_FEATURE_EMBEDDED_BUN").is_none() {
        return;
    }
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("Cargo did not provide OUT_DIR"));
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
