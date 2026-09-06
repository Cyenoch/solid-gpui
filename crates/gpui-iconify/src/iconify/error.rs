use std::fmt;

/// Failure while reading or resolving Iconify build input.
#[derive(Debug)]
pub enum CodegenError {
    /// `icons.json` / `info.json` could not be parsed.
    Json(String),
    /// `allowlist.toml` could not be parsed.
    Toml(String),
    /// A vendor file could not be read.
    Io(String),
    /// Allowlist name is not in the vendor JSON.
    MissingIcon { prefix: String, name: String },
    /// Allowlist name resolves to a hidden icon.
    HiddenIcon { prefix: String, name: String },
    /// Alias parent is missing.
    MissingParent { name: String, parent: String },
    /// Alias parent chain loops.
    AliasCycle { name: String },
    /// Allowlist names a preset whose cargo feature is off.
    DisabledPreset { prefix: String },
    /// Allowlist names a preset that is not vendored by this crate.
    UnknownPreset { prefix: String },
    /// Allowlist names no embeddable icons.
    EmptyCatalog,
}

impl fmt::Display for CodegenError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(message) => write!(f, "iconify json: {message}"),
            Self::Toml(message) => write!(f, "allowlist toml: {message}"),
            Self::Io(message) => write!(f, "vendor io: {message}"),
            Self::MissingIcon { prefix, name } => write!(f, "missing icon {prefix}:{name}"),
            Self::HiddenIcon { prefix, name } => {
                write!(f, "hidden icon {prefix}:{name} cannot be embedded")
            }
            Self::MissingParent { name, parent } => {
                write!(f, "alias {name} missing parent {parent}")
            }
            Self::AliasCycle { name } => write!(f, "alias cycle at {name}"),
            Self::DisabledPreset { prefix } => {
                write!(f, "preset {prefix} is listed but its cargo feature is off")
            }
            Self::UnknownPreset { prefix } => {
                write!(f, "preset {prefix} is not vendored")
            }
            Self::EmptyCatalog => write!(f, "no icons were selected for embedding"),
        }
    }
}

impl std::error::Error for CodegenError {}
