use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use futures::channel::mpsc;
use futures::{SinkExt, StreamExt};
use gpui::{
    AnyElement, App, Bounds, Context, Element, ElementId, ElementInputHandler, Entity,
    EntityInputHandler, FocusHandle, InteractiveElement, IntoElement, ParentElement, Point, Render,
    ScrollStrategy, SharedString, StatefulInteractiveElement, Styled, UTF16Selection,
    UniformListScrollHandle, Window, div, px, rgba, uniform_list,
};
use thiserror::Error;

use crate::protocol::{
    COMMAND_BLUR, COMMAND_FOCUS, COMMAND_SCROLL_TO_END, COMMAND_SCROLL_TO_INDEX,
    COMMAND_SET_SELECTION, Command, CommandResult, EVENT_BLUR, EVENT_CHANGE, EVENT_FOCUS,
    EVENT_SELECTION, Easing, Event, HostProperties, Patch, PatchOperation, ProtocolError, Snapshot,
    Style, TextInputEvent, TextInputProperties,
};
use crate::transport::RuntimeAdapter;
use crate::tree::{
    KIND_PRESSABLE, KIND_RAW_TEXT, KIND_TEXT, KIND_TEXT_INPUT, KIND_VIRTUAL_LIST, NodeStore,
    StoredNode, TreeError,
};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Tree(#[from] TreeError),
}

#[derive(Debug, Default)]
struct NativeInputState {
    text: String,
    selection: Range<usize>,
    marked: Option<Range<usize>>,
    edit_seq: u32,
    focused: bool,
}

impl NativeInputState {
    fn replace(&mut self, range: Option<Range<usize>>, text: &str) {
        let range = range.unwrap_or_else(|| self.selection.clone());
        let cursor = replace_utf16(&mut self.text, range, text);
        self.selection = cursor..cursor;
        self.marked = None;
        self.edit_seq = self.edit_seq.wrapping_add(1);
    }

    fn replace_marked(
        &mut self,
        range: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
    ) {
        let range = range.unwrap_or_else(|| self.selection.clone());
        let start = range.start;
        let inserted_end = replace_utf16(&mut self.text, range, text);
        self.marked = Some(start..inserted_end);
        self.selection = selected.unwrap_or(inserted_end..inserted_end);
        self.edit_seq = self.edit_seq.wrapping_add(1);
    }

    fn unmark(&mut self) -> bool {
        if self.marked.take().is_some() {
            self.edit_seq = self.edit_seq.wrapping_add(1);
            true
        } else {
            false
        }
    }

    fn set_selection(&mut self, selection: Range<usize>) {
        self.selection = selection;
    }

    fn apply_controlled(&mut self, input: &TextInputProperties) {
        if input.controlled && self.edit_seq <= input.ack_edit_seq && self.marked.is_none() {
            self.text = input.value.clone();
            self.selection = input.selection_start as usize..input.selection_end as usize;
            self.marked = input
                .marked_start
                .map(|start| start as usize..input.marked_end.unwrap_or(start) as usize);
        }
    }
}

#[derive(Debug, Clone)]
struct AnimationState {
    opacity_from: f32,
    opacity_target: f32,
    opacity_active: bool,
    background_from: Option<u32>,
    background_target: Option<u32>,
    background_active: bool,
    start: Instant,
    delay: Duration,
    duration: Duration,
    easing: Easing,
    generation: u32,
    completion_sent: bool,
}

impl AnimationState {
    fn at_target(style: &Style) -> Self {
        Self {
            opacity_from: style.opacity.unwrap_or(1.0),
            opacity_target: style.opacity.unwrap_or(1.0),
            opacity_active: false,
            background_from: style.background_rgba,
            background_target: style.background_rgba,
            background_active: false,
            start: Instant::now(),
            delay: Duration::ZERO,
            duration: Duration::ZERO,
            easing: Easing::Linear,
            generation: 0,
            completion_sent: true,
        }
    }
    fn retarget(&mut self, style: &Style, now: Instant) {
        let Some(transition) = style.transition.as_ref() else {
            *self = Self::at_target(style);
            return;
        };
        let (sampled_opacity, sampled_background, _) = self.values(now, false);
        let next_opacity = style.opacity.unwrap_or(1.0);
        let next_background = style.background_rgba;
        self.opacity_from = if transition.properties & crate::protocol::TRANSITION_OPACITY != 0 {
            sampled_opacity
        } else {
            next_opacity
        };
        self.opacity_target = next_opacity;
        self.opacity_active = transition.properties & crate::protocol::TRANSITION_OPACITY != 0
            && (self.opacity_from - next_opacity).abs() > f32::EPSILON;
        self.background_from =
            if transition.properties & crate::protocol::TRANSITION_BACKGROUND_COLOR != 0 {
                sampled_background
            } else {
                next_background
            };
        self.background_target = next_background;
        self.background_active =
            transition.properties & crate::protocol::TRANSITION_BACKGROUND_COLOR != 0
                && self.background_from != self.background_target;
        self.start = now;
        self.delay = Duration::from_millis(transition.delay_ms as u64);
        self.duration = Duration::from_millis(transition.duration_ms as u64);
        self.easing = transition.easing;
        self.generation = self.generation.wrapping_add(1);
        self.completion_sent = false;
    }

