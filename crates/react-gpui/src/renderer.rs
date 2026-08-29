use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::ops::Range;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(test)]
use gpui::ListOffset;
use gpui::{
    Context, Element, FocusHandle, IntoElement, ListAlignment, ListState, Render, Styled,
    Subscription, Window, WindowAppearance as GpuiWindowAppearance, div, px,
};
use thiserror::Error;

use crate::protocol::{
    Command, CommandResult, CommandValue, Event, HostProperties, Patch, PatchOperation,
    ProtocolError, Snapshot, Style, WindowAppearance,
};
use crate::transport::{RuntimeAdapter, send_event_or_exit};
use crate::tree::{
    KIND_PRESSABLE, KIND_TEXT, KIND_TEXT_INPUT, KIND_VIEW, KIND_VIRTUAL_LIST, NodeStore,
    StoredNode, TreeError,
};

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
use input::{NativeInputState, TextInputLayout};
use paint::RenderedBounds;
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

fn virtual_list_ancestor(store: &NodeStore, mut node_id: u32) -> Option<u32> {
    loop {
        let node = store.get(node_id)?;
        if node.kind == KIND_VIRTUAL_LIST {
            return Some(node.id);
        }
        if node.parent_id == 0 {
            return None;
        }
        node_id = node.parent_id;
    }
}
fn protocol_window_appearance(appearance: GpuiWindowAppearance) -> WindowAppearance {
    match appearance {
        GpuiWindowAppearance::Light | GpuiWindowAppearance::VibrantLight => WindowAppearance::Light,
        GpuiWindowAppearance::Dark | GpuiWindowAppearance::VibrantDark => WindowAppearance::Dark,
    }
}

/// `render` and never become application state.
pub struct ReactRoot {
    store: NodeStore,
    runtime: Arc<dyn RuntimeAdapter>,
    next_sequence: Arc<AtomicU32>,
    input_states: HashMap<u32, NativeInputState>,
    text_input_layouts: HashMap<u32, TextInputLayout>,
    selectable_text_layouts: HashMap<u32, TextInputLayout>,
    selectable_text_selections: HashMap<u32, Range<usize>>,
    focus_handles: HashMap<u32, FocusHandle>,
    focused_node: Option<(u32, u32)>,
    active_input: Option<u32>,
    text_input_drag_anchor: Option<(u32, usize)>,
    selectable_text_drag_anchor: Option<(u32, usize)>,
    commands: Vec<Command>,
    virtual_lists: HashMap<u32, ListState>,
    virtual_ranges: HashMap<u32, (u32, u32)>,
    virtual_item_sizes: HashMap<u32, f32>,
    pending_visible_ranges: Rc<RefCell<HashMap<u32, (u32, u32)>>>,
    reported_visible_ranges: HashMap<u32, (u32, u32)>,
    active_drag_type: Rc<RefCell<Option<String>>>,
    rendered_bounds: RenderedBounds,
    reported_layout_bounds: HashMap<u32, (f32, f32, f32, f32)>,
    animation_states: HashMap<u32, AnimationState>,
    animation_styles: HashMap<u32, Option<Style>>,
    frame_styles: HashMap<u32, Style>,
    animation_frame_requested: bool,
    focus_observers: HashMap<u32, (Subscription, Subscription)>,
    focus_lost_observer: Option<Subscription>,
    window_observers: Option<(Subscription, Subscription, Subscription)>,
    window_observation_scheduled: bool,
    last_window_size: Option<(f32, f32)>,
    last_window_scale_factor: Option<f32>,
    last_window_active: Option<bool>,
    last_window_appearance: Option<WindowAppearance>,
}

