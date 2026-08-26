use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(test)]
use gpui::ListOffset;
use gpui::{
    Bounds, Context, Element, FocusHandle, IntoElement, ListAlignment, ListState, Pixels, Render,
    ShapedLine, Styled, Subscription, Window, WindowAppearance as GpuiWindowAppearance, div, px,
};
use thiserror::Error;

use crate::protocol::{
    Command, CommandResult, CommandValue, Event, HostProperties, Patch, PatchOperation,
    ProtocolError, Snapshot, Style, WindowAppearance,
};
use crate::transport::{RuntimeAdapter, send_event_or_exit};
use crate::tree::{KIND_VIRTUAL_LIST, NodeStore, TreeError};

mod animation;
mod commands;
mod commit_reader;
mod events;
mod input;
mod paint;

#[cfg(test)]
use crate::protocol::{Easing, TextInputProperties};
use animation::AnimationState;
#[cfg(test)]
use animation::animation_target_changed;
use input::NativeInputState;
#[cfg(test)]
use std::time::{Duration, Instant};

#[derive(Debug, Error)]
pub enum RenderError {
    #[error(transparent)]
    Protocol(#[from] ProtocolError),
    #[error(transparent)]
    Tree(#[from] TreeError),
}

fn committed_child_index(absolute_index: u32, range_start: u32, range_end: u32) -> Option<u32> {
    if absolute_index >= range_start && absolute_index < range_end {
        Some(absolute_index - range_start)
    } else {
        None
    }
}
fn protocol_window_appearance(appearance: GpuiWindowAppearance) -> WindowAppearance {
    match appearance {
        GpuiWindowAppearance::Light | GpuiWindowAppearance::VibrantLight => WindowAppearance::Light,
        GpuiWindowAppearance::Dark | GpuiWindowAppearance::VibrantDark => WindowAppearance::Dark,
    }
}
pub(super) struct TextInputLayout {
    pub(super) line: ShapedLine,
    pub(super) bounds: Bounds<Pixels>,
    pub(super) content: String,
    pub(super) placeholder: bool,
}

/// `render` and never become application state.
pub struct ReactRoot {
    store: NodeStore,
    runtime: Arc<dyn RuntimeAdapter>,
    next_sequence: Arc<AtomicU32>,
    input_states: HashMap<u32, NativeInputState>,
    text_input_layouts: HashMap<u32, TextInputLayout>,
    focus_handles: HashMap<u32, FocusHandle>,
    active_input: Option<u32>,
    text_input_drag_anchor: Option<(u32, usize)>,
    commands: Vec<Command>,
    virtual_lists: HashMap<u32, ListState>,
    virtual_ranges: HashMap<u32, (u32, u32)>,
    virtual_item_sizes: HashMap<u32, f32>,
    pending_visible_ranges: Rc<RefCell<HashMap<u32, (u32, u32)>>>,
    reported_visible_ranges: HashMap<u32, (u32, u32)>,
    active_drag_type: Rc<RefCell<Option<String>>>,
    reported_layout_bounds: HashMap<u32, (f32, f32, f32, f32)>,
    animation_states: HashMap<u32, AnimationState>,
    animation_styles: HashMap<u32, Option<Style>>,
    frame_styles: HashMap<u32, Style>,
    animation_frame_requested: bool,
    window_observers: Option<(Subscription, Subscription, Subscription)>,
    window_observation_scheduled: bool,
    last_window_size: Option<(f32, f32)>,
    last_window_active: Option<bool>,
    last_window_appearance: Option<WindowAppearance>,
}

impl ReactRoot {
    pub fn new(runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self {
            store: NodeStore::empty(),
            runtime,
            next_sequence: Arc::new(AtomicU32::new(1)),
            input_states: HashMap::new(),
            text_input_layouts: HashMap::new(),
            focus_handles: HashMap::new(),
            active_input: None,
            text_input_drag_anchor: None,
            commands: Vec::new(),
            virtual_lists: HashMap::new(),
            virtual_ranges: HashMap::new(),
            virtual_item_sizes: HashMap::new(),
            pending_visible_ranges: Rc::new(RefCell::new(HashMap::new())),
            reported_visible_ranges: HashMap::new(),
            active_drag_type: Rc::new(RefCell::new(None)),
            reported_layout_bounds: HashMap::new(),
            animation_states: HashMap::new(),
            animation_styles: HashMap::new(),
            frame_styles: HashMap::new(),
            animation_frame_requested: false,
            window_observers: None,
            window_observation_scheduled: false,
            last_window_size: None,
            last_window_active: None,
            last_window_appearance: None,
        }
    }
    pub fn store(&self) -> &NodeStore {
        &self.store
    }

    /// Emit a command acknowledgement using this surface's event sequence.
    ///
    /// Host-owned events must allocate their sequence at the same emission
    /// point as renderer-owned events so the per-surface dispatcher observes
    /// one monotonic sequence space.
    pub fn emit_command_ack(
        &self,
        request_id: u32,
        command: u32,
        node_id: u32,
        success: bool,
        error: Option<String>,
        value: Option<CommandValue>,
    ) {
        let event = Event::command_result(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            CommandResult {
                request_id,
                command,
                node_id,
                success,
                error,
                value,
            },
        );
        send_event_or_exit(
            self.runtime.as_ref(),
            "surface command result event",
            &event,
        );
    }

    /// Notify the renderer that this native surface closed before teardown.
    pub fn emit_surface_closed(&self) {
        let event = Event::surface_closed(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
        );
        send_event_or_exit(self.runtime.as_ref(), "surface closed event", &event);
    }
    /// Emit a host menu action for the owning root surface.
    pub fn emit_action(&self, action: String) {
        let event = Event::action(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            action,
        );
        send_event_or_exit(self.runtime.as_ref(), "menu action event", &event);
    }
    /// Emit a response to a host system notification for this root surface.
    pub fn emit_notification_response(&self, tag: String, action_id: Option<String>) {
        let event = Event::notification_response(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            tag,
            action_id,
        );
        send_event_or_exit(self.runtime.as_ref(), "notification response event", &event);
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
            self.reported_layout_bounds.clear();
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
            self.reported_layout_bounds
                .retain(|id, _| !affected.contains(id));
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
        self.text_input_layouts.clear();
        self.focus_handles.clear();
        self.active_input = None;
        self.text_input_drag_anchor = None;
        self.active_drag_type.borrow_mut().take();
        self.virtual_lists.clear();
        self.virtual_ranges.clear();
        self.virtual_item_sizes.clear();
        self.reported_visible_ranges.clear();
        self.reported_layout_bounds.clear();
        self.pending_visible_ranges.borrow_mut().clear();
        self.animation_states.clear();
        self.animation_styles.clear();
        self.window_observation_scheduled = false;
        self.last_window_size = None;
        self.last_window_active = None;
    }

    #[cfg(test)]
    fn reconcile_virtual_lists(&mut self) {
        self.reconcile_virtual_lists_for(None);
    }

    fn reconcile_virtual_lists_for(&mut self, affected: Option<&HashSet<u32>>) {
        self.virtual_lists.retain(|id, _| {
            self.store
                .get(*id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
        });
        self.virtual_ranges.retain(|id, _| {
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
            let item_count = list.item_count as usize;
            let estimated = px(list.estimated_item_size);
            let previous_estimate = self.virtual_item_sizes.insert(id, list.estimated_item_size);
            let state = self.virtual_lists.entry(id).or_insert_with(|| {
                ListState::new(item_count, ListAlignment::Top, px(2048.0))
                    .with_uniform_item_height(estimated)
            });
            if state.item_count() != item_count {
                state.reset_with_uniform_height(item_count, estimated);
            } else if previous_estimate.is_some_and(|previous| previous != list.estimated_item_size)
            {
                let scroll_top = state.logical_scroll_top();
                state.reset_with_uniform_height(item_count, estimated);
                state.scroll_to(scroll_top);
            }
            let next_range = (list.range_start, list.range_end);
            let previous_range = self.virtual_ranges.insert(id, next_range);
            if previous_range.is_some_and(|previous| previous != next_range)
                && next_range.0 < next_range.1
            {
                state.remeasure_items(next_range.0 as usize..next_range.1 as usize);
            }
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
        send_event_or_exit(
            self.runtime.as_ref(),
            "VirtualList visible range event",
            &event,
        );
    }
    fn emit_layout_bounds(&mut self, node_id: u32, frame: (f32, f32, f32, f32)) {
        if ![frame.0, frame.1, frame.2, frame.3]
            .into_iter()
            .all(f32::is_finite)
            || self.reported_layout_bounds.get(&node_id) == Some(&frame)
        {
            return;
        }
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.listener_id == 0 {
            return;
        }
        self.reported_layout_bounds.insert(node_id, frame);
        let event = Event::layout(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            frame.0,
            frame.1,
            frame.2,
            frame.3,
        );
        send_event_or_exit(self.runtime.as_ref(), "layout event", &event);
    }
    fn ensure_window_observers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.window_observers.is_some() {
            return;
        }
        let resize = cx.observe_window_bounds(window, |root, window, cx| {
            root.schedule_window_observation(window, cx);
        });
        let activation = cx.observe_window_activation(window, |root, window, cx| {
            root.schedule_window_observation(window, cx);
        });
        let appearance = cx.observe_window_appearance(window, |root, window, cx| {
            root.schedule_window_observation(window, cx);
        });
        self.window_observers = Some((resize, activation, appearance));
        self.schedule_window_observation(window, cx);
    }

    fn schedule_window_observation(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.window_observation_scheduled {
            return;
        }
        self.window_observation_scheduled = true;
        let entity = cx.entity();
        window.on_next_frame(move |window, app| {
            let size = window.viewport_size();
            let width = f32::from(size.width);
            let height = f32::from(size.height);
            let active = window.is_window_active();
            let appearance = protocol_window_appearance(window.appearance());
            entity.update(app, |root, _| {
                root.emit_window_observation(width, height, active, appearance);
            });
        });
    }

    fn emit_window_observation(
        &mut self,
        width: f32,
        height: f32,
        active: bool,
        appearance: WindowAppearance,
    ) {
        self.window_observation_scheduled = false;
        if self.last_window_size != Some((width, height)) {
            self.last_window_size = Some((width, height));
            let event = Event::window_resize(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                1,
                0,
                width,
                height,
            );
            send_event_or_exit(self.runtime.as_ref(), "window resize event", &event);
        }
        if self.last_window_active != Some(active) {
            self.last_window_active = Some(active);
            let event = Event::window_activation(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                1,
                0,
                active,
            );
            send_event_or_exit(self.runtime.as_ref(), "window activation event", &event);
        }
        if self.last_window_appearance != Some(appearance) {
            self.last_window_appearance = Some(appearance);
            let event = Event::window_appearance(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                appearance,
            );
            send_event_or_exit(self.runtime.as_ref(), "window appearance event", &event);
        }
    }
}

impl Render for ReactRoot {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_window_observers(window, cx);
        self.process_commands(window, cx);
        self.prepare_animation_frame(window, cx);
        let entity = cx.entity();
        self.store
            .root()
            .map(|root| self.render_node(root, &entity))
            .unwrap_or_else(|| div().size_full().into_any())
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::protocol::{
        EventPayload, Node, Patch, PatchOperation, TRANSITION_BACKGROUND_COLOR, TRANSITION_HEIGHT,
        TRANSITION_OPACITY, TRANSITION_WIDTH, Transition, UPDATE_LISTENER, UPDATE_PROPERTIES,
        VirtualListProperties,
    };
    use crate::transport::InMemoryAdapter;
    use crate::tree::KIND_VIEW;
    use gpui::AppContext as _;

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
            max_length: None,
            selection_reversed: false,
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
    #[test]
    fn accessibility_role_mapping_matches_protocol_roles() {
        assert_eq!(super::paint::accessibility_role(1), None);
        assert_eq!(
            super::paint::accessibility_role(2),
            Some(gpui::accesskit::Role::Button)
        );
        assert_eq!(
            super::paint::accessibility_role(3),
            Some(gpui::accesskit::Role::Label)
        );
        assert_eq!(
            super::paint::accessibility_role(4),
            Some(gpui::accesskit::Role::TextInput)
        );
        assert_eq!(
            super::paint::accessibility_role(5),
            Some(gpui::accesskit::Role::CheckBox)
        );
        assert_eq!(
            super::paint::accessibility_role(6),
            Some(gpui::accesskit::Role::Heading)
        );
        assert_eq!(super::paint::accessibility_role(7), None);
    }
    #[test]
    fn placeholder_display_is_visual_only() {
        assert_eq!(
            super::paint::input_display_text(String::new(), Some("Name")),
            ("Name".into(), true)
        );
        assert_eq!(
            super::paint::input_display_text("actual".into(), Some("Name")),
            ("actual".into(), false)
        );
        assert_eq!(
            super::paint::input_display_text(String::new(), None),
            ("".into(), false)
        );
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
            width_from: None,
            width_target: None,
            width_active: false,
            height_from: None,
            height_target: None,
            height_active: false,
            start: now - Duration::from_millis(50),
            delay: Duration::ZERO,
            duration: Duration::from_millis(100),
            easing: Easing::Linear,
            generation: 1,
            completion_sent: false,
        };
        let (opacity, color, _, _, done) = state.values(now, false);
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
            width_from: None,
            width_target: None,
            width_active: false,
            height_from: None,
            height_target: None,
            height_active: false,
            start: now - Duration::from_millis(50),
            delay: Duration::from_millis(100),
            duration: Duration::from_millis(100),
            easing: Easing::EaseIn,
            generation: 4,
            completion_sent: false,
        };
        let (opacity, color, _, _, done) = state.values(now, false);
        assert_eq!(opacity, 0.0);
        assert_eq!(color, Some(0xff000000));
        assert!(!done);
        state.start = now - Duration::from_millis(150);
        let (opacity, _, _, _, done) = state.values(now, false);
        assert!((opacity - 0.25).abs() < 0.02);
        assert!(!done);
        let (opacity, color, _, _, done) = state.values(now, true);
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
        let (_, halfway_add, _, _, _done) = state.values(now, false);
        assert_eq!(halfway_add, Some(0x11223380));
        state.retarget(&remove, now);
        state.start = now - Duration::from_millis(50);
        let (_, halfway_remove, _, _, done) = state.values(now, false);
        assert!(!done);
        assert_eq!(state.background_target, None);
        assert_eq!(halfway_remove, Some(0x11223340));
    }
    #[test]
    fn width_and_height_interpolate_and_retarget_from_sampled_values() {
        let old = Style {
            width: Some(100.0),
            height: Some(40.0),
            transition: Some(transition(TRANSITION_WIDTH | TRANSITION_HEIGHT)),
            ..Style::default()
        };
        let first = Style {
            width: Some(200.0),
            height: Some(80.0),
            transition: Some(transition(TRANSITION_WIDTH | TRANSITION_HEIGHT)),
            ..Style::default()
        };
        let second = Style {
            width: Some(300.0),
            height: Some(100.0),
            transition: Some(transition(TRANSITION_WIDTH | TRANSITION_HEIGHT)),
            ..Style::default()
        };
        let now = Instant::now();
        let mut state = AnimationState::at_target(&old);
        state.retarget(&first, now);
        state.start = now - Duration::from_millis(50);
        let (_, _, width, height, done) = state.values(now, false);
        assert!(!done);
        assert!((width.unwrap() - 150.0).abs() < 0.02);
        assert!((height.unwrap() - 60.0).abs() < 0.02);
        state.retarget(&second, now);
        assert!((state.width_from.unwrap() - 150.0).abs() < 0.03);
        assert!((state.height_from.unwrap() - 60.0).abs() < 0.03);
        state.start = now - Duration::from_millis(100);
        let (_, _, width, height, done) = state.values(now, false);
        assert!(done);
        assert_eq!(width, Some(300.0));
        assert_eq!(height, Some(100.0));
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
        assert!(runtime.take_event().expect("second event result").is_none());

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
    fn virtual_list_state_persists_remeasures_and_cleans_up() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime);
        root.store
            .apply_snapshot(virtual_list_snapshot(20.0))
            .expect("VirtualList snapshot");
        root.reconcile_virtual_lists();
        let state = root.virtual_lists.get(&2).expect("initial list state");
        assert_eq!(state.item_count(), 100);
        state.scroll_to(ListOffset {
            item_ix: 20,
            offset_in_item: px(4.0),
        });

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
                        focusable: false,
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
        let state = root.virtual_lists.get(&2).expect("persistent list state");
        assert_eq!(state.item_count(), 100);
        assert_eq!(state.logical_scroll_top().item_ix, 20);

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
                    focusable: false,
                }],
            ))
            .expect("estimated size patch");
        root.reconcile_virtual_lists();
        assert_eq!(root.virtual_item_sizes.get(&2), Some(&32.0));
        assert_eq!(
            root.virtual_lists
                .get(&2)
                .expect("resized list state")
                .logical_scroll_top()
                .item_ix,
            20
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
        assert!(!root.virtual_lists.contains_key(&2));
        assert!(!root.virtual_item_sizes.contains_key(&2));
        assert!(!root.virtual_ranges.contains_key(&2));
        assert!(!root.reported_visible_ranges.contains_key(&2));
        assert!(!root.pending_visible_ranges.borrow().contains_key(&2));
    }

    #[gpui::test]
    fn variable_height_list_replaces_estimates_with_measured_rows(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(120.0), px(100.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut first = Node::new(3, 2, 0, KIND_VIEW);
        first.style = Some(Style {
            height: Some(20.0),
            ..Style::default()
        });
        let mut second = Node::new(4, 2, 1, KIND_VIEW);
        second.style = Some(Style {
            height: Some(120.0),
            ..Style::default()
        });
        let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
        list.style = Some(Style {
            height: Some(100.0),
            ..Style::default()
        });
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 2,
            range_start: 0,
            range_end: 2,
            estimated_item_size: 50.0,
            overscan: 1,
        }));
        let snapshot = Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), list, first, second],
        );
        let payload = snapshot.encode().expect("encode variable-height snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply variable-height snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw variable-height list");
        cx.run_until_parked();

        let list_state = root.read_with(cx, |root, _| {
            root.virtual_lists.get(&2).cloned().expect("list state")
        });
        assert_eq!(list_state.item_count(), 2);
        assert_eq!(list_state.is_scrolled_to_end(), Some(false));
        list_state.scroll_to_end();
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw list at end");
        cx.run_until_parked();
        assert_eq!(list_state.is_scrolled_to_end(), Some(true));
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
    fn layout_events_are_deduplicated_per_node_and_preserve_float_bounds() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime.clone());
        let mut node = Node::new(2, 1, 0, KIND_VIEW);
        node.listener_id = 7;
        root.store
            .apply_snapshot(Snapshot::new(
                7,
                3,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW), node],
            ))
            .expect("layout snapshot");
        root.emit_layout_bounds(2, (12.5, -3.25, 100.0, 48.75));
        root.emit_layout_bounds(2, (12.5, -3.25, 100.0, 48.75));
        let first = runtime
            .take_event()
            .expect("first layout result")
            .expect("first layout event");
        assert_eq!(
            first.payload,
            Some(EventPayload::Layout {
                x: 12.5,
                y: -3.25,
                width: 100.0,
                height: 48.75,
            })
        );
        assert!(
            runtime
                .take_event()
                .expect("layout dedupe result")
                .is_none()
        );
        root.emit_layout_bounds(2, (12.5, -3.25, 101.0, 48.75));
        assert!(
            runtime
                .take_event()
                .expect("changed layout result")
                .is_some()
        );
    }

    #[test]
    fn window_observations_emit_initial_values_and_dedupe_changes() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime.clone());
        root.emit_window_observation(800.0, 600.0, true, WindowAppearance::Light);
        root.emit_window_observation(800.0, 600.0, true, WindowAppearance::Light);
        let resize = runtime
            .take_event()
            .expect("initial resize result")
            .expect("initial resize event");
        assert_eq!(
            resize.payload,
            Some(EventPayload::WindowResize {
                width: 800.0,
                height: 600.0
            })
        );
        let activation = runtime
            .take_event()
            .expect("initial activation result")
            .expect("initial activation event");
        assert_eq!(
            activation.payload,
            Some(EventPayload::WindowActivation { active: true })
        );
        let appearance = runtime
            .take_event()
            .expect("initial appearance result")
            .expect("initial appearance event");
        assert_eq!(
            appearance.payload,
            Some(EventPayload::WindowAppearance {
                appearance: WindowAppearance::Light
            })
        );
        assert!(runtime.take_event().expect("dedupe result").is_none());
        root.emit_window_observation(801.0, 600.0, true, WindowAppearance::Light);
        assert!(matches!(
            runtime
                .take_event()
                .expect("resize change result")
                .expect("resize change event")
                .payload,
            Some(EventPayload::WindowResize {
                width: 801.0,
                height: 600.0
            })
        ));
        root.emit_window_observation(801.0, 600.0, false, WindowAppearance::Light);
        assert!(matches!(
            runtime
                .take_event()
                .expect("activation change result")
                .expect("activation change event")
                .payload,
            Some(EventPayload::WindowActivation { active: false })
        ));
        root.emit_window_observation(801.0, 600.0, false, WindowAppearance::Dark);
        assert!(matches!(
            runtime
                .take_event()
                .expect("appearance change result")
                .expect("appearance change event")
                .payload,
            Some(EventPayload::WindowAppearance {
                appearance: WindowAppearance::Dark
            })
        ));
    }

    #[test]
    fn virtual_list_maps_absolute_indices_only_inside_committed_range() {
        assert_eq!(committed_child_index(500, 500, 502), Some(0));
        assert_eq!(committed_child_index(501, 500, 502), Some(1));
        assert_eq!(committed_child_index(499, 500, 502), None);
        assert_eq!(committed_child_index(502, 500, 502), None);
    }
}
