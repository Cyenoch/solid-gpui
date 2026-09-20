use std::collections::{BTreeSet, HashMap, HashSet};
use std::sync::atomic::Ordering;
use std::time::Duration;
use web_time::Instant;

use gpui::{App, Context, Task, Window};

use crate::protocol::{Easing, Event, Style, Transition};
use crate::transport::send_event_or_exit;
use crate::tree::StoredNode;

use super::SolidRoot;

#[derive(Debug, Clone)]
pub(super) struct AnimationState {
    pub(super) opacity_from: f32,
    pub(super) opacity_target: f32,
    pub(super) opacity_active: bool,
    pub(super) background_from: Option<u32>,
    pub(super) background_target: Option<u32>,
    pub(super) background_active: bool,
    pub(super) width_from: Option<f32>,
    pub(super) width_target: Option<f32>,
    pub(super) width_active: bool,
    pub(super) height_from: Option<f32>,
    pub(super) height_target: Option<f32>,
    pub(super) height_active: bool,
    pub(super) start: Instant,
    pub(super) delay: Duration,
    pub(super) duration: Duration,
    pub(super) easing: Easing,
    pub(super) properties: u32,
    pub(super) generation: u32,
    pub(super) completion_sent: bool,
}
impl AnimationState {
    pub(super) fn at_target(style: &Style) -> Self {
        Self {
            opacity_from: style.opacity.unwrap_or(1.0),
            opacity_target: style.opacity.unwrap_or(1.0),
            opacity_active: false,
            background_from: style.background_rgba,
            background_target: style.background_rgba,
            background_active: false,
            width_from: style.width,
            width_target: style.width,
            width_active: false,
            height_from: style.height,
            height_target: style.height,
            height_active: false,
            start: Instant::now(),
            delay: Duration::ZERO,
            duration: Duration::ZERO,
            easing: Easing::Linear,
            properties: style
                .transition
                .as_ref()
                .map_or(0, |transition| transition.properties),
            generation: 0,
            completion_sent: true,
        }
    }

    #[cfg(test)]
    pub(super) fn retarget(&mut self, style: &Style, now: Instant) {
        let Some(transition) = style.transition.as_ref() else {
            *self = Self::at_target(style);
            return;
        };
        self.retarget_with_transition(style, transition, now);
    }

    pub(super) fn retarget_with_transition(
        &mut self,
        style: &Style,
        transition: &Transition,
        now: Instant,
    ) {
        let (sampled_opacity, sampled_background, sampled_width, sampled_height, _) =
            self.values(now, false);
        let next_opacity = style.opacity.unwrap_or(1.0);
        let next_background = style.background_rgba;
        let next_width = style.width;
        let next_height = style.height;
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
        self.width_from = if transition.properties & crate::protocol::TRANSITION_WIDTH != 0
            && sampled_width.is_some()
            && next_width.is_some()
        {
            sampled_width
        } else {
            next_width
        };
        self.width_target = next_width;
        self.width_active = transition.properties & crate::protocol::TRANSITION_WIDTH != 0
            && self
                .width_from
                .zip(next_width)
                .is_some_and(|(from, target)| (from - target).abs() > f32::EPSILON);
        self.height_from = if transition.properties & crate::protocol::TRANSITION_HEIGHT != 0
            && sampled_height.is_some()
            && next_height.is_some()
        {
            sampled_height
        } else {
            next_height
        };
        self.height_target = next_height;
        self.height_active = transition.properties & crate::protocol::TRANSITION_HEIGHT != 0
            && self
                .height_from
                .zip(next_height)
                .is_some_and(|(from, target)| (from - target).abs() > f32::EPSILON);
        self.start = now;
        self.delay = Duration::from_millis(transition.delay_ms as u64);
        self.duration = Duration::from_millis(transition.duration_ms as u64);
        self.easing = transition.easing;
        self.properties = transition.properties;
        self.generation = self.generation.wrapping_add(1);
        self.completion_sent = false;
    }

