use super::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File, OpenOptions},
    io::{Read, Write},
    path::{Component, Path},
};

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn checkpoint(point: &str) -> Result<(), String> {
    #[cfg(all(test, target_os = "macos"))]
    return super::test_io::checkpoint(point);
    #[cfg(not(all(test, target_os = "macos")))]
    {
        let _ = point;
        Ok(())
    }
}

pub(super) fn relative_path(text: &str) -> Result<PathBuf, String> {
    if text.is_empty()
        || text.len() > 1024
        || text.contains(['\\', ':', '\0'])
        || text
            .split('/')
            .any(|part| part.is_empty() || part == "." || part == "..")
    {
        return Err("unsafe update archive path".into());
    }
    let path = PathBuf::from(text);
    if path.components().count() > 32
        || path
            .components()
            .any(|p| !matches!(p, Component::Normal(_)))
    {
        return Err("unsafe update archive path".into());
    }
    Ok(path)
}

fn no_links(path: &Path) -> Result<(), String> {
    for ancestor in path.ancestors() {
        let metadata = fs::symlink_metadata(ancestor).map_err(err)?;
        if metadata.file_type().is_symlink() {
            return Err("update paths must not contain symlinks".into());
        }
    }
    Ok(())
}

fn transaction(config: &UpdaterConfig) -> PathBuf {
    config.install_path.with_file_name(format!(
        ".{}.solid-update",
        config.install_path.file_name().unwrap().to_string_lossy()
    ))
}

pub(super) struct InstallationLock {
    pub(super) file: File,
}

impl Drop for InstallationLock {
    fn drop(&mut self) {
        #[cfg(unix)]
        {
            use std::os::fd::AsRawFd;
            // A forked child can retain the shared descriptor until exec.
            // Release ownership before closing our handle.
            unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
        }
    }
}

pub(super) fn lock(config: &UpdaterConfig) -> Result<InstallationLock, String> {
    if !config.install_path.is_absolute()
        || config.install_path.extension().is_none_or(|s| s != "app")
        || !config.install_path.is_dir()
    {
        return Err("install path must be an existing absolute .app directory".into());
    }
    no_links(&config.install_path)?;
    if config.install_path.canonicalize().map_err(err)? != config.install_path {
        return Err("install path must be canonical".into());
    }
    let root = transaction(config);
    match fs::create_dir(&root) {
        Ok(()) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&root, fs::Permissions::from_mode(0o700)).map_err(err)?;
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => no_links(&root)?,
        Err(e) => return Err(err(e)),
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let metadata = fs::metadata(&root).map_err(err)?;
        if !metadata.is_dir()
            || metadata.uid() != unsafe { libc::geteuid() }
            || metadata.permissions().mode() & 0o077 != 0
        {
            return Err(
                "update transaction directory must be private and owned by the current user".into(),
            );
        }
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW);
    }
    let lock = options.open(root.join("lock")).map_err(err)?;
    #[cfg(unix)]
    {
        use std::os::fd::AsRawFd;
        if unsafe { libc::flock(lock.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } != 0 {
            return Err("another signed updater owns this installation".into());
        }
    }
    Ok(InstallationLock { file: lock })
}

