use std::borrow::Cow;
use std::collections::HashSet;
use std::ops::Range;
use std::sync::atomic::Ordering;

use gpui::{
    App, Bounds, Context, EntityInputHandler, Pixels, Point, ShapedLine, TextAlign, UTF16Selection,
    Window, WrappedLine, point, px, size,
};

use crate::protocol::{
    EVENT_BLUR, EVENT_CHANGE, EVENT_FOCUS, EVENT_SELECTION, Event, HostProperties, TextInputEvent,
    TextInputProperties,
};
use crate::transport::send_event_or_exit;
use crate::tree::{KIND_PRESSABLE, KIND_TEXT, KIND_VIEW};

use super::ReactRoot;

#[derive(Debug, Default)]
pub(super) struct NativeInputState {
    pub(super) text: String,
    pub(super) selection: Range<usize>,
    pub(super) selection_reversed: bool,
    pub(super) marked: Option<Range<usize>>,
    pub(super) edit_seq: u32,
    pub(super) focused: bool,
    pub(super) max_length: Option<usize>,
}

pub(super) enum TextInputTextLayout {
    Single {
        line: Box<ShapedLine>,
        line_height: Pixels,
    },
    Multiline {
        lines: Vec<WrappedLine>,
        line_starts: Vec<usize>,
        line_height: Pixels,
    },
}

impl Default for TextInputTextLayout {
    fn default() -> Self {
        Self::Single {
            line: Box::new(ShapedLine::default()),
            line_height: px(0.0),
        }
    }
}

pub(super) struct TextInputLayout {
    pub(super) text: TextInputTextLayout,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) content: String,
    pub(super) placeholder: bool,
}

#[derive(Clone, Copy)]
pub(super) struct TextInputPosition {
    pub(super) point: Point<Pixels>,
    pub(super) line_index: usize,
    pub(super) line_width: Pixels,
    pub(super) line_height: Pixels,
}

impl TextInputTextLayout {
    pub(super) fn position_for_utf8(&self, index: usize) -> TextInputPosition {
        match self {
            Self::Single { line, line_height } => {
                let index = index.min(line.len());
                TextInputPosition {
                    point: point(line.x_for_index(index), px(0.0)),
                    line_index: 0,
                    line_width: line.width(),
                    line_height: *line_height,
                }
            }
            Self::Multiline {
                lines,
                line_starts,
                line_height,
            } => {
                let mut physical_line = 0;
                for (paragraph, line) in lines.iter().enumerate() {
                    let start = line_starts.get(paragraph).copied().unwrap_or_default();
                    let end = start + line.len();
                    if index <= end || paragraph + 1 == lines.len() {
                        let local = index.saturating_sub(start).min(line.len());
                        let point = line
                            .position_for_index(local, *line_height)
                            .unwrap_or_else(|| point(px(0.0), px(0.0)));
                        let wrapped_line = if f32::from(*line_height) > 0.0 {
                            (f32::from(point.y) / f32::from(*line_height)).max(0.0) as usize
                        } else {
                            0
                        };
                        return TextInputPosition {
                            point,
                            line_index: physical_line + wrapped_line,
                            line_width: line.width(),
                            line_height: *line_height,
                        };
                    }
                    physical_line += line.wrap_boundaries().len() + 1;
                }
                TextInputPosition {
                    point: point(px(0.0), px(0.0)),
                    line_index: physical_line,
                    line_width: px(0.0),
                    line_height: *line_height,
                }
            }
        }
    }

    pub(super) fn closest_index_for_point(&self, at: Point<Pixels>) -> usize {
        match self {
            Self::Single { line, .. } => line.closest_index_for_x(at.x),
            Self::Multiline {
                lines,
                line_starts,
                line_height,
            } => {
                let mut y = 0.0;
                let point_y = f32::from(at.y);
                for (paragraph, line) in lines.iter().enumerate() {
                    let line_height_px = f32::from(line.size(*line_height).height);
                    if point_y < y + line_height_px || paragraph + 1 == lines.len() {
                        let local_point = point(at.x, px((f32::from(at.y) - y).max(0.0)));
                        let local = line
                            .closest_index_for_position(local_point, *line_height)
                            .unwrap_or_else(|index| index)
                            .min(line.len());
                        return line_starts.get(paragraph).copied().unwrap_or_default() + local;
                    }
                    y += line_height_px;
                }
                line_starts.last().copied().unwrap_or_default()
                    + lines.last().map(WrappedLine::len).unwrap_or_default()
            }
        }
    }