    pub(super) fn progress(&self, now: Instant, reduce_motion: bool) -> (f32, bool) {
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

    pub(super) fn values(
        &self,
        now: Instant,
        reduce_motion: bool,
    ) -> (f32, Option<u32>, Option<f32>, Option<f32>, bool) {
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
        let width = interpolate_length(
            self.width_from,
            self.width_target,
            self.width_active,
            progress,
        );
        let height = interpolate_length(
            self.height_from,
            self.height_target,
            self.height_active,
            progress,
        );
        (opacity, background, width, height, done)
    }

    pub(super) fn active(&self) -> bool {
        self.opacity_active || self.background_active || self.width_active || self.height_active
    }
}

/// Owns all animation timing state for a [`SolidRoot`].
///
/// Besides the per-node maps, the book keeps two compact registries that bound
/// per-frame work to nodes that actually animate: `animating` lists nodes whose
/// transition is in flight, and `delayed` orders the deadlines of in-flight
/// transitions that have not started yet. A single executor timer wakes the
/// earliest delayed transition, so a transition inside its delay requests no
/// frames at all; the wake is cancelled and rearmed whenever that deadline
/// changes, the node disappears, or the book is reset.
pub(super) struct AnimationBook {
    pub(super) states: HashMap<u32, AnimationState>,
    pub(super) styles: HashMap<u32, Option<Style>>,
    pub(super) frame_styles: HashMap<u32, Style>,
    frame_requested: bool,
    animating: HashSet<u32>,
    delayed: BTreeSet<(Instant, u32)>,
    timer: Option<Task<()>>,
    timer_deadline: Option<Instant>,
    timer_epoch: u64,
}

impl AnimationBook {
    pub(super) fn new() -> Self {
        Self {
            states: HashMap::new(),
            styles: HashMap::new(),
            frame_styles: HashMap::new(),
            frame_requested: false,
            animating: HashSet::new(),
            delayed: BTreeSet::new(),
            timer: None,
            timer_deadline: None,
            timer_epoch: 0,
        }
    }

    /// Drop every animation, including any pending deadline wake.
    pub(super) fn reset(&mut self) {
        self.cancel_timer();
        self.states.clear();
        self.styles.clear();
        self.frame_styles.clear();
        self.animating.clear();
        self.delayed.clear();
    }

    /// Cancel the deadline wake and forget a node that left the tree.
    pub(super) fn remove_node(&mut self, node_id: u32) {
        self.unregister_in_flight(node_id);
        self.states.remove(&node_id);
        self.styles.remove(&node_id);
        self.frame_styles.remove(&node_id);
    }

    /// Forget bookkeeping for nodes that no longer exist in the tree.
    pub(super) fn retain(&mut self, alive: impl Fn(u32) -> bool) {
        self.states.retain(|id, _| alive(*id));
        self.styles.retain(|id, _| alive(*id));
        self.frame_styles.retain(|id, _| alive(*id));
        self.animating.retain(|id| alive(*id));
        self.delayed.retain(|(deadline, id)| {
            alive(*id)
                && self.states.get(id).is_some_and(|state| {
                    state.active()
                        && !state.delay.is_zero()
                        && state.start + state.delay == *deadline
                })
        });
    }

    /// Record `node_id` as in flight under its current state. The caller must
    /// unregister any previous registration first so that `delayed` holds at
    /// most one entry per in-flight state, keyed by its current deadline.
    fn register_in_flight(&mut self, node_id: u32, now: Instant) {
        let Some(state) = self.states.get(&node_id) else {
            return;
        };
        if !state.active() {
            return;
        }
        self.animating.insert(node_id);
        if !state.delay.is_zero() {
            let deadline = state.start + state.delay;
            if deadline > now {
                self.delayed.insert((deadline, node_id));
            }
        }
    }

    fn unregister_in_flight(&mut self, node_id: u32) {
        self.animating.remove(&node_id);
        if let Some(state) = self.states.get(&node_id)
            && state.active()
            && !state.delay.is_zero()
        {
            self.delayed.remove(&(state.start + state.delay, node_id));
        }
    }

    /// Keep exactly one executor timer alive for the earliest pending delay.
    fn ensure_timer(&mut self, cx: &mut Context<SolidRoot>) {
        let Some(&(deadline, _)) = self.delayed.first() else {
            self.cancel_timer();
            return;
        };
        if deadline <= animation_now(cx) {
            // The transition is already due: one notified redraw starts it and
            // prunes the entry, so there is nothing left to wait for.
            self.cancel_timer();
            cx.notify();
            return;
        }
        if self.timer_deadline == Some(deadline) {
            return;
        }
        self.arm_timer(deadline, cx);
    }

