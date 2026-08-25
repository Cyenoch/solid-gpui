use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::atomic::Ordering;

use gpui::{Bounds, Context, EntityInputHandler, Point, UTF16Selection, Window};

use crate::protocol::{
    EVENT_BLUR, EVENT_CHANGE, EVENT_FOCUS, EVENT_SELECTION, Event, HostProperties, TextInputEvent,
    TextInputProperties,
};
use crate::transport::send_event_or_exit;
use crate::tree::{KIND_PRESSABLE, KIND_VIEW};

use super::ReactRoot;

#[derive(Debug, Default)]
pub(super) struct NativeInputState {
    pub(super) text: String,
    pub(super) selection: Range<usize>,
    pub(super) marked: Option<Range<usize>>,
    pub(super) edit_seq: u32,
    pub(super) focused: bool,
    pub(super) max_length: Option<usize>,
}

fn truncate_utf16(value: &str, max_length: usize) -> Cow<'_, str> {
    let mut units = 0;
    let mut end = 0;
    for character in value.chars() {
        let next_units = units + character.len_utf16();
        if next_units > max_length {
            break;
        }
        units = next_units;
        end += character.len_utf8();
    }
    if end == value.len() {
        Cow::Borrowed(value)
    } else {
        Cow::Owned(value[..end].to_owned())
    }
}

impl NativeInputState {
    pub(super) fn set_max_length(&mut self, max_length: Option<usize>) {
        self.max_length = max_length;
        let Some(max_length) = max_length else {
            return;
        };
        if self.text.encode_utf16().count() <= max_length {
            return;
        }
        self.text = truncate_utf16(&self.text, max_length).into_owned();
        self.selection.start = self.selection.start.min(max_length);
        self.selection.end = self.selection.end.min(max_length);
        if let Some(marked) = self.marked.as_mut() {
            marked.start = marked.start.min(max_length);
            marked.end = marked.end.min(max_length);
        }
    }

    pub(super) fn replace(&mut self, range: Option<Range<usize>>, text: &str) {
        let range = range.unwrap_or_else(|| self.selection.clone());
        let available = self.max_length.map(|max_length| {
            max_length.saturating_sub(
                self.text
                    .encode_utf16()
                    .count()
                    .saturating_sub(range.end.saturating_sub(range.start)),
            )
        });
        let replacement = available.map(|limit| truncate_utf16(text, limit));
        let replacement = replacement.as_deref().unwrap_or(text);
        let cursor = replace_utf16(&mut self.text, range, replacement);
        self.selection = cursor..cursor;
        self.marked = None;
        self.edit_seq = self.edit_seq.wrapping_add(1);
    }

    pub(super) fn replace_marked(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        let range = range.unwrap_or_else(|| self.selection.clone());
        let available = self.max_length.map(|max_length| {
            max_length.saturating_sub(
                self.text
                    .encode_utf16()
                    .count()
                    .saturating_sub(range.end.saturating_sub(range.start)),
            )
        });
        let replacement = available.map(|limit| truncate_utf16(text, limit));
        let replacement = replacement.as_deref().unwrap_or(text);
        let start = range.start;
        let inserted_end = replace_utf16(&mut self.text, range, replacement);
        self.marked = Some(start..inserted_end);
        let selected = selected.unwrap_or(inserted_end..inserted_end);
        self.selection = selected.start.min(inserted_end)..selected.end.min(inserted_end);
        self.edit_seq = self.edit_seq.wrapping_add(1);
    }

    pub(super) fn unmark(&mut self) -> bool {
        if self.marked.take().is_some() {
            self.edit_seq = self.edit_seq.wrapping_add(1);
            true
        } else {
            false
        }
    }

    pub(super) fn set_selection(&mut self, selection: Range<usize>) {
        self.selection = selection;
    }