    fn progress(&self, now: Instant, reduce_motion: bool) -> (f32, bool) {
        if reduce_motion {
            return (1.0, true);
        }
        if self.duration.is_zero() {
            return (1.0, true);
        }
        let elapsed = now.saturating_duration_since(self.start);
        if elapsed < self.delay {
            return (0.0, false);
        }
        let active = elapsed - self.delay;
        let raw = (active.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        (ease(self.easing, raw), raw >= 1.0)
    }

    fn values(&self, now: Instant, reduce_motion: bool) -> (f32, Option<u32>, bool) {
        let (progress, done) = self.progress(now, reduce_motion);
        let opacity = if self.opacity_active {
            self.opacity_from + (self.opacity_target - self.opacity_from) * progress
        } else {
            self.opacity_target
        };
        let background = if self.background_active {
            let from = match (self.background_from, self.background_target) {
                (Some(from), _) => from,
                (None, Some(target)) => transparent_variant(target),
                (None, None) => 0,
            };
            let target = match (self.background_target, self.background_from) {
                (Some(target), _) => target,
                (None, Some(from)) => transparent_variant(from),
                (None, None) => 0,
            };
            Some(interpolate_rgba(from, target, progress))
        } else {
            self.background_target
        };
        (opacity, background, done)
    }

    fn active(&self) -> bool {
        self.opacity_active || self.background_active
    }
}

fn ease(easing: Easing, value: f32) -> f32 {
    match easing {
        Easing::Linear => value,
        Easing::EaseIn => value * value,
        Easing::EaseOut => 1.0 - (1.0 - value) * (1.0 - value),
        Easing::EaseInOut => {
            if value < 0.5 {
                2.0 * value * value
            } else {
                1.0 - (-2.0 * value + 2.0).powi(2) / 2.0
            }
        }
    }
}

fn interpolate_rgba(from: u32, target: u32, progress: f32) -> u32 {
    let mut result = 0u32;
    for shift in [24, 16, 8, 0] {
        let from_channel = ((from >> shift) & 0xff) as f32;
        let target_channel = ((target >> shift) & 0xff) as f32;
        let channel = (from_channel + (target_channel - from_channel) * progress).round() as u32;
        result |= channel.min(0xff) << shift;
    }
    result
}
fn transparent_variant(color: u32) -> u32 {
    color & 0xffffff00
}
fn animation_target_changed(previous: &Style, current: &Style) -> bool {
    previous.opacity != current.opacity || previous.background_rgba != current.background_rgba
}

fn committed_child_index(absolute_index: u32, range_start: u32, range_end: u32) -> Option<u32> {
    if absolute_index >= range_start && absolute_index < range_end {
        Some(absolute_index - range_start)
    } else {
        None
    }
}
/// The sole persistent GPUI entity for a React surface. The tree itself is
/// retained in `NodeStore`; GPUI element values are rebuilt ephemerally in
/// `render` and never become application state.
pub struct ReactRoot {
    store: NodeStore,
    runtime: Arc<dyn RuntimeAdapter>,
    next_sequence: Arc<AtomicU32>,
    input_states: HashMap<u32, NativeInputState>,
    focus_handles: HashMap<u32, FocusHandle>,
    active_input: Option<u32>,
    commands: Vec<Command>,
    virtual_handles: HashMap<u32, UniformListScrollHandle>,
    virtual_item_sizes: HashMap<u32, f32>,
    pending_visible_ranges: Rc<RefCell<HashMap<u32, (u32, u32)>>>,
    reported_visible_ranges: HashMap<u32, (u32, u32)>,
    animation_states: HashMap<u32, AnimationState>,
    animation_styles: HashMap<u32, Option<Style>>,
    frame_styles: HashMap<u32, Style>,
    animation_frame_requested: bool,
}

impl ReactRoot {
    pub fn new(runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self {
            store: NodeStore::empty(),
            runtime,
            next_sequence: Arc::new(AtomicU32::new(1)),
            input_states: HashMap::new(),
            focus_handles: HashMap::new(),
            active_input: None,
            commands: Vec::new(),
            virtual_handles: HashMap::new(),
            virtual_item_sizes: HashMap::new(),
            pending_visible_ranges: Rc::new(RefCell::new(HashMap::new())),
            reported_visible_ranges: HashMap::new(),
            animation_states: HashMap::new(),
            animation_styles: HashMap::new(),
            frame_styles: HashMap::new(),
            animation_frame_requested: false,
        }
    }

    pub fn store(&self) -> &NodeStore {
        &self.store
    }

    /// Decode, validate, atomically commit, and notify exactly once. This
    /// method is intended to run from a GPUI foreground callback.
    pub fn apply_payload(
        &mut self,
        payload: &[u8],
        cx: &mut Context<Self>,
    ) -> Result<(), RenderError> {
        if let Ok(snapshot) = Snapshot::decode(payload) {
            let reset_native_state = snapshot.base_revision == 0
                || snapshot.surface_id != self.store.surface_id()
                || snapshot.epoch != self.store.epoch();
            self.store.apply_snapshot(snapshot)?;
            if reset_native_state {
                self.reset_native_state();
            }
            self.reconcile_input_states(cx, None);
            self.reconcile_virtual_lists_for(None);
            self.reconcile_animation_states(cx, None);
        } else if let Ok(patch) = Patch::decode(payload) {
            let affected: HashSet<u32> = patch
                .operations
                .iter()
                .map(|operation| match operation {
                    PatchOperation::Create(node) => node.id,
                    PatchOperation::Update { id, .. }
                    | PatchOperation::Move { id, .. }
                    | PatchOperation::Delete { id } => *id,
                })
                .collect();
            self.store.apply_patch(patch)?;
            self.reconcile_input_states(cx, Some(&affected));
            self.reconcile_virtual_lists_for(Some(&affected));
            self.reconcile_animation_states(cx, Some(&affected));
        } else {
            self.commands.push(Command::decode(payload)?);
        }
        cx.notify();
        Ok(())
    }
    fn reset_native_state(&mut self) {
        self.input_states.clear();
        self.focus_handles.clear();
        self.active_input = None;
        self.virtual_handles.clear();
        self.virtual_item_sizes.clear();
        self.reported_visible_ranges.clear();
        self.pending_visible_ranges.borrow_mut().clear();
        self.animation_states.clear();
        self.animation_styles.clear();
        self.frame_styles.clear();
    }