    fn arm_timer(&mut self, deadline: Instant, cx: &mut Context<SolidRoot>) {
        self.cancel_timer();
        self.timer_epoch = self.timer_epoch.wrapping_add(1);
        let epoch = self.timer_epoch;
        self.timer_deadline = Some(deadline);
        let executor = cx.background_executor().clone();
        // Create the timer eagerly so its deadline is anchored to the current
        // clock, not to whenever the spawned future is first polled.
        let timer = executor.timer(deadline.saturating_duration_since(animation_now(cx)));
        self.timer = Some(cx.spawn(async move |this, cx| {
            timer.await;
            let _ = this.update(cx, |root, cx| {
                root.animation.on_timer_fired(epoch, deadline, cx)
            });
        }));
    }

    /// Dropping the task cancels the wake; the epoch also invalidates one that
    /// already resumed.
    fn cancel_timer(&mut self) {
        let had_timer = self.timer.take().is_some();
        let had_deadline = self.timer_deadline.take().is_some();
        if had_timer || had_deadline {
            self.timer_epoch = self.timer_epoch.wrapping_add(1);
        }
    }

    fn on_timer_fired(&mut self, epoch: u64, deadline: Instant, cx: &mut Context<SolidRoot>) {
        if self.timer_epoch != epoch || self.timer_deadline != Some(deadline) {
            return; // A retarget, deletion or reset already superseded this wake.
        }
        self.cancel_timer();
        // The due entry is still registered: the notified redraw starts the
        // transition and rearms whatever delay remains.
        self.ensure_timer(cx);
    }

    #[cfg(test)]
    pub(super) fn animating_ids(&self) -> HashSet<u32> {
        self.animating.clone()
    }

