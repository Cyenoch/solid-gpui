use super::MotionSource;
use tokio::sync::watch;
use windows::{
    Foundation::TypedEventHandler,
    UI::ViewManagement::{UISettings, UISettingsAnimationsEnabledChangedEventArgs},
};
pub(super) struct Subscription {
    settings: UISettings,
    token: i64,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        if let Err(error) = self.settings.RemoveAnimationsEnabledChanged(self.token) {
            eprintln!("solid-gpui-host: could not remove system motion observer: {error}");
        }
    }
}
fn read(settings: &UISettings) -> MotionSource {
    match settings.AnimationsEnabled() {
        Ok(enabled) => MotionSource::Available { reduced: !enabled },
        Err(error) => MotionSource::Unavailable {
            message: error.to_string(),
        },
    }
}
pub(super) fn subscribe(sender: watch::Sender<MotionSource>) -> Result<Subscription, String> {
    let settings = UISettings::new().map_err(|error| error.to_string())?;
    let changes = sender.clone();
    let ordering = std::sync::Arc::new(std::sync::Mutex::new(()));
    let callback_ordering = ordering.clone();
    let token = settings
        .AnimationsEnabledChanged(&TypedEventHandler::<
            UISettings,
            UISettingsAnimationsEnabledChangedEventArgs,
        >::new(move |settings, _| {
            let _order = callback_ordering.lock().unwrap();
            if let Some(settings) = settings.as_ref() {
                changes.send_replace(read(settings));
            }
            Ok(())
        }))
        .map_err(|error| format!("motion notification API unavailable: {error}"))?;
    let _order = ordering.lock().unwrap();
    sender.send_replace(read(&settings));
    Ok(Subscription { settings, token })
}