impl ReactRoot {
    pub fn new(runtime: Arc<dyn RuntimeAdapter>) -> Self {
        Self {
            store: NodeStore::empty(),
            runtime,
            input_states: HashMap::new(),
            text_input_layouts: HashMap::new(),
            selectable_text_layouts: HashMap::new(),
            selectable_text_selections: HashMap::new(),
            focus_handles: HashMap::new(),
            focused_node: None,
            active_input: None,
            text_input_drag_anchor: None,
            selectable_text_drag_anchor: None,
            next_sequence: Arc::new(AtomicU32::new(1)),
            virtual_lists: HashMap::new(),
            virtual_ranges: HashMap::new(),
            virtual_item_sizes: HashMap::new(),
            pending_visible_ranges: Rc::new(RefCell::new(HashMap::new())),
            reported_visible_ranges: HashMap::new(),
            rendered_bounds: Rc::new(RefCell::new(HashMap::new())),
            active_drag_type: Rc::new(RefCell::new(None)),
            reported_layout_bounds: HashMap::new(),
            animation_states: HashMap::new(),
            commands: Vec::new(),
            animation_styles: HashMap::new(),
            frame_styles: HashMap::new(),
            animation_frame_requested: false,
            focus_observers: HashMap::new(),
            focus_lost_observer: None,
            window_observers: None,
            window_observation_scheduled: false,
            last_window_size: None,
            last_window_scale_factor: None,
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
    /// Notify the renderer that the native window close was vetoed pending a JS decision.
    pub fn emit_close_requested(&self, request_id: u32) {
        let event = Event::close_requested(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            request_id,
        );
        send_event_or_exit(self.runtime.as_ref(), "close requested event", &event);
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
            self.reconcile_selectable_text_states(cx, None);
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
            self.reconcile_selectable_text_states(cx, Some(&affected));
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
        self.selectable_text_layouts.clear();
        self.selectable_text_selections.clear();
        self.selectable_text_drag_anchor = None;
        self.focus_handles.clear();
        self.focus_observers.clear();
        self.focused_node = None;
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
        self.last_window_scale_factor = None;
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
        let affected_virtual_lists: HashSet<u32> = affected
            .into_iter()
            .flat_map(|ids| ids.iter().copied())
            .filter_map(|id| virtual_list_ancestor(&self.store, id))
            .collect();
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
            let previous_count = state.item_count();
            if previous_count != item_count {
                let scroll_top = state.logical_scroll_top();
                let was_at_end = previous_count > 0 && scroll_top.item_ix >= previous_count;
                state.reset_with_uniform_height(item_count, estimated);
                if was_at_end && item_count > 0 {
                    state.scroll_to_end();
                } else {
                    state.scroll_to(scroll_top);
                }
            } else if previous_estimate.is_some_and(|previous| previous != list.estimated_item_size)
            {
                let scroll_top = state.logical_scroll_top();
                state.reset_with_uniform_height(item_count, estimated);
                state.scroll_to(scroll_top);
            }
            let next_range = (list.range_start, list.range_end);
            let previous_range = self.virtual_ranges.insert(id, next_range);
            if (previous_range.is_some_and(|previous| previous != next_range)
                || affected_virtual_lists.contains(&id))
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

    fn ensure_focus_observers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus_lost_observer.is_none() {
            let focus_lost = cx.on_focus_lost(window, |root, window, cx| {
                root.handle_focus_lost(window, cx);
            });
            self.focus_lost_observer = Some(focus_lost);
        }
        let focusable_ids: HashSet<u32> = self
            .store
            .iter()
            .filter(|node| {
                matches!(node.kind, KIND_VIEW | KIND_PRESSABLE | KIND_TEXT)
                    && node.focusable
                    && node.listener_id != 0
            })
            .map(|node| node.id)
            .collect();
        self.focus_observers
            .retain(|node_id, _| focusable_ids.contains(node_id));
        for node_id in focusable_ids {
            if self.focus_observers.contains_key(&node_id) {
                continue;
            }
            let Some(handle) = self.focus_handles.get(&node_id).cloned() else {
                continue;
            };
            let focus = cx.on_focus(&handle, window, move |root, _, _| {
                root.emit_focus_event(node_id, true);
            });
            let blur = cx.on_blur(&handle, window, move |root, _, _| {
                root.emit_focus_event(node_id, false);
            });
            self.focus_observers.insert(node_id, (focus, blur));
        }
    }

    fn node_can_receive_focus(node: &StoredNode) -> bool {
        match node.kind {
            KIND_VIEW | KIND_PRESSABLE | KIND_TEXT => node.focusable && node.listener_id != 0,
            KIND_TEXT_INPUT => matches!(
                node.host_properties.as_ref(),
                Some(HostProperties::TextInput(input)) if !input.disabled
            ),
            _ => false,
        }
    }

    fn first_focus_handle(&self) -> Option<FocusHandle> {
        fn visit(root: &ReactRoot, node: &StoredNode) -> Option<FocusHandle> {
            if ReactRoot::node_can_receive_focus(node)
                && let Some(handle) = root.focus_handles.get(&node.id)
            {
                return Some(handle.clone());
            }
            node.children(&root.store)
                .find_map(|child| visit(root, child))
        }

        self.store.root().and_then(|root| visit(self, root))
    }

    fn handle_focus_lost(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let detached_focus = self
            .focused_node
            .take()
            .filter(|(node_id, _)| self.store.get(*node_id).is_none());
        if let Some((node_id, listener_id)) = detached_focus
            && !self.focus_observers.contains_key(&node_id)
        {
            self.emit_focus_event_for_listener(node_id, listener_id, false);
        } else if detached_focus.is_none() {
            return;
        }

        let restore = window.focus_lost_restore_target(cx).and_then(|target| {
            self.focus_handles
                .values()
                .find(|handle| handle == &&target)
                .cloned()
        });
        if let Some(handle) = restore.or_else(|| self.first_focus_handle()) {
            window.focus(&handle, cx);
        }
    }

    fn emit_focus_event_for_listener(&mut self, node_id: u32, listener_id: u32, focused: bool) {
        let event = Event::focus(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            listener_id,
            focused,
        );
        send_event_or_exit(self.runtime.as_ref(), "focus event", &event);
    }

    fn emit_focus_event(&mut self, node_id: u32, focused: bool) {
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if !matches!(node.kind, KIND_VIEW | KIND_PRESSABLE | KIND_TEXT)
            || !node.focusable
            || node.listener_id == 0
        {
            return;
        }
        let listener_id = node.listener_id;
        if focused {
            self.focused_node = Some((node_id, listener_id));
        } else if self.focused_node == Some((node_id, listener_id)) {
            self.focused_node = None;
        }
        self.emit_focus_event_for_listener(node_id, listener_id, focused);
    }
    fn emit_pointer_down_outside(&mut self, node_id: u32, listener_id: u32, x: f32, y: f32) {
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.kind != crate::tree::KIND_VIEW
            || listener_id == 0
            || node.listener_id != listener_id
            || node.style.as_ref().and_then(|style| style.position) != Some(2)
            || !x.is_finite()
            || !y.is_finite()
        {
            return;
        }
        let event = Event::pointer_down_outside(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            listener_id,
            x,
            y,
        );
        send_event_or_exit(self.runtime.as_ref(), "pointer down outside event", &event);
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
            let scale_factor = window.scale_factor();
            let active = window.is_window_active();
            let appearance = protocol_window_appearance(window.appearance());
            entity.update(app, |root, _| {
                root.emit_window_observation(width, height, scale_factor, active, appearance);
            });
        });
    }

