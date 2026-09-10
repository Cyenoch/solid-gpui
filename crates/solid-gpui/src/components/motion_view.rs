//! Retained native motion and presence. Only completion crosses back to JavaScript.
use super::motion_types::{
    MotionAnimation, MotionEasing, MotionStagger, MotionTarget, PreparedMotion, timing,
};
use crate::native::{ComponentDefinition, Event, NativeChildren, NativeView};
use gpui::{Context, IntoElement, ParentElement, Render, Styled, Window, div, px};
use gpui_base::motion::{self, MotionStatus};
use std::time::Duration;

#[crate::native_type]
#[derive(Clone, Default, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct MotionProps {
    pub target: MotionTarget,
    pub animation: MotionAnimation,
    pub playback_id: u32,
    pub stagger: Option<MotionStagger>,
}
impl MotionProps {
    fn prepare(&self) -> Result<PreparedMotion, String> {
        let mut animation = self.animation.clone();
        if let Some(stagger) = &self.stagger {
            let offset = stagger.delay_ms()?;
            match &mut animation {
                MotionAnimation::Transition { delay_ms, .. }
                | MotionAnimation::Keyframes { delay_ms, .. } => {
                    *delay_ms = delay_ms
                        .checked_add(offset)
                        .ok_or("stagger delay overflow")?;
                }
                MotionAnimation::Spring { .. } => {
                    return Err("stagger requires a transition or keyframes".into());
                }
            }
        }
        animation.prepare()
    }
}
#[crate::native_type]
#[derive(Clone)]
#[serde(rename_all = "camelCase")]
pub struct MotionComplete {
    pub playback_id: u32,
}
struct Motion {
    props: MotionProps,
    prepared: PreparedMotion,
    children: NativeChildren,
    event: Event<MotionComplete>,
    completed: bool,
    generation: u64,
}
impl NativeView for Motion {
    type Props = MotionProps;
    type Event = MotionComplete;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "complete"
    }
    fn validate_props(props: &Self::Props) -> Result<(), String> {
        props.target.validate()?;
        props.prepare().map(|_| ())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        let prepared = props.prepare().expect("validated native motion");
        Self {
            props,
            prepared,
            children,
            event,
            completed: false,
            generation: 0,
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, _: &mut Context<Self>) {
        if props != self.props {
            self.completed = false;
        }
        if props.animation != self.props.animation
            || props.stagger != self.props.stagger
            || props.playback_id != self.props.playback_id
        {
            self.prepared = props.prepare().expect("validated native motion");
            self.generation = self
                .generation
                .checked_add(1)
                .expect("motion generation exhausted");
        }
        self.props = props;
    }
}
impl Render for Motion {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let target = self.props.target;
        let (value, finished) = match &self.prepared {
            PreparedMotion::Transition(policy) => {
                let sample = motion::transition_with_status(
                    "motion-target",
                    target,
                    policy.clone(),
                    window,
                    cx,
                );
                (
                    sample.value,
                    matches!(sample.status, MotionStatus::Idle | MotionStatus::Finished),
                )
            }
            PreparedMotion::Spring(policy) => {
                let value = MotionTarget {
                    x: motion::spring("motion-x", target.x, policy.with_epsilon(0.1), window, cx),
                    y: motion::spring("motion-y", target.y, policy.with_epsilon(0.1), window, cx),
                    opacity: motion::spring("motion-opacity", target.opacity, *policy, window, cx),
                };
                (value, value == target)
            }
            PreparedMotion::Keyframes(frames, timing) => {
                let sample = motion::animate_keyframes(
                    gpui::ElementId::Integer(self.generation),
                    frames,
                    timing.clone(),
                    window,
                    cx,
                );
                (sample.value, sample.status == MotionStatus::Finished)
            }
        };
        if finished && !self.completed {
            self.completed = true;
            self.event.emit(MotionComplete {
                playback_id: self.props.playback_id,
            });
        }
        div()
            .relative()
            .left(px(value.x))
            .top(px(value.y))
            .opacity(value.opacity.clamp(0., 1.))
            .child(self.children.content())
    }
}

#[crate::native_type]
#[derive(Clone, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct NativePresenceProps {
    pub present: bool,
    pub generation: u32,
    pub duration_ms: u32,
    pub easing: MotionEasing,
    pub offset_y: f32,
    pub reveal: bool,
}
impl Default for NativePresenceProps {
    fn default() -> Self {
        Self {
            present: true,
            generation: 0,
            duration_ms: 180,
            easing: MotionEasing::EaseOut,
            offset_y: 8.,
            reveal: false,
        }
    }
}
#[crate::native_type]
#[derive(Clone)]
pub struct PresenceComplete {
    pub present: bool,
    pub generation: u32,
}
struct NativePresence {
    props: NativePresenceProps,
    children: NativeChildren,
    event: Event<PresenceComplete>,
    completed: bool,
}
impl NativeView for NativePresence {
    type Props = NativePresenceProps;
    type Event = PresenceComplete;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "complete"
    }
    fn validate_props(props: &Self::Props) -> Result<(), String> {
        timing(props.duration_ms, 0)?;
        if !props.offset_y.is_finite() || props.offset_y.abs() > 100000. {
            return Err("presence offsetY must be finite and within 100000 pixels".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self {
            props,
            children,
            event,
            completed: false,
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, _: &mut Context<Self>) {
        if props != self.props {
            self.completed = false;
        }
        self.props = props;
    }
}
impl Render for NativePresence {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sample = motion::Presence::new("native-presence", self.props.present)
            .transition(
                motion::Transition::new(Duration::from_millis(self.props.duration_ms as u64))
                    .easing(self.props.easing.into()),
            )
            .sample(window, cx);
        if sample.status == MotionStatus::Finished && !self.completed {
            self.completed = true;
            self.event.emit(PresenceComplete {
                present: self.props.present,
                generation: self.props.generation,
            });
        }
        if !sample.should_render() {
            return gpui::Empty.into_any_element();
        }
        let child = div()
            .relative()
            .top(px(self.props.offset_y * (1. - sample.progress)))
            .opacity(sample.progress.clamp(0., 1.))
            .child(self.children.content())
            .into_any_element();
        if self.props.reveal {
            motion::MotionReveal::new("presence-reveal", sample.progress, child).into_any_element()
        } else {
            child
        }
    }
}
pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<Motion>("Motion").with_contract(concat!(
            include_str!("motion_types.rs"),
            include_str!("motion_view.rs")
        )),
        ComponentDefinition::view::<NativePresence>("NativePresence")
            .with_contract(include_str!("motion_view.rs")),
    ]
}