    pub(super) fn paint(
        &self,
        origin: Point<Pixels>,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut App,
    ) {
        match self {
            Self::Single { line, line_height } => {
                let _ = line.paint(
                    origin,
                    *line_height,
                    TextAlign::Left,
                    Some(bounds.size.width),
                    window,
                    cx,
                );
            }
            Self::Multiline {
                lines, line_height, ..
            } => {
                let mut y = px(0.0);
                for line in lines {
                    let _ = line.paint(
                        origin + point(px(0.0), y),
                        *line_height,
                        TextAlign::Left,
                        Some(bounds),
                        window,
                        cx,
                    );
                    y += line.size(*line_height).height;
                }
            }
        }
    }

    pub(super) fn bounds_for_range(
        &self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
    ) -> Bounds<Pixels> {
        let start = self.position_for_utf8(range.start);
        let end = self.position_for_utf8(range.end);
        let origin = bounds.origin;
        if range.start == range.end {
            return Bounds::new(origin + start.point, size(px(2.0), start.line_height));
        }
        if start.line_index == end.line_index {
            let start_x = f32::from(start.point.x);
            let end_x = f32::from(end.point.x);
            return Bounds::from_corners(
                origin + point(px(start_x.min(end_x)), start.point.y),
                origin + point(px(start_x.max(end_x)), start.point.y + start.line_height),
            );
        }
        let first = Bounds::from_corners(
            origin + start.point,
            origin + point(start.line_width, start.point.y + start.line_height),
        );
        let last = Bounds::from_corners(
            origin + point(px(0.0), end.point.y),
            origin + point(end.point.x, end.point.y + end.line_height),
        );
        first.union(&last)
    }

    pub(super) fn selection_bounds_per_line(
        &self,
        range: Range<usize>,
        bounds: Bounds<Pixels>,
    ) -> Vec<Bounds<Pixels>> {
        if range.start >= range.end {
            return Vec::new();
        }
        match self {
            Self::Single { line, line_height } => (line.x_for_index(range.end.min(line.len()))
                > line.x_for_index(range.start.min(line.len())))
            .then(|| {
                let start = line.x_for_index(range.start.min(line.len()));
                let end = line.x_for_index(range.end.min(line.len()));
                Bounds::new(
                    bounds.origin + point(start, px(0.0)),
                    size(end - start, *line_height),
                )
            })
            .into_iter()
            .collect(),
            Self::Multiline {
                lines,
                line_starts,
                line_height,
            } => {
                let mut result = Vec::new();
                let mut visual_line = 0usize;
                for (paragraph, line) in lines.iter().enumerate() {
                    let paragraph_start = line_starts.get(paragraph).copied().unwrap_or_default();
                    let mut row_start = 0usize;
                    let mut row_ends: Vec<usize> = line
                        .wrap_boundaries()
                        .iter()
                        .map(|boundary| {
                            line.runs()[boundary.run_ix].glyphs[boundary.glyph_ix].index
                        })
                        .collect();
                    row_ends.push(line.len());
                    for row_end in row_ends {
                        let global_start = paragraph_start + row_start;
                        let global_end = paragraph_start + row_end;
                        let start = range.start.max(global_start);
                        let end = range.end.min(global_end);
                        if start < end {
                            let local_start = start - paragraph_start;
                            let local_end = end - paragraph_start;
                            let start_x = if local_start <= row_start {
                                px(0.0)
                            } else {
                                line.position_for_index(local_start, *line_height)
                                    .map(|position| position.x)
                                    .unwrap_or(px(0.0))
                            };
                            let end_x = line
                                .position_for_index(local_end.min(row_end), *line_height)
                                .map(|position| position.x)
                                .unwrap_or(line.width());
                            if end_x > start_x {
                                result.push(Bounds::new(
                                    bounds.origin
                                        + point(px(0.0), *line_height * visual_line as f32)
                                        + point(start_x, px(0.0)),
                                    size(end_x - start_x, *line_height),
                                ));
                            }
                        }
                        row_start = row_end;
                        visual_line += 1;
                    }
                }
                result
            }
        }
    }
}
pub(super) fn line_starts(text: &str) -> Vec<usize> {
    let mut starts = vec![0];
    for (index, character) in text.char_indices() {
        if character == '\n' {
            starts.push(index + character.len_utf8());
        }
    }
    starts
}

