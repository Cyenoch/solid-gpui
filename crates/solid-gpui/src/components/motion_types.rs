//! Bounded data contracts for native motion; all sampling stays in gpui-base.
use gpui_base::motion::{self, Interpolate};
use std::time::Duration;

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct MotionTarget {
    pub x: f32,
    pub y: f32,
    pub opacity: f32,
}
impl Default for MotionTarget {
    fn default() -> Self {
        Self {
            x: 0.,
            y: 0.,
            opacity: 1.,
        }
    }
}
impl MotionTarget {
    pub(super) fn validate(&self) -> Result<(), String> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || self.x.abs() > 100000.
            || self.y.abs() > 100000.
            || !self.opacity.is_finite()
            || !(0.0..=1.0).contains(&self.opacity)
        {
            return Err(
                "motion requires finite offsets within 100000 pixels and opacity in 0..1".into(),
            );
        }
        Ok(())
    }
}
impl Interpolate for MotionTarget {
    fn interpolate(&self, target: &Self, progress: f32) -> Self {
        Self {
            x: self.x.interpolate(&target.x, progress),
            y: self.y.interpolate(&target.y, progress),
            opacity: self.opacity.interpolate(&target.opacity, progress),
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MotionEasing {
    Linear,
    Ease,
    EaseIn,
    #[default]
    EaseOut,
    EaseInOut,
}
impl From<MotionEasing> for motion::Easing {
    fn from(value: MotionEasing) -> Self {
        match value {
            MotionEasing::Linear => Self::Linear,
            MotionEasing::Ease => Self::Ease,
            MotionEasing::EaseIn => Self::EaseIn,
            MotionEasing::EaseOut => Self::EaseOut,
            MotionEasing::EaseInOut => Self::EaseInOut,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum MotionDirection {
    #[default]
    Normal,
    Reverse,
    Alternate,
    AlternateReverse,
}
impl From<MotionDirection> for motion::PlaybackDirection {
    fn from(value: MotionDirection) -> Self {
        match value {
            MotionDirection::Normal => Self::Normal,
            MotionDirection::Reverse => Self::Reverse,
            MotionDirection::Alternate => Self::Alternate,
            MotionDirection::AlternateReverse => Self::AlternateReverse,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
pub struct MotionKeyframe {
    pub offset: f32,
    pub value: MotionTarget,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum MotionAnimation {
    Transition {
        duration_ms: u32,
        #[serde(default)]
        delay_ms: i32,
        #[serde(default)]
        easing: MotionEasing,
    },
    Spring {
        response_ms: u32,
        #[serde(default = "damping")]
        damping: f32,
        #[serde(default = "travel")]
        travel: bool,
    },
    Keyframes {
        frames: Vec<MotionKeyframe>,
        duration_ms: u32,
        #[serde(default)]
        delay_ms: i32,
        #[serde(default = "iterations")]
        iterations: Option<u32>,
        #[serde(default)]
        direction: MotionDirection,
        #[serde(default)]
        easing: MotionEasing,
    },
}
fn damping() -> f32 {
    1.
}
fn travel() -> bool {
    true
}
fn iterations() -> Option<u32> {
    Some(1)
}
impl Default for MotionAnimation {
    fn default() -> Self {
        Self::Transition {
            duration_ms: 200,
            delay_ms: 0,
            easing: MotionEasing::EaseOut,
        }
    }
}
pub(super) enum PreparedMotion {
    Transition(motion::Transition),
    Spring(motion::Spring),
    Keyframes(motion::Keyframes<MotionTarget>, motion::Timing),
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum StaggerOrigin {
    #[default]
    First,
    Last,
    Center,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotionStagger {
    pub index: usize,
    pub count: usize,
    pub interval_ms: u32,
    #[serde(default)]
    pub origin: StaggerOrigin,
}
impl MotionStagger {
    pub(super) fn delay_ms(&self) -> Result<i32, String> {
        if self.count == 0
            || self.count > 1024
            || self.index >= self.count
            || self.interval_ms > 60000
        {
            return Err(
                "stagger requires 1..1024 items, a valid index, and an interval within 60000 ms"
                    .into(),
            );
        }
        let origin = match self.origin {
            StaggerOrigin::First => motion::StaggerOrigin::First,
            StaggerOrigin::Last => motion::StaggerOrigin::Last,
            StaggerOrigin::Center => motion::StaggerOrigin::Center,
        };
        let delay = motion::Stagger::new(Duration::from_millis(self.interval_ms as u64), origin)
            .delay(self.index, self.count)
            .as_millis();
        if delay > 60000 {
            return Err("stagger delay exceeds 60000 ms".into());
        }
        Ok(delay as i32)
    }
}
pub(super) fn delay(ms: i32) -> motion::SignedDuration {
    let duration = Duration::from_millis(ms.unsigned_abs() as u64);
    if ms < 0 {
        motion::SignedDuration::negative(duration)
    } else {
        motion::SignedDuration::positive(duration)
    }
}
pub(super) fn timing(duration_ms: u32, delay_ms: i32) -> Result<(), String> {
    if duration_ms > 60000 || delay_ms.unsigned_abs() > 60000 {
        return Err("motion duration and absolute delay must not exceed 60000 ms".into());
    }
    Ok(())
}
impl MotionAnimation {
    pub(super) fn prepare(&self) -> Result<PreparedMotion, String> {
        Ok(match self {
            Self::Transition {
                duration_ms,
                delay_ms,
                easing,
            } => {
                timing(*duration_ms, *delay_ms)?;
                PreparedMotion::Transition(
                    motion::Transition::new(Duration::from_millis(*duration_ms as u64))
                        .delay(delay(*delay_ms))
                        .easing((*easing).into()),
                )
            }
            Self::Spring {
                response_ms,
                damping,
                travel,
            } => {
                timing(*response_ms, 0)?;
                if !damping.is_finite() || !(0.05..=10.).contains(damping) {
                    return Err("spring damping must be between 0.05 and 10".into());
                }
                PreparedMotion::Spring(
                    motion::Spring::new(Duration::from_millis(*response_ms as u64))
                        .try_with_damping(*damping)
                        .map_err(|e| e.to_string())?
                        .with_travel(*travel),
                )
            }
            Self::Keyframes {
                frames,
                duration_ms,
                delay_ms,
                iterations,
                direction,
                easing,
            } => {
                timing(*duration_ms, *delay_ms)?;
                if !(2..=128).contains(&frames.len())
                    || iterations.is_some_and(|n| n == 0 || n > 10000)
                {
                    return Err("keyframes require 2..128 stops and 1..10000 iterations, or null for repetition".into());
                }
                for frame in frames {
                    frame.value.validate()?;
                }
                let frames = motion::Keyframes::try_new(
                    frames
                        .iter()
                        .map(|frame| motion::Keyframe::new(frame.offset, frame.value)),
                )
                .map_err(|e| format!("invalid keyframes: {e:?}"))?;
                let count = iterations.map_or(motion::IterationCount::Infinite, |n| {
                    motion::IterationCount::Finite(n as u64)
                });
                PreparedMotion::Keyframes(
                    frames,
                    motion::Timing::new(Duration::from_millis(*duration_ms as u64))
                        .delay(delay(*delay_ms))
                        .iterations(count)
                        .direction((*direction).into())
                        .ease((*easing).into()),
                )
            }
        })
    }
}
