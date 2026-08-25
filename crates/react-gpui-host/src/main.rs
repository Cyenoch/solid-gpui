use gpui::{App, AppContext, Bounds, WindowBounds, WindowOptions, px, size};
#[cfg(feature = "embedded-bun")]
use react_gpui::Event;
use react_gpui::{ProcessAdapter, ReactRoot, RuntimeAdapter};
#[cfg(feature = "embedded-bun")]
use react_gpui_bun::EmbeddedBunAdapter;
use std::env;
use std::ffi::OsString;
#[cfg(feature = "embedded-bun")]
use std::path::PathBuf;
use std::process::Command;
use std::sync::Arc;

const COMMAND_ENV: &str = "REACT_GPUI_RENDERER_COMMAND";
const ARGS_ENV: &str = "REACT_GPUI_RENDERER_ARGS";
const DEFAULT_EMBEDDED_ENTRY: &str = "packages/react-gpui/examples/counter.tsx";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RuntimeMode {
    Process,
    Embedded,
}

fn main() {
    if env::args().any(|arg| arg == "--help" || arg == "-h") {
        print_help();
        return;
    }

    let (mode, renderer_args) = match runtime_selection() {
        Ok(selection) => selection,
        Err(error) => {
            eprintln!("react-gpui-host: {error}");
            std::process::exit(2);
        }
    };
    let runtime = match start_runtime(mode, &renderer_args) {
        Ok(runtime) => runtime,
        Err(error) => {
            eprintln!("react-gpui-host: {error}");
            std::process::exit(1);
        }
    };

    let runtime_for_quit = Arc::clone(&runtime);
    let runtime_for_root = Arc::clone(&runtime);
    gpui_platform::application().run(move |cx: &mut App| {
        let mut root_entity = None;
        let bounds = Bounds::centered(None, size(px(800.0), px(600.0)), cx);
        if let Err(error) = cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| {
                let root = cx.new(|_| {
                    ReactRoot::new(Arc::clone(&runtime_for_root) as Arc<dyn RuntimeAdapter>)
                });
                root_entity = Some(root.clone());
                root
            },
        ) {
            eprintln!("react-gpui-host: failed to open GPUI window: {error}");
            let _ = runtime_for_quit.shutdown();
            return;
        }

        let Some(root_entity) = root_entity else {
            eprintln!("react-gpui-host: GPUI did not return a root entity");
            let _ = runtime_for_quit.shutdown();
            return;
        };
        ReactRoot::start_commit_reader(root_entity, runtime_for_root, cx);
        cx.on_app_quit(move |_| {
            let runtime = Arc::clone(&runtime_for_quit);
            async move {
                let _ = runtime.shutdown();
            }
        })
        .detach();
        cx.activate(true);
    });
}

fn runtime_selection() -> Result<(RuntimeMode, Vec<OsString>), String> {
    let mut mode = RuntimeMode::Process;
    let mut args = Vec::new();
    let mut input = env::args_os().skip(1);
    while let Some(arg) = input.next() {
        if arg == "--runtime" {
            let Some(value) = input.next() else {
                return Err("--runtime requires `process` or `embedded`".to_owned());
            };
            mode = match value.to_string_lossy().as_ref() {
                "process" => RuntimeMode::Process,
                "embedded" => RuntimeMode::Embedded,
                value => return Err(format!("unknown runtime `{value}`")),
            };
        } else if arg == "--embedded" {
            mode = RuntimeMode::Embedded;
        } else {
            args.push(arg);
        }
    }
    Ok((mode, args))
}

fn start_runtime(
    mode: RuntimeMode,
    renderer_args: &[OsString],
) -> Result<Arc<dyn RuntimeAdapter>, String> {
    match mode {
        RuntimeMode::Process => ProcessAdapter::spawn(renderer_command(renderer_args))
            .map(|runtime| runtime as Arc<dyn RuntimeAdapter>)
            .map_err(|error| format!("failed to spawn process renderer: {error}")),
        RuntimeMode::Embedded => start_embedded(renderer_args),
    }
}

fn start_embedded(renderer_args: &[OsString]) -> Result<Arc<dyn RuntimeAdapter>, String> {
    #[cfg(feature = "embedded-bun")]
    {
        let entry = renderer_args
            .iter()
            .find(|arg| *arg != "--smoke-press" && *arg != "--watch")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_EMBEDDED_ENTRY));
        let runtime = EmbeddedBunAdapter::start(&entry)
            .map_err(|error| format!("failed to start embedded Bun: {error}"))?;
        if renderer_args.iter().any(|arg| arg == "--watch") {
            runtime
                .watch(&entry)
                .map_err(|error| format!("failed to watch embedded entry: {error}"))?;
        }
        if renderer_args.iter().any(|arg| arg == "--smoke-press") {
            let runtime_for_smoke = Arc::clone(&runtime);
            let watch_enabled = renderer_args.iter().any(|arg| arg == "--watch");
            std::thread::spawn(move || {
                std::thread::sleep(std::time::Duration::from_millis(2000));
                let sent = runtime_for_smoke
                    .send_event(&Event::press(1, 1, 1, 1, 7, 1))
                    .is_ok();
                std::thread::sleep(std::time::Duration::from_millis(2000));
                eprintln!(
                    "react-gpui-host: embedded smoke press sent={sent}, commits={}, status={:?}",
                    runtime_for_smoke.commit_count(),
                    runtime_for_smoke.runtime_status()
                );
                if watch_enabled {
                    std::thread::sleep(std::time::Duration::from_millis(3000));
                    eprintln!(
                        "react-gpui-host: embedded refresh commits={}, queued={}, status={:?}",
                        runtime_for_smoke.commit_count(),
                        runtime_for_smoke.refresh_count(),
                        runtime_for_smoke.runtime_status()
                    );
                }
            });
        }
        return Ok(runtime as Arc<dyn RuntimeAdapter>);
    }
    #[cfg(not(feature = "embedded-bun"))]
    {
        let _ = renderer_args;
        Err("embedded runtime is not compiled; use `--features embedded-bun`".to_owned())
    }
}

fn renderer_command(renderer_args: &[OsString]) -> Command {
    let mut command = renderer_args
        .first()
        .cloned()
        .map(Command::new)
        .unwrap_or_else(|| {
            let executable = env::var_os(COMMAND_ENV).unwrap_or_else(|| "bun".into());
            Command::new(executable)
        });
    if renderer_args.len() > 1 {
        command.args(&renderer_args[1..]);
    } else if renderer_args.is_empty() {
        if let Some(configured_args) = env::var_os(ARGS_ENV) {
            command.args(configured_args.to_string_lossy().split_whitespace());
        }
    }
    command
}

fn print_help() {
    println!(
        "react-gpui-host\n\nUsage:\n  react-gpui-host [--runtime process] [renderer-command [args...]]\n  react-gpui-host --runtime embedded [entry.tsx]\n\nRuntime selection:\n  --runtime process    child-process ProcessAdapter (default)\n  --runtime embedded   in-process Bun/JSC adapter (build with --features embedded-bun)\n  --watch              watch the embedded entry and apply Fast Refresh updates\n  --smoke-press        send one current-wire press for embedded counter smoke\n\nConfiguration:\n  {COMMAND_ENV}  renderer executable (process mode; default: bun)\n  {ARGS_ENV}     whitespace-separated default renderer args\n\nThe embedded default entry is {DEFAULT_EMBEDDED_ENTRY}. Both runtimes use\nlength-prefixed MessagePack Commit Batches and current-wire events."
    );
}
