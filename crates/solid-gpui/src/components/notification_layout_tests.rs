//! Toast card layout, asserted against a really-drawn frame.
//!
//! These tests live beside the adapter instead of inside the vendored card: the
//! card's geometry is what a reader sees, so it has to be measured from real
//! paint bounds rather than from the style values that fed it. The card itself
//! registers the selectors (`vendor/gpui-kit/.../notification.rs`).
use crate::components::host::ComponentHost;
use crate::host::HostProfile;
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::{HostProperties, InMemoryAdapter, Node, Snapshot};
use gpui::{
    AnyWindowHandle, Bounds, Pixels, TestAppContext, VisualTestContext, WindowBounds,
    WindowOptions, px, size,
};
use std::sync::Arc;
use std::time::Duration;

const CARD: &str = "notification-card";
const ICON: &str = "notification-icon";
const COPY: &str = "notification-copy";
const CLOSE: &str = "notification-close-button";
/// The inset from the card's own edge to every slot inside it.
const CARD_INSET: Pixels = px(16.);
/// The gap between two neighbouring slots.
const SLOT_GAP: Pixels = px(12.);
/// The close button's own height (`Button::small`).
const CLOSE_SIZE: Pixels = px(24.);
/// The card draws a hairline border around its padding.
const BORDER: Pixels = px(1.);
/// Layout rounds to whole pixels, so a glyph may sit half a pixel off centre.
const TOLERANCE: Pixels = px(0.5);

fn notification_snapshot(message: &str) -> Snapshot {
    let props = format!(
        r#"{{"open":true,"kind":"success","placement":"topCenter","message":{}}}"#,
        serde_json::to_string(message).expect("a message is JSON-encodable")
    );
    let mut toast = Node::new(2, 1, 0, crate::KIND_EXTENSION);
    toast.host_properties = Some(HostProperties::Extension(ExtensionProperties {
        provider_id: super::native_module().id(),
        catalog_digest: super::native_module().digest(),
        entry_id: super::native_module()
            .component_id("Notification")
            .expect("Notification is in the catalog"),
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(props.into_bytes()),
        }],
        event_ids: Arc::from([]),
    }));
    Snapshot::new(
        1,
        1,
        0,
        1,
        vec![Node::new(1, 0, 0, crate::KIND_VIEW), toast],
    )
}

/// Show one toast in a real provider window and let its enter motion finish.
fn drawn_toast(cx: &mut TestAppContext, message: &str) -> VisualTestContext {
    let mut profile = ComponentHost::default();
    let runtime = InMemoryAdapter::new();
    let extensions = profile.extension_registry();
    let (window, solid_root) = cx.update(|app| {
        profile.initialize(app);
        let options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                None,
                size(px(480.), px(320.)),
                app,
            ))),
            ..Default::default()
        };
        profile
            .open_window(options, runtime.clone(), extensions, app)
            .expect("provider window opens")
    });
    let mut visual = VisualTestContext::from_window(AnyWindowHandle::from(window), cx);
    visual.update(|window, cx| {
        let message =
            crate::protocol::decode_message(&notification_snapshot(message).encode().unwrap())
                .unwrap();
        solid_root.update(cx, |root, cx| {
            root.apply_decoded_message_in_window(message, window, cx)
                .unwrap()
        })
    });
    // The card is pushed through the window's notification layer and animates
    // in; measure it once that motion has settled.
    visual
        .background_executor
        .advance_clock(Duration::from_millis(600));
    visual.run_until_parked();
    visual.update(|window, cx| window.draw(cx).clear(cx));
    visual
}

fn bounds(visual: &mut VisualTestContext, selector: &'static str) -> Bounds<Pixels> {
    visual
        .debug_bounds(selector)
        .unwrap_or_else(|| panic!("{selector} is painted"))
}

fn left(bounds: Bounds<Pixels>) -> Pixels {
    bounds.origin.x
}

fn right(bounds: Bounds<Pixels>) -> Pixels {
    bounds.origin.x + bounds.size.width
}

fn top(bounds: Bounds<Pixels>) -> Pixels {
    bounds.origin.y
}

fn bottom(bounds: Bounds<Pixels>) -> Pixels {
    bounds.origin.y + bounds.size.height
}

fn center_y(bounds: Bounds<Pixels>) -> Pixels {
    bounds.origin.y + bounds.size.height / 2.
}

fn assert_close(actual: Pixels, expected: Pixels, what: &str) {
    assert!(
        (actual - expected).abs() <= TOLERANCE,
        "{what}: {actual:?} is not within {TOLERANCE:?} of {expected:?}"
    );
}

/// Every inset and gap in the card is the same, and the icon and the close
/// button stand on the copy's first line. The card used to draw the icon at a
/// hard-coded offset (18 px down, 16 px in) with the copy padded around it, so
/// the icon sat below the text as soon as the line height changed — and the
/// close button was invisible until the card was hovered, which is how a reader
/// could end up with no way to dismiss a toast at all.
#[gpui::test]
fn a_single_line_toast_insets_every_slot_by_the_same_amount(cx: &mut TestAppContext) {
    let mut visual = drawn_toast(cx, "已复制链接");

    let card = bounds(&mut visual, CARD);
    let icon = bounds(&mut visual, ICON);
    let copy = bounds(&mut visual, COPY);
    let close = bounds(&mut visual, CLOSE);

    assert_close(left(icon) - left(card), CARD_INSET + BORDER, "left inset");
    assert_close(top(icon) - top(card), CARD_INSET + BORDER, "top inset");
    assert_close(
        right(card) - right(close),
        CARD_INSET + BORDER,
        "right inset",
    );
    assert_close(
        bottom(card) - bottom(copy),
        CARD_INSET + BORDER,
        "bottom inset",
    );
    assert_close(left(copy) - right(icon), SLOT_GAP, "icon/copy gap");

    assert_close(top(icon), top(copy), "icon starts with the copy");
    assert!(icon.size.height >= copy.size.height, "{icon:?} vs {copy:?}");
    assert_close(center_y(close), center_y(copy), "close button is centred");
    assert_close(close.size.height, CLOSE_SIZE, "close button height");
}

/// A wrapped message keeps the icon and the close button on the first line
/// instead of centring them on the card.
#[gpui::test]
fn a_wrapped_toast_aligns_its_slots_with_the_first_line(cx: &mut TestAppContext) {
    let mut visual = drawn_toast(
        cx,
        "无法打开下载位置：目标目录已经不存在，可能已被移动或删除，请在任务详情中选择新的保存位置后重试。",
    );

    let card = bounds(&mut visual, CARD);
    let icon = bounds(&mut visual, ICON);
    let copy = bounds(&mut visual, COPY);
    let close = bounds(&mut visual, CLOSE);

    assert!(
        copy.size.height > icon.size.height * 1.5,
        "precondition: the copy wrapped ({copy:?})"
    );
    let first_line = top(copy) + icon.size.height / 2.;
    assert_close(center_y(icon), first_line, "icon centres on the first line");
    assert_close(
        center_y(close),
        first_line,
        "close centres on the first line",
    );
    assert!(
        (center_y(icon) - center_y(card)).abs() > px(8.),
        "the icon must not be centred on a multi-line card ({card:?} {icon:?})"
    );
    assert_close(
        bottom(card) - bottom(copy),
        CARD_INSET + BORDER,
        "bottom inset",
    );
}