#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
struct Identity {
    dev: u64,
    ino: u64,
}
fn identity(path: &Path) -> Result<Identity, String> {
    no_links(path)?;
    let metadata = fs::metadata(path).map_err(err)?;
    if !metadata.is_dir() {
        return Err("update bundle must be a directory".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        Ok(Identity {
            dev: metadata.dev(),
            ino: metadata.ino(),
        })
    }
    #[cfg(not(unix))]
    {
        Err("bundle installation is unsupported on this platform".into())
    }
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum RetainedBundle {
    Old,
    New,
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
enum Phase {
    Staged,
    Restoring,
    Cleaning { keep: RetainedBundle },
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Record {
    old: Identity,
    new: Identity,
    signed: SignedFeed,
    phase: Phase,
}

fn read_record(config: &UpdaterConfig) -> Result<Option<Record>, String> {
    let path = transaction(config).join("record.json");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(err(e)),
        Ok(metadata)
            if !metadata.is_file()
                || metadata.file_type().is_symlink()
                || metadata.len() > 32768 =>
        {
            return Err("unsafe update transaction record".into());
        }
        Ok(_) => {}
    }
    let mut bytes = Vec::new();
    File::open(path)
        .map_err(err)?
        .take(32769)
        .read_to_end(&mut bytes)
        .map_err(err)?;
    let record: Record = serde_json::from_slice(&bytes).map_err(err)?;
    record.signed.verify(config)?;
    Ok(Some(record))
}

fn sync_dir(path: &Path) -> Result<(), String> {
    File::open(path).map_err(err)?.sync_all().map_err(err)
}

fn remove_candidate(config: &UpdaterConfig) -> Result<(), String> {
    let path = transaction(config).join("candidate");
    match fs::symlink_metadata(&path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(err(e)),
        Ok(metadata) if metadata.is_dir() && !metadata.file_type().is_symlink() => {
            #[cfg(all(test, target_os = "macos"))]
            if let Err(error) = checkpoint("partial-candidate-remove") {
                // Reproduce a recursive removal that deleted files before its
                // final directory removal failed. Directory identity survives.
                let executable = path.join(&config.executable);
                if executable.is_file() {
                    fs::remove_file(executable).map_err(err)?;
                }
                return Err(error);
            }
            fs::remove_dir_all(path).map_err(err)
        }
        Ok(_) => Err("unsafe update candidate path".into()),
    }
}

fn write_record(config: &UpdaterConfig, record: &Record) -> Result<(), String> {
    let root = transaction(config);
    let mut file = tempfile::Builder::new()
        .prefix("record-")
        .tempfile_in(&root)
        .map_err(err)?;
    file.write_all(&serde_json::to_vec(record).map_err(err)?)
        .map_err(err)?;
    file.as_file().sync_all().map_err(err)?;
    file.persist(root.join("record.json")).map_err(err)?;
    checkpoint("record-write-sync")?;
    if matches!(record.phase, Phase::Cleaning { .. }) {
        checkpoint("cleanup-phase-sync")?;
    }
    sync_dir(&root)
}

fn candidate_identity(config: &UpdaterConfig) -> Result<Option<Identity>, String> {
    let candidate = transaction(config).join("candidate");
    match fs::symlink_metadata(&candidate) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(err(e)),
        Ok(_) => identity(&candidate).map(Some),
    }
}

fn validate_cleanup(
    config: &UpdaterConfig,
    record: &Record,
    keep: RetainedBundle,
) -> Result<(), String> {
    let (retained, discarded) = match keep {
        RetainedBundle::Old => (&record.old, &record.new),
        RetainedBundle::New => (&record.new, &record.old),
    };
    if &identity(&config.install_path)? != retained
        || candidate_identity(config)?.is_some_and(|candidate| &candidate != discarded)
    {
        return Err("cleanup found unexpected bundle identities; transaction retained".into());
    }
    Ok(())
}

fn finish_cleanup(
    config: &UpdaterConfig,
    record: &Record,
    keep: RetainedBundle,
) -> Result<(), String> {
    validate_cleanup(config, record, keep)?;
    // Repeat both directory barriers before deleting the discarded bundle. A
    // restart may have observed Cleaning before its prior barrier succeeded.
    sync_dir(config.install_path.parent().unwrap())?;
    sync_dir(&transaction(config))?;
    checkpoint("remove-candidate")?;
    remove_candidate(config)?;
    checkpoint("cleanup-sync")?;
    sync_dir(&transaction(config))?;
    // The installed bundle and candidate deletion are now durable. A crash or
    // sync failure after unlink may leave either no record or Cleaning with no
    // candidate; both represent the same completed filesystem transaction.
    checkpoint("remove-record")?;
    fs::remove_file(transaction(config).join("record.json")).map_err(err)?;
    checkpoint("record-remove-sync")?;
    sync_dir(&transaction(config))
}

fn begin_cleanup(
    config: &UpdaterConfig,
    mut record: Record,
    keep: RetainedBundle,
) -> Result<(), String> {
    validate_cleanup(config, &record, keep)?;
    record.phase = Phase::Cleaning { keep };
    write_record(config, &record)?;
    finish_cleanup(config, &record, keep)
}

fn restore(config: &UpdaterConfig, mut record: Record) -> Result<(), String> {
    let current = identity(&config.install_path)?;
    let candidate =
        candidate_identity(config)?.ok_or("rollback candidate is missing; transaction retained")?;
    if current == record.new && candidate == record.old {
        record.phase = Phase::Restoring;
        write_record(config, &record)?;
        checkpoint("restore-exchange")?;
        exchange(&config.install_path, &transaction(config).join("candidate"))?;
    } else if current != record.old || candidate != record.new {
        return Err("rollback found unexpected bundle identities; transaction retained".into());
    }
    checkpoint("restore-sync")?;
    sync_dir(config.install_path.parent().unwrap())?;
    sync_dir(&transaction(config))?;
    begin_cleanup(config, record, RetainedBundle::Old)
}

pub(super) fn recover(config: &UpdaterConfig) -> Result<(), String> {
    cleanup_staging(config)?;
    let Some(record) = read_record(config)? else {
        return remove_candidate(config);
    };
    match record.phase {
        Phase::Cleaning { keep } => return finish_cleanup(config, &record, keep),
        Phase::Restoring => return restore(config, record),
        Phase::Staged => {}
    }
    let current = identity(&config.install_path)?;
    let candidate =
        candidate_identity(config)?.ok_or("staged candidate is missing; transaction retained")?;
    if current == record.new && candidate == record.old {
        return Ok(());
    }
    if current == record.old && candidate == record.new {
        return begin_cleanup(config, record, RetainedBundle::Old);
    }
    Err("update recovery found unexpected bundle identities; preserve transaction for manual recovery".into())
}

fn cleanup_staging(config: &UpdaterConfig) -> Result<(), String> {
    for entry in fs::read_dir(transaction(config)).map_err(err)? {
        let entry = entry.map_err(err)?;
        let name = entry.file_name();
        let name = name.to_str().ok_or("invalid update transaction entry")?;
        if name.starts_with("stage-") && entry.file_type().map_err(err)?.is_dir() {
            fs::remove_dir_all(entry.path()).map_err(err)?;
        } else if name.starts_with("record-") && entry.file_type().map_err(err)?.is_file() {
            fs::remove_file(entry.path()).map_err(err)?;
        }
    }
    Ok(())
}

pub(super) fn pending(config: &UpdaterConfig) -> Result<bool, String> {
    if let Some(record) = read_record(config)? {
        if let Phase::Cleaning { keep } = record.phase {
            validate_cleanup(config, &record, keep)?;
            return Ok(false);
        }
        let current = identity(&config.install_path)?;
        let candidate = candidate_identity(config)?
            .ok_or("update candidate is missing; transaction retained")?;
        if current == record.new && candidate == record.old {
            return Ok(true);
        }
        if current == record.old && candidate == record.new {
            return Ok(false);
        }
        Err("pending update bundle identities changed; transaction retained".into())
    } else {
        Ok(false)
    }
}

fn extract(
    config: &UpdaterConfig,
    manifest: &Manifest,
    bytes: &[u8],
    destination: &Path,
    context: &NativeCallContext,
) -> Result<PathBuf, String> {
    let mut archive = tar::Archive::new(bytes);
    let mut names = BTreeSet::new();
    let mut total = 0u64;
    for entry in archive.entries().map_err(err)?.raw(true) {
        context.check_cancelled()?;
        let mut entry = entry.map_err(err)?;
        if entry.header().as_ustar().is_none() {
            return Err("update archive requires USTAR headers".into());
        }
        let kind = entry.header().entry_type();
        if kind != tar::EntryType::Regular && kind != tar::EntryType::Directory {
            return Err("archive links, special files and extended headers are forbidden".into());
        }
        let raw_name = entry.path_bytes();
        let raw_name = std::str::from_utf8(&raw_name).map_err(err)?;
        let name = if kind == tar::EntryType::Directory {
            raw_name.strip_suffix('/').unwrap_or(raw_name)
        } else {
            raw_name
        };
        let relative = relative_path(name)?;
        if relative.components().next().unwrap().as_os_str()
            != std::ffi::OsStr::new(&manifest.bundle)
            || !names.insert(relative.clone())
        {
            return Err("archive must contain one exact bundle with no duplicate paths".into());
        }
        if names.len() > config.max_entries as usize {
            return Err("archive entry count exceeds limit".into());
        }
        let size = entry.header().size().map_err(err)?;
        total = total.checked_add(size).ok_or("archive size overflow")?;
        if total > u64::from(config.max_unpacked_bytes) {
            return Err("archive unpacked bytes exceed limit".into());
        }
        let path = destination.join(relative);
        if kind == tar::EntryType::Directory {
            if size != 0 {
                return Err("archive directories must have zero size".into());
            }
            fs::create_dir_all(path).map_err(err)?;
        } else {
            fs::create_dir_all(path.parent().unwrap()).map_err(err)?;
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&path)
                .map_err(err)?;
            let mut copied = 0u64;
            let mut buffer = [0; 65536];
            loop {
                context.check_cancelled()?;
                let n = entry.read(&mut buffer).map_err(err)?;
                if n == 0 {
                    break;
                }
                file.write_all(&buffer[..n]).map_err(err)?;
                copied += n as u64;
            }
            if copied != size {
                return Err("truncated archive entry".into());
            }
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                // Never preserve setuid, setgid, sticky bits, ownership or xattrs.
                let executable = entry.header().mode().map_err(err)? & 0o111 != 0;
                fs::set_permissions(
                    &path,
                    fs::Permissions::from_mode(if executable { 0o755 } else { 0o644 }),
                )
                .map_err(err)?;
            }
            file.sync_all().map_err(err)?;
        }
    }
    if archive.into_inner().iter().any(|byte| *byte != 0) {
        return Err("update archive contains nonzero trailing data".into());
    }
    let bundle = destination.join(&manifest.bundle);
    let executable = bundle.join(&manifest.executable);
    let metadata = fs::symlink_metadata(executable).map_err(err)?;
    if !metadata.is_file() {
        return Err("archive is missing the configured executable".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err("configured executable lacks execute permission".into());
        }
    }
    sync_tree(&bundle, context)?;
    Ok(bundle)
}

