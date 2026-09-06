use std::sync::atomic::{AtomicU32, Ordering};

use gpui::{MouseButton, NavigationDirection, Pixels, Point, ScrollDelta, ScrollWheelEvent, Size};

use crate::protocol::{
    EVENT_POINTER, EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, KeyAction, POINTER_BUTTON_BACK,
    POINTER_BUTTON_FORWARD, POINTER_BUTTON_LEFT, POINTER_BUTTON_MIDDLE, POINTER_BUTTON_RIGHT,
    SCROLL_DELTA_LINES, SCROLL_DELTA_PIXELS,
};
use crate::transport::{RuntimeAdapter, send_event_or_exit};

fn key_modifiers(modifiers: &gpui::Modifiers) -> Vec<String> {
    let mut values = Vec::with_capacity(5);
    if modifiers.control {
        values.push("ctrl".to_string());
    }
    if modifiers.alt {
        values.push("alt".to_string());
    }
    if modifiers.shift {
        values.push("shift".to_string());
    }
    if modifiers.platform {
        values.push("cmd".to_string());
    }
    if modifiers.function {
        values.push("function".to_string());
    }
    values
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_key_event(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    key: &str,
    modifiers: &gpui::Modifiers,
    action: KeyAction,
) {
    if key.is_empty() {
        return;
    }
    let event = Event::key(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        key.to_string(),
        key_modifiers(modifiers),
        action,
    );
    send_event_or_exit(runtime, "key event", event);
}

fn pointer_button(button: MouseButton) -> u32 {
    match button {
        MouseButton::Left => POINTER_BUTTON_LEFT,
        MouseButton::Right => POINTER_BUTTON_RIGHT,
        MouseButton::Middle => POINTER_BUTTON_MIDDLE,
        MouseButton::Navigate(NavigationDirection::Back) => POINTER_BUTTON_BACK,
        MouseButton::Navigate(NavigationDirection::Forward) => POINTER_BUTTON_FORWARD,
    }
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_pointer_event(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    event_type: u32,
    event_button: MouseButton,
    modifiers: &gpui::Modifiers,
    click_count: usize,
    position: Point<Pixels>,
    viewport_size: Size<Pixels>,
) {
    let button = pointer_button(event_button);
    let action = if event_type == EVENT_POINTER_DOWN {
        EVENT_POINTER_DOWN
    } else {
        EVENT_POINTER_UP
    };
    let x = clamp_coordinate(position.x.as_f32(), viewport_size.width.as_f32());
    let y = clamp_coordinate(position.y.as_f32(), viewport_size.height.as_f32());
    let event = Event::pointer(
        EVENT_POINTER,
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        button,
        key_modifiers(modifiers),
        action,
        click_count.max(1).min(u32::MAX as usize) as u32,
        x,
        y,
    );
    send_event_or_exit(runtime, "pointer event", event);
}
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_pointer_move(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    position: Point<Pixels>,
    modifiers: &gpui::Modifiers,
    viewport_size: Size<Pixels>,
) {
    let x = clamp_coordinate(position.x.as_f32(), viewport_size.width.as_f32());
    let y = clamp_coordinate(position.y.as_f32(), viewport_size.height.as_f32());
    let event = Event::pointer_move(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        x,
        y,
        key_modifiers(modifiers),
    );
    send_event_or_exit(runtime, "pointer move event", event);
}

fn clamp_coordinate(value: f32, upper_bound: f32) -> f32 {
    if !value.is_finite() {
        return 0.0;
    }
    value.clamp(0.0, upper_bound.max(0.0))
}
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_scroll_event(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    event: &ScrollWheelEvent,
) {
    let (delta_kind, dx, dy) = match event.delta {
        ScrollDelta::Pixels(delta) => (SCROLL_DELTA_PIXELS, delta.x.as_f32(), delta.y.as_f32()),
        ScrollDelta::Lines(delta) => (SCROLL_DELTA_LINES, delta.x, delta.y),
    };
    let event = Event::scroll(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        delta_kind,
        dx,
        dy,
        event.position.x.as_f32(),
        event.position.y.as_f32(),
        key_modifiers(&event.modifiers),
    );
    send_event_or_exit(runtime, "scroll event", event);
}
#[allow(clippy::too_many_arguments)]
pub(super) fn emit_drag_over(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    drag_type: &str,
) {
    let event = Event::drag_over(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        drag_type.to_owned(),
    );
    send_event_or_exit(runtime, "drag over event", event);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_drag_drop(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    drag_type: &str,
) {
    let event = Event::drag_drop(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        drag_type.to_owned(),
    );
    send_event_or_exit(runtime, "drag drop event", event);
}

#[allow(clippy::too_many_arguments)]
pub(super) fn emit_external_file_drop(
    runtime: &dyn RuntimeAdapter,
    sequence: &AtomicU32,
    surface_id: u32,
    epoch: u32,
    revision: u32,
    node_id: u32,
    listener_id: u32,
    paths: Vec<String>,
) {
    let event = Event::external_file_drop(
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        paths,
    );
    send_event_or_exit(runtime, "external file drop event", event);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{EVENT_POINTER_DOWN, EVENT_POINTER_UP, EventPayload};
    use crate::transport::InMemoryAdapter;

    #[test]
    fn pointer_coordinates_are_clamped_before_wire_encoding() {
        let runtime = InMemoryAdapter::new();
        let sequence = AtomicU32::new(1);
        emit_pointer_event(
            runtime.as_ref(),
            &sequence,
            7,
            3,
            1,
            2,
            9,
            EVENT_POINTER_DOWN,
            MouseButton::Left,
            &gpui::Modifiers::none(),
            1,
            gpui::point(gpui::px(-10.0), gpui::px(900.0)),
            gpui::size(gpui::px(800.0), gpui::px(600.0)),
        );

        let event = runtime
            .take_event()
            .expect("in-memory event should decode")
            .expect("pointer event should be queued");
        let EventPayload::Pointer(pointer) = event.payload else {
            panic!("expected pointer payload");
        };
        assert_eq!(pointer.action, EVENT_POINTER_DOWN);
        assert_eq!(pointer.x, 0.0);
        assert_eq!(pointer.y, 600.0);
    }
    #[test]
    fn pointer_move_coordinates_are_clamped_before_wire_encoding() {
        let runtime = InMemoryAdapter::new();
        let sequence = AtomicU32::new(1);
        emit_pointer_move(
            runtime.as_ref(),
            &sequence,
            7,
            3,
            1,
            2,
            9,
            gpui::point(gpui::px(-10.0), gpui::px(900.0)),
            &gpui::Modifiers::none(),
            gpui::size(gpui::px(800.0), gpui::px(600.0)),
        );
        let event = runtime
            .take_event()
            .expect("in-memory event should decode")
            .expect("pointer move should be queued");
        let EventPayload::PointerMove(pointer) = event.payload else {
            panic!("expected pointer move payload");
        };
        assert_eq!((pointer.x, pointer.y), (0.0, 600.0));
    }

    #[test]
    fn zero_click_count_mouse_up_is_normalized_before_wire_encoding() {
        let runtime = InMemoryAdapter::new();
        let sequence = AtomicU32::new(1);
        emit_pointer_event(
            runtime.as_ref(),
            &sequence,
            7,
            3,
            1,
            2,
            9,
            EVENT_POINTER_UP,
            MouseButton::Left,
            &gpui::Modifiers::none(),
            0,
            gpui::point(gpui::px(12.5), gpui::px(24.0)),
            gpui::size(gpui::px(800.0), gpui::px(600.0)),
        );

        let event = runtime
            .take_event()
            .expect("in-memory event should decode")
            .expect("pointer event should be queued");
        let EventPayload::Pointer(pointer) = event.payload else {
            panic!("expected pointer payload");
        };
        assert_eq!(pointer.action, EVENT_POINTER_UP);
        assert_eq!(pointer.click_count, 1);
        assert_eq!(pointer.x, 12.5);
        assert_eq!(pointer.y, 24.0);
    }
}