    fn reconcile_input_states(&mut self, cx: &mut Context<Self>, affected: Option<&HashSet<u32>>) {
        self.input_states.retain(|id, _| {
            self.store.get(*id).is_some_and(|node| {
                matches!(node.host_properties, Some(HostProperties::TextInput(_)))
            })
        });
        self.focus_handles
            .retain(|id, _| self.input_states.contains_key(id));
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
                .filter(|node| matches!(node.host_properties, Some(HostProperties::TextInput(_))))
                .map(|node| node.id)
                .collect(),
        };
        for id in ids {
            let Some(HostProperties::TextInput(input)) = self
                .store
                .get(id)
                .and_then(|node| node.host_properties.clone())
            else {
                continue;
            };
            let state = self
                .input_states
                .entry(id)
                .or_insert_with(|| NativeInputState {
                    text: input.value.clone(),
                    selection: input.selection_start as usize..input.selection_end as usize,
                    marked: input
                        .marked_start
                        .map(|start| start as usize..input.marked_end.unwrap_or(start) as usize),
                    edit_seq: 0,
                    focused: false,
                });
            self.focus_handles
                .entry(id)
                .or_insert_with(|| cx.focus_handle());
            state.apply_controlled(&input);
        }
    }

    #[cfg(test)]
    fn reconcile_virtual_lists(&mut self) {
        self.reconcile_virtual_lists_for(None);
    }

