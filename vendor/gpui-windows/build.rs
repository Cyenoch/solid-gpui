#![allow(clippy::disallowed_methods, reason = "build scripts are exempt")]

use shader_compilation::compile_shaders;

fn main() {
    // Shader generation follows the target cargo compiles for, not the host this
    // script runs on: `cfg(target_os = "windows")` in a build script means the
    // host, so Windows cross-compiled from another host would skip the shaders
    // that `directx_renderer.rs` requires.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS")
        .expect("cargo sets CARGO_CFG_TARGET_OS when it runs a build script");
    if target_os != "windows" {
        return;
    }

    // `directx_renderer.rs` selects embedded bytecode with the target's
    // `cfg(not(debug_assertions))`; cargo reports that profile setting to build
    // scripts as CARGO_CFG_DEBUG_ASSERTIONS, whereas this script's own
    // `debug_assertions` follows whichever profile compiled the script.
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_some() {
        return;
    }

    compile_shaders();
}

mod shader_compilation {
    use std::{
        fs,
        io::Write,
        path::{Path, PathBuf},
        process::{self, Command},
    };

    pub fn compile_shaders() {
        println!("cargo:rerun-if-env-changed=GPUI_FXC_PATH");
        println!("cargo:rerun-if-env-changed=PATH");
        let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let shader_path = manifest_dir.join("src/shaders.hlsl");
        let emoji_shader_path = manifest_dir.join("src/color_text_raster.hlsl");
        // Both sources above `#include` this file, so the compiled bytecode
        // depends on its contents as well.
        let include_path = manifest_dir.join("src/alpha_correction.hlsl");
        for path in [&shader_path, &emoji_shader_path, &include_path] {
            println!("cargo:rerun-if-changed={}", path.display());
        }

        let fxc_path = find_fxc_compiler();
        let out_dir = std::env::var("OUT_DIR").unwrap();

        // Define all modules
        let modules = [
            "quad",
            "shadow",
            "path_rasterization",
            "path_sprite",
            "underline",
            "monochrome_sprite",
            "subpixel_sprite",
            "polychrome_sprite",
        ];

        let rust_binding_path = format!("{out_dir}/shaders_bytes.rs");
        if Path::new(&rust_binding_path).exists() {
            fs::remove_file(&rust_binding_path)
                .expect("Failed to remove existing Rust binding file");
        }
        for module in modules {
            compile_shader_for_module(
                module,
                &out_dir,
                &fxc_path,
                shader_path.to_str().unwrap(),
                &rust_binding_path,
            );
        }
        compile_shader_for_module(
            "emoji_rasterization",
            &out_dir,
            &fxc_path,
            emoji_shader_path.to_str().unwrap(),
            &rust_binding_path,
        );
    }

    /// Locate `binary` in the newest installed Windows SDK.
    ///
    /// Reading the SDK's registry keys and executing the returned `fxc.exe` are
    /// Windows-host capabilities, so this item exists only on that host.
    #[cfg(windows)]
    pub fn find_latest_windows_sdk_binary(
        binary: &str,
    ) -> Result<Option<PathBuf>, Box<dyn std::error::Error>> {
        let key = windows_registry::LOCAL_MACHINE
            .open("SOFTWARE\\WOW6432Node\\Microsoft\\Microsoft SDKs\\Windows\\v10.0")?;

        let install_folder: String = key.get_string("InstallationFolder")?; // "C:\Program Files (x86)\Windows Kits\10\"
        let install_folder_bin = Path::new(&install_folder).join("bin");

        let mut versions: Vec<_> = std::fs::read_dir(&install_folder_bin)?
            .flatten()
            .filter(|entry| entry.path().is_dir())
            .filter_map(|entry| entry.file_name().into_string().ok())
            .collect();

        versions.sort_by_key(|s| {
            s.split('.')
                .filter_map(|p| p.parse().ok())
                .collect::<Vec<u32>>()
        });

        let arch = match std::env::consts::ARCH {
            "x86_64" => "x64",
            "aarch64" => "arm64",
            _ => Err(format!(
                "Unsupported architecture: {}",
                std::env::consts::ARCH
            ))?,
        };

        if let Some(highest_version) = versions.last() {
            return Ok(Some(
                install_folder_bin
                    .join(highest_version)
                    .join(arch)
                    .join(binary),
            ));
        }

        Ok(None)
    }

