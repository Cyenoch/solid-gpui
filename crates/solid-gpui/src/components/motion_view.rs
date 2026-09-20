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
                MotionAnimation::Spring { .. } | MotionAnimation::Sequence { .. } => {
                    return Err("stagger requires a transition or keyframes".into());
                }
            }
        }
        animation.prepare()
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionComplete {
    pub playback_id: u32,
}
struct Motion {
    props: MotionProps,
    prepared: PreparedMotion,
    sequence: Option<motion::Sequence<MotionTarget>>,
    children: NativeChildren,
    event: Event<MotionComplete>,
    completed: bool,
    generation: u64,
}
impl Motion {
    /// Builds the retained sequence for a playback key. Rebuilt only when the
    /// generation moves — a new mount or a replay — so frames sample the kept
    /// sequence without rebuilding or reallocating its step chain.
    fn sequence_for(
        prepared: &PreparedMotion,
        generation: u64,
    ) -> Option<motion::Sequence<MotionTarget>> {
        match prepared {
            PreparedMotion::Sequence { from, steps } => Some(
                motion::Sequence::new(gpui::ElementId::Integer(generation), *from)
                    .with_steps(steps.iter().cloned()),
            ),
            _ => None,
        }
    }
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
        let sequence = Self::sequence_for(&prepared, 0);
        Self {
            props,
            prepared,
            sequence,
            children,
            event,
            completed: false,
            generation: 0,
        }
    }
    fn update(&mut self, props: Self::Props, _: &mut Window, _: &mut Context<Self>) {
        let playback_changed = props.animation != self.props.animation
            || props.stagger != self.props.stagger
            || props.playback_id != self.props.playback_id;
        if playback_changed {
            self.completed = false;
            self.prepared = props.prepare().expect("validated native motion");
            self.generation = self
                .generation
                .checked_add(1)
                .expect("motion generation exhausted");
            self.sequence = Self::sequence_for(&self.prepared, self.generation);
        } else if props.target != self.props.target
            && !matches!(self.prepared, PreparedMotion::Sequence { .. })
        {
            // A retarget re-arms completion only for the variants that sample
            // `target` live. A sequence owns its timeline and ignores a
            // mid-flight `target` change, so its completion is re-armed by a
            // playback change alone.
            self.completed = false;
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
            PreparedMotion::Sequence { .. } => {
                // Retained per playback key (built in mount/update when the
                // generation moves); borrowed sampling keeps every frame
                // allocation-free. The generation in the key owns the
                // playback: re-rendering with it continues the sequence, and
                // a replay starts a fresh key at `from`. Reduced motion
                // settles on the last step inside `sample_ref` and reports
                // `Finished` without requesting frames.
                let sample = self
                    .sequence
                    .as_ref()
                    .expect("validated native sequence")
                    .sample_ref(window, cx);
                (*sample.value(), sample.is_finished())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::EventPayload;
    use crate::components::motion_types::{MotionSequenceStep, StaggerOrigin};
    use crate::components::test_support::Fixture;
    use gpui::TestAppContext;

    fn sequence_props(stagger: Option<MotionStagger>) -> MotionProps {
        MotionProps {
            target: MotionTarget::default(),
            animation: MotionAnimation::Sequence {
                from: MotionTarget::default(),
                steps: vec![MotionSequenceStep {
                    target: MotionTarget::default(),
                    duration_ms: 100,
                    delay_ms: 0,
                    easing: MotionEasing::EaseOut,
                }],
            },
            playback_id: 1,
            stagger,
        }
    }

    #[test]
    fn a_sequence_prepares_without_stagger_and_rejects_it_with_one() {
        assert!(sequence_props(None).prepare().is_ok());
        assert!(
            sequence_props(Some(MotionStagger {
                index: 0,
                count: 2,
                interval_ms: 40,
                origin: StaggerOrigin::First,
            }))
            .prepare()
            .is_err()
        );
    }

    /// Two 100ms linear steps on a 200ms timeline.
    fn two_step_sequence(playback_id: u32) -> MotionProps {
        MotionProps {
            target: MotionTarget::default(),
            animation: MotionAnimation::Sequence {
                from: MotionTarget {
                    x: 0.,
                    y: 0.,
                    opacity: 0.,
                },
                steps: vec![
                    MotionSequenceStep {
                        target: MotionTarget {
                            x: 0.,
                            y: 0.,
                            opacity: 1.,
                        },
                        duration_ms: 100,
                        delay_ms: 0,
                        easing: MotionEasing::Linear,
                    },
                    MotionSequenceStep {
                        target: MotionTarget {
                            x: 10.,
                            y: 0.,
                            opacity: 1.,
                        },
                        duration_ms: 100,
                        delay_ms: 0,
                        easing: MotionEasing::Linear,
                    },
                ],
            },
            playback_id,
            stagger: None,
        }
    }

    fn draw(f: &Fixture<Motion>, cx: &mut TestAppContext) {
        f.window
            .update(cx, |_, window, _| window.refresh())
            .unwrap();
        cx.run_until_parked();
    }

    fn completions(f: &Fixture<Motion>) -> Vec<MotionComplete> {
        let mut result = vec![];
        while let Some(e) = f.runtime.take_event().unwrap() {
            if let EventPayload::Extension {
                event_id: 1,
                fields,
                ..
            } = e.payload
            {
                let crate::protocol::ExtensionValue::Bytes(bytes) = &fields[0].value else {
                    panic!("typed event")
                };
                result.push(crate::native::decode_json(bytes).unwrap());
            }
        }
        result
    }

    #[gpui::test]
    fn a_sequence_completes_once_ignores_target_moves_and_replays_by_playback_id(
        cx: &mut TestAppContext,
    ) {
        let f = Fixture::<Motion>::new(two_step_sequence(1), cx);
        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "a fresh sequence is mid-flight, not finished"
        );

        cx.executor().advance_clock(Duration::from_millis(150));
        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "the second step is still playing at 150ms of 200ms"
        );

        cx.executor().advance_clock(Duration::from_millis(100));
        draw(&f, cx);
        assert_eq!(
            completions(&f),
            vec![MotionComplete { playback_id: 1 }],
            "completion fires once the last step settles"
        );

        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "a finished sequence does not re-emit on later frames"
        );

        // A `target` move is not a playback change: a sequence owns its
        // timeline, so completion stays settled instead of firing again.
        f.update(cx, |v, w, cx| {
            let mut p = v.props.clone();
            p.target = MotionTarget {
                x: 5.,
                y: 0.,
                opacity: 1.,
            };
            v.update(p, w, cx);
        });
        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "a target move without a playback change never re-emits completion"
        );

        f.update(cx, |v, w, cx| {
            let mut p = v.props.clone();
            p.playback_id = 2;
            v.update(p, w, cx);
        });
        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "a replayed sequence restarts from its origin mid-flight"
        );

        cx.executor().advance_clock(Duration::from_millis(250));
        draw(&f, cx);
        assert_eq!(
            completions(&f),
            vec![MotionComplete { playback_id: 2 }],
            "the replay reports its own playbackId"
        );
    }

    #[gpui::test]
    fn reduced_motion_settles_a_sequence_and_completes_immediately(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_reduce_motion(true));
        let f = Fixture::<Motion>::new(two_step_sequence(7), cx);
        draw(&f, cx);
        assert_eq!(
            completions(&f),
            vec![MotionComplete { playback_id: 7 }],
            "reduced motion adopts the last step without waiting on the clock"
        );

        draw(&f, cx);
        assert!(
            completions(&f).is_empty(),
            "the immediate completion is emitted once"
        );
    }
}