    pub(super) fn apply_controlled(&mut self, input: &TextInputProperties) {
        if input.controlled && self.edit_seq <= input.ack_edit_seq && self.marked.is_none() {
            self.text = self
                .max_length
                .map(|max_length| truncate_utf16(&input.value, max_length).into_owned())
                .unwrap_or_else(|| input.value.clone());
            self.selection = input.selection_start as usize..input.selection_end as usize;
            self.selection.start = self.selection.start.min(self.text.encode_utf16().count());
            self.selection.end = self.selection.end.min(self.text.encode_utf16().count());
            self.marked = input
                .marked_start
                .map(|start| start as usize..input.marked_end.unwrap_or(start) as usize);
        }
    }
}

impl ReactRoot {
    pub(super) fn reconcile_input_states(
        &mut self,
        cx: &mut Context<Self>,
        affected: Option<&HashSet<u32>>,
    ) {
        self.input_states.retain(|id, _| {
            self.store.get(*id).is_some_and(|node| {
                matches!(node.host_properties, Some(HostProperties::TextInput(_)))
            })
        });
        self.focus_handles.retain(|id, _| {
            self.input_states.contains_key(id)
                || self.store.get(*id).is_some_and(|node| {
                    (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
                        && node.focusable
                        && node.listener_id != 0
                })
        });
        if self
            .active_input
            .is_some_and(|id| !self.input_states.contains_key(&id))
        {
            self.active_input = None;
        }
        let ids: Vec<u32> = match affected {
            Some(ids) => ids.iter().copied().collect(),
            None => self
                .store
                .iter()
                .filter(|node| {
                    matches!(node.host_properties, Some(HostProperties::TextInput(_)))
                        || ((node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
                            && node.focusable
                            && node.listener_id != 0)
                })
                .map(|node| node.id)
                .collect(),
        };
        for id in ids {
            let Some(node) = self.store.get(id) else {
                continue;
            };
            if let Some(HostProperties::TextInput(input)) = node.host_properties.clone() {
                let state = self
                    .input_states
                    .entry(id)
                    .or_insert_with(|| NativeInputState {
                        text: input.value.clone(),
                        selection: input.selection_start as usize..input.selection_end as usize,
                        marked: input.marked_start.map(|start| {
                            start as usize..input.marked_end.unwrap_or(start) as usize
                        }),
                        edit_seq: 0,
                        focused: false,
                        max_length: input.max_length.map(|value| value as usize),
                    });
                self.focus_handles
                    .entry(id)
                    .or_insert_with(|| cx.focus_handle());
                state.set_max_length(input.max_length.map(|value| value as usize));
                state.apply_controlled(&input);
            } else if (node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
                && node.focusable
                && node.listener_id != 0
            {
                self.focus_handles
                    .entry(id)
                    .or_insert_with(|| cx.focus_handle());
            }
        }
    }

    pub(super) fn emit_input_event(&self, node_id: u32, event_type: u32) {
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.listener_id == 0 {
            return;
        }
        let Some(state) = self.input_states.get(&node_id) else {
            return;
        };
        let event = Event::text_input(
            event_type,
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            TextInputEvent {
                text: state.text.clone(),
                selection_start: state.selection.start as u32,
                selection_end: state.selection.end as u32,
                marked_start: state.marked.as_ref().map(|range| range.start as u32),
                marked_end: state.marked.as_ref().map(|range| range.end as u32),
                edit_seq: state.edit_seq,
            },
        );
        send_event_or_exit(self.runtime.as_ref(), "TextInput event", &event);
    }
    pub(super) fn emit_submit_event(&self, node_id: u32) {
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        let Some(HostProperties::TextInput(input)) = node.host_properties.as_ref() else {
            return;
        };
        if input.multiline || node.listener_id == 0 {
            return;
        }
        let event = Event::submit_with_text(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            self.input_states
                .get(&node_id)
                .map(|state| state.text.clone())
                .unwrap_or_else(|| input.value.clone()),
        );
        send_event_or_exit(self.runtime.as_ref(), "TextInput submit event", &event);
    }

    pub(super) fn set_input_focus(&mut self, node_id: u32, focused: bool) {
        if focused {
            if let Some(previous) = self.active_input.take()
                && previous != node_id
            {
                if let Some(state) = self.input_states.get_mut(&previous) {
                    state.focused = false;
                }
                self.emit_input_event(previous, EVENT_BLUR);
            }
            self.active_input = Some(node_id);
        }
        let changed = self
            .input_states
            .get(&node_id)
            .is_some_and(|state| state.focused != focused);
        if !changed {
            if !focused && self.active_input == Some(node_id) {
                self.active_input = None;
            }
            return;
        }
        if let Some(state) = self.input_states.get_mut(&node_id) {
            state.focused = focused;
        }
        self.emit_input_event(node_id, if focused { EVENT_FOCUS } else { EVENT_BLUR });
        if !focused && self.active_input == Some(node_id) {
            self.active_input = None;
        }
    }
}

fn utf16_byte_index(text: &str, offset: usize) -> usize {
    let mut units = 0;
    for (index, ch) in text.char_indices() {
        if units >= offset {
            return index;
        }
        units += ch.len_utf16();
        if units >= offset {
            return index + ch.len_utf8();
        }
    }
    text.len()
}

fn replace_utf16(text: &mut String, range: Range<usize>, replacement: &str) -> usize {
    let start = utf16_byte_index(text, range.start);
    let end = utf16_byte_index(text, range.end);
    text.replace_range(start..end, replacement);
    range.start + replacement.encode_utf16().count()
}

impl ReactRoot {
    fn active_input_state(&self) -> Option<&NativeInputState> {
        self.active_input.and_then(|id| self.input_states.get(&id))
    }

    fn active_input_state_mut(&mut self) -> Option<&mut NativeInputState> {
        let id = self.active_input?;
        self.input_states.get_mut(&id)
    }
}

impl EntityInputHandler for ReactRoot {
    fn text_for_range(
        &mut self,
        range: Range<usize>,
        adjusted: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let state = self.active_input_state()?;
        let start = utf16_byte_index(&state.text, range.start);
        let end = utf16_byte_index(&state.text, range.end);
        *adjusted = Some(range);
        Some(state.text[start..end].to_string())
    }

    fn selected_text_range(
        &mut self,
        _ignore_disabled_input: bool,

        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        let state = self.active_input_state()?;
        Some(UTF16Selection {
            range: state.selection.clone(),
            reversed: false,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.active_input_state()
            .and_then(|state| state.marked.clone())
    }

    fn unmark_text(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        if let Some(state) = self.active_input_state_mut()
            && state.unmark()
        {
            cx.notify();
        }
        if let Some(id) = self.active_input {
            self.emit_input_event(id, EVENT_CHANGE);
            self.emit_input_event(id, EVENT_SELECTION);
        }
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = self.active_input_state_mut() {
            state.replace(range, text);
            cx.notify();
        }
        if let Some(id) = self.active_input {
            self.emit_input_event(id, EVENT_CHANGE);
            self.emit_input_event(id, EVENT_SELECTION);
        }
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(state) = self.active_input_state_mut() {
            state.replace_marked(range, new_text, new_selected_range);
            cx.notify();
        }
        if let Some(id) = self.active_input {
            self.emit_input_event(id, EVENT_CHANGE);
            self.emit_input_event(id, EVENT_SELECTION);
        }
    }

    fn bounds_for_range(
        &mut self,
        _range_utf16: Range<usize>,
        element_bounds: Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<gpui::Pixels>> {
        Some(element_bounds)
    }

    fn character_index_for_point(
        &mut self,
        _point: Point<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.active_input_state().map(|state| state.selection.end)
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn max_length_limits_utf16_replacements_without_splitting_surrogates() {
        let mut state = NativeInputState {
            text: "ab".into(),
            selection: 1..1,
            max_length: Some(4),
            ..Default::default()
        };
        state.replace(None, "😀x");
        assert_eq!(state.text, "a😀b");
        assert_eq!(state.selection, 3..3);
        assert_eq!(state.text.encode_utf16().count(), 4);
    }
}
