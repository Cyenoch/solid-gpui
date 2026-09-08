use super::MotionSource;
use block2::RcBlock;
use objc2::{rc::Retained, runtime::ProtocolObject};
use objc2_app_kit::{NSWorkspace, NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification};
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObjectProtocol, NSOperationQueue};
use std::ptr::NonNull;
use tokio::sync::watch;

pub(super) struct Subscription {
    center: Retained<NSNotificationCenter>,
    observer: Retained<ProtocolObject<dyn NSObjectProtocol>>,
}
impl Drop for Subscription {
    fn drop(&mut self) {
        // This App-owned guard is created and destroyed on the GPUI main thread.
        unsafe {
            self.center.removeObserver((*self.observer).as_ref());
        }
    }
}
pub(super) fn subscribe(sender: watch::Sender<MotionSource>) -> Result<Subscription, String> {
    let workspace = NSWorkspace::sharedWorkspace();
    let center = workspace.notificationCenter();
    let changes = sender.clone();
    let callback = RcBlock::new(move |_: NonNull<NSNotification>| {
        changes.send_replace(MotionSource::Available {
            reduced: NSWorkspace::sharedWorkspace().accessibilityDisplayShouldReduceMotion(),
        });
    });
    // Register before reading. The notification center copies the block, and
    // the main operation queue serializes callbacks with the initial query.
    let observer = unsafe {
        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceAccessibilityDisplayOptionsDidChangeNotification),
            None,
            Some(&NSOperationQueue::mainQueue()),
            &callback,
        )
    };
    sender.send_replace(MotionSource::Available {
        reduced: workspace.accessibilityDisplayShouldReduceMotion(),
    });
    Ok(Subscription { center, observer })
}