#[cfg(test)]
pub(super) fn multiline_utf16_position(text: &str, offset: usize) -> (usize, usize) {
    let byte = utf16_byte_index(text, offset);
    let starts = line_starts(text);
    let mut line = 0;
    for (index, start) in starts.iter().enumerate() {
        if *start > byte {
            break;
        }
        line = index;
    }
    (line, byte - starts[line])
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
        self.selection_reversed = false;
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
        self.selection_reversed = false;
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
        self.selection_reversed = false;
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
            self.selection_reversed = input.selection_reversed;
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
        self.text_input_layouts
            .retain(|id, _| self.input_states.contains_key(id));
        self.focus_handles.retain(|id, _| {
            self.input_states.contains_key(id)
                || self.store.get(*id).is_some_and(|node| {
                    ((node.kind == KIND_VIEW || node.kind == KIND_PRESSABLE)
                        && node.focusable
                        && node.listener_id != 0)
                        || (node.kind == KIND_TEXT && node.selectable)
                })
        });
        if self
            .active_input
            .is_some_and(|id| !self.input_states.contains_key(&id))
        {
            self.active_input = None;
            self.text_input_drag_anchor = None;
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
                self.text_input_layouts.remove(&id);
                let state = self
                    .input_states
                    .entry(id)
                    .or_insert_with(|| NativeInputState {
                        text: input.value.clone(),
                        selection: input.selection_start as usize..input.selection_end as usize,
                        selection_reversed: input.selection_reversed,
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

    pub(super) fn reconcile_selectable_text_states(
        &mut self,
        cx: &mut Context<Self>,
        _affected: Option<&HashSet<u32>>,
    ) {
        self.selectable_text_selections.retain(|id, selection| {
            let Some(node) = self.store.get(*id) else {
                return false;
            };
            if node.kind != KIND_TEXT || !node.selectable {
                return false;
            }
            let len = node.text_content.as_ref().map_or(0, |text| text.len());
            Self::clamp_utf8_range(
                selection,
                node.text_content.as_deref().unwrap_or_default(),
                len,
            );
            true
        });
        self.selectable_text_layouts.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_TEXT && node.selectable)
        });
        if self
            .selectable_text_drag_anchor
            .is_some_and(|(id, _)| !self.selectable_text_selections.contains_key(&id))
        {
            self.selectable_text_drag_anchor = None;
        }
        for node in self
            .store
            .iter()
            .filter(|node| node.kind == KIND_TEXT && node.selectable)
        {
            let len = node.text_content.as_ref().map_or(0, |text| text.len());
            let selection = self
                .selectable_text_selections
                .entry(node.id)
                .or_insert_with(|| 0..0);
            Self::clamp_utf8_range(
                selection,
                node.text_content.as_deref().unwrap_or_default(),
                len,
            );
            self.focus_handles
                .entry(node.id)
                .or_insert_with(|| cx.focus_handle());
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
                reversed: state.selection_reversed,
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
                self.text_input_drag_anchor = None;
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
                self.text_input_drag_anchor = None;
            }
            return;
        }
        if let Some(state) = self.input_states.get_mut(&node_id) {
            state.focused = focused;
        }
        self.emit_input_event(node_id, if focused { EVENT_FOCUS } else { EVENT_BLUR });
        if !focused && self.active_input == Some(node_id) {
            self.active_input = None;
            self.text_input_drag_anchor = None;
        }
    }

    fn text_input_index_for_point(
        &mut self,
        node_id: u32,
        point: Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Option<usize> {
        self.active_input = Some(node_id);
        let fallback = self
            .input_states
            .get(&node_id)
            .map(|state| state.selection.end)?;
        Some(
            EntityInputHandler::character_index_for_point(self, point, window, cx)
                .unwrap_or(fallback),
        )
    }

    pub(super) fn begin_text_input_selection(
        &mut self,
        node_id: u32,
        point: Point<gpui::Pixels>,
        extend: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.set_input_focus(node_id, true);
        let fallback = self
            .input_states
            .get(&node_id)
            .map(|state| state.selection.end)
            .unwrap_or_default();
        let index = self
            .text_input_index_for_point(node_id, point, window, cx)
            .unwrap_or(fallback);
        let anchor = self
            .input_states
            .get(&node_id)
            .map(|state| {
                if extend {
                    if state.selection_reversed {
                        state.selection.end
                    } else {
                        state.selection.start
                    }
                } else {
                    index
                }
            })
            .unwrap_or(index);
        self.text_input_drag_anchor = Some((node_id, anchor));
        let (selection, reversed) = selection_from_anchor(anchor, index);
        if let Some(state) = self.input_states.get_mut(&node_id) {
            state.selection = selection;
            state.selection_reversed = reversed;
        }
        self.emit_input_event(node_id, EVENT_SELECTION);
        cx.notify();
    }

    pub(super) fn update_text_input_selection(
        &mut self,
        node_id: u32,
        point: Point<gpui::Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some((anchor_node, anchor)) = self.text_input_drag_anchor else {
            return;
        };
        if anchor_node != node_id {
            return;
        }
        let fallback = self
            .input_states
            .get(&node_id)
            .map(|state| state.selection.end)
            .unwrap_or(anchor);
        let index = self
            .text_input_index_for_point(node_id, point, window, cx)
            .unwrap_or(fallback);
        let (selection, reversed) = selection_from_anchor(anchor, index);
        let changed = self.input_states.get(&node_id).is_some_and(|state| {
            state.selection != selection || state.selection_reversed != reversed
        });
        if !changed {
            return;
        }
        if let Some(state) = self.input_states.get_mut(&node_id) {
            state.selection = selection;
            state.selection_reversed = reversed;
        }
        self.emit_input_event(node_id, EVENT_SELECTION);
        cx.notify();
    }

    pub(super) fn end_text_input_selection(&mut self, node_id: u32) {
        if self
            .text_input_drag_anchor
            .is_some_and(|(anchor_node, _)| anchor_node == node_id)
        {
            self.text_input_drag_anchor = None;
        }
    }

    fn selectable_text_index_for_point(&self, node_id: u32, point: Point<Pixels>) -> Option<usize> {
        let layout = self.selectable_text_layouts.get(&node_id)?;
        let local = point.relative_to(&layout.bounds.origin);
        Some(
            layout
                .text
                .closest_index_for_point(local)
                .min(layout.content.len()),
        )
    }

    pub(super) fn begin_selectable_text_selection(
        &mut self,
        node_id: u32,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let index = self
            .selectable_text_index_for_point(node_id, point)
            .unwrap_or_default();
        self.selectable_text_drag_anchor = Some((node_id, index));
        self.selectable_text_selections
            .insert(node_id, index..index);
        cx.notify();
    }

    pub(super) fn update_selectable_text_selection(
        &mut self,
        node_id: u32,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) {
        let Some((anchor_node, anchor)) = self.selectable_text_drag_anchor else {
            return;
        };
        if anchor_node != node_id {
            return;
        }
        let head = self
            .selectable_text_index_for_point(node_id, point)
            .unwrap_or(anchor);
        let (selection, _) = selection_from_anchor(anchor, head);
        if self.selectable_text_selections.get(&node_id) == Some(&selection) {
            return;
        }
        self.selectable_text_selections.insert(node_id, selection);
        cx.notify();
    }

    pub(super) fn end_selectable_text_selection(&mut self, node_id: u32) {
        if self
            .selectable_text_drag_anchor
            .is_some_and(|(anchor_node, _)| anchor_node == node_id)
        {
            self.selectable_text_drag_anchor = None;
        }
    }

    pub(super) fn selected_selectable_text(&self, node_id: u32) -> Option<String> {
        let node = self.store.get(node_id)?;
        let text = node.text_content.as_deref()?;
        let range = self.selectable_text_selections.get(&node_id)?;
        if range.start >= range.end
            || range.end > text.len()
            || !text.is_char_boundary(range.start)
            || !text.is_char_boundary(range.end)
        {
            return None;
        }
        Some(text[range.clone()].to_owned())
    }
    pub(super) fn handle_text_input_navigation(
        &mut self,
        node_id: u32,
        key: &str,
        extend: bool,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self.input_states.get(&node_id) else {
            return;
        };
        let Some((selection, reversed)) = move_selection(
            &state.text,
            &state.selection,
            state.selection_reversed,
            key,
            extend,
        ) else {
            return;
        };
        let changed = state.selection != selection || state.selection_reversed != reversed;
        if !changed {
            return;
        }
        if let Some(state) = self.input_states.get_mut(&node_id) {
            state.selection = selection;
            state.selection_reversed = reversed;
        }
        self.active_input = Some(node_id);
        self.emit_input_event(node_id, EVENT_SELECTION);
        cx.notify();
    }
    fn clamp_utf8_range(range: &mut Range<usize>, text: &str, len: usize) {
        range.start = range.start.min(len);
        range.end = range.end.min(len);
        while range.start > 0 && !text.is_char_boundary(range.start) {
            range.start -= 1;
        }
        while range.end > 0 && !text.is_char_boundary(range.end) {
            range.end -= 1;
        }
        if range.start > range.end {
            std::mem::swap(&mut range.start, &mut range.end);
        }
    }
}

