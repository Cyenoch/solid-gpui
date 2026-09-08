//! App-owned bounded work with coalesced progress and explicit cancellation.
use solid_gpui::{
    gpui::{self, App, Context, IntoElement, ParentElement, Render, Window, div},
    native::{ComponentDefinition, Event, NativeChildren, NativeView},
    native_type,
};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

const MAX_ENTRIES: u32 = 100_000;
const MAX_PENDING_DIRECTORIES: usize = 4_096;
const UPDATE_INTERVAL: Duration = Duration::from_millis(50);
static ACTIVE_JOBS: AtomicUsize = AtomicUsize::new(0);

#[native_type]
#[derive(Clone, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct ScanProps {
    pub path: Option<String>,
    pub request_id: u32,
}

#[native_type]
#[derive(Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScanProgress {
    pub request_id: u32,
    pub files: u32,
    pub directories: u32,
    pub finished: bool,
    pub error: Option<String>,
}

type Latest = Arc<Mutex<Option<ScanProgress>>>;
struct Permit;
impl Drop for Permit {
    fn drop(&mut self) {
        ACTIVE_JOBS.fetch_sub(1, Ordering::AcqRel);
    }
}
struct Job {
    cancelled: Arc<AtomicBool>,
    _observer: gpui::Task<()>,
}
impl Drop for Job {
    fn drop(&mut self) {
        self.cancelled.store(true, Ordering::Release);
    }
}

fn scan(
    path: &Path,
    cancelled: &AtomicBool,
    latest: &Latest,
    progress: &mut ScanProgress,
) -> Result<(), String> {
    let metadata = std::fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Choose a directory, not a symbolic link".into());
    }
    progress.directories = 1;
    let mut pending = vec![PathBuf::from(path)];
    let mut last_update = Instant::now();
    while let Some(directory) = pending.pop() {
        if cancelled.load(Ordering::Acquire) {
            return Err("Scan cancelled".into());
        }
        for entry in std::fs::read_dir(directory).map_err(|error| error.to_string())? {
            if cancelled.load(Ordering::Acquire) {
                return Err("Scan cancelled".into());
            }
            if progress.files + progress.directories >= MAX_ENTRIES {
                return Err("Scan reached its 100,000-entry limit".into());
            }
            let entry = entry.map_err(|error| error.to_string())?;
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_dir() {
                if pending.len() >= MAX_PENDING_DIRECTORIES {
                    return Err("Scan reached its pending-directory limit".into());
                }
                progress.directories += 1;
                pending.push(entry.path());
            } else {
                // Symlinks are counted but never followed, so cycles cannot expand work.
                progress.files += 1;
            }
            if last_update.elapsed() >= UPDATE_INTERVAL {
                *latest.lock().unwrap() = Some(progress.clone());
                last_update = Instant::now();
            }
        }
    }
    Ok(())
}

pub struct WorkspaceScan {
    props: ScanProps,
    event: Event<ScanProgress>,
    progress: ScanProgress,
    job: Option<Job>,
    mounted: bool,
}
impl WorkspaceScan {
    fn start(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.mounted {
            return;
        }
        self.job = None;
        self.progress = ScanProgress {
            request_id: self.props.request_id,
            ..Default::default()
        };
        let Some(path) = self.props.path.clone() else {
            cx.notify();
            return;
        };
        if ACTIVE_JOBS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |count| {
                (count < 2).then_some(count + 1)
            })
            .is_err()
        {
            self.progress.finished = true;
            self.progress.error =
                Some("Two scans are still running; retry after they finish".into());
            self.event.emit(self.progress.clone());
            cx.notify();
            return;
        }
        let permit = Permit;
        let cancelled = Arc::new(AtomicBool::new(false));
        let worker_cancelled = cancelled.clone();
        let latest: Latest = Arc::new(Mutex::new(None));
        let worker_latest = latest.clone();
        let mut progress = self.progress.clone();
        let worker = std::thread::Builder::new()
            .name("website-workspace-scan".into())
            .spawn(move || {
                let _permit = permit;
                progress.error = scan(
                    Path::new(&path),
                    &worker_cancelled,
                    &worker_latest,
                    &mut progress,
                )
                .err();
                progress.finished = true;
                *worker_latest.lock().unwrap() = Some(progress);
            });
        if let Err(error) = worker {
            self.progress.finished = true;
            self.progress.error = Some(format!("Could not start scan: {error}"));
            self.event.emit(self.progress.clone());
            cx.notify();
            return;
        }
        let executor = cx.background_executor().clone();
        let observer = cx.spawn_in(window, async move |view, cx| {
            loop {
                executor.timer(UPDATE_INTERVAL).await;
                let progress = latest.lock().unwrap().take();
                if let Some(progress) = progress {
                    let finished = progress.finished;
                    let updated = cx.update(|_, cx| {
                        view.update(cx, |view, cx| {
                            view.progress = progress;
                            view.event.emit(view.progress.clone());
                            cx.notify();
                        })
                    });
                    if finished || !matches!(updated, Ok(Ok(()))) {
                        break;
                    }
                }
            }
        });
        self.job = Some(Job {
            cancelled,
            _observer: observer,
        });
        cx.notify();
    }
}
impl NativeView for WorkspaceScan {
    type Props = ScanProps;
    type Event = ScanProgress;
    fn mount(
        props: ScanProps,
        event: Event<ScanProgress>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        cx.defer_in(window, |view, window, cx| view.start(window, cx));
        Self {
            props,
            event,
            progress: ScanProgress::default(),
            job: None,
            mounted: true,
        }
    }
    fn update(&mut self, props: ScanProps, window: &mut Window, cx: &mut Context<Self>) {
        if self.props != props {
            self.props = props;
            self.start(window, cx);
        }
    }
    fn unmount(&mut self, _: &mut Window, _: &mut App) {
        self.mounted = false;
        self.job = None;
    }
    fn validate_props(props: &ScanProps) -> Result<(), String> {
        if props.path.as_ref().is_some_and(|path| {
            path.len() > 4096
                || path.chars().any(char::is_control)
                || !Path::new(path).is_absolute()
        }) {
            return Err("Scan path must be an absolute path of at most 4096 bytes".into());
        }
        Ok(())
    }
    fn event_name() -> &'static str {
        "progress"
    }
}
impl Render for WorkspaceScan {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().child(format!(
            "{} files · {} directories",
            self.progress.files, self.progress.directories
        ))
    }
}
pub fn definition() -> ComponentDefinition {
    ComponentDefinition::view::<WorkspaceScan>("WorkspaceScan")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn real_directory_walk_counts_entries_without_following_cycles_and_observes_cancellation() {
        let path = std::env::temp_dir().join(format!("solid-gpui-scan-{}", std::process::id()));
        std::fs::create_dir_all(path.join("nested")).unwrap();
        std::fs::write(path.join("nested/file.txt"), "content").unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(&path, path.join("cycle")).unwrap();
        let cancelled = AtomicBool::new(false);
        let latest = Arc::new(Mutex::new(None));
        let mut progress = ScanProgress::default();
        let result = scan(&path, &cancelled, &latest, &mut progress);
        std::fs::remove_dir_all(&path).unwrap();
        result.unwrap();
        assert_eq!(progress.directories, 2);
        assert_eq!(progress.files, if cfg!(unix) { 2 } else { 1 });
        cancelled.store(true, Ordering::Release);
        let error = scan(&std::env::temp_dir(), &cancelled, &latest, &mut progress).unwrap_err();
        assert_eq!(error, "Scan cancelled");
    }
}