    fn reconcile_virtual_lists_for(&mut self, affected: Option<&HashSet<u32>>) {
        self.virtual_handles.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
        });
        self.virtual_item_sizes.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
        });
        self.reported_visible_ranges.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
        });
        self.pending_visible_ranges.borrow_mut().retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
        });
        let ids: Vec<u32> = match affected {
            Some(ids) => ids.iter().copied().collect(),
            None => self
                .store
                .iter()
                .filter(|node| node.kind == KIND_VIRTUAL_LIST)
                .map(|node| node.id)
                .collect(),
        };
        for id in ids {
            let Some(node) = self.store.get(id) else {
                continue;
            };
            let Some(HostProperties::VirtualList(list)) = node.host_properties.as_ref() else {
                continue;
            };
            self.virtual_handles
                .entry(id)
                .or_insert_with(UniformListScrollHandle::new);
            if self
                .virtual_item_sizes
                .insert(id, list.estimated_item_size)
                .is_some_and(|previous| previous != list.estimated_item_size)
            {
                if let Some(handle) = self.virtual_handles.get(&id) {
                    handle.0.borrow_mut().last_item_size = None;
                }
            }
        }
    }

    fn prune_animation_states(&mut self) {
        self.animation_states
            .retain(|id, _| self.store.get(*id).is_some());
        self.animation_styles
            .retain(|id, _| self.store.get(*id).is_some());
    }

    fn reconcile_animation_states(
        &mut self,
        cx: &mut Context<Self>,
        affected: Option<&HashSet<u32>>,
    ) {
        self.prune_animation_states();
        let ids: Vec<u32> = match affected {
            Some(ids) => ids.iter().copied().collect(),
            None => self.store.iter().map(|node| node.id).collect(),
        };
        for node_id in ids {
            let current = self.store.get(node_id).and_then(|node| node.style.clone());
            let previous = self
                .animation_styles
                .insert(node_id, current.clone())
                .flatten();
            let Some(style) = current.as_ref() else {
                self.animation_states.remove(&node_id);
                continue;
            };
            if style.transition.is_none() {
                self.animation_states
                    .insert(node_id, AnimationState::at_target(style));
                continue;
            }
            let Some(previous) = previous else {
                self.animation_states
                    .insert(node_id, AnimationState::at_target(style));
                continue;
            };
            if !animation_target_changed(&previous, style) {
                continue;
            }
            self.retarget_animation(node_id, style, cx.reduce_motion());
        }
    }

    fn retarget_animation(&mut self, node_id: u32, style: &Style, reduce_motion: bool) {
        let now = Instant::now();
        let state = self
            .animation_states
            .entry(node_id)
            .or_insert_with(|| AnimationState::at_target(style));
        state.retarget(style, now);
        if reduce_motion || !state.active() {
            self.finish_animation(node_id);
        }
    }

    fn finish_animation(&mut self, node_id: u32) {
        let Some(state) = self.animation_states.get_mut(&node_id) else {
            return;
        };
        if state.completion_sent {
            state.opacity_active = false;
            state.background_active = false;
            return;
        }
        state.opacity_active = false;
        state.background_active = false;
        state.opacity_from = state.opacity_target;
        state.background_from = state.background_target;
        state.completion_sent = true;
        let generation = state.generation;
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.listener_id == 0 {
            return;
        }
        let event = Event::animation_complete(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            generation,
        );
        if let Err(error) = self.runtime.send_event(&event) {
            eprintln!("react-gpui-host: failed to send animation event: {error}");
        }
    }
    fn emit_input_event(&self, node_id: u32, event_type: u32) {
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
        if let Err(error) = self.runtime.send_event(&event) {
            eprintln!("react-gpui-host: failed to send text input event: {error}");
        }
    }

    fn emit_visible_range(&mut self, node_id: u32, start: u32, end: u32) {
        if self.reported_visible_ranges.get(&node_id) == Some(&(start, end)) {
            return;
        }
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.kind != KIND_VIRTUAL_LIST || node.listener_id == 0 {
            return;
        }
        self.reported_visible_ranges.insert(node_id, (start, end));
        let event = Event::visible_range(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            start,
            end,
        );
        if let Err(error) = self.runtime.send_event(&event) {
            eprintln!("react-gpui-host: failed to send VirtualList range: {error}");
        }
    }

    fn set_input_focus(&mut self, node_id: u32, focused: bool) {
        if focused {
            if let Some(previous) = self.active_input.take() {
                if previous != node_id {
                    if let Some(state) = self.input_states.get_mut(&previous) {
                        state.focused = false;
                    }
                    self.emit_input_event(previous, EVENT_BLUR);
                }
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

    /// Start a dedicated blocking frame reader and forward each complete frame
    /// to the GPUI foreground executor. The transport thread never touches App,
    /// Window, Entity, or NodeStore state. The channel is bounded; a stalled
    /// foreground executor applies backpressure to the renderer process.
    pub fn start_commit_reader(entity: Entity<Self>, runtime: Arc<dyn RuntimeAdapter>, cx: &App) {
        let (mut sender, mut receiver) = mpsc::channel::<Vec<u8>>(32);
        thread::Builder::new()
            .name("react-gpui-commit-reader".into())
            .spawn(move || {
                loop {
                    let payload = match runtime.recv_commit() {
                        Ok(Some(payload)) => payload,
                        Ok(None) => break,
                        Err(error) => {
                            eprintln!("react-gpui-host: renderer transport error: {error}");
                            break;
                        }
                    };
                    if futures::executor::block_on(sender.send(payload)).is_err() {
                        break;
                    }
                }
            })
            .expect("failed to start React GPUI commit reader");

        cx.spawn(async move |cx| {
            while let Some(payload) = receiver.next().await {
                let result = entity.update(cx, |root, cx| root.apply_payload(&payload, cx));
                if let Err(error) = result {
                    eprintln!("react-gpui-host: rejected renderer commit: {error}");
                }
            }
        })
        .detach();
    }

    fn prepare_animation_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.frame_styles.clear();
        self.animation_frame_requested = false;
        let active_ids: Vec<u32> = self
            .animation_states
            .iter()
            .filter_map(|(id, state)| state.active().then_some(*id))
            .collect();
        for node_id in active_ids {
            let Some(state) = self.animation_states.get(&node_id).cloned() else {
                continue;
            };
            let Some(node_style) = self.store.get(node_id).and_then(|node| node.style.as_ref())
            else {
                self.animation_states.remove(&node_id);
                continue;
            };
            let (opacity, background, done) = state.values(Instant::now(), cx.reduce_motion());
            let mut frame_style = node_style.clone();
            if state.opacity_active {
                frame_style.opacity = Some(if done { state.opacity_target } else { opacity });
            }
            if state.background_active {
                frame_style.background_rgba = if done {
                    state.background_target
                } else {
                    background
                };
            }
            if done {
                self.finish_animation(node_id);
            } else {
                self.frame_styles.insert(node_id, frame_style);
                if window.is_window_active() && !self.animation_frame_requested {
                    self.animation_frame_requested = true;
                    window.request_animation_frame();
                }
            }
        }
    }

    fn style_for_node<'a>(&'a self, node: &'a StoredNode) -> Option<&'a Style> {
        self.frame_styles.get(&node.id).or(node.style.as_ref())
    }

    fn render_node(
        &self,
        node: &StoredNode,
        entity: &Entity<Self>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let style = self.style_for_node(node);
        if node.kind == KIND_TEXT_INPUT {
            let Some(HostProperties::TextInput(input)) = node.host_properties.as_ref() else {
                return div().id(ElementId::Integer(node.id as u64)).into_any();
            };
            let focus = self
                .focus_handles
                .get(&node.id)
                .cloned()
                .expect("TextInput focus handle is reconciled before render");
            let input_entity = entity.clone();
            let input_focus = focus.clone();
            let input_id = node.id;
            let mut input_element = apply_style(div(), style);
            input_element = input_element.on_children_prepainted(move |bounds, window, app| {
                if input_focus.is_focused(window) {
                    input_entity.update(app, |root, _| root.set_input_focus(input_id, true));
                    if let Some(bounds) = bounds.into_iter().next() {
                        window.handle_input(
                            &input_focus,
                            ElementInputHandler::new(bounds, input_entity.clone()),
                            app,
                        );
                    }
                }
            });
            let display_text = self
                .input_states
                .get(&node.id)
                .map(|state| state.text.clone())
                .unwrap_or_else(|| input.value.clone());
            let input_element = input_element
                .child(SharedString::from(display_text))
                .id(ElementId::Integer(node.id as u64))
                .focusable()
                .track_focus(&focus);
            return apply_accessibility(input_element, node).into_any();
        }
        if node.kind == KIND_VIRTUAL_LIST {
            let Some(HostProperties::VirtualList(list)) = node.host_properties.as_ref() else {
                return div().id(ElementId::Integer(node.id as u64)).into_any();
            };
            let handle = self
                .virtual_handles
                .get(&node.id)
                .cloned()
                .expect("VirtualList scroll handle is reconciled before render");
            let list_id = node.id;
            let item_count = list.item_count as usize;
            let committed_start = list.range_start;
            let committed_end = list.range_end;
            let estimated = list.estimated_item_size;
            let overscan = list.overscan;
            let pending = Rc::clone(&self.pending_visible_ranges);
            let list_entity = entity.clone();
            let mut list_element = uniform_list(
                ElementId::Integer(list_id as u64),
                item_count,
                move |range, window, app| {
                    let start = range.start as u32;
                    let end = range.end as u32;
                    let report_start = start.saturating_sub(overscan);
                    let report_end = end.saturating_add(overscan).min(item_count as u32);
                    pending
                        .borrow_mut()
                        .insert(list_id, (report_start, report_end));
                    let pending_for_frame = Rc::clone(&pending);
                    let entity_for_frame = list_entity.clone();
                    window.on_next_frame(move |_, app| {
                        let range = pending_for_frame.borrow_mut().remove(&list_id);
                        if let Some((start, end)) = range {
                            entity_for_frame.update(app, |root, _| {
                                root.emit_visible_range(list_id, start, end)
                            });
                        }
                    });
                    list_entity.update(app, |root, cx| {
                        range
                            .map(|absolute_index| {
                                let child = committed_child_index(
                                    absolute_index as u32,
                                    committed_start,
                                    committed_end,
                                )
                                .and_then(|offset| root.store.get_child_at(list_id, offset))
                                .cloned();
                                let row_id = ((list_id as u64) << 32) | absolute_index as u64;
                                let mut row = div().id(ElementId::Integer(row_id)).h(px(estimated));
                                if let Some(child) = child {
                                    row = row.child(root.render_node(
                                        &child,
                                        &list_entity,
                                        window,
                                        cx,
                                    ));
                                }
                                row.into_any()
                            })
                            .collect::<Vec<_>>()
                    })
                },
            )
            .track_scroll(&handle);
            list_element = apply_style(list_element, style);
            return list_element.into_any();
        }
        let mut element = div().id(ElementId::Integer(node.id as u64));
        if node.id == 1 {
            element = element.size_full().flex().flex_col();
        }
        element = apply_style(element, style);
        if node.kind == KIND_RAW_TEXT {
            return element
                .child(
                    node.text
                        .as_ref()
                        .map(|text| SharedString::new(Arc::clone(text)))
                        .unwrap_or_default(),
                )
                .into_any();
        }
        if node.kind == KIND_TEXT {
            if let Some(text) = node.text_content.as_ref() {
                element = element.child(SharedString::new(Arc::clone(text)));
            }
        } else {
            element = element.children(
                node.children(&self.store)
                    .map(|child| self.render_node(child, entity, window, cx)),
            );
        }
        element = apply_accessibility(element, node);
        if node.kind == KIND_PRESSABLE && node.listener_id != 0 {
            let runtime = Arc::clone(&self.runtime);
            let sequence = Arc::clone(&self.next_sequence);
            let surface_id = self.store.surface_id();
            let epoch = self.store.epoch();
            let revision = self.store.revision();
            let node_id = node.id;
            let listener_id = node.listener_id;
            element = element.on_click(move |_, _, _| {
                let event = Event::press(
                    surface_id,
                    epoch,
                    revision,
                    sequence.fetch_add(1, Ordering::Relaxed),
                    node_id,
                    listener_id,
                );
                if let Err(error) = runtime.send_event(&event) {
                    eprintln!("react-gpui-host: failed to send press event: {error}");
                }
            });
        }
        element.into_any()
    }

    fn process_commands(&mut self, window: &mut Window, cx: &mut App) {
        let mut refresh = false;
        for command in std::mem::take(&mut self.commands) {
            let mut success = true;
            let mut error = None;
            if command.surface_id != self.store.surface_id() || command.epoch != self.store.epoch()
            {
                success = false;
                error = Some("surface or epoch mismatch".to_string());
            } else if command.after_revision != self.store.revision() {
                success = false;
                error = Some("command revision is stale".to_string());
            } else {
                match self.store.get(command.node_id) {
                    Some(node) if node.kind == KIND_TEXT_INPUT => {
                        let disabled = matches!(node.host_properties.as_ref(), Some(HostProperties::TextInput(input)) if input.disabled);
                        if disabled {
                            success = false;
                            error = Some("TextInput is disabled".to_string());
                        } else {
                            match command.kind {
                                COMMAND_FOCUS => {
                                    if let Some(handle) = self.focus_handles.get(&command.node_id) {
                                        window.focus(handle, cx);
                                        self.set_input_focus(command.node_id, true);
                                    } else {
                                        success = false;
                                        error = Some("TextInput is not mounted".to_string());
                                    }
                                }
                                COMMAND_BLUR => {
                                    window.blur();
                                    self.set_input_focus(command.node_id, false);
                                }
                                COMMAND_SET_SELECTION => {
                                    if let Some((start, end)) = command.payload {
                                        let length = self
                                            .input_states
                                            .get(&command.node_id)
                                            .map(|state| state.text.encode_utf16().count())
                                            .unwrap_or(0);
                                        if start > end || end as usize > length {
                                            success = false;
                                            error = Some(
                                                "selection is outside UTF-16 text range"
                                                    .to_string(),
                                            );
                                        } else if let Some(state) =
                                            self.input_states.get_mut(&command.node_id)
                                        {
                                            state.set_selection(start as usize..end as usize);
                                            self.emit_input_event(command.node_id, EVENT_SELECTION);
                                        }
                                    } else {
                                        success = false;
                                        error = Some("selection payload is required".to_string());
                                    }
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown TextInput command".to_string());
                                }
                            }
                        }
                    }
                    Some(node) if node.kind == KIND_VIRTUAL_LIST => {
                        let list =
                            node.host_properties
                                .as_ref()
                                .and_then(|properties| match properties {
                                    HostProperties::VirtualList(list) => Some(list),
                                    _ => None,
                                });
                        let handle = self.virtual_handles.get(&command.node_id);
                        if list.is_none() {
                            success = false;
                            error = Some("VirtualList properties are missing".to_string());
                        } else if handle.is_none() {
                            success = false;
                            error = Some("VirtualList is not mounted".to_string());
                        } else {
                            let list = list.expect("checked above");
                            let handle = handle.expect("checked above");
                            match command.kind {
                                COMMAND_SCROLL_TO_INDEX => {
                                    if let Some((index, _)) = command.payload {
                                        if index >= list.item_count {
                                            success = false;
                                            error = Some(
                                                "VirtualList index is out of range".to_string(),
                                            );
                                        } else {
                                            handle.scroll_to_item_strict(
                                                index as usize,
                                                ScrollStrategy::Top,
                                            );
                                            refresh = true;
                                        }
                                    } else {
                                        success = false;
                                        error =
                                            Some("scroll index payload is required".to_string());
                                    }
                                }
                                COMMAND_SCROLL_TO_END => {
                                    handle.scroll_to_bottom();
                                    refresh = true;
                                }
                                _ => {
                                    success = false;
                                    error = Some("unknown VirtualList command".to_string());
                                }
                            }
                        }
                    }
                    Some(_) => {
                        success = false;
                        error = Some("node does not support commands".to_string());
                    }
                    None => {
                        success = false;
                        error = Some("host node is missing".to_string());
                    }
                }
            }
            let result = Event::command_result(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                CommandResult {
                    request_id: command.request_id,
                    command: command.kind,
                    node_id: command.node_id,
                    success,
                    error,
                },
            );
            let _ = self.runtime.send_event(&result);
        }
        if refresh {
            window.refresh();
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
        if let Some(state) = self.active_input_state_mut() {
            if state.unmark() {
                cx.notify();
            }
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

fn apply_accessibility<E: StatefulInteractiveElement>(mut element: E, node: &StoredNode) -> E {
    let Some(accessibility) = node.accessibility.as_ref() else {
        return element;
    };
    let role = match accessibility.role {
        1 => gpui::accesskit::Role::GenericContainer,
        2 => gpui::accesskit::Role::Button,
        3 => gpui::accesskit::Role::Label,
        4 => gpui::accesskit::Role::TextInput,
        5 => gpui::accesskit::Role::CheckBox,
        6 => gpui::accesskit::Role::Heading,
        _ => return element,
    };
    if role != gpui::accesskit::Role::GenericContainer {
        element = element.role(role);
    }
    element = element.accessibility_id(Arc::clone(&node.accessibility_id));
    if let Some(label) = &accessibility.label {
        element = element.aria_label(label.clone());
    }
    if let Some(description) = &accessibility.description {
        element = element.aria_description(description.clone());
    }
    if let Some(selected) = accessibility.selected {
        element = element.aria_selected(selected);
    }
    if let Some(checked) = accessibility.checked {
        element = element.aria_toggled(if checked {
            gpui::accesskit::Toggled::True
        } else {
            gpui::accesskit::Toggled::False
        });
    }
    if let Some(value) = &accessibility.value {
        element = element.aria_value(value.clone());
    }
    element
}

impl Render for ReactRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.process_commands(window, cx);
        self.prepare_animation_frame(window, cx);
        let entity = cx.entity();
        self.store
            .root()
            .map(|root| self.render_node(root, &entity, window, cx))
            .unwrap_or_else(|| div().size_full().into_any())
    }
}

fn apply_style<E: Styled>(mut element: E, style: Option<&Style>) -> E {
    let Some(style) = style else { return element };
    if let Some(width) = style.width {
        element = element.w(px(width));
    }
    if let Some(height) = style.height {
        element = element.h(px(height));
    }
    if let Some(direction) = style.flex_direction {
        element = match direction {
            1 => element.flex_row(),
            2 => element.flex_col(),
            _ => element,
        };
    }
    if let Some(grow) = style.flex_grow {
        element.style().flex_grow = Some(grow);
    }
    if let Some(padding) = style.padding {
        element = element.p(px(padding));
    }
    if let Some(gap) = style.gap {
        element = element.gap(px(gap));
    }
    if let Some(background) = style.background_rgba {
        element = element.bg(rgba(background));
    }
    if let Some(color) = style.color_rgba {
        element = element.text_color(rgba(color));
    }
    if let Some(opacity) = style.opacity {
        element = element.opacity(opacity);
    }
    element
}
#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::protocol::{
        EventPayload, Node, Patch, PatchOperation, TRANSITION_BACKGROUND_COLOR, TRANSITION_OPACITY,
        Transition, UPDATE_LISTENER, UPDATE_PROPERTIES, VirtualListProperties,
    };
    use crate::transport::InMemoryAdapter;
    use crate::tree::KIND_VIEW;

    fn controlled(value: &str, ack_edit_seq: u32) -> TextInputProperties {
        TextInputProperties {
            value: value.into(),
            placeholder: None,
            multiline: false,
            disabled: false,
            controlled: true,
            ack_edit_seq,
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
        }
    }
    fn transition(properties: u32) -> Transition {
        Transition {
            duration_ms: 100,
            delay_ms: 0,
            easing: Easing::Linear,
            properties,
        }
    }

    fn animated_style(opacity: Option<f32>, background_rgba: Option<u32>) -> Style {
        Style {
            opacity,
            background_rgba,
            transition: Some(transition(TRANSITION_OPACITY | TRANSITION_BACKGROUND_COLOR)),
            ..Style::default()
        }
    }

    fn root_with_animated_node(runtime: Arc<InMemoryAdapter>, style: Style) -> ReactRoot {
        let mut root = ReactRoot::new(runtime);
        let mut node = Node::new(1, 0, 0, crate::tree::KIND_VIEW);
        node.listener_id = 7;
        node.style = Some(style);
        root.store
            .apply_snapshot(Snapshot::new(7, 3, 0, 1, vec![node]))
            .expect("animated root snapshot");
        root
    }
    fn virtual_list_snapshot(estimated_item_size: f32) -> Snapshot {
        let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
        list.listener_id = 12;
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 100,
            range_start: 0,
            range_end: 4,
            estimated_item_size,
            overscan: 2,
        }));
        Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), list])
    }

    #[test]
    fn ascii_surrogate_marked_and_unmark_transitions_are_ordered() {
        let mut state = NativeInputState {
            text: "A😀".into(),
            selection: 3..3,
            ..Default::default()
        };
        state.replace(Some(1..3), "x");
        assert_eq!(state.text, "Ax");
        assert_eq!(state.selection, 2..2);
        assert_eq!(state.edit_seq, 1);
        state.replace_marked(Some(2..2), "你", Some(3..3));
        assert_eq!(state.text, "Ax你");
        assert_eq!(state.marked, Some(2..3));
        assert_eq!(state.edit_seq, 2);
        assert!(state.unmark());
        assert_eq!(state.marked, None);
        assert_eq!(state.edit_seq, 3);
    }

    #[test]
    fn stale_controlled_values_are_ignored_until_acknowledged() {
        let mut state = NativeInputState {
            text: "native".into(),
            selection: 6..6,
            edit_seq: 4,
            ..Default::default()
        };
        state.apply_controlled(&controlled("stale", 3));
        assert_eq!(state.text, "native");
        state.apply_controlled(&controlled("accepted", 4));
        assert_eq!(state.text, "accepted");
    }

    #[test]
    fn easing_and_rgba_samples_are_deterministic() {
        let now = Instant::now();
        let state = AnimationState {
            opacity_from: 0.0,
            opacity_target: 1.0,
            opacity_active: true,
            background_from: Some(0x000000ff),
            background_target: Some(0xffffffff),
            background_active: true,
            start: now - Duration::from_millis(50),
            delay: Duration::ZERO,
            duration: Duration::from_millis(100),
            easing: Easing::Linear,
            generation: 1,
            completion_sent: false,
        };
        let (opacity, color, done) = state.values(now, false);
        assert!((opacity - 0.5).abs() < 0.02);
        assert_eq!(color, Some(0x808080ff));
        assert!(!done);
    }
    #[test]
    fn delayed_easing_and_reduced_motion_samples_are_stable() {
        let now = Instant::now();
        let mut state = AnimationState {
            opacity_from: 0.0,
            opacity_target: 1.0,
            opacity_active: true,
            background_from: None,
            background_target: Some(0xff0000ff),
            background_active: true,
            start: now - Duration::from_millis(50),
            delay: Duration::from_millis(100),
            duration: Duration::from_millis(100),
            easing: Easing::EaseIn,
            generation: 4,
            completion_sent: false,
        };
        let (opacity, color, done) = state.values(now, false);
        assert_eq!(opacity, 0.0);
        assert_eq!(color, Some(0xff000000));
        assert!(!done);
        state.start = now - Duration::from_millis(150);
        let (opacity, _, done) = state.values(now, false);
        assert!((opacity - 0.25).abs() < 0.02);
        assert!(!done);
        let (opacity, color, done) = state.values(now, true);
        assert_eq!(opacity, 1.0);
        assert_eq!(color, Some(0xff0000ff));
        assert!(done);
    }

    #[test]
    fn retarget_starts_from_the_sampled_midpoint_and_ignores_unrelated_style_changes() {
        let old = animated_style(Some(0.0), Some(0x000000ff));
        let first = animated_style(Some(1.0), Some(0xffffffff));
        let second = Style {
            opacity: Some(0.25),
            background_rgba: Some(0x00ff00ff),
            transition: Some(transition(TRANSITION_OPACITY | TRANSITION_BACKGROUND_COLOR)),
            ..Style::default()
        };
        let now = Instant::now();
        let mut state = AnimationState::at_target(&old);
        state.retarget(&first, now);
        state.start = now - Duration::from_millis(50);
        state.retarget(&second, now);
        assert!((state.opacity_from - 0.5).abs() < 0.03);
        assert!((state.opacity_target - 0.25).abs() < f32::EPSILON);
        assert_eq!(state.generation, 2);
        let unrelated = Style {
            width: Some(12.0),
            ..second.clone()
        };
        assert!(!animation_target_changed(&second, &unrelated));
    }

    #[test]
    fn background_add_and_remove_interpolate_through_transparent() {
        let old = animated_style(None, None);
        let add = animated_style(None, Some(0x112233ff));
        let remove = animated_style(None, None);
        let now = Instant::now();
        let mut state = AnimationState::at_target(&old);
        state.retarget(&add, now);
        state.start = now - Duration::from_millis(50);
        let (_, halfway_add, done) = state.values(now, false);
        assert!(!done);
        assert_eq!(halfway_add, Some(0x11223380));
        state.retarget(&remove, now);
        state.start = now - Duration::from_millis(50);
        let (_, halfway_remove, done) = state.values(now, false);
        assert!(!done);
        assert_eq!(state.background_target, None);
        assert_eq!(halfway_remove, Some(0x11223340));
    }

    #[test]
    fn completion_is_once_reduced_motion_is_immediate_and_deletion_cancels() {
        let runtime = InMemoryAdapter::new();
        let style = animated_style(Some(1.0), Some(0xffffffff));
        let mut root = root_with_animated_node(runtime.clone(), style.clone());
        let mut state = AnimationState::at_target(&style);
        state.opacity_active = true;
        state.opacity_from = 0.0;
        state.opacity_target = 1.0;
        state.completion_sent = false;
        state.generation = 4;
        root.animation_states.insert(1, state);
        root.finish_animation(1);
        root.finish_animation(1);
        let event = runtime
            .take_event()
            .expect("event result")
            .expect("completion");
        assert_eq!(event.event_type, crate::protocol::EVENT_ANIMATION_COMPLETE);
        assert!(matches!(
            runtime.take_event().expect("second event result"),
            None
        ));

        let runtime = InMemoryAdapter::new();
        let mut root = root_with_animated_node(runtime.clone(), animated_style(Some(0.0), None));
        let target = animated_style(Some(1.0), None);
        root.animation_states.insert(
            1,
            AnimationState::at_target(&animated_style(Some(0.0), None)),
        );
        root.retarget_animation(1, &target, true);
        assert!(!root.animation_states.get(&1).expect("state").active());
        assert!(
            runtime
                .take_event()
                .expect("reduced event result")
                .is_some()
        );

        root.animation_states.insert(
            99,
            AnimationState::at_target(&animated_style(Some(0.0), None)),
        );
        root.prune_animation_states();
        assert!(!root.animation_states.contains_key(&99));
    }
    #[test]
    fn virtual_list_handle_persists_invalidates_measurements_and_cleans_up() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime);
        root.store
            .apply_snapshot(virtual_list_snapshot(20.0))
            .expect("VirtualList snapshot");
        root.reconcile_virtual_lists();
        let identity = Rc::as_ptr(&root.virtual_handles.get(&2).expect("initial handle").0);

        root.store
            .apply_patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![
                    PatchOperation::Update {
                        id: 2,
                        mask: UPDATE_LISTENER,
                        style: None,
                        text: None,
                        listener_id: 13,
                        host_properties: None,
                        accessibility: None,
                    },
                    PatchOperation::Create(Node::new(3, 1, 1, KIND_VIEW)),
                    PatchOperation::Move {
                        id: 2,
                        parent_id: 3,
                        index: 0,
                    },
                ],
            ))
            .expect("unrelated parent move");
        root.reconcile_virtual_lists();
        assert_eq!(
            Rc::as_ptr(&root.virtual_handles.get(&2).expect("persistent handle").0),
            identity
        );

        root.virtual_handles
            .get(&2)
            .expect("measured handle")
            .0
            .borrow_mut()
            .last_item_size = Some(gpui::ItemSize {
            item: gpui::size(gpui::px(10.0), gpui::px(20.0)),
            contents: gpui::size(gpui::px(10.0), gpui::px(20.0)),
        });
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                2,
                3,
                vec![PatchOperation::Update {
                    id: 2,
                    mask: UPDATE_PROPERTIES,
                    style: None,
                    text: None,
                    listener_id: 12,
                    host_properties: Some(HostProperties::VirtualList(VirtualListProperties {
                        item_count: 100,
                        range_start: 0,
                        range_end: 4,
                        estimated_item_size: 32.0,
                        overscan: 2,
                    })),
                    accessibility: None,
                }],
            ))
            .expect("estimated size patch");
        root.reconcile_virtual_lists();
        assert!(
            root.virtual_handles
                .get(&2)
                .expect("resized handle")
                .0
                .borrow()
                .last_item_size
                .is_none()
        );

        root.reported_visible_ranges.insert(2, (0, 4));
        root.pending_visible_ranges.borrow_mut().insert(2, (0, 4));
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                3,
                4,
                vec![PatchOperation::Delete { id: 2 }],
            ))
            .expect("VirtualList delete");
        root.reconcile_virtual_lists();
        assert!(!root.virtual_handles.contains_key(&2));
        assert!(!root.virtual_item_sizes.contains_key(&2));
        assert!(!root.reported_visible_ranges.contains_key(&2));
        assert!(!root.pending_visible_ranges.borrow().contains_key(&2));
    }

    #[test]
    fn visible_range_events_are_deduplicated_per_node() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime.clone());
        root.store
            .apply_snapshot(virtual_list_snapshot(20.0))
            .expect("VirtualList snapshot");
        root.emit_visible_range(2, 0, 4);
        root.emit_visible_range(2, 0, 4);
        let first = runtime
            .take_event()
            .expect("first range result")
            .expect("first range event");
        assert_eq!(
            first.payload,
            Some(EventPayload::VisibleRange { start: 0, end: 4 })
        );
        assert!(runtime.take_event().expect("dedupe result").is_none());
        root.emit_visible_range(2, 1, 5);
        let changed = runtime
            .take_event()
            .expect("changed range result")
            .expect("changed range event");
        assert_eq!(
            changed.payload,
            Some(EventPayload::VisibleRange { start: 1, end: 5 })
        );
    }

    #[test]
    fn virtual_list_maps_absolute_indices_only_inside_committed_range() {
        assert_eq!(committed_child_index(500, 500, 502), Some(0));
        assert_eq!(committed_child_index(501, 500, 502), Some(1));
        assert_eq!(committed_child_index(499, 500, 502), None);
        assert_eq!(committed_child_index(502, 500, 502), None);
    }
}