    #[cfg(test)]
    pub(super) fn delayed_ids(&self) -> HashSet<u32> {
        self.delayed.iter().map(|(_, id)| *id).collect()
    }
}

/// Animation time follows the executor clock so production timing matches the
/// scheduler that owns the deadline wake and tests can advance it deterministically.
fn animation_now(cx: &App) -> Instant {
    cx.background_executor().now()
}

fn interpolate_length(
    from: Option<f32>,
    target: Option<f32>,
    active: bool,
    progress: f32,
) -> Option<f32> {
    if active {
        from.zip(target)
            .map(|(from, target)| from + (target - from) * progress)
    } else {
        target
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

#[cfg(test)]
pub(super) fn animation_target_changed(previous: &Style, current: &Style) -> bool {
    let properties = current
        .transition
        .as_ref()
        .or(previous.transition.as_ref())
        .map_or(0, |transition| transition.properties);
    animation_target_changed_for_properties(previous, current, properties)
}

fn animation_target_changed_for_properties(
    previous: &Style,
    current: &Style,
    properties: u32,
) -> bool {
    (properties & crate::protocol::TRANSITION_OPACITY != 0 && previous.opacity != current.opacity)
        || (properties & crate::protocol::TRANSITION_BACKGROUND_COLOR != 0
            && previous.background_rgba != current.background_rgba)
        || (properties & crate::protocol::TRANSITION_WIDTH != 0 && previous.width != current.width)
        || (properties & crate::protocol::TRANSITION_HEIGHT != 0
            && previous.height != current.height)
}
impl SolidRoot {
    pub(super) fn prune_animation_states(&mut self) {
        self.animation.retain(|id| self.store.get(id).is_some());
    }

    fn prune_animation_states_for(&mut self, affected: Option<&HashSet<u32>>) {
        if let Some(ids) = affected {
            for id in ids {
                if self.store.get(*id).is_none() {
                    self.animation.remove_node(*id);
                }
            }
        } else {
            self.prune_animation_states();
        }
    }

    pub(super) fn reconcile_animation_states(
        &mut self,
        cx: &mut Context<Self>,
        affected: Option<&HashSet<u32>>,
    ) {
        self.prune_animation_states_for(affected);
        let ids: Vec<u32> = match affected {
            Some(ids) => ids.iter().copied().collect(),
            None => self.store.iter().map(|node| node.id).collect(),
        };
        for node_id in ids {
            let Some(style) = self.store.get(node_id).and_then(|node| node.style.clone()) else {
                self.animation.remove_node(node_id);
                continue;
            };
            let previous = self
                .animation
                .styles
                .insert(node_id, Some(style.clone()))
                .flatten();
            let Some(previous) = previous else {
                self.animation
                    .states
                    .insert(node_id, AnimationState::at_target(&style));
                continue;
            };
            let transition = style.transition.as_ref().or(previous.transition.as_ref());
            let active_properties = self
                .animation
                .states
                .get(&node_id)
                .filter(|state| state.active())
                .map(|state| state.properties);
            let properties = transition
                .map(|transition| transition.properties)
                .or(active_properties)
                .unwrap_or(0);
            if !animation_target_changed_for_properties(&previous, &style, properties) {
                if transition.is_none() && active_properties.is_none() {
                    self.animation
                        .states
                        .insert(node_id, AnimationState::at_target(&style));
                }
                continue;
            }
            if let Some(transition) = transition {
                self.retarget_animation(node_id, &style, transition, cx);
            } else {
                self.retarget_existing_animation(node_id, &style, cx);
            }
        }
    }

    pub(super) fn retarget_animation(
        &mut self,
        node_id: u32,
        style: &Style,
        transition: &Transition,
        cx: &mut Context<Self>,
    ) {
        let now = animation_now(cx);
        self.animation.unregister_in_flight(node_id);
        let state = self
            .animation
            .states
            .entry(node_id)
            .or_insert_with(|| AnimationState::at_target(style));
        state.retarget_with_transition(style, transition, now);
        if cx.reduce_motion() || !state.active() {
            self.finish_animation(node_id);
            return;
        }
        self.animation.register_in_flight(node_id, now);
        self.animation.ensure_timer(cx);
    }

    pub(super) fn retarget_existing_animation(
        &mut self,
        node_id: u32,
        style: &Style,
        cx: &mut Context<Self>,
    ) {
        let Some(state) = self
            .animation
            .states
            .get(&node_id)
            .filter(|state| state.active())
        else {
            self.animation
                .states
                .insert(node_id, AnimationState::at_target(style));
            return;
        };
        let transition = Transition {
            duration_ms: state.duration.as_millis() as u32,
            delay_ms: state.delay.as_millis() as u32,
            easing: state.easing,
            properties: state.properties,
        };
        self.retarget_animation(node_id, style, &transition, cx);
    }

    pub(super) fn finish_animation(&mut self, node_id: u32) {
        let Some(state) = self.animation.states.get_mut(&node_id) else {
            return;
        };
        let was_active = state.active();
        if was_active && !state.delay.is_zero() {
            let deadline = state.start + state.delay;
            self.animation.delayed.remove(&(deadline, node_id));
        }
        state.opacity_active = false;
        state.background_active = false;
        state.width_active = false;
        state.height_active = false;
        if was_active {
            self.animation.animating.remove(&node_id);
        }
        if state.completion_sent {
            return;
        }
        state.opacity_from = state.opacity_target;
        state.background_from = state.background_target;
        state.width_from = state.width_target;
        state.height_from = state.height_target;
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
        send_event_or_exit(self.runtime.as_ref(), "animation complete event", event);
    }

    pub(super) fn prepare_animation_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.animation.frame_styles.clear();
        self.animation.frame_requested = false;
        let now = animation_now(cx);
        let reduce_motion = cx.reduce_motion();
        let animating: Vec<u32> = self.animation.animating.iter().copied().collect();
        let mut frames_pending = false;
        for node_id in animating {
            let Some(state) = self.animation.states.get(&node_id) else {
                self.animation.animating.remove(&node_id);
                continue;
            };
            if !state.active() {
                self.animation.animating.remove(&node_id);
                continue;
            }
            let Some(node_style) = self
                .store
                .get(node_id)
                .and_then(|node| node.style.as_ref())
                .cloned()
            else {
                self.animation.remove_node(node_id);
                continue;
            };
            let (opacity, background, width, height, done) = state.values(now, reduce_motion);
            if done {
                self.finish_animation(node_id);
                continue;
            }
            let mut frame_style = node_style;
            if state.opacity_active {
                frame_style.opacity = Some(opacity);
            }
            if state.background_active {
                frame_style.background_rgba = background;
            }
            if state.width_active {
                frame_style.width = width;
            }
            if state.height_active {
                frame_style.height = height;
            }
            // The sampled style is always what paint consumes: inside the delay
            // it holds the pre-transition look, so the patched target style on
            // the node never leaks into a frame.
            self.animation.frame_styles.insert(node_id, frame_style);
            let deadline = state.start + state.delay;
            if now < deadline {
                // No visual progress yet: only frame requests are gated; the
                // deadline wake will start the transition.
                continue;
            }
            self.animation.delayed.remove(&(deadline, node_id));
            frames_pending = true;
        }
        self.animation
            .delayed
            .retain(|(_, id)| self.animation.animating.contains(id));
        if frames_pending && window.is_window_active() {
            self.animation.frame_requested = true;
            window.request_animation_frame();
        }
        self.animation.ensure_timer(cx);
    }

    pub(super) fn style_for_node<'a>(&'a self, node: &'a StoredNode) -> Option<&'a Style> {
        self.animation
            .frame_styles
            .get(&node.id)
            .or(node.style.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::time::Duration;

    use gpui::{AppContext as _, TestAppContext, px, size};

    use super::SolidRoot;
    use crate::protocol::{
        DecodedMessage, Easing, EventPayload, Node, Patch, PatchOperation, Snapshot, Style,
        TRANSITION_OPACITY, Transition, UPDATE_STYLE,
    };
    use crate::transport::InMemoryAdapter;
    use crate::tree::KIND_VIEW;

    const SURFACE: u32 = 1;
    const EPOCH: u32 = 1;
    const LISTENER: u32 = 7;

    fn window_size() -> gpui::Size<gpui::Pixels> {
        size(px(400.0), px(300.0))
    }

    fn opacity_transition(duration_ms: u32, delay_ms: u32) -> Transition {
        Transition {
            duration_ms,
            delay_ms,
            easing: Easing::Linear,
            properties: TRANSITION_OPACITY,
        }
    }

    fn opacity_style(opacity: f32, transition: Option<Transition>) -> Style {
        Style {
            opacity: Some(opacity),
            transition,
            ..Style::default()
        }
    }

    fn view_node(id: u32, parent_id: u32, index: u32, opacity: f32) -> Node {
        let mut node = Node::new(id, parent_id, index, KIND_VIEW);
        node.listener_id = LISTENER;
        node.style = Some(opacity_style(opacity, None));
        node
    }

    fn style_patch(
        base_revision: u32,
        revision: u32,
        updates: Vec<(u32, Style)>,
    ) -> DecodedMessage {
        DecodedMessage::Patch(Patch::new(
            SURFACE,
            EPOCH,
            base_revision,
            revision,
            updates
                .into_iter()
                .map(|(id, style)| PatchOperation::Update {
                    id,
                    mask: UPDATE_STYLE,
                    style: Some(style),
                    text: None,
                    listener_id: LISTENER,
                    host_properties: None,
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                    observes_layout: false,
                })
                .collect(),
        ))
    }

    fn delete_patch(base_revision: u32, revision: u32, ids: &[u32]) -> DecodedMessage {
        DecodedMessage::Patch(Patch::new(
            SURFACE,
            EPOCH,
            base_revision,
            revision,
            ids.iter()
                .map(|id| PatchOperation::Delete { id: *id })
                .collect(),
        ))
    }

    fn apply(root: &gpui::Entity<SolidRoot>, cx: &mut TestAppContext, message: DecodedMessage) {
        root.update(cx, |root, cx| {
            root.apply_decoded_message(message, cx)
                .expect("message applies");
        });
    }

    fn snapshot(epoch: u32, nodes: Vec<Node>) -> DecodedMessage {
        DecodedMessage::Snapshot(Snapshot::new(SURFACE, epoch, 0, 1, nodes))
    }

    /// Opens an activated window and returns its runtime, window and root.
    fn open_surface(
        cx: &mut TestAppContext,
        activate: bool,
    ) -> (
        Arc<InMemoryAdapter>,
        gpui::WindowHandle<SolidRoot>,
        gpui::Entity<SolidRoot>,
    ) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(window_size(), {
            let runtime = runtime.clone();
            move |_, _| SolidRoot::new(runtime)
        });
        let root = window.root(cx).expect("root entity");
        let _ = window.update(cx, |_, window, cx| {
            cx.set_reduce_motion(false);
            if activate {
                window.activate_window();
            }
        });
        cx.run_until_parked();
        let active = window
            .update(cx, |_, window, _| window.is_window_active())
            .expect("window state");
        assert_eq!(active, activate, "window activation precondition");
        (runtime, window, root)
    }

    /// Draws one frame; returns the next-frame demand it produced plus the
    /// painted per-node opacities the renderer will composite.
    ///
    /// Drawing must happen under `cx.update_window`, whose closure only holds
    /// an `AnyView` handle: `window.update` would keep the root entity
    /// borrowed while `draw` re-enters it.
    ///
    /// Before drawing, pending frame callbacks are drained (bounded): each
    /// drained animation callback notifies and re-requests while a transition
    /// is animating, so without settling one stale callback would leak into
    /// the next measured frame. Silence stages settle to zero; animating
    /// stages keep re-registering and simply exhaust the bound.
    fn draw_frame(
        window: &gpui::WindowHandle<SolidRoot>,
        root: &gpui::Entity<SolidRoot>,
        cx: &mut TestAppContext,
    ) -> (usize, HashMap<u32, Option<f32>>) {
        for _ in 0..4 {
            let pending = cx
                .update_window((*window).into(), |_, window, cx| {
                    window.simulate_next_frame(cx)
                })
                .expect("settle frame callbacks");
            if pending == 0 {
                break;
            }
        }
        let demand = cx
            .update_window((*window).into(), |_, window, cx| {
                window.draw(cx).clear(cx);
                window.simulate_next_frame(cx)
            })
            .expect("draw");
        let painted = root.read_with(cx, |root, _| {
            root.animation
                .frame_styles
                .iter()
                .map(|(id, style)| (*id, style.opacity))
                .collect()
        });
        (demand, painted)
    }

    fn drain_completions(runtime: &InMemoryAdapter) -> Vec<(u32, u32)> {
        let mut completions = Vec::new();
        while let Some(event) = runtime.take_event().expect("read test event") {
            if let EventPayload::AnimationComplete { generation } = event.payload {
                completions.push((event.meta.node_id, generation));
            }
        }
        completions
    }

    #[gpui::test]
    fn delayed_transition_stays_silent_until_its_deadline(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(
                    1,
                    opacity_style(0.25, Some(opacity_transition(10_000, 60_000))),
                )],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);

        // Before the deadline there is no visual progress, so repeated draws
        // must demand no frames and must keep presenting the pre-transition
        // value instead of jumping to the patched target.
        for _ in 0..8 {
            let (demand, painted) = draw_frame(&window, &root, cx);
            assert_eq!(demand, 0, "delayed transition must not demand frames");
            assert_eq!(
                painted.get(&1),
                Some(&Some(1.0)),
                "delayed transition must hold its source style"
            );
        }
        assert!(drain_completions(&runtime).is_empty());

        // At the deadline the wake starts the transition.
        cx.executor()
            .advance_clock(Duration::from_secs(60) + Duration::from_millis(50));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "deadline wake must start frame demand");
        let started = painted.get(&1).copied().flatten().expect("progress paints");
        assert!(started < 1.0, "progress moved toward the target");

        // The transition completes exactly once, then everything goes quiet.
        cx.executor().advance_clock(Duration::from_secs(10));
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0, "finished transition must not demand frames");
        assert!(painted.is_empty());
        assert_eq!(drain_completions(&runtime), vec![(1, 1)]);
        for _ in 0..4 {
            let (demand, painted) = draw_frame(&window, &root, cx);
            assert_eq!(demand, 0);
            assert!(painted.is_empty());
        }
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn retarget_replaces_a_pending_deadline_and_reverses(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        // Delayed 100ms toward 0.25.
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(1, opacity_style(0.25, Some(opacity_transition(100, 100))))],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);

        // Halfway into the delay, reverse toward 0.75 with a 300ms delay.
        cx.executor().advance_clock(Duration::from_millis(50));
        apply(
            &root,
            cx,
            style_patch(
                2,
                3,
                vec![(1, opacity_style(0.75, Some(opacity_transition(100, 300))))],
            ),
        );
        cx.run_until_parked();

        // Past the original deadline nothing has started or demanded frames,
        // and the presented style still holds the pre-retarget value.
        cx.executor().advance_clock(Duration::from_millis(60));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0, "the cancelled deadline must not start progress");
        assert_eq!(
            painted.get(&1),
            Some(&Some(1.0)),
            "the transition keeps holding its pre-retarget value"
        );
        assert!(drain_completions(&runtime).is_empty());

        // The replacement deadline starts the reversed transition.
        cx.executor().advance_clock(Duration::from_millis(250));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "the replacement deadline must start demand");
        let started = painted.get(&1).copied().flatten().expect("reversal paints");
        assert!(
            started > 0.75 && started < 1.0,
            "reversal moves from 1.0 toward 0.75"
        );

        cx.executor().advance_clock(Duration::from_millis(150));
        draw_frame(&window, &root, cx);
        assert_eq!(drain_completions(&runtime), vec![(1, 2)]);

        // No stale wake fires afterwards.
        cx.executor().advance_clock(Duration::from_secs(5));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn retargeting_pulls_a_deadline_earlier(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(1, opacity_style(0.5, Some(opacity_transition(100, 60_000))))],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);

        // A value change with a short delay must move the wake earlier.
        apply(
            &root,
            cx,
            style_patch(
                2,
                3,
                vec![(1, opacity_style(0.25, Some(opacity_transition(100, 100))))],
            ),
        );
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(150));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "the earlier deadline must start demand");
        let started = painted.get(&1).copied().flatten().expect("progress paints");
        assert!(
            started > 0.25 && started < 1.0,
            "progress moves from 1.0 toward the new 0.25 target"
        );

        cx.executor().advance_clock(Duration::from_millis(100));
        draw_frame(&window, &root, cx);
        assert_eq!(drain_completions(&runtime), vec![(1, 2)]);

        // Neither the cancelled 60s wake nor a duplicate completion may fire.
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn active_transitions_progress_while_delayed_ones_wait(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(
            &root,
            cx,
            snapshot(
                EPOCH,
                vec![
                    view_node(1, 0, 0, 1.0),
                    view_node(2, 1, 0, 1.0),
                    view_node(3, 1, 1, 1.0),
                ],
            ),
        );
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![
                    (1, opacity_style(0.5, Some(opacity_transition(10_000, 0)))),
                    (2, opacity_style(0.25, Some(opacity_transition(100, 5_000)))),
                    (
                        3,
                        opacity_style(0.75, Some(opacity_transition(100, 20_000))),
                    ),
                ],
            ),
        );
        cx.run_until_parked();

        // Node 1 animates immediately while nodes 2 and 3 hold their source
        // style and wait.
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "the active transition demands frames");
        assert_eq!(painted.get(&1), Some(&Some(1.0)));
        assert_eq!(
            painted.get(&2),
            Some(&Some(1.0)),
            "delayed transition holds its source style"
        );
        assert_eq!(painted.get(&3), Some(&Some(1.0)));
        assert!(drain_completions(&runtime).is_empty());

        // Node 2 starts exactly at its own deadline while node 1 keeps going.
        cx.executor().advance_clock(Duration::from_millis(5_000));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1);
        assert!(painted.contains_key(&1));
        assert_eq!(
            painted.get(&2),
            Some(&Some(1.0)),
            "node 2 starts at its deadline"
        );
        assert_eq!(
            painted.get(&3),
            Some(&Some(1.0)),
            "node 3 still holds its source style"
        );

        // Node 2 completes once; node 1 is still animating.
        cx.executor().advance_clock(Duration::from_millis(150));
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1);
        assert!(painted.contains_key(&1));
        assert!(!painted.contains_key(&2));
        assert_eq!(painted.get(&3), Some(&Some(1.0)));
        assert_eq!(drain_completions(&runtime), vec![(2, 1)]);

        // Node 1 completes; node 3 alone keeps holding its source style.
        cx.executor().advance_clock(Duration::from_millis(5_000));
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(
            demand, 0,
            "a lone delayed transition must not demand frames"
        );
        assert_eq!(
            painted,
            HashMap::from([(3, Some(1.0))]),
            "node 3 keeps presenting its source style while delayed"
        );
        assert_eq!(drain_completions(&runtime), vec![(1, 1)]);

        // Node 3's wake was rearmed by the earlier ones.
        cx.executor().advance_clock(Duration::from_millis(9_850));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1);
        assert_eq!(
            painted.get(&3),
            Some(&Some(1.0)),
            "node 3 starts at its deadline"
        );

        cx.executor().advance_clock(Duration::from_millis(150));
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert_eq!(drain_completions(&runtime), vec![(3, 1)]);
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn zero_duration_transition_completes_once_without_a_frame_loop(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(1, opacity_style(0.25, Some(opacity_transition(0, 0))))],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);
        assert_eq!(drain_completions(&runtime), vec![(1, 1)]);
        for _ in 0..4 {
            let (demand, painted) = draw_frame(&window, &root, cx);
            assert_eq!(demand, 0, "a zero-duration transition must not loop frames");
            assert!(painted.is_empty());
        }
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn reduced_motion_completes_immediately_without_frames(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        let _ = window.update(cx, |_, _, cx| cx.set_reduce_motion(true));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(
                    1,
                    opacity_style(0.25, Some(opacity_transition(5_000, 5_000))),
                )],
            ),
        );
        cx.run_until_parked();
        // Reduced motion finishes at the retarget, before any draw.
        assert_eq!(drain_completions(&runtime), vec![(1, 1)]);
        // Warm-up frame: drains the startup window-observation callback so the
        // demand assertions below see only animation frames.
        draw_frame(&window, &root, cx);
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        cx.executor().advance_clock(Duration::from_secs(60));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn deletion_cancels_pending_deadlines_and_active_transitions(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(
            &root,
            cx,
            snapshot(
                EPOCH,
                vec![view_node(1, 0, 0, 1.0), view_node(2, 1, 0, 1.0)],
            ),
        );
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![
                    (1, opacity_style(0.75, Some(opacity_transition(100_000, 0)))),
                    (2, opacity_style(0.25, Some(opacity_transition(100, 5_000)))),
                ],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);
        // Delete the delayed child; its pending wake must be cancelled with it.
        apply(&root, cx, delete_patch(2, 3, &[2]));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(
            demand >= 1,
            "the surviving active transition keeps animating"
        );
        assert!(painted.contains_key(&1));
        assert!(!painted.contains_key(&2), "deleted node stops painting");

        // Advancing past the deleted node's deadline must stay quiet: the
        // wake was cancelled with the node and no completion may fire.
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert!(demand >= 1);
        assert!(painted.contains_key(&1));
        assert!(!painted.contains_key(&2));
        assert!(drain_completions(&runtime).is_empty());

        // The survivor completes exactly once and everything goes quiet.
        cx.executor().advance_clock(Duration::from_secs(80));
        draw_frame(&window, &root, cx);
        assert_eq!(drain_completions(&runtime), vec![(1, 1)]);
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn epoch_reset_drops_pending_deadlines(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, true);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(1, opacity_style(0.25, Some(opacity_transition(100, 5_000))))],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);

        // A fresh epoch replaces the surface and every pending wake with it.
        apply(&root, cx, snapshot(2, vec![view_node(1, 0, 0, 1.0)]));
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(30));
        cx.run_until_parked();
        // Warm-up frame: the reset also clears the renderer's window
        // observation baseline, so the first draw re-emits one observation
        // callback that is not animation demand.
        draw_frame(&window, &root, cx);
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0);
        assert!(painted.is_empty());
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn window_teardown_with_pending_wake_is_quiet(cx: &mut TestAppContext) {
        let runtime = InMemoryAdapter::new();
        let window = cx.open_window(window_size(), {
            let runtime = runtime.clone();
            move |_, _| SolidRoot::new(runtime)
        });
        let root = window.root(cx).expect("root entity");
        let _ = window.update(cx, |_, window, cx| {
            cx.set_reduce_motion(false);
            window.activate_window();
        });
        cx.run_until_parked();
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(
                    1,
                    opacity_style(0.25, Some(opacity_transition(100, 60_000))),
                )],
            ),
        );
        cx.run_until_parked();

        cx.update_window(window.into(), |_, window, _| window.remove_window())
            .expect("remove window");
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_secs(120));
        cx.run_until_parked();
        assert!(drain_completions(&runtime).is_empty());
    }

    #[gpui::test]
    fn inactive_windows_defer_frame_demand_until_reactivation(cx: &mut TestAppContext) {
        let (runtime, window, root) = open_surface(cx, false);
        apply(&root, cx, snapshot(EPOCH, vec![view_node(1, 0, 0, 1.0)]));
        apply(
            &root,
            cx,
            style_patch(
                1,
                2,
                vec![(
                    1,
                    opacity_style(0.25, Some(opacity_transition(60_000, 1_000))),
                )],
            ),
        );
        cx.run_until_parked();
        draw_frame(&window, &root, cx);

        // Past the deadline an inactive window paints progress but must not
        // spin the frame loop.
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        let (demand, painted) = draw_frame(&window, &root, cx);
        assert_eq!(demand, 0, "inactive windows must not demand frames");
        let started = painted
            .get(&1)
            .copied()
            .flatten()
            .expect("due transition paints while inactive");
        assert!(started < 1.0);
        assert!(drain_completions(&runtime).is_empty());

        // Reactivation resumes the frame loop while the transition is active.
        let _ = window.update(cx, |_, window, _| window.activate_window());
        cx.run_until_parked();
        let (demand, _) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "reactivation must resume frame demand");
        let (demand, _) = draw_frame(&window, &root, cx);
        assert!(demand >= 1, "the frame loop keeps running while animating");
        assert!(drain_completions(&runtime).is_empty());
    }
}
