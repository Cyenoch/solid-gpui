use super::SolidRoot;
use crate::native::text::*;
use crate::protocol::HostProperties;
use crate::tree::{KIND_TEXT, KIND_VIRTUAL_LIST};
use gpui::{Context, Pixels, Point, Window};
use std::{
    collections::HashMap,
    ops::Range,
    sync::{Arc, OnceLock},
};
use unicode_segmentation::UnicodeSegmentation;

const MAX_TEXT_BYTES: usize = 256 * 1024;
const MAX_SPANS: usize = 4096;
const MAX_MATCHES: usize = 1024;

#[derive(Clone, PartialEq, Eq)]
struct Run {
    id: u32,
    text: Arc<str>,
    virtual_row: Option<(u32, u32, u32)>,
}
#[derive(Clone, PartialEq, Eq)]
struct Selected {
    run: Run,
    range: Range<usize>,
}
#[derive(Default)]
pub(super) struct DocumentText {
    runs: Vec<Run>,
    index: HashMap<u32, usize>,
    revision: u32,
    selection_revision: u32,
    anchor: Option<TextPosition>,
    head: Option<TextPosition>,
    selected: Vec<Selected>,
    detached: bool,
    dragging: bool,
    forward: bool,
    search_revision: u32,
    search: Option<TextSearchSnapshot>,
    search_ranges: HashMap<u32, Vec<Range<usize>>>,
    observers: Vec<(u32, u32)>,
    emitted: Option<TextSelectionChange>,
    emit_suppressed: bool,
    focus: Option<gpui::FocusHandle>,
    focus_owner: Option<gpui::FocusHandle>,
    scroll_scheduled: bool,
    drag_generation: u32,
    drag_point: Point<Pixels>,
}

fn utf16(text: &str, byte: usize) -> u32 {
    text[..byte].encode_utf16().count() as u32
}
fn byte(text: &str, offset: u32) -> Result<usize, String> {
    let mut units = 0;
    for (index, cluster) in text.grapheme_indices(true) {
        if units == offset {
            return Ok(index);
        }
        units += cluster.encode_utf16().count() as u32;
    }
    if units == offset {
        Ok(text.len())
    } else {
        Err("text offset must address a UTF-16 grapheme boundary".into())
    }
}
fn snap(text: &str, index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain([text.len()])
        .take_while(|i| *i <= index)
        .last()
        .unwrap_or(0)
}
fn span(selected: &Selected) -> TextSpan {
    TextSpan {
        node_id: selected.run.id,
        start: utf16(&selected.run.text, selected.range.start),
        end: utf16(&selected.run.text, selected.range.end),
    }
}

