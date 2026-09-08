//! Application-owned system motion subscription and explicit override policy.
use gpui::{App, Global, Task};
use tokio::sync::watch;

#[cfg(target_os = "macos")]
#[path = "motion/macos.rs"]
mod platform;
#[cfg(target_os = "windows")]
#[path = "motion/windows.rs"]
mod platform;
#[cfg(target_os = "linux")]
#[path = "motion/linux.rs"]
mod platform;

#[cfg(target_family = "wasm")]
#[path = "motion/web.rs"]
mod platform;

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum MotionMode {
    #[default]
    System,
    Reduced,
    Full,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(tag = "status", rename_all = "camelCase")]
pub enum MotionSource {
    Starting,
    Available { reduced: bool },
    Unavailable { message: String },
}

#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct MotionPreference {
    pub mode: MotionMode,
    pub reduced: bool,
    pub source: MotionSource,
}
impl Default for MotionPreference {
    fn default() -> Self {
        // Do not start decorative motion before the system preference is known.
        Self {
            mode: MotionMode::System,
            reduced: true,
            source: MotionSource::Starting,
        }
    }
}
impl MotionPreference {
    fn source_changed(&mut self, source: MotionSource) {
        if self.mode == MotionMode::System
            && let MotionSource::Available { reduced } = source
        {
            self.reduced = reduced;
        }
        self.source = source;
    }
    fn set_mode(&mut self, mode: MotionMode) -> Result<(), String> {
        let reduced = match mode {
            MotionMode::Full => false,
            MotionMode::Reduced => true,
            MotionMode::System => match &self.source {
                MotionSource::Available { reduced } => *reduced,
                MotionSource::Starting => {
                    return Err("system-motion-unavailable: preference is still loading".into());
                }
                MotionSource::Unavailable { message } => {
                    return Err(format!("system-motion-unavailable: {message}"));
                }
            },
        };
        self.mode = mode;
        self.reduced = reduced;
        Ok(())
    }
}
struct Motion {
    preference: MotionPreference,
    _subscription: Option<platform::Subscription>,
    _receiver: Task<()>,
}
impl Global for Motion {}

pub fn initialize(cx: &mut App) {
    let (sender, receiver) = watch::channel(MotionSource::Starting);
    let subscription = match platform::subscribe(sender.clone()) {
        Ok(subscription) => Some(subscription),
        Err(message) => {
            sender.send_replace(MotionSource::Unavailable { message });
            None
        }
    };
    drop(sender);
    install(cx, subscription, receiver);
}

fn install(
    cx: &mut App,
    subscription: Option<platform::Subscription>,
    mut receiver: watch::Receiver<MotionSource>,
) {
    let mut preference = MotionPreference::default();
    preference.source_changed(receiver.borrow_and_update().clone());
    cx.set_reduce_motion(preference.reduced);
    let task = cx.spawn(async move |cx| {
        while receiver.changed().await.is_ok() {
            let source = receiver.borrow_and_update().clone();
            cx.update(|cx| {
                let state = &mut cx.global_mut::<Motion>().preference;
                state.source_changed(source);
                let reduced = state.reduced;
                cx.set_reduce_motion(reduced);
            });
        }
    });
    cx.set_global(Motion {
        preference,
        _subscription: subscription,
        _receiver: task,
    });
}
pub fn get(cx: &App) -> Result<MotionPreference, String> {
    cx.try_global::<Motion>()
        .map(|motion| motion.preference.clone())
        .ok_or_else(|| {
            "system-motion-unavailable: this host did not initialize motion preferences".into()
        })
}
pub fn set(mode: MotionMode, cx: &mut App) -> Result<MotionPreference, String> {
    let mut state = get(cx)?;
    state.set_mode(mode)?;
    cx.global_mut::<Motion>().preference = state.clone();
    cx.set_reduce_motion(state.reduced);
    Ok(state)
}

#[cfg(all(test, feature = "gpui-component"))]
pub(crate) fn initialize_for_test(cx: &mut App) {
    let (_, receiver) = watch::channel(MotionSource::Available { reduced: false });
    install(cx, None, receiver);
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_updates_respect_override_and_source_loss_preserves_effective_motion() {
        let mut state = MotionPreference::default();
        assert!(state.reduced);
        state.source_changed(MotionSource::Available { reduced: false });
        assert!(!state.reduced);
        state.set_mode(MotionMode::Reduced).unwrap();
        state.source_changed(MotionSource::Available { reduced: false });
        assert!(state.reduced);
        state.set_mode(MotionMode::System).unwrap();
        assert!(!state.reduced);
        state.source_changed(MotionSource::Unavailable {
            message: "subscription ended".into(),
        });
        assert!(!state.reduced);
        state.set_mode(MotionMode::Reduced).unwrap();
        assert!(state.set_mode(MotionMode::System).is_err());
        assert_eq!(state.mode, MotionMode::Reduced);
    }
}
