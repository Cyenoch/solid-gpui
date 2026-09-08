use super::MotionSource;
use tokio::sync::watch;
use wasm_bindgen::{JsCast, closure::Closure};
use web_sys::{MediaQueryList, MediaQueryListEvent};

pub(super) struct Subscription {
    query: MediaQueryList,
    callback: Closure<dyn FnMut(MediaQueryListEvent)>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        let _ = self
            .query
            .remove_event_listener_with_callback("change", self.callback.as_ref().unchecked_ref());
    }
}
pub(super) fn subscribe(sender: watch::Sender<MotionSource>) -> Result<Subscription, String> {
    let query = web_sys::window()
        .ok_or("browser window is unavailable")?
        .match_media("(prefers-reduced-motion: reduce)")
        .map_err(|error| format!("motion media query failed: {error:?}"))?
        .ok_or("motion media query is unavailable")?;
    sender.send_replace(MotionSource::Available {
        reduced: query.matches(),
    });
    let callback = Closure::new(move |event: MediaQueryListEvent| {
        sender.send_replace(MotionSource::Available {
            reduced: event.matches(),
        });
    });
    query
        .add_event_listener_with_callback("change", callback.as_ref().unchecked_ref())
        .map_err(|error| format!("motion subscription failed: {error:?}"))?;
    Ok(Subscription { query, callback })
}
