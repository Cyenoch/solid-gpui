use std::{io, path::PathBuf, process::Command, sync::Arc};

use crate::ProcessAdapter;

/// A Rust-owned Vite runtime. When launched by the Vite plugin, it attaches to
/// that server instead of creating another module graph or rebuilding the host.
pub struct Vite {
    root: PathBuf,
    config_file: Option<PathBuf>,
}

impl Vite {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self {
            root: root.into(),
            config_file: None,
        }
    }

    pub fn config_file(mut self, path: impl Into<PathBuf>) -> Self {
        self.config_file = Some(path.into());
        self
    }

    /// Return an ordinary command so callers can configure Bun's environment.
    pub fn command(&self) -> io::Result<Command> {
        if let Some(command) = runner_command()? {
            return Ok(command);
        }
        let options = serde_json::json!({
            "configFile": self.config_file,
            "nativeHost": std::env::current_exe()?,
        });
        let mut command = Command::new("bun");
        command.current_dir(&self.root)
            .arg("--conditions=browser")
            .arg("--eval")
            .arg(format!(
                "import {{ startDev }} from '@solid-gpui/vite/dev'; const server = await startDev({options}); for (const signal of ['SIGINT', 'SIGTERM']) process.once(signal, () => {{ void server.close().finally(() => process.exit(0)); }});"
            ));
        Ok(command)
    }

    pub fn spawn(&self) -> io::Result<Arc<ProcessAdapter>> {
        ProcessAdapter::spawn(self.command()?)
    }
}

pub(crate) fn runner_command() -> io::Result<Option<Command>> {
    let Some(value) = std::env::var_os("SOLID_GPUI_VITE_RUNNER") else {
        return Ok(None);
    };
    let args: Vec<String> = serde_json::from_str(&value.to_string_lossy())
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidInput, error))?;
    let Some(executable) = args.first().filter(|value| !value.is_empty()) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Empty Vite runner command",
        ));
    };
    let mut command = Command::new(executable);
    command
        .args(&args[1..])
        .env_remove("SOLID_GPUI_VITE_RUNNER");
    Ok(Some(command))
}