fn sync_tree(path: &Path, context: &NativeCallContext) -> Result<(), String> {
    context.check_cancelled()?;
    for entry in fs::read_dir(path).map_err(err)? {
        let entry = entry.map_err(err)?;
        if entry.file_type().map_err(err)?.is_dir() {
            sync_tree(&entry.path(), context)?;
        }
    }
    sync_dir(path)
}

#[cfg(target_os = "macos")]
fn exchange(a: &Path, b: &Path) -> Result<(), String> {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let a = CString::new(a.as_os_str().as_bytes()).map_err(err)?;
    let b = CString::new(b.as_os_str().as_bytes()).map_err(err)?;
    // RENAME_SWAP is one filesystem operation: the install path is never absent.
    if unsafe {
        libc::renameatx_np(
            libc::AT_FDCWD,
            a.as_ptr(),
            libc::AT_FDCWD,
            b.as_ptr(),
            libc::RENAME_SWAP,
        )
    } != 0
    {
        return Err(format!(
            "atomic bundle exchange failed: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(())
}
#[cfg(not(target_os = "macos"))]
fn exchange(_: &Path, _: &Path) -> Result<(), String> {
    Err("atomic app bundle installation is unsupported on this platform".into())
}

pub(super) fn install(
    config: &UpdaterConfig,
    manifest: &Manifest,
    signed: &SignedFeed,
    bytes: &[u8],
    context: &NativeCallContext,
) -> Result<(), String> {
    let root = transaction(config);
    recover(config)?;
    let stage = tempfile::Builder::new()
        .prefix("stage-")
        .tempdir_in(&root)
        .map_err(err)?;
    let bundle = extract(config, manifest, bytes, stage.path(), context)?;
    context.check_cancelled()?;
    let candidate = root.join("candidate");
    let record = Record {
        old: identity(&config.install_path)?,
        new: identity(&bundle)?,
        signed: signed.clone(),
        phase: Phase::Staged,
    };
    fs::rename(bundle, &candidate).map_err(err)?;
    write_record(config, &record)?;
    if let Err(error) = context
        .check_cancelled()
        .and_then(|_| checkpoint("before-exchange"))
        .and_then(|_| exchange(&config.install_path, &candidate))
    {
        return match begin_cleanup(config, record, RetainedBundle::Old) {
            Ok(()) => Err(error),
            Err(cleanup) => Err(format!(
                "{error}; cleanup failed: {cleanup}; transaction retained"
            )),
        };
    }
    // No await after the commit boundary. Cancellation and durability failures
    // restore the prior bundle synchronously; a crash is recovered using inode IDs.
    if let Err(error) = checkpoint("post-exchange-sync")
        .and_then(|_| sync_dir(config.install_path.parent().unwrap()))
        .and_then(|_| sync_dir(&root))
        .and_then(|_| context.check_cancelled())
    {
        return match restore(config, record) {
            Ok(()) => Err(error),
            Err(rollback) => Err(format!(
                "{error}; rollback failed: {rollback}; transaction retained"
            )),
        };
    }
    Ok(())
}

pub(super) fn rollback(config: &UpdaterConfig) -> Result<(), String> {
    if !pending(config)? {
        return Err("no pending update to roll back".into());
    }
    restore(config, read_record(config)?.unwrap())
}

pub(super) fn confirm(config: &UpdaterConfig) -> Result<u32, String> {
    if !pending(config)? {
        return Err("no pending update to confirm".into());
    }
    let record = read_record(config)?.unwrap();
    if !matches!(record.phase, Phase::Staged) {
        return Err(
            "update has a recorded rollback decision; complete recovery before confirmation".into(),
        );
    }
    let manifest = record.signed.verify(config)?;
    if config.current_sequence != manifest.sequence {
        return Err(
            "confirmation requires the newly restarted application's release sequence".into(),
        );
    }
    begin_cleanup(config, record, RetainedBundle::New)?;
    Ok(manifest.sequence)
}