pub(super) fn utf16_byte_index(text: &str, offset: usize) -> usize {
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
pub(super) fn utf8_byte_to_utf16(text: &str, offset: usize) -> usize {
    let mut byte_index = 0;
    let mut utf16_index = 0;
    for ch in text.chars() {
        if byte_index >= offset {
            break;
        }
        byte_index += ch.len_utf8();
        utf16_index += ch.len_utf16();
    }
    utf16_index
}
pub(super) fn selection_from_anchor(anchor: usize, head: usize) -> (Range<usize>, bool) {
    if head < anchor {
        (head..anchor, true)
    } else {
        (anchor..head, false)
    }
}

fn previous_utf16_boundary(text: &str, offset: usize) -> usize {
    let mut current = 0;
    for character in text.chars() {
        let next = current + character.len_utf16();
        if next >= offset {
            return current;
        }
        current = next;
    }
    current
}

fn next_utf16_boundary(text: &str, offset: usize) -> usize {
    let mut current = 0;
    for character in text.chars() {
        let next = current + character.len_utf16();
        if offset < next {
            return next;
        }
        current = next;
    }
    current
}

pub(super) fn move_selection(
    text: &str,
    selection: &Range<usize>,
    reversed: bool,
    key: &str,
    extend: bool,
) -> Option<(Range<usize>, bool)> {
    let is_left = key == "left" || key == "ArrowLeft";
    let is_right = key == "right" || key == "ArrowRight";
    let is_home = key == "home" || key == "Home" || key == "up" || key == "ArrowUp";
    let is_end = key == "end" || key == "End" || key == "down" || key == "ArrowDown";
    if !is_left && !is_right && !is_home && !is_end {
        return None;
    }

    let (anchor, head) = if reversed {
        (selection.end, selection.start)
    } else {
        (selection.start, selection.end)
    };
    let next_head = if is_left {
        if !extend && !selection.is_empty() {
            selection.start
        } else {
            previous_utf16_boundary(text, head)
        }
    } else if is_right {
        if !extend && !selection.is_empty() {
            selection.end
        } else {
            next_utf16_boundary(text, head)
        }
    } else if is_home {
        0
    } else {
        text.encode_utf16().count()
    };

    if extend {
        Some(selection_from_anchor(anchor, next_head))
    } else {
        Some((next_head..next_head, false))
    }
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
    fn emit_input_change_and_selection(&self) {
        if let Some(id) = self.active_input {
            self.emit_input_event(id, EVENT_CHANGE);
            self.emit_input_event(id, EVENT_SELECTION);
        }
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
            reversed: state.selection_reversed,
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
        self.emit_input_change_and_selection();
    }

    fn replace_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.active_input {
            self.text_input_layouts.remove(&id);
        }
        if let Some(state) = self.active_input_state_mut() {
            state.replace(range, text);
            cx.notify();
        }
        self.emit_input_change_and_selection();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range: Option<Range<usize>>,
        new_text: &str,
        new_selected_range: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(id) = self.active_input {
            self.text_input_layouts.remove(&id);
        }
        if let Some(state) = self.active_input_state_mut() {
            state.replace_marked(range, new_text, new_selected_range);
            cx.notify();
        }
        self.emit_input_change_and_selection();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        element_bounds: Bounds<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<gpui::Pixels>> {
        let Some(id) = self.active_input else {
            return Some(element_bounds);
        };
        let Some(state) = self.input_states.get(&id) else {
            return Some(element_bounds);
        };
        let Some(layout) = self.text_input_layouts.get(&id) else {
            return Some(element_bounds);
        };
        if layout.placeholder || layout.content != state.text {
            debug_assert_eq!(layout.content, state.text);
            return Some(element_bounds);
        }
        let start = utf16_byte_index(&state.text, range_utf16.start);
        let end = utf16_byte_index(&state.text, range_utf16.end);
        Some(layout.text.bounds_for_range(start..end, layout.bounds))
    }

    fn character_index_for_point(
        &mut self,
        point: Point<gpui::Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let id = self.active_input?;
        let state = self.input_states.get(&id)?;
        let Some(layout) = self.text_input_layouts.get(&id) else {
            return Some(state.selection.end);
        };
        if layout.placeholder || layout.content != state.text {
            debug_assert_eq!(layout.content, state.text);
            return Some(state.selection.end);
        }
        let local = layout.bounds.localize(&point)?;
        let utf8_offset = layout.text.closest_index_for_point(local);
        Some(utf8_byte_to_utf16(&layout.content, utf8_offset))
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
    #[test]
    fn text_edits_clear_reversed_selection_orientation() {
        let mut state = NativeInputState {
            text: "abcd".into(),
            selection: 1..3,
            selection_reversed: true,
            ..Default::default()
        };
        state.replace(None, "x");
        assert_eq!(state.selection, 2..2);
        assert!(!state.selection_reversed);
    }

    #[test]
    fn utf16_and_utf8_offsets_round_trip_at_astral_boundaries() {
        let text = "a😀中";
        assert_eq!(utf16_byte_index(text, 0), 0);
        assert_eq!(utf16_byte_index(text, 1), 1);
        assert_eq!(utf16_byte_index(text, 2), 5);
        assert_eq!(utf16_byte_index(text, 3), 5);
        assert_eq!(utf16_byte_index(text, 4), 8);
        assert_eq!(utf16_byte_index(text, 5), 8);
        assert_eq!(utf8_byte_to_utf16(text, 0), 0);
        assert_eq!(utf8_byte_to_utf16(text, 1), 1);
        assert_eq!(utf8_byte_to_utf16(text, 5), 3);
        assert_eq!(utf8_byte_to_utf16(text, 8), 4);
    }

    #[test]
    fn selection_anchor_normalizes_drag_direction_and_preserves_reversed_head() {
        assert_eq!(selection_from_anchor(5, 2), (2..5, true));
        assert_eq!(selection_from_anchor(2, 5), (2..5, false));
        assert_eq!(selection_from_anchor(3, 3), (3..3, false));
    }

    #[test]
    fn selectable_selection_clamps_changed_text_to_utf8_boundaries() {
        let mut range = 1..8;
        ReactRoot::clamp_utf8_range(&mut range, "a😀", "a😀".len());
        assert_eq!(range, 1..5);
        let mut reversed = Range { start: 8, end: 1 };
        ReactRoot::clamp_utf8_range(&mut reversed, "a😀", "a😀".len());
        assert_eq!(reversed, 1..5);
    }

    #[test]
    fn shift_navigation_respects_utf16_boundaries_and_home_end_aliases() {
        let text = "a😀b";
        assert_eq!(
            move_selection(text, &(3..3), false, "left", true),
            Some((1..3, true))
        );
        assert_eq!(
            move_selection(text, &(1..3), true, "right", true),
            Some((3..3, false))
        );
        assert_eq!(
            move_selection(text, &(1..3), true, "right", false),
            Some((3..3, false))
        );
        assert_eq!(
            move_selection(text, &(1..1), false, "up", true),
            Some((0..1, true))
        );
        assert_eq!(
            move_selection(text, &(1..1), false, "down", false),
            Some((4..4, false))
        );
    }

    #[test]
    fn navigation_accepts_platform_arrow_key_names() {
        assert_eq!(
            move_selection("ab", &(1..1), false, "ArrowLeft", false),
            Some((0..0, false))
        );
        assert_eq!(
            move_selection("ab", &(1..1), false, "ArrowRight", false),
            Some((2..2, false))
        );
        assert_eq!(
            move_selection("ab", &(1..1), false, "ArrowUp", false),
            Some((0..0, false))
        );
        assert_eq!(
            move_selection("ab", &(1..1), false, "ArrowDown", false),
            Some((2..2, false))
        );
        assert_eq!(
            move_selection("ab", &(1..1), false, "Home", false),
            Some((0..0, false))
        );
        assert_eq!(
            move_selection("ab", &(1..1), false, "End", false),
            Some((2..2, false))
        );
    }
    #[test]
    fn multiline_utf16_positions_cover_emoji_empty_lines_and_trailing_newline() {
        let text = "a😀\n中\n\n";
        assert_eq!(line_starts(text), vec![0, 6, 10, 11]);
        assert_eq!(multiline_utf16_position(text, 0), (0, 0));
        assert_eq!(multiline_utf16_position(text, 1), (0, 1));
        assert_eq!(multiline_utf16_position(text, 3), (0, 5));
        assert_eq!(multiline_utf16_position(text, 4), (1, 0));
        assert_eq!(multiline_utf16_position(text, 5), (1, 3));
        assert_eq!(multiline_utf16_position(text, 6), (2, 0));
        assert_eq!(multiline_utf16_position(text, 7), (3, 0));
    }
}
