use std::collections::HashSet;
use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use gpui::{Context, Window};

use crate::protocol::{Easing, Event, Style};
use crate::transport::send_event_or_exit;
use crate::tree::StoredNode;

use super::ReactRoot;

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
            generation: 0,
            completion_sent: true,
        }
    }
    pub(super) fn retarget(&mut self, style: &Style, now: Instant) {
        let Some(transition) = style.transition.as_ref() else {
            *self = Self::at_target(style);
            return;
        };
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
pub(super) fn animation_target_changed(previous: &Style, current: &Style) -> bool {
    let properties = current
        .transition
        .as_ref()
        .map_or(0, |transition| transition.properties);
    (properties & crate::protocol::TRANSITION_OPACITY != 0 && previous.opacity != current.opacity)
        || (properties & crate::protocol::TRANSITION_BACKGROUND_COLOR != 0
            && previous.background_rgba != current.background_rgba)
        || (properties & crate::protocol::TRANSITION_WIDTH != 0 && previous.width != current.width)
        || (properties & crate::protocol::TRANSITION_HEIGHT != 0
            && previous.height != current.height)
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

impl ReactRoot {
    pub(super) fn prune_animation_states(&mut self) {
        self.animation_states
            .retain(|id, _| self.store.get(*id).is_some());
        self.animation_styles
            .retain(|id, _| self.store.get(*id).is_some());
    }

    pub(super) fn reconcile_animation_states(
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

    pub(super) fn retarget_animation(&mut self, node_id: u32, style: &Style, reduce_motion: bool) {
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

    pub(super) fn finish_animation(&mut self, node_id: u32) {
        let Some(state) = self.animation_states.get_mut(&node_id) else {
            return;
        };
        if state.completion_sent {
            state.opacity_active = false;
            state.background_active = false;
            state.width_active = false;
            state.height_active = false;
            return;
        }
        state.opacity_active = false;
        state.background_active = false;
        state.width_active = false;
        state.height_active = false;
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
        send_event_or_exit(self.runtime.as_ref(), "animation complete event", &event);
    }

    pub(super) fn prepare_animation_frame(&mut self, window: &mut Window, cx: &mut Context<Self>) {
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
            let (opacity, background, width, height, done) =
                state.values(Instant::now(), cx.reduce_motion());
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
            if state.width_active {
                frame_style.width = if done { state.width_target } else { width };
            }
            if state.height_active {
                frame_style.height = if done { state.height_target } else { height };
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

    pub(super) fn style_for_node<'a>(&'a self, node: &'a StoredNode) -> Option<&'a Style> {
        self.frame_styles.get(&node.id).or(node.style.as_ref())
    }
}
