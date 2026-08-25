use std::sync::atomic::{AtomicU32, Ordering};

use gpui::{MouseButton, NavigationDirection, ScrollDelta, ScrollWheelEvent};

use crate::protocol::{
    EVENT_POINTER_DOWN, EVENT_POINTER_UP, Event, KeyAction, POINTER_BUTTON_BACK,
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
    send_event_or_exit(runtime, "key event", &event);
}

fn pointer_button(button: MouseButton) -> Option<u32> {
    match button {
        MouseButton::Left => Some(POINTER_BUTTON_LEFT),
        MouseButton::Right => Some(POINTER_BUTTON_RIGHT),
        MouseButton::Middle => Some(POINTER_BUTTON_MIDDLE),
        MouseButton::Navigate(NavigationDirection::Back) => Some(POINTER_BUTTON_BACK),
        MouseButton::Navigate(NavigationDirection::Forward) => Some(POINTER_BUTTON_FORWARD),
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
) {
    let Some(button) = pointer_button(event_button) else {
        return;
    };
    let action = if event_type == EVENT_POINTER_DOWN {
        EVENT_POINTER_DOWN
    } else {
        EVENT_POINTER_UP
    };
    let event = Event::pointer(
        event_type,
        surface_id,
        epoch,
        revision,
        sequence.fetch_add(1, Ordering::Relaxed),
        node_id,
        listener_id,
        button,
        key_modifiers(modifiers),
        action,
        click_count.min(u32::MAX as usize) as u32,
    );
    send_event_or_exit(runtime, "pointer event", &event);
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
    send_event_or_exit(runtime, "scroll event", &event);
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
    send_event_or_exit(runtime, "drag over event", &event);
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
    send_event_or_exit(runtime, "drag drop event", &event);
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
    send_event_or_exit(runtime, "external file drop event", &event);
}