    /// You can set the `GPUI_FXC_PATH` environment variable to specify the path to the fxc.exe compiler.
    fn find_fxc_compiler() -> String {
        if let Some(path) = std::env::var_os("GPUI_FXC_PATH") {
            let path = path
                .into_string()
                .expect("GPUI_FXC_PATH must be valid UTF-8");
            assert!(
                Path::new(&path).is_file(),
                "GPUI_FXC_PATH must name a compiler file: {path}"
            );
            return path;
        }

        // Try to find in PATH
        // NOTE: This has to be `where.exe` on Windows, not `where`, it must be ended with `.exe`
        if let Ok(output) = std::process::Command::new("where.exe")
            .arg("fxc.exe")
            .output()
            && output.status.success()
        {
            let paths = String::from_utf8_lossy(&output.stdout);
            if let Some(path) = paths
                .lines()
                .map(str::trim)
                .find(|path| Path::new(path).is_file())
            {
                return path.to_owned();
            }
        }

        #[cfg(windows)]
        if let Ok(Some(path)) = find_latest_windows_sdk_binary("fxc.exe") {
            return path.to_string_lossy().into_owned();
        }

        let target = std::env::var("TARGET").unwrap_or_else(|_| "unknown".to_string());
        let host = std::env::var("HOST").unwrap_or_else(|_| "unknown".to_string());
        println!(
            "cargo::error=fxc.exe not found for the Windows release shader build (target {target}, host {host}).\n\
             cargo::error=Release builds embed DXBC bytecode compiled at build time; debug builds compile the embedded HLSL at runtime and need no compiler.\n\
             cargo::error=Searched GPUI_FXC_PATH, PATH, and the newest installed Windows SDK.\n\
             cargo::error=Prerequisite: run the release build on a Windows host with the Windows SDK installed, or set GPUI_FXC_PATH to an fxc.exe that this host can execute."
        );
        process::exit(1);
    }

    fn compile_shader_for_module(
        module: &str,
        out_dir: &str,
        fxc_path: &str,
        shader_path: &str,
        rust_binding_path: &str,
    ) {
        // Compile vertex shader
        let output_file = format!("{}/{}_vs.h", out_dir, module);
        let const_name = format!("{}_VERTEX_BYTES", module.to_uppercase());
        compile_shader_impl(
            fxc_path,
            &format!("{module}_vertex"),
            &output_file,
            &const_name,
            shader_path,
            "vs_4_1",
        );
        generate_rust_binding(&const_name, &output_file, rust_binding_path);

        // Compile fragment shader
        let output_file = format!("{}/{}_ps.h", out_dir, module);
        let const_name = format!("{}_FRAGMENT_BYTES", module.to_uppercase());
        compile_shader_impl(
            fxc_path,
            &format!("{module}_fragment"),
            &output_file,
            &const_name,
            shader_path,
            "ps_4_1",
        );
        generate_rust_binding(&const_name, &output_file, rust_binding_path);
    }

    fn compile_shader_impl(
        fxc_path: &str,
        entry_point: &str,
        output_path: &str,
        var_name: &str,
        shader_path: &str,
        target: &str,
    ) {
        let output = Command::new(fxc_path)
            .args([
                "/T",
                target,
                "/E",
                entry_point,
                "/Fh",
                output_path,
                "/Vn",
                var_name,
                "/O3",
                shader_path,
            ])
            .output();

        match output {
            Ok(result) => {
                if result.status.success() {
                    return;
                }
                println!(
                    "cargo::error=Shader compilation failed for {}:\n{}",
                    entry_point,
                    String::from_utf8_lossy(&result.stderr)
                );
                process::exit(1);
            }
            Err(e) => {
                println!("cargo::error=Failed to run fxc for {}: {}", entry_point, e);
                process::exit(1);
            }
        }
    }

    fn generate_rust_binding(const_name: &str, head_file: &str, output_path: &str) {
        let header_content = fs::read_to_string(head_file).expect("Failed to read header file");
        let const_definition = {
            let global_var_start = header_content.find("const BYTE").unwrap();
            let global_var = &header_content[global_var_start..];
            let equal = global_var.find('=').unwrap();
            global_var[equal + 1..].trim()
        };
        let rust_binding = format!(
            "const {}: &[u8] = &{}\n",
            const_name,
            const_definition.replace('{', "[").replace('}', "]")
        );
        let mut options = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(output_path)
            .expect("Failed to open Rust binding file");
        options
            .write_all(rust_binding.as_bytes())
            .expect("Failed to write Rust binding file");
    }
}