impl SolidRoot {
    pub(super) fn reconcile_document_text(&mut self) {
        self.document_text_clips.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_TEXT && node.selectable)
        });
        static CONTRACT: OnceLock<([u8; 16], [u8; 32], u32)> = OnceLock::new();
        let (provider, digest, entry) = *CONTRACT.get_or_init(|| {
            let module = crate::native::text::native_module();
            (
                module.id(),
                module.digest(),
                module.component_id("TextSelectionObserver").unwrap(),
            )
        });
        let mut observers: Vec<_> = self.store.iter().filter(|node| matches!(&node.host_properties,
            Some(HostProperties::Extension(p)) if p.provider_id == provider && p.catalog_digest == digest
            && p.entry_id == entry)).map(|node| (node.id, node.listener_id)).collect();
        observers.sort_unstable();
        if self.document_text.observers != observers {
            self.document_text.observers = observers;
            self.document_text.emitted = None;
        }
        let mut runs = Vec::new();
        let mut stack = self
            .store
            .root()
            .map(|node| vec![(node.id, None)])
            .unwrap_or_default();
        while let Some((id, row)) = stack.pop() {
            let Some(node) = self.store.get(id) else {
                continue;
            };
            if node.kind == KIND_TEXT {
                if node.selectable {
                    runs.push(Run {
                        id,
                        text: node.text_content.clone().unwrap_or_default(),
                        virtual_row: row,
                    });
                }
                // A styled Text subtree is one shaped logical paragraph.
                continue;
            }
            let children: Vec<_> = node
                .children(&self.store)
                .map(|child| {
                    let row = if node.kind == KIND_VIRTUAL_LIST {
                        match &node.host_properties {
                            Some(HostProperties::VirtualList(list)) => {
                                Some((id, list.data_revision, list.range_start + child.index))
                            }
                            _ => row,
                        }
                    } else {
                        row
                    };
                    (child.id, row)
                })
                .collect();
            stack.extend(children.into_iter().rev());
        }
        if self.document_text.runs == runs {
            return;
        }
        let state = &mut self.document_text;
        state.revision = state.revision.wrapping_add(1);
        state.index = runs
            .iter()
            .enumerate()
            .map(|(i, run)| (run.id, i))
            .collect();
        state.runs = runs;
        state.search = None;
        state.search_ranges.clear();
        state.search_revision = state.search_revision.wrapping_add(1);
        let changed = state.selected.iter().any(|selected| {
            if let Some(i) = state.index.get(&selected.run.id) {
                return state.runs[*i] != selected.run;
            }
            // Only eviction beyond a still-live virtual materialization window
            // retains copy bytes; deletion/filter/data replacement clears selection.
            !selected.run.virtual_row.is_some_and(|(id, revision, index)| {
                self.store.get(id).is_some_and(|node| matches!(&node.host_properties,
                    Some(HostProperties::VirtualList(list)) if list.data_revision == revision
                    && index < list.item_count && (index < list.range_start || index >= list.range_end)))
            })
        });
        if changed {
            // Preserve the existing single-paragraph clamping contract. A
            // multi-paragraph edit invalidates the whole selection atomically.
            if let (Some(mut anchor), Some(mut head)) = (state.anchor.clone(), state.head.clone())
                && anchor.node_id == head.node_id
                && let Some(&i) = state.index.get(&anchor.node_id)
                && state
                    .selected
                    .iter()
                    .all(|selected| selected.run.virtual_row == state.runs[i].virtual_row)
            {
                let text = &state.runs[i].text;
                let clamp = |offset| {
                    utf16(
                        text,
                        snap(text, super::input::utf16_byte_index(text, offset as usize)),
                    )
                };
                anchor.offset = clamp(anchor.offset);
                head.offset = clamp(head.offset);
                let request = TextSelectionRequest {
                    text_revision: state.revision,
                    anchor,
                    head,
                };
                if self.set_text_selection(request).is_ok() {
                    return;
                }
            }
            self.clear_text_selection();
            return;
        }
        let detached = state
            .selected
            .iter()
            .any(|selected| !state.index.contains_key(&selected.run.id));
        state.detached = detached;
        if !detached && let (Some(anchor), Some(head)) = (state.anchor.clone(), state.head.clone())
        {
            let request = TextSelectionRequest {
                text_revision: state.revision,
                anchor,
                head,
            };
            if self.set_text_selection(request).is_err() {
                self.clear_text_selection();
            }
        }
    }

    pub fn text_selection_snapshot(&self) -> Result<TextSelectionSnapshot, String> {
        let state = &self.document_text;
        let mut text = String::new();
        for (i, selected) in state.selected.iter().enumerate() {
            if i != 0 {
                text.push('\n');
            }
            text.push_str(&selected.run.text[selected.range.clone()]);
        }
        Ok(TextSelectionSnapshot {
            text_revision: state.revision,
            selection_revision: state.selection_revision,
            anchor: state.anchor.clone(),
            head: state.head.clone(),
            spans: state.selected.iter().map(span).collect(),
            text,
            detached: state.detached,
        })
    }

    pub fn set_text_selection(&mut self, request: TextSelectionRequest) -> Result<(), String> {
        let state = &self.document_text;
        if request.text_revision != state.revision {
            return Err("text revision is obsolete".into());
        }
        let resolve = |position: &TextPosition| -> Result<(usize, usize), String> {
            let index = *state
                .index
                .get(&position.node_id)
                .ok_or("selection node is not committed selectable text")?;
            Ok((index, byte(&state.runs[index].text, position.offset)?))
        };
        let a = resolve(&request.anchor)?;
        let b = resolve(&request.head)?;
        let (start, end) = if a <= b { (a, b) } else { (b, a) };
        if end.0 - start.0 + 1 > MAX_SPANS {
            return Err("selection exceeds 4096 paragraphs".into());
        }
        let mut selected = Vec::new();
        let mut bytes = 0;
        let mut retained_bytes = 0;
        for i in start.0..=end.0 {
            let run = &state.runs[i];
            let range = (if i == start.0 { start.1 } else { 0 })..(if i == end.0 {
                end.1
            } else {
                run.text.len()
            });
            bytes += range.len() + 1;
            if bytes > MAX_TEXT_BYTES {
                return Err("selection exceeds 256 KiB".into());
            }
            retained_bytes += run.text.len();
            if retained_bytes > MAX_TEXT_BYTES {
                return Err("selection snapshots exceed 256 KiB".into());
            }
            if !range.is_empty() || start.0 != end.0 {
                selected.push(Selected {
                    run: run.clone(),
                    range,
                });
            }
        }
        let state = &mut self.document_text;
        if state.anchor.as_ref() != Some(&request.anchor)
            || state.head.as_ref() != Some(&request.head)
            || state.detached
            || state.selected != selected
        {
            state.selection_revision = state.selection_revision.wrapping_add(1);
        }
        state.anchor = Some(request.anchor);
        state.head = Some(request.head);
        state.forward = a <= b;
        state.selected = selected;
        state.detached = false;
        self.selectable_text_selections.clear();
        self.selectable_text_selections.extend(
            state
                .selected
                .iter()
                .map(|selected| (selected.run.id, selected.range.clone())),
        );
        self.emit_document_selection_change();
        Ok(())
    }

    pub fn clear_text_selection(&mut self) {
        let state = &mut self.document_text;
        if state.anchor.take().is_some() {
            state.selection_revision = state.selection_revision.wrapping_add(1);
        }
        state.head = None;
        state.selected.clear();
        state.detached = false;
        state.dragging = false;
        self.selectable_text_selections.clear();
        self.emit_document_selection_change();
    }

    pub(super) fn document_text_hit(
        &self,
        point: Point<Pixels>,
        node: Option<u32>,
    ) -> Option<TextPosition> {
        if self.last_painted_revision != Some(self.store.revision()) {
            return None;
        }
        let state = &self.document_text;
        let (id, layout) = self
            .selectable_text_layouts
            .iter()
            .filter(|(id, _)| node.is_none_or(|node| **id == node))
            .filter(|(id, _)| {
                state.index.contains_key(id) && self.document_text_clips.contains_key(id)
            })
            .min_by(|(a, _), (b, _)| {
                let distance = |id: &u32| {
                    let bounds = self.document_text_clips[id];
                    let dx = (bounds.left() - point.x)
                        .max(point.x - bounds.right())
                        .max(gpui::px(0.));
                    let dy = (bounds.top() - point.y)
                        .max(point.y - bounds.bottom())
                        .max(gpui::px(0.));
                    f32::from(dx).powi(2) + f32::from(dy).powi(2)
                };
                distance(a)
                    .total_cmp(&distance(b))
                    .then(state.index[a].cmp(&state.index[b]))
            })?;
        let run = &state.runs[state.index[id]];
        if layout.content.as_str() != run.text.as_ref() {
            return None;
        }
        let index = snap(
            &run.text,
            layout
                .text
                .closest_index_for_point(point.relative_to(&layout.bounds.origin)),
        );
        Some(TextPosition {
            node_id: *id,
            offset: utf16(&run.text, index),
        })
    }

    pub(super) fn begin_document_selection(
        &mut self,
        node_id: u32,
        point: Point<Pixels>,
        extend: bool,
        click_count: usize,
        cx: &mut Context<Self>,
    ) {
        let Some(head) = self.document_text_hit(point, Some(node_id)) else {
            return;
        };
        self.document_text
            .focus
            .get_or_insert_with(|| cx.focus_handle());
        self.document_text.focus_owner = self.focus_handles.get(&node_id).cloned();
        self.document_text.drag_generation = self.document_text.drag_generation.wrapping_add(1);
        self.document_text.scroll_scheduled = false;
        self.document_text.drag_point = point;
        if !extend && click_count >= 2 {
            let run = &self.document_text.runs[self.document_text.index[&node_id]];
            let index = byte(&run.text, head.offset).expect("native hit is a grapheme boundary");
            let range = if click_count >= 3 {
                0..run.text.len()
            } else {
                run.text
                    .unicode_word_indices()
                    .find(|(start, word)| *start <= index && index <= *start + word.len())
                    .map(|(start, word)| start..start + word.len())
                    .unwrap_or_else(|| {
                        index
                            ..run.text[index..]
                                .graphemes(true)
                                .next()
                                .map_or(index, |cluster| index + cluster.len())
                    })
            };
            let request = TextSelectionRequest {
                text_revision: self.document_text.revision,
                anchor: TextPosition {
                    node_id,
                    offset: utf16(&run.text, range.start),
                },
                head: TextPosition {
                    node_id,
                    offset: utf16(&run.text, range.end),
                },
            };
            if self.set_text_selection(request).is_ok() {
                self.document_text.dragging = true;
                cx.notify();
            }
            return;
        }
        let anchor = if extend {
            self.document_text.anchor.clone().unwrap_or(head.clone())
        } else {
            head.clone()
        };
        if self
            .set_text_selection(TextSelectionRequest {
                text_revision: self.document_text.revision,
                anchor,
                head,
            })
            .is_ok()
        {
            self.document_text.dragging = true;
            cx.notify();
        }
    }

    pub(super) fn drag_document_selection(&mut self, point: Point<Pixels>, cx: &mut Context<Self>) {
        self.document_text.drag_point = point;
        if !self.document_text.dragging {
            return;
        }
        let Some(head) = self.document_text_hit(point, None) else {
            return;
        };
        if self.document_text.head.as_ref() == Some(&head) {
            return;
        }
        let Some(anchor) = self.document_text.anchor.clone() else {
            return;
        };
        if self.document_text.detached {
            let previous = self.document_text.selected.clone();
            let forward = self.document_text.forward;
            let overlap =
                if forward {
                    previous.iter().enumerate().find(|(_, selected)| {
                        self.document_text.index.contains_key(&selected.run.id)
                    })
                } else {
                    previous.iter().enumerate().rev().find(|(_, selected)| {
                        self.document_text.index.contains_key(&selected.run.id)
                    })
                };
            let Some((index, overlap)) = overlap else {
                return;
            };
            let position = TextPosition {
                node_id: overlap.run.id,
                offset: utf16(
                    &overlap.run.text,
                    if forward {
                        overlap.range.start
                    } else {
                        overlap.range.end
                    },
                ),
            };
            let head_index = self.document_text.index[&head.node_id];
            let overlap_index = self.document_text.index[&position.node_id];
            if forward && head_index < overlap_index || !forward && head_index > overlap_index {
                return;
            }
            let retained = if forward {
                &previous[..index]
            } else {
                &previous[index + 1..]
            };
            let committed_bytes: usize = self
                .document_text
                .runs
                .iter()
                .skip(overlap_index.min(head_index))
                .take(overlap_index.abs_diff(head_index) + 1)
                .map(|run| run.text.len() + 1)
                .sum();
            if retained.len() + overlap_index.abs_diff(head_index) + 1 > MAX_SPANS
                || retained
                    .iter()
                    .map(|selected| selected.run.text.len() + 1)
                    .sum::<usize>()
                    + committed_bytes
                    > MAX_TEXT_BYTES
            {
                return;
            }
            self.document_text.emit_suppressed = true;
            let result = self.set_text_selection(TextSelectionRequest {
                text_revision: self.document_text.revision,
                anchor: position,
                head,
            });
            self.document_text.emit_suppressed = false;
            if result.is_err() {
                return;
            }
            let state = &mut self.document_text;
            if forward {
                let mut selected = retained.to_vec();
                selected.append(&mut state.selected);
                state.selected = selected;
            } else {
                state.selected.extend_from_slice(retained);
            }
            state.anchor = Some(anchor);
            state.forward = forward;
            state.detached = true;
            self.selectable_text_selections.extend(
                state
                    .selected
                    .iter()
                    .filter(|selected| state.index.contains_key(&selected.run.id))
                    .map(|selected| (selected.run.id, selected.range.clone())),
            );
            self.emit_document_selection_change();
            cx.notify();
            return;
        }
        if self
            .set_text_selection(TextSelectionRequest {
                text_revision: self.document_text.revision,
                anchor,
                head,
            })
            .is_ok()
        {
            cx.notify();
        }
    }
    pub(super) fn end_document_selection(&mut self) {
        self.document_text.dragging = false;
    }
    pub(super) fn document_selection_focus(&self) -> Option<gpui::FocusHandle> {
        self.document_text.focus.clone()
    }
    pub(super) fn document_selection_dragging(&self) -> bool {
        self.document_text.dragging
    }
    pub(super) fn schedule_document_scroll(&mut self) -> Option<u32> {
        if !self.document_text.dragging || self.document_text.scroll_scheduled {
            return None;
        }
        self.document_text.scroll_scheduled = true;
        Some(self.document_text.drag_generation)
    }
    pub(super) fn advance_document_scroll(
        &mut self,
        generation: u32,
        cx: &mut Context<Self>,
    ) -> bool {
        if generation != self.document_text.drag_generation {
            return false;
        }
        self.document_text.scroll_scheduled = false;
        let point = self.document_text.drag_point;
        self.drag_document_selection(point, cx);
        self.autoscroll_document_selection(point, cx)
    }
    pub(super) fn autoscroll_document_selection(
        &mut self,
        point: Point<Pixels>,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.document_text.dragging {
            return false;
        }
        let Some(head) = &self.document_text.head else {
            return false;
        };
        let Some(&i) = self.document_text.index.get(&head.node_id) else {
            return false;
        };
        let Some((list, _, _)) = self.document_text.runs[i].virtual_row else {
            return false;
        };
        let Some(viewport) = self.virtual_lists.get(&list) else {
            return false;
        };
        let bounds = viewport.viewport_bounds();
        let delta = if point.y < bounds.top() + gpui::px(16.) {
            gpui::px(8.)
        } else if point.y > bounds.bottom() - gpui::px(16.) {
            gpui::px(-8.)
        } else {
            return false;
        };
        let old = viewport.scroll_px_offset_for_scrollbar();
        viewport.set_offset_from_scrollbar(old + gpui::point(gpui::px(0.), delta));
        if viewport.scroll_px_offset_for_scrollbar() == old {
            return false;
        }
        cx.notify();
        true
    }
    pub(super) fn restore_document_selection_focus(
        &self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.document_text.detached
            && self
                .document_text
                .focus_owner
                .as_ref()
                .is_some_and(|focus| focus.is_focused(window))
            && let Some(focus) = &self.document_text.focus
        {
            window.focus(focus, cx);
        }
    }

    pub(super) fn navigate_document_selection(
        &mut self,
        key: &str,
        extend: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.document_text.detached {
            return false;
        }
        let state = &self.document_text;
        let Some(mut head) = state.head.clone() else {
            return false;
        };
        if !extend && (key == "left" || key == "right") && state.anchor.as_ref() != Some(&head) {
            let anchor = state.anchor.clone().expect("selection owns an anchor");
            let backwards = key == "left";
            if backwards == state.forward {
                head = anchor;
            }
            let request = TextSelectionRequest {
                text_revision: state.revision,
                anchor: head.clone(),
                head,
            };
            if self.set_text_selection(request).is_ok() {
                cx.notify();
            }
            return true;
        }
        let Some(&i) = state.index.get(&head.node_id) else {
            return false;
        };
        let run = &state.runs[i];
        let Ok(current) = byte(&run.text, head.offset) else {
            return false;
        };
        if key == "up" || key == "down" {
            let Some(layout) = self.selectable_text_layouts.get(&head.node_id) else {
                return false;
            };
            let position = layout.text.position_for_utf8(current);
            let direction = if key == "up" { -1. } else { 1. };
            let point = layout.bounds.origin
                + position.point
                + gpui::point(gpui::px(0.1), position.line_height * (direction + 0.5));
            let Some(head) = self.document_text_hit(point, None) else {
                return false;
            };
            let anchor = if extend {
                state.anchor.clone().unwrap_or(head.clone())
            } else {
                head.clone()
            };
            if self
                .set_text_selection(TextSelectionRequest {
                    text_revision: state.revision,
                    anchor,
                    head,
                })
                .is_ok()
            {
                cx.notify();
            }
            return true;
        }
        let boundaries: Vec<_> = run
            .text
            .grapheme_indices(true)
            .map(|(i, _)| i)
            .chain([run.text.len()])
            .collect();
        let next = match key {
            "left" => boundaries.iter().copied().rev().find(|i| *i < current),
            "right" => boundaries.iter().copied().find(|i| *i > current),
            "home" => Some(0),
            "end" => Some(run.text.len()),
            _ => return false,
        };
        if let Some(next) = next {
            head.offset = utf16(&run.text, next);
        } else {
            let index = if key == "left" {
                i.checked_sub(1)
            } else {
                (i + 1 < state.runs.len()).then_some(i + 1)
            };
            let Some(index) = index else { return true };
            let run = &state.runs[index];
            head = TextPosition {
                node_id: run.id,
                offset: if key == "left" {
                    run.text.encode_utf16().count() as u32
                } else {
                    0
                },
            };
        }
        let anchor = if extend {
            state.anchor.clone().unwrap_or(head.clone())
        } else {
            head.clone()
        };
        if self
            .set_text_selection(TextSelectionRequest {
                text_revision: state.revision,
                anchor,
                head,
            })
            .is_ok()
        {
            cx.notify();
        }
        true
    }

    pub(super) fn select_document_all(&mut self, cx: &mut Context<Self>) -> bool {
        let state = &self.document_text;
        let (Some(first), Some(last)) = (state.runs.first(), state.runs.last()) else {
            return false;
        };
        let request = TextSelectionRequest {
            text_revision: state.revision,
            anchor: TextPosition {
                node_id: first.id,
                offset: 0,
            },
            head: TextPosition {
                node_id: last.id,
                offset: last.text.encode_utf16().count() as u32,
            },
        };
        if self.set_text_selection(request).is_err() {
            return false;
        }
        cx.notify();
        true
    }

    pub fn search_text(
        &mut self,
        request: TextSearchRequest,
    ) -> Result<TextSearchSnapshot, String> {
        if request.query.len() > 4096 {
            return Err("search query exceeds 4096 bytes".into());
        }
        let state = &mut self.document_text;
        if let Some(search) = &state.search
            && search.query == request.query
        {
            return Ok(search.clone());
        }
        let mut flat = String::new();
        let mut starts = Vec::new();
        let mut boundaries = Vec::new();
        for run in state.runs.iter().filter(|_| !request.query.is_empty()) {
            if starts.len() == MAX_SPANS {
                return Err("search exceeds 4096 committed paragraphs".into());
            }
            if !starts.is_empty() {
                flat.push('\n');
            }
            starts.push(flat.len());
            if flat.len() + run.text.len() > MAX_TEXT_BYTES {
                return Err("committed search text exceeds 256 KiB".into());
            }
            flat.push_str(&run.text);
            let mut units = 0;
            let mut positions = Vec::new();
            for (index, cluster) in run.text.grapheme_indices(true) {
                positions.push((index, units));
                units += cluster.encode_utf16().count() as u32;
            }
            positions.push((run.text.len(), units));
            boundaries.push(positions);
        }
        let mut matches = Vec::new();
        let mut truncated = false;
        let mut span_count = 0;
        let mut search_ranges: HashMap<u32, Vec<Range<usize>>> = HashMap::new();
        if !request.query.is_empty() {
            for (start, _) in flat.match_indices(&request.query) {
                let end = start + request.query.len();
                let mut spans = Vec::new();
                let first = starts
                    .partition_point(|offset| *offset <= start)
                    .saturating_sub(1);
                let last = starts.partition_point(|offset| *offset < end);
                for i in first..last {
                    let run = &state.runs[i];
                    let from = start.saturating_sub(starts[i]).min(run.text.len());
                    let to = end.saturating_sub(starts[i]).min(run.text.len());
                    let a = boundaries[i].binary_search_by_key(&from, |(offset, _)| *offset);
                    let b = boundaries[i].binary_search_by_key(&to, |(offset, _)| *offset);
                    if from < to
                        && let (Ok(a), Ok(b)) = (a, b)
                    {
                        spans.push(TextSpan {
                            node_id: run.id,
                            start: boundaries[i][a].1,
                            end: boundaries[i][b].1,
                        });
                    } else if from < to {
                        spans.clear();
                        break;
                    }
                }
                if spans.is_empty() {
                    continue;
                }
                if matches.len() == MAX_MATCHES || span_count + spans.len() > MAX_SPANS {
                    truncated = true;
                    break;
                }
                span_count += spans.len();
                for span in &spans {
                    let index = state.index[&span.node_id];
                    let positions = &boundaries[index];
                    let a = positions
                        .binary_search_by_key(&span.start, |(_, units)| *units)
                        .expect("search boundary");
                    let b = positions
                        .binary_search_by_key(&span.end, |(_, units)| *units)
                        .expect("search boundary");
                    search_ranges
                        .entry(span.node_id)
                        .or_default()
                        .push(positions[a].0..positions[b].0);
                }
                matches.push(spans);
            }
        }
        state.search_revision = state.search_revision.wrapping_add(1);
        let search = TextSearchSnapshot {
            text_revision: state.revision,
            search_revision: state.search_revision,
            query: request.query,
            matches,
            active_match: None,
            truncated,
        };
        state.search_ranges = search_ranges;
        state.search = Some(search.clone());
        self.emit_document_selection_change();
        Ok(search)
    }
    pub fn text_search_snapshot(&self) -> Result<TextSearchSnapshot, String> {
        let state = &self.document_text;
        Ok(state.search.clone().unwrap_or(TextSearchSnapshot {
            text_revision: state.revision,
            search_revision: state.search_revision,
            query: String::new(),
            matches: Vec::new(),
            active_match: None,
            truncated: false,
        }))
    }
    pub fn select_text_search_match(
        &mut self,
        request: TextSearchSelection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<TextSelectionSnapshot, String> {
        let state = &self.document_text;
        let search = state.search.as_ref().ok_or("search has been invalidated")?;
        if request.text_revision != state.revision
            || request.search_revision != search.search_revision
        {
            return Err("search revision is obsolete".into());
        }
        let spans = search
            .matches
            .get(request.match_index as usize)
            .ok_or("search match index is out of range")?;
        let first = spans.first().ok_or("empty search match")?;
        let last = spans.last().unwrap();
        let node_id = first.node_id;
        self.set_text_selection(TextSelectionRequest {
            text_revision: state.revision,
            anchor: TextPosition {
                node_id,
                offset: first.start,
            },
            head: TextPosition {
                node_id: last.node_id,
                offset: last.end,
            },
        })?;
        self.document_text.search.as_mut().unwrap().active_match = Some(request.match_index);
        if let Some(&i) = self.document_text.index.get(&node_id)
            && let Some((list, _, index)) = self.document_text.runs[i].virtual_row
            && let Some(state) = self.virtual_lists.get(&list)
        {
            state.scroll_to_reveal_item(index as usize);
        }
        if let Some(focus) = self.focus_handles.get(&node_id) {
            window.focus(focus, cx);
        }
        cx.notify();
        self.text_selection_snapshot()
    }
    pub(super) fn search_ranges(&self, node_id: u32) -> Vec<Range<usize>> {
        self.document_text
            .search_ranges
            .get(&node_id)
            .cloned()
            .unwrap_or_default()
    }
    pub(super) fn emit_document_selection_change(&mut self) {
        let state = &mut self.document_text;
        if state.emit_suppressed {
            return;
        }
        let change = TextSelectionChange {
            text_revision: state.revision,
            selection_revision: state.selection_revision,
            search_revision: state.search_revision,
            selected_paragraphs: state.selected.len() as u32,
            detached: state.detached,
        };
        if state.emitted.as_ref() == Some(&change) {
            return;
        }
        state.emitted = Some(change.clone());
        for (id, _) in &state.observers {
            let Some(node) = self.store.get(*id) else {
                continue;
            };
            let Some(HostProperties::Extension(p)) = &node.host_properties else {
                continue;
            };
            let sink = super::extensions::ExtensionEventSink::new(
                self.extension_event_state.clone(),
                *id,
                node.listener_id,
                p.event_ids.clone(),
            );
            if sink.is_subscribed(1) {
                let fields = vec![crate::protocol::ExtensionField {
                    id: 1,
                    value: crate::protocol::ExtensionValue::Bytes(
                        crate::native::encode_json(&change).expect("bounded revision notification"),
                    ),
                }];
                let _ = sink.emit(1, fields);
            }
        }
    }
}