    fn emit_window_observation(
        &mut self,
        width: f32,
        height: f32,
        scale_factor: f32,
        active: bool,
        appearance: WindowAppearance,
    ) {
        self.window_observation_scheduled = false;
        if self.last_window_size != Some((width, height))
            || self.last_window_scale_factor != Some(scale_factor)
        {
            self.last_window_size = Some((width, height));
            self.last_window_scale_factor = Some(scale_factor);
            let event = Event::window_resize_with_scale(
                self.store.surface_id(),
                self.store.epoch(),
                self.store.revision(),
                self.next_sequence.fetch_add(1, Ordering::Relaxed),
                1,
                0,
                width,
                height,
                scale_factor,
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
        self.rendered_bounds.borrow_mut().clear();
        self.ensure_window_observers(window, cx);
        self.ensure_focus_observers(window, cx);
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
mod image_tests {
    use gpui::{
        Context, Image, ImageSource, InteractiveElement, IntoElement, ObjectFit, Render,
        RenderImage, Styled, StyledImage, TestAppContext, Window, div, img, red,
    };
    use image::{Frame, ImageBuffer, Rgba};
    use smallvec::SmallVec;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::time::Duration;

    struct ImageTestView {
        source: ImageSource,
        id: &'static str,
    }

    impl Render for ImageTestView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            img(self.source.clone())
                .id(self.id)
                .size_full()
                .object_fit(ObjectFit::Contain)
                .with_loading(|| {
                    div()
                        .size_full()
                        .bg(red())
                        .debug_selector(|| "image-loading".to_owned())
                        .into_any_element()
                })
        }
    }
    fn test_image() -> Arc<RenderImage> {
        let frame = Frame::new(ImageBuffer::from_pixel(1, 1, Rgba([0, 0, 0, 0])));
        Arc::new(RenderImage::new(SmallVec::from_elem(frame, 1)))
    }

    #[gpui::test]
    fn image_loading_fallback_appears_after_delay_with_stable_id(cx: &mut TestAppContext) {
        let calls = Arc::new(AtomicUsize::new(0));
        let source_calls = calls.clone();
        let ready = Arc::new(AtomicBool::new(false));
        let source_ready = ready.clone();
        let image = test_image();
        let source = ImageSource::Custom(Arc::new(move |_, _| {
            source_calls.fetch_add(1, Ordering::Relaxed);
            if source_ready.load(Ordering::Relaxed) {
                Some(Ok(image.clone()))
            } else {
                None
            }
        }));
        let (_view, visual) = cx.add_window_view(|_, _| ImageTestView {
            source,
            id: "react-gpui-image-loading-test",
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(calls.load(Ordering::Relaxed) > 0);
        visual.executor().advance_clock(Duration::from_millis(201));
        visual.run_until_parked();
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(calls.load(Ordering::Relaxed) > 1);
        ready.store(true, Ordering::Relaxed);
        visual.update(|window, cx| window.draw(cx).clear(cx));
        assert!(calls.load(Ordering::Relaxed) > 2);
    }

    #[gpui::test]
    fn svg_bytes_render_through_img_loader(cx: &mut TestAppContext) {
        let image = Arc::new(Image::from_bytes(
            gpui::ImageFormat::Svg,
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="8" height="4"><rect width="8" height="4" fill="#38bdf8"/></svg>"##.to_vec(),
        ));
        let image_for_view = image.clone();
        let (_view, visual) = cx.add_window_view(|_, _| ImageTestView {
            source: ImageSource::Image(image_for_view),
            id: "react-gpui-svg-test",
        });
        let image_for_decode = image.clone();
        visual.update(|window, cx| {
            let _ = image_for_decode.get_render_image(window, cx);
            window.draw(cx).clear(cx);
        });
        visual.run_until_parked();
        let rendered = visual.update(|window, cx| image.get_render_image(window, cx));
        assert_eq!(rendered.map(|image| image.frame_count()), Some(1));
    }
}

#[cfg(test)]
mod input_tests {
    use super::*;
    use crate::protocol::{
        COMMAND_MESSAGE, COMMAND_SCROLL_TO_END, COMMAND_SCROLL_TO_INDEX, EVENT_COMMAND_RESULT,
        EventPayload, Node, Patch, PatchOperation, TRANSITION_BACKGROUND_COLOR, TRANSITION_HEIGHT,
        TRANSITION_OPACITY, TRANSITION_WIDTH, Transition, UPDATE_LISTENER, UPDATE_PROPERTIES,
        UPDATE_STYLE, UPDATE_TEXT, VirtualListProperties,
    };
    use crate::transport::InMemoryAdapter;
    use crate::tree::{KIND_RAW_TEXT, KIND_TEXT, KIND_VIEW};
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
    fn text_input_snapshot(value: &str, multiline: bool) -> Snapshot {
        let mut input = Node::new(2, 1, 0, crate::tree::KIND_TEXT_INPUT);
        input.listener_id = 1;
        input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
            value: value.into(),
            placeholder: None,
            multiline,
            disabled: false,
            controlled: true,
            ack_edit_seq: 0,
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            max_length: None,
            selection_reversed: false,
        }));
        Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), input])
    }

    fn latest_text_selection(runtime: &InMemoryAdapter) -> (u32, u32, bool) {
        let mut result = None;
        while let Some(event) = runtime.take_event().expect("read input event") {
            if event.event_type == crate::protocol::EVENT_SELECTION
                && let Some(EventPayload::TextInput(input)) = event.payload
            {
                result = Some((input.selection_start, input.selection_end, input.reversed));
            }
        }
        result.expect("selection event")
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
        assert_eq!(
            super::paint::accessibility_role(7),
            Some(gpui::accesskit::Role::Link)
        );
        assert_eq!(super::paint::accessibility_role(8), None);
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
            width_from: Some(100.0),
            width_target: Some(200.0),
            width_active: true,
            height_from: Some(40.0),
            height_target: Some(80.0),
            height_active: true,
            start: now - Duration::from_millis(50),
            delay: Duration::ZERO,
            duration: Duration::from_millis(100),
            easing: Easing::Linear,
            properties: TRANSITION_OPACITY
                | TRANSITION_BACKGROUND_COLOR
                | TRANSITION_WIDTH
                | TRANSITION_HEIGHT,
            generation: 1,
            completion_sent: false,
        };
        let (opacity, color, width, height, done) = state.values(now, false);
        assert!((opacity - 0.5).abs() < 0.02);
        assert_eq!(color, Some(0x808080ff));
        assert!((width.unwrap() - 150.0).abs() < 0.02);
        assert!((height.unwrap() - 60.0).abs() < 0.02);
        assert!(!done);
    }

    #[test]
    fn easing_curves_apply_to_every_supported_property() {
        let now = Instant::now();
        let properties =
            TRANSITION_OPACITY | TRANSITION_BACKGROUND_COLOR | TRANSITION_WIDTH | TRANSITION_HEIGHT;
        for (easing, expected_progress, expected_color, expected_width, expected_height) in [
            (Easing::Linear, 0.5, 0x808080ff, 150.0, 60.0),
            (Easing::EaseIn, 0.25, 0x404040ff, 125.0, 50.0),
            (Easing::EaseOut, 0.75, 0xbfbfbfff, 175.0, 70.0),
            (Easing::EaseInOut, 0.5, 0x808080ff, 150.0, 60.0),
        ] {
            let state = AnimationState {
                opacity_from: 0.0,
                opacity_target: 1.0,
                opacity_active: true,
                background_from: Some(0x000000ff),
                background_target: Some(0xffffffff),
                background_active: true,
                width_from: Some(100.0),
                width_target: Some(200.0),
                width_active: true,
                height_from: Some(40.0),
                height_target: Some(80.0),
                height_active: true,
                start: now - Duration::from_millis(50),
                delay: Duration::ZERO,
                duration: Duration::from_millis(100),
                easing,
                properties,
                generation: 1,
                completion_sent: false,
            };
            let (opacity, color, width, height, done) = state.values(now, false);
            assert!((opacity - expected_progress).abs() < 0.02);
            assert_eq!(color, Some(expected_color));
            assert!((width.unwrap() - expected_width).abs() < 0.02);
            assert!((height.unwrap() - expected_height).abs() < 0.02);
            assert!(!done);
        }
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
            properties: TRANSITION_OPACITY | TRANSITION_BACKGROUND_COLOR,
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

    #[gpui::test]
    fn removed_transition_retargets_supported_properties_from_previous_style(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(160.0), px(80.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let previous = animated_style(Some(0.25), Some(0x112233ff));
        let mut node = Node::new(1, 0, 0, KIND_VIEW);
        node.listener_id = 7;
        node.style = Some(previous);
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![node]);
        let snapshot_payload = snapshot.encode().expect("encode animation snapshot");
        root.update(cx, |root, cx| root.apply_payload(&snapshot_payload, cx))
            .expect("apply animation snapshot");

        let current = Style::default();
        let patch = Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 1,
                mask: UPDATE_STYLE,
                style: Some(current),
                text: None,
                listener_id: 7,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        );
        let patch_payload = patch.encode().expect("encode animation patch");
        root.update(cx, |root, cx| root.apply_payload(&patch_payload, cx))
            .expect("apply animation patch");
        root.update(cx, |root, _| {
            root.animation_states
                .get_mut(&1)
                .expect("reverse animation state")
                .start = Instant::now() - Duration::from_millis(50);
        });

        let (opacity, background, generation, active) = root.read_with(cx, |root, _| {
            let state = root.animation_states.get(&1).expect("animation state");
            let (opacity, background, _, _, _) = state.values(Instant::now(), false);
            (opacity, background, state.generation, state.active())
        });
        assert!(active);
        assert_eq!(generation, 1);
        assert!((opacity - 0.625).abs() < 0.03);
        let background = background.expect("background sample");
        assert_eq!(background & 0xffffff00, 0x11223300);
        assert!((0x70..=0x90).contains(&(background & 0xff)));
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
        let transition = target.transition.as_ref().expect("target transition");
        root.retarget_animation(1, &target, transition, true);
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
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
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
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
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
    #[test]
    fn virtual_list_state_preserves_scroll_across_count_changes() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime);
        root.store
            .apply_snapshot(virtual_list_snapshot(20.0))
            .expect("VirtualList snapshot");
        root.reconcile_virtual_lists();

        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 40,
                offset_in_item: px(4.0),
            });
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![PatchOperation::Update {
                    id: 2,
                    mask: UPDATE_PROPERTIES,
                    style: None,
                    text: None,
                    listener_id: 12,
                    host_properties: Some(HostProperties::VirtualList(VirtualListProperties {
                        item_count: 200,
                        range_start: 0,
                        range_end: 4,
                        estimated_item_size: 20.0,
                        overscan: 2,
                    })),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                }],
            ))
            .expect("grow VirtualList");
        root.reconcile_virtual_lists();
        let state = root.virtual_lists.get(&2).expect("grown list state");
        assert_eq!(state.logical_scroll_top().item_ix, 40);
        assert_eq!(state.logical_scroll_top().offset_in_item, px(4.0));

        state.scroll_to(ListOffset {
            item_ix: 90,
            offset_in_item: px(3.0),
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
                        item_count: 20,
                        range_start: 0,
                        range_end: 4,
                        estimated_item_size: 20.0,
                        overscan: 2,
                    })),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                }],
            ))
            .expect("shrink VirtualList");
        root.reconcile_virtual_lists();
        let state = root.virtual_lists.get(&2).expect("shrunk list state");
        assert_eq!(state.logical_scroll_top().item_ix, 20);
        assert_eq!(state.logical_scroll_top().offset_in_item, px(0.0));

        root.store
            .apply_patch(Patch::new(
                7,
                3,
                3,
                4,
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
                        estimated_item_size: 20.0,
                        overscan: 2,
                    })),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                }],
            ))
            .expect("restore VirtualList");
        root.reconcile_virtual_lists();
        let state = root.virtual_lists.get(&2).expect("restored list state");
        assert_eq!(state.logical_scroll_top().item_ix, 100);
        assert_eq!(state.logical_scroll_top().offset_in_item, px(0.0));
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
        assert_eq!(
            list_state
                .bounds_for_item(0)
                .map(|bounds| bounds.size.height),
            Some(px(20.0))
        );
        assert_eq!(
            list_state
                .bounds_for_item(1)
                .map(|bounds| bounds.size.height),
            Some(px(120.0))
        );
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

    #[gpui::test]
    fn virtual_list_estimate_is_replaced_when_an_unmeasured_row_is_committed(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(120.0), px(100.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut first = Node::new(3, 2, 0, KIND_VIEW);
        first.style = Some(Style {
            height: Some(40.0),
            ..Style::default()
        });
        let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
        list.style = Some(Style {
            height: Some(100.0),
            ..Style::default()
        });
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 3,
            range_start: 0,
            range_end: 1,
            estimated_item_size: 10.0,
            overscan: 1,
        }));
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), list, first]);
        let payload = snapshot.encode().expect("encode estimated list snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply estimated list snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw estimated list");
        cx.run_until_parked();

        let list_state = root.read_with(cx, |root, _| {
            root.virtual_lists.get(&2).cloned().expect("list state")
        });
        assert_eq!(
            list_state
                .bounds_for_item(0)
                .map(|bounds| bounds.size.height),
            Some(px(40.0))
        );
        assert_eq!(
            list_state
                .bounds_for_item(1)
                .map(|bounds| bounds.size.height),
            Some(px(10.0))
        );

        let mut committed = Node::new(4, 2, 0, KIND_VIEW);
        committed.style = Some(Style {
            height: Some(70.0),
            ..Style::default()
        });
        let patch = Patch::new(
            7,
            3,
            1,
            2,
            vec![
                PatchOperation::Delete { id: 3 },
                PatchOperation::Create(committed),
                PatchOperation::Update {
                    id: 2,
                    mask: UPDATE_PROPERTIES,
                    style: None,
                    text: None,
                    listener_id: 0,
                    host_properties: Some(HostProperties::VirtualList(VirtualListProperties {
                        item_count: 3,
                        range_start: 1,
                        range_end: 2,
                        estimated_item_size: 10.0,
                        overscan: 1,
                    })),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                },
            ],
        );
        let patch_payload = patch.encode().expect("encode committed row patch");
        root.update(cx, |root, cx| root.apply_payload(&patch_payload, cx))
            .expect("apply committed row patch");
        list_state.scroll_to(ListOffset {
            item_ix: 1,
            offset_in_item: px(0.0),
        });
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw committed row");
        cx.run_until_parked();
        assert_eq!(
            list_state
                .bounds_for_item(1)
                .map(|bounds| bounds.size.height),
            Some(px(70.0))
        );
    }
    #[gpui::test]
    fn virtual_list_boundary_consumes_wheel_before_outer_view(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(120.0), px(100.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut outer = Node::new(1, 0, 0, KIND_VIEW);
        outer.listener_id = 7;
        let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
        list.listener_id = 12;
        list.style = Some(Style {
            height: Some(100.0),
            ..Style::default()
        });
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 100,
            range_start: 0,
            range_end: 4,
            estimated_item_size: 20.0,
            overscan: 2,
        }));
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![outer, list]);
        let payload = snapshot.encode().expect("encode boundary snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply boundary snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw boundary list");
        cx.run_until_parked();
        while runtime
            .take_event()
            .expect("drain initial boundary events")
            .is_some()
        {}

        let list_state = root.read_with(cx, |root, _| {
            root.virtual_lists.get(&2).cloned().expect("list state")
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(px(60.0), px(50.0)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.0), px(-100.0))),
            ..Default::default()
        });
        let offset = list_state.logical_scroll_top();
        assert!(offset.item_ix > 0 || offset.offset_in_item > px(0.0));

        let mut saw_visible_range = false;
        let mut saw_outer_scroll = false;
        while let Some(event) = runtime.take_event().expect("read boundary event") {
            saw_visible_range |= event.event_type == crate::protocol::EVENT_VISIBLE_RANGE;
            saw_outer_scroll |=
                event.event_type == crate::protocol::EVENT_SCROLL && event.node_id == 1;
        }
        assert!(
            saw_visible_range,
            "list should emit a range after consuming wheel"
        );
        assert!(
            !saw_outer_scroll,
            "outer View must not receive the list wheel"
        );
    }
    #[gpui::test]
    fn virtual_list_commands_scroll_estimated_rows_and_reject_end_index(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(120.0), px(100.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut list = Node::new(2, 1, 0, KIND_VIRTUAL_LIST);
        list.style = Some(Style {
            height: Some(100.0),
            ..Style::default()
        });
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 100,
            range_start: 0,
            range_end: 4,
            estimated_item_size: 20.0,
            overscan: 2,
        }));
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), list]);
        let snapshot_payload = snapshot.encode().expect("encode command snapshot");
        root.update(cx, |root, cx| root.apply_payload(&snapshot_payload, cx))
            .expect("apply command snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw command list");
        cx.run_until_parked();
        while runtime
            .take_event()
            .expect("drain initial list events")
            .is_some()
        {}

        let scroll_to_index = Command {
            protocol: crate::protocol::PROTOCOL_VERSION,
            message: COMMAND_MESSAGE,
            surface_id: 7,
            epoch: 3,
            after_revision: 1,
            request_id: 1,
            node_id: 2,
            kind: COMMAND_SCROLL_TO_INDEX,
            payload: Some((99, 0)),
            title: None,
            body: None,
            actions: None,
            menus: None,
            keybindings: None,
            window_options: None,
            scroll_offset: None,
            image: None,
        };
        let command_payload = scroll_to_index.encode().expect("encode index command");
        root.update(cx, |root, cx| root.apply_payload(&command_payload, cx))
            .expect("queue index command");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw index command");
        cx.run_until_parked();
        let index_ack = loop {
            let event = runtime
                .take_event()
                .expect("read index command event")
                .expect("index command acknowledgement");
            if event.event_type == EVENT_COMMAND_RESULT {
                break event;
            }
        };
        match index_ack.payload {
            Some(EventPayload::CommandResult(result)) => {
                assert_eq!(result.request_id, 1);
                assert!(result.success);
                assert_eq!(result.error, None);
            }
            payload => panic!("unexpected index command payload: {payload:?}"),
        }
        let list_state = root.read_with(cx, |root, _| {
            root.virtual_lists.get(&2).cloned().expect("list state")
        });
        assert_eq!(list_state.logical_scroll_top().item_ix, 95);
        assert!(list_state.bounds_for_item(99).is_some());

        let scroll_to_end = Command {
            request_id: 2,
            kind: COMMAND_SCROLL_TO_END,
            payload: None,
            ..scroll_to_index.clone()
        };
        let command_payload = scroll_to_end.encode().expect("encode end command");
        root.update(cx, |root, cx| root.apply_payload(&command_payload, cx))
            .expect("queue end command");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw end command");
        cx.run_until_parked();
        let end_ack = loop {
            let event = runtime
                .take_event()
                .expect("read end command event")
                .expect("end command acknowledgement");
            if event.event_type == EVENT_COMMAND_RESULT {
                break event;
            }
        };
        match end_ack.payload {
            Some(EventPayload::CommandResult(result)) => {
                assert_eq!(result.request_id, 2);
                assert!(result.success);
            }
            payload => panic!("unexpected end command payload: {payload:?}"),
        }
        assert_eq!(list_state.logical_scroll_top().item_ix, 95);
        assert!(list_state.bounds_for_item(99).is_some());

        let invalid_index = Command {
            request_id: 3,
            kind: COMMAND_SCROLL_TO_INDEX,
            payload: Some((100, 0)),
            ..scroll_to_index
        };
        let command_payload = invalid_index.encode().expect("encode invalid command");
        root.update(cx, |root, cx| root.apply_payload(&command_payload, cx))
            .expect("queue invalid command");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw invalid command");
        cx.run_until_parked();
        let invalid_ack = loop {
            let event = runtime
                .take_event()
                .expect("read invalid command event")
                .expect("invalid command acknowledgement");
            if event.event_type == EVENT_COMMAND_RESULT {
                break event;
            }
        };
        match invalid_ack.payload {
            Some(EventPayload::CommandResult(result)) => {
                assert_eq!(result.request_id, 3);
                assert!(!result.success);
                assert_eq!(
                    result.error.as_deref(),
                    Some("VirtualList index is out of range")
                );
            }
            payload => panic!("unexpected invalid command payload: {payload:?}"),
        }
        assert_eq!(list_state.logical_scroll_top().item_ix, 95);
    }
    #[gpui::test]
    fn overflowing_multiline_text_input_keeps_caret_inside_element_bounds(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(300.0), px(160.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let value = "one\ntwo\nthree\nfour\nfive\nsix";
        let mut input = Node::new(2, 1, 0, crate::tree::KIND_TEXT_INPUT);
        input.style = Some(Style {
            width: Some(160.0),
            height: Some(40.0),
            overflow: Some(2),
            flex_shrink: Some(1.0),
            ..Style::default()
        });
        input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
            value: value.into(),
            placeholder: None,
            multiline: true,
            disabled: false,
            controlled: true,
            ack_edit_seq: 0,
            selection_start: value.encode_utf16().count() as u32,
            selection_end: value.encode_utf16().count() as u32,
            marked_start: None,
            marked_end: None,
            max_length: None,
            selection_reversed: false,
        }));
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), input]);
        let payload = snapshot
            .encode()
            .expect("encode overflowing multiline snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply overflowing multiline snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw overflowing multiline text input");
        cx.run_until_parked();

        let (viewport, caret, content_height, scroll_offset) = root.read_with(cx, |root, _| {
            let layout = root
                .text_input_layouts
                .get(&2)
                .expect("overflowing multiline layout");
            let state = root.input_states.get(&2).expect("overflowing input state");
            let position = layout
                .text
                .position_for_utf8(super::input::utf16_byte_index(
                    &state.text,
                    state.selection.end,
                ));
            let content_height = match &layout.text {
                super::input::TextInputTextLayout::Multiline {
                    lines, line_height, ..
                } => lines
                    .iter()
                    .map(|line| line.size(*line_height).height)
                    .sum::<gpui::Pixels>(),
                super::input::TextInputTextLayout::Single { .. } => px(0.0),
            };
            (
                layout.bounds,
                gpui::Bounds::new(
                    layout.bounds.origin + position.point - layout.scroll_offset,
                    gpui::size(px(2.0), position.line_height),
                ),
                content_height,
                layout.scroll_offset,
            )
        });
        assert!(
            content_height > viewport.size.height,
            "test setup must overflow: content height {content_height:?}, viewport {viewport:?}"
        );
        assert!(
            scroll_offset.y > px(0.0),
            "caret-follow offset must advance"
        );
        assert!(
            caret.origin.y >= viewport.origin.y
                && caret.bottom_right().y <= viewport.bottom_right().y,
            "caret {caret:?} is outside viewport {viewport:?}"
        );
    }

    #[gpui::test]
    fn multiline_text_input_uses_wrapped_layout_and_preserves_empty_lines(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(160.0), px(180.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut input = Node::new(2, 1, 0, crate::tree::KIND_TEXT_INPUT);
        input.listener_id = 1;
        input.host_properties = Some(HostProperties::TextInput(TextInputProperties {
            value: "a😀\n\n中\n".into(),
            placeholder: None,
            multiline: true,
            disabled: false,
            controlled: true,
            ack_edit_seq: 0,
            selection_start: 0,
            selection_end: 0,
            marked_start: None,
            marked_end: None,
            max_length: None,
            selection_reversed: false,
        }));
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), input]);
        let payload = snapshot.encode().expect("encode multiline snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply multiline snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw multiline text input");
        cx.run_until_parked();

        let (line_count, line_starts, content) = root.read_with(cx, |root, _| {
            let layout = root.text_input_layouts.get(&2).expect("multiline layout");
            match &layout.text {
                super::input::TextInputTextLayout::Multiline {
                    lines, line_starts, ..
                } => (lines.len(), line_starts.clone(), layout.content.clone()),
                super::input::TextInputTextLayout::Single { .. } => {
                    panic!("expected wrapped layout")
                }
            }
        });
        assert_eq!(line_count, 4);
        assert_eq!(line_starts, vec![0, 6, 7, 11]);
        assert_eq!(content, "a😀\n\n中\n");
    }
    #[gpui::test]
    fn text_input_multi_click_dispatch_selects_words_lines_and_copy(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(300.0), px(120.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let snapshot = text_input_snapshot("one 😀 café next\nsecond line\n", true);
        let payload = snapshot.encode().expect("encode multi-click snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply multi-click snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw multi-click text input");
        cx.run_until_parked();

        let (word_point, first_word_point, next_word_point, second_line_point) =
            root.read_with(cx, |root, _| {
                let layout = root.text_input_layouts.get(&2).expect("text input layout");
                let point_for = |byte| {
                    let position = layout.text.position_for_utf8(byte);
                    layout.bounds.origin
                        + position.point
                        + gpui::point(px(2.0), position.line_height * 0.5)
                };
                let second_line_point = match &layout.text {
                    super::input::TextInputTextLayout::Multiline { line_height, .. } => {
                        layout.bounds.origin + gpui::point(px(2.0), *line_height * 1.5)
                    }
                    super::input::TextInputTextLayout::Single { .. } => {
                        panic!("expected multiline layout")
                    }
                };
                (
                    point_for(10),
                    point_for(1),
                    point_for(16),
                    second_line_point,
                )
            });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_event(gpui::MouseDownEvent {
            position: word_point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 2,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (7, 11, false));
        visual.simulate_mouse_up(word_point, gpui::MouseButton::Left, gpui::Modifiers::none());

        visual.simulate_keystrokes("shift-right");
        assert_eq!(latest_text_selection(&runtime), (7, 12, false));

        visual.simulate_event(gpui::MouseDownEvent {
            position: word_point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 2,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (7, 11, false));
        visual.simulate_mouse_move(
            next_word_point,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
        assert_eq!(latest_text_selection(&runtime), (7, 16, false));
        visual.simulate_mouse_up(
            next_word_point,
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_event(gpui::MouseDownEvent {
            position: word_point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 2,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (7, 11, false));
        visual.simulate_mouse_move(
            first_word_point,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
        assert_eq!(latest_text_selection(&runtime), (0, 11, true));
        visual.simulate_mouse_up(
            first_word_point,
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );

        visual.simulate_event(gpui::MouseDownEvent {
            position: second_line_point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 3,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (17, 28, false));
        visual.simulate_mouse_move(
            first_word_point,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
        assert_eq!(latest_text_selection(&runtime), (0, 28, true));
        visual.simulate_mouse_up(
            first_word_point,
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_keystrokes("cmd-c");
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("one 😀 café next\nsecond line".to_owned())
        );
        visual.simulate_event(gpui::MouseDownEvent {
            position: second_line_point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 3,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (17, 28, false));
        visual.simulate_mouse_up(
            second_line_point,
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_keystrokes("cmd-c");
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("second line".to_owned())
        );
    }

    #[gpui::test]
    fn text_input_triple_click_dispatch_selects_entire_single_line(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(300.0), px(80.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let snapshot = text_input_snapshot("one two", false);
        let payload = snapshot.encode().expect("encode single-line snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply single-line snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw single-line text input");
        cx.run_until_parked();

        let point = root.read_with(cx, |root, _| {
            let layout = root.text_input_layouts.get(&2).expect("text input layout");
            let position = layout.text.position_for_utf8(5);
            layout.bounds.origin + position.point + gpui::point(px(2.0), position.line_height * 0.5)
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_event(gpui::MouseDownEvent {
            position: point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 3,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (0, 7, false));
    }

    #[gpui::test]
    fn text_input_keyboard_editing_dispatches_clipboard_selection_and_word_navigation(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(300.0), px(80.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let snapshot = text_input_snapshot("one 😀 two", false);
        let payload = snapshot.encode().expect("encode keyboard snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply keyboard snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw keyboard text input");
        cx.run_until_parked();

        let point = root.read_with(cx, |root, _| {
            let layout = root.text_input_layouts.get(&2).expect("text input layout");
            let position = layout.text.position_for_utf8(10);
            layout.bounds.origin + position.point + gpui::point(px(2.0), position.line_height * 0.5)
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_event(gpui::MouseDownEvent {
            position: point,
            button: gpui::MouseButton::Left,
            modifiers: gpui::Modifiers::none(),
            click_count: 2,
            first_mouse: false,
        });
        assert_eq!(latest_text_selection(&runtime), (7, 10, false));
        visual.simulate_mouse_up(point, gpui::MouseButton::Left, gpui::Modifiers::none());

        visual.simulate_keystrokes("cmd-c");
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("two".to_owned())
        );
        visual.simulate_keystrokes("cmd-x");
        let (text, selection, reversed) = root.read_with(cx, |root, _| {
            let state = root.input_states.get(&2).expect("text input state");
            (
                state.text.clone(),
                state.selection.clone(),
                state.selection_reversed,
            )
        });
        assert_eq!(text, "one 😀 ");
        assert_eq!(selection, 7..7);
        assert!(!reversed);

        cx.write_to_clipboard(gpui::ClipboardItem::new_string("inserted".into()));
        visual.simulate_keystrokes("cmd-v");
        let (text, selection) = root.read_with(cx, |root, _| {
            let state = root.input_states.get(&2).expect("pasted text input state");
            (state.text.clone(), state.selection.clone())
        });
        assert_eq!(text, "one 😀 inserted");
        assert_eq!(selection, 15..15);

        visual.simulate_keystrokes("cmd-a");
        let selection = root.read_with(cx, |root, _| {
            root.input_states
                .get(&2)
                .expect("selected-all text input state")
                .selection
                .clone()
        });
        assert_eq!(selection, 0..15);
        visual.simulate_keystrokes("end home");
        let selection = root.read_with(cx, |root, _| {
            root.input_states
                .get(&2)
                .expect("home/end text input state")
                .selection
                .clone()
        });
        assert_eq!(selection, 0..0);

        visual.simulate_keystrokes("alt-right");
        let selection = root.read_with(cx, |root, _| {
            root.input_states
                .get(&2)
                .expect("first word movement state")
                .selection
                .clone()
        });
        assert_eq!(selection, 3..3);
        visual.simulate_keystrokes("alt-right alt-left shift-alt-left");
        let (selection, reversed) = root.read_with(cx, |root, _| {
            let state = root.input_states.get(&2).expect("word movement state");
            (state.selection.clone(), state.selection_reversed)
        });
        assert_eq!(selection, 0..7);
        assert!(reversed);
    }

    #[gpui::test]
    fn text_input_undo_redo_real_dispatch_coalesces_and_emits_change(
        cx: &mut gpui::TestAppContext,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(300.0), px(80.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let payload = text_input_snapshot("", false)
            .encode()
            .expect("encode undo snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply undo snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw undo input");
        cx.run_until_parked();
        let point = root.read_with(cx, |root, _| {
            let layout = root.text_input_layouts.get(&2).expect("undo input layout");
            layout.bounds.origin + gpui::point(px(2.0), px(8.0))
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_mouse_down(point, gpui::MouseButton::Left, gpui::Modifiers::none());
        visual.simulate_mouse_up(point, gpui::MouseButton::Left, gpui::Modifiers::none());
        visual.simulate_keystrokes("a b");
        assert_eq!(
            root.read_with(cx, |root, _| root.input_states[&2].text.clone()),
            "ab"
        );
        visual.simulate_keystrokes("cmd-z");
        assert_eq!(
            root.read_with(cx, |root, _| root.input_states[&2].text.clone()),
            ""
        );
        visual.simulate_keystrokes("shift-cmd-z");
        assert_eq!(
            root.read_with(cx, |root, _| root.input_states[&2].text.clone()),
            "ab"
        );
        visual.simulate_keystrokes("cmd-z");
        visual.simulate_keystrokes("c");
        visual.simulate_keystrokes("shift-cmd-z");
        assert_eq!(
            root.read_with(cx, |root, _| root.input_states[&2].text.clone()),
            "c"
        );
        let mut saw_reverted_change = false;
        while let Some(event) = runtime.take_event().expect("read undo event") {
            if event.event_type == crate::protocol::EVENT_CHANGE
                && let Some(EventPayload::TextInput(input)) = event.payload
                && input.text.is_empty()
            {
                saw_reverted_change = true;
            }
        }
        assert!(
            saw_reverted_change,
            "undo must emit the normal change event"
        );
    }

    #[gpui::test]
    fn selectable_text_shapes_and_retains_host_selection_geometry(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(160.0), px(120.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("ReactRoot test window");
        let mut text = Node::new(2, 1, 0, KIND_TEXT);
        text.selectable = true;
        let mut raw = Node::new(3, 2, 0, KIND_RAW_TEXT);
        raw.text = Some("selectable text".into());
        let snapshot = Snapshot::new(7, 3, 0, 1, vec![Node::new(1, 0, 0, KIND_VIEW), text, raw]);
        let payload = snapshot.encode().expect("encode selectable snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply selectable snapshot");
        cx.update_window(window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
        })
        .expect("draw selectable text");
        cx.run_until_parked();

        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_mouse_down(
            gpui::point(px(1.0), px(8.0)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_mouse_move(
            gpui::point(px(110.0), px(8.0)),
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
        visual.simulate_mouse_up(
            gpui::point(px(110.0), px(8.0)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        visual.simulate_keystrokes("cmd-c");

        let (content, selection_rows) = root.read_with(cx, |root, _| {
            let layout = root
                .selectable_text_layouts
                .get(&2)
                .expect("selectable layout");
            (
                layout.content.clone(),
                layout
                    .text
                    .selection_bounds_per_line(0..10, layout.bounds)
                    .len(),
            )
        });
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some("selectable ".to_string()),
        );
        let text_patch = Patch::new(
            7,
            3,
            1,
            2,
            vec![PatchOperation::Update {
                id: 3,
                mask: UPDATE_TEXT,
                style: None,
                text: Some("x".into()),
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        );
        let text_payload = text_patch.encode().expect("encode text update");
        root.update(cx, |root, cx| root.apply_payload(&text_payload, cx))
            .expect("apply text update");
        let clamped = root.read_with(cx, |root, _| {
            root.selectable_text_selections.get(&2).cloned()
        });
        assert_eq!(clamped, Some(0..1));

        let delete_patch = Patch::new(7, 3, 2, 3, vec![PatchOperation::Delete { id: 2 }]);
        let delete_payload = delete_patch.encode().expect("encode selectable delete");
        root.update(cx, |root, cx| root.apply_payload(&delete_payload, cx))
            .expect("apply selectable delete");
        root.read_with(cx, |root, _| {
            assert!(!root.selectable_text_selections.contains_key(&2));
            assert!(!root.selectable_text_layouts.contains_key(&2));
            assert!(!root.focus_handles.contains_key(&2));
        });
        assert_eq!(content, "selectable text");
        assert!(selection_rows > 0);
    }

    #[gpui::test]
    fn selectable_rich_text_runs_share_selection_geometry_and_copy(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(260.0), px(120.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("rich selectable root");
        let mut text = Node::new(2, 1, 0, KIND_TEXT);
        text.selectable = true;
        let mut first = Node::new(3, 2, 0, KIND_RAW_TEXT);
        first.text = Some("first ".into());
        let mut run = Node::new(4, 2, 1, KIND_TEXT);
        run.style = Some(Style {
            color_rgba: Some(0xff0000ff),
            ..Style::default()
        });
        let mut second = Node::new(5, 4, 0, KIND_RAW_TEXT);
        second.text = Some("second".into());
        let snapshot = Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), text, first, run, second],
        );
        let payload = snapshot.encode().expect("encode rich selectable snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply rich selectable snapshot");
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .expect("draw rich selectable text");
        cx.run_until_parked();
        let point = root.read_with(cx, |root, _| {
            let layout = root
                .selectable_text_layouts
                .get(&2)
                .expect("rich selectable layout");
            let start = layout.bounds.origin + gpui::point(px(1.0), px(8.0));
            let end = layout.bounds.origin
                + layout.text.position_for_utf8(12).point
                + gpui::point(px(1.0), px(8.0));
            (start, end, layout.bounds)
        });
        let mut visual = gpui::VisualTestContext::from_window(window.into(), cx);
        visual.simulate_mouse_down(point.0, gpui::MouseButton::Left, gpui::Modifiers::none());
        visual.simulate_mouse_move(
            point.1,
            Some(gpui::MouseButton::Left),
            gpui::Modifiers::none(),
        );
        visual.simulate_mouse_up(point.1, gpui::MouseButton::Left, gpui::Modifiers::none());
        visual.simulate_keystrokes("cmd-c");
        let (content, selection) = root.read_with(cx, |root, _| {
            let layout = root
                .selectable_text_layouts
                .get(&2)
                .expect("rich selectable layout");
            (
                layout.content.clone(),
                root.selectable_text_selections
                    .get(&2)
                    .cloned()
                    .expect("selection"),
            )
        });
        assert_eq!(content, "first second");
        assert!(
            selection.start < 6 && selection.end > 6,
            "selection must cross run boundary: {selection:?}"
        );
        assert_eq!(
            cx.read_from_clipboard().and_then(|item| item.text()),
            Some(content[selection.clone()].to_string())
        );
    }

    #[gpui::test]
    fn interactive_text_run_gets_tab_stop_and_enter_press(cx: &mut gpui::TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(240.0), px(80.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("interactive Text root");
        let paragraph = Node::new(2, 1, 0, KIND_TEXT);
        let mut run = Node::new(3, 2, 0, KIND_TEXT);
        run.listener_id = 9;
        run.focusable = true;
        let mut raw = Node::new(4, 3, 0, KIND_RAW_TEXT);
        raw.text = Some("link".into());
        let snapshot = Snapshot::new(
            7,
            3,
            0,
            1,
            vec![Node::new(1, 0, 0, KIND_VIEW), paragraph, run, raw],
        );
        root.update(cx, |root, cx| {
            root.apply_payload(&snapshot.encode().unwrap(), cx)
        })
        .expect("apply interactive Text snapshot");
        cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .expect("draw interactive Text");
        cx.run_until_parked();
        root.read_with(cx, |root, _| assert!(root.focus_handles.contains_key(&3)));
        cx.update_window(window.into(), |_, window, cx| window.focus_next(cx))
            .expect("focus next");
        cx.run_until_parked();
        cx.update_window(window.into(), |_, window, cx| {
            assert!(root.read_with(cx, |root, _| { root.focus_handles[&3].is_focused(window) }));
            window.dispatch_keystroke(gpui::Keystroke::parse("enter").unwrap(), cx)
        })
        .expect("dispatch Enter");
        let mut event = None;
        while let Some(next) = runtime.take_event().expect("read event") {
            if next.event_type == crate::protocol::EVENT_PRESS {
                event = Some(next);
                break;
            }
        }
        let event = event.expect("Text press event");
        assert_eq!((event.node_id, event.listener_id), (3, 9));
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
    #[gpui::test]
    fn layout_next_frame_callbacks_dedupe_repeated_draws(cx: &mut gpui::TestAppContext) {
        const MEASURED_NODES: u32 = 128;
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(gpui::size(px(800.0), px(600.0)), {
            let runtime = runtime.clone();
            move |_, _| ReactRoot::new(runtime)
        });
        let root = window.root(cx).expect("layout test root");
        let mut nodes = Vec::with_capacity(MEASURED_NODES as usize + 1);
        nodes.push(Node::new(1, 0, 0, KIND_VIEW));
        for id in 2..=MEASURED_NODES + 1 {
            let mut node = Node::new(id, 1, id - 2, KIND_VIEW);
            node.listener_id = id;
            nodes.push(node);
        }
        let payload = Snapshot::new(7, 3, 0, 1, nodes)
            .encode()
            .expect("encode layout storm snapshot");
        root.update(cx, |root, cx| root.apply_payload(&payload, cx))
            .expect("apply layout storm snapshot");

        let started = Instant::now();
        for _ in 0..2 {
            cx.update_window(window.into(), |_, window, cx| {
                window.draw(cx).clear(cx);
            })
            .expect("draw layout storm frame");
        }
        cx.update_window(window.into(), |_, window, cx| {
            window.simulate_next_frame(cx);
        })
        .expect("deliver layout storm callbacks");
        cx.run_until_parked();

        let mut layout_events = 0;
        while let Some(event) = runtime.take_event().expect("read layout storm event") {
            if event.event_type == crate::protocol::EVENT_LAYOUT {
                layout_events += 1;
            }
        }
        let elapsed = started.elapsed();
        eprintln!(
            "perf_event_storm: layout measured_nodes={MEASURED_NODES} draws=2 callbacks={} emitted_events={} elapsed={:.3}ms",
            MEASURED_NODES * 2,
            layout_events,
            elapsed.as_secs_f64() * 1_000.0,
        );
        assert_eq!(layout_events, MEASURED_NODES);
    }

    #[test]
    fn window_observations_emit_initial_values_and_dedupe_changes() {
        let runtime = InMemoryAdapter::new();
        let mut root = ReactRoot::new(runtime.clone());
        root.emit_window_observation(800.0, 600.0, 1.0, true, WindowAppearance::Light);
        root.emit_window_observation(800.0, 600.0, 1.0, true, WindowAppearance::Light);
        let resize = runtime
            .take_event()
            .expect("initial resize result")
            .expect("initial resize event");
        assert_eq!(
            resize.payload,
            Some(EventPayload::WindowResize {
                width: 800.0,
                height: 600.0,
                scale_factor: 1.0,
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
        root.emit_window_observation(801.0, 600.0, 1.0, true, WindowAppearance::Light);
        assert!(matches!(
            runtime
                .take_event()
                .expect("resize change result")
                .expect("resize change event")
                .payload,
            Some(EventPayload::WindowResize {
                width: 801.0,
                height: 600.0,
                scale_factor: 1.0,
            })
        ));
        root.emit_window_observation(801.0, 600.0, 2.0, true, WindowAppearance::Light);
        assert!(matches!(
            runtime
                .take_event()
                .expect("scale-factor change result")
                .expect("scale-factor change event")
                .payload,
            Some(EventPayload::WindowResize {
                width: 801.0,
                height: 600.0,
                scale_factor: 2.0,
            })
        ));
        root.emit_window_observation(801.0, 600.0, 2.0, false, WindowAppearance::Light);
        assert!(matches!(
            runtime
                .take_event()
                .expect("activation change result")
                .expect("activation change event")
                .payload,
            Some(EventPayload::WindowActivation { active: false })
        ));
        root.emit_window_observation(801.0, 600.0, 2.0, false, WindowAppearance::Dark);
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
