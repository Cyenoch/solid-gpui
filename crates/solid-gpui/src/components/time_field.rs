//! Retained segmented time editing; application values are civil times, not timestamps.
use super::ControlSize;
use crate::native::{
    ControlledBinding, Deserialize, Event, NativeChildren, NativeView, Serialize, TS, ViewCommand,
};
use chrono::NaiveTime;
use gpui::{AppContext, Context, Entity, Focusable, IntoElement, Render, Subscription, Window};
use gpui_component::{
    Disableable, Sizable,
    time_field::{TimeFieldEvent, TimeFieldState},
};

#[derive(Clone, Debug, PartialEq, Serialize, TS)]
pub struct CivilTime(String);
impl Default for CivilTime {
    fn default() -> Self {
        Self("00:00:00".into())
    }
}
impl<'de> Deserialize<'de> for CivilTime {
    fn deserialize<D: serde::Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let value = String::deserialize(d)?;
        let time =
            NaiveTime::parse_from_str(&value, "%H:%M:%S").map_err(serde::de::Error::custom)?;
        if value.len() != 8
            || time.format("%H:%M:%S").to_string() != value
            || value.ends_with(":60")
        {
            return Err(serde::de::Error::custom(
                "time must be HH:MM:SS, from 00:00:00 to 23:59:59",
            ));
        }
        Ok(Self(value))
    }
}
impl CivilTime {
    pub(super) fn native(&self) -> NaiveTime {
        NaiveTime::parse_from_str(&self.0, "%H:%M:%S").expect("validated civil time")
    }
    pub(super) fn from_native(time: NaiveTime) -> Self {
        Self(time.format("%H:%M:%S").to_string())
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TimePrecision {
    #[default]
    Minute,
    Second,
}
impl From<TimePrecision> for gpui_component::time_field::TimePrecision {
    fn from(value: TimePrecision) -> Self {
        match value {
            TimePrecision::Minute => Self::Minute,
            TimePrecision::Second => Self::Second,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HourCycle {
    H12,
    #[default]
    H23,
}
impl From<HourCycle> for gpui_component::time_field::HourCycle {
    fn from(value: HourCycle) -> Self {
        match value {
            HourCycle::H12 => Self::H12,
            HourCycle::H23 => Self::H23,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct TimeFieldProps {
    pub value: Option<CivilTime>,
    pub default_value: CivilTime,
    pub ack_edit_seq: u32,
    pub precision: TimePrecision,
    pub hour_cycle: HourCycle,
    pub disabled: bool,
    pub invalid: bool,
    pub size: ControlSize,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct TimeChange {
    pub value: CivilTime,
    pub edit_seq: u32,
}
pub struct TimeField {
    state: Entity<TimeFieldState>,
    props: TimeFieldProps,
    event: Event<TimeChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for TimeField {
    type Props = TimeFieldProps;
    type Event = TimeChange;
    fn mount(
        props: Self::Props,
        event: Event<TimeChange>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut state = TimeFieldState::new(window, cx)
                .precision(props.precision.into())
                .hour_cycle(props.hour_cycle.into());
            state.set_time(
                props
                    .value
                    .as_ref()
                    .unwrap_or(&props.default_value)
                    .native(),
                window,
                cx,
            );
            state
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| {
            let TimeFieldEvent::Change(time) = event;
            this.edit_seq = this
                .edit_seq
                .checked_add(1)
                .expect("time edit sequence exhausted");
            this.event.emit(TimeChange {
                value: CivilTime::from_native(*time),
                edit_seq: this.edit_seq,
            });
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        self.state.update(cx, |state, cx| {
            if props.precision != self.props.precision {
                state.set_precision(props.precision.into(), window, cx);
            }
            if props.hour_cycle != self.props.hour_cycle {
                state.set_hour_cycle(props.hour_cycle.into(), window, cx);
            }
            if let Some(value) = &props.value
                && (self.props.value.is_none() || props.ack_edit_seq >= self.edit_seq)
                && state.time() != value.native()
            {
                state.set_time(value.native(), window, cx);
            }
        });
        self.props = props;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_props: &["value"],
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("getValue", |this, (): (), _, cx| {
                Ok(CivilTime::from_native(this.state.read(cx).time()))
            }),
            ViewCommand::new("focus", |this, (): (), window, cx| {
                if this.props.disabled {
                    return Err("disabled time field cannot be focused".into());
                }
                this.state.read(cx).focus_handle(cx).focus(window, cx);
                Ok(())
            }),
        ]
    }
}
impl Render for TimeField {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_component::time_field::TimeField::new(&self.state)
            .with_size(self.props.size)
            .disabled(self.props.disabled)
            .invalid(self.props.invalid)
    }
}
pub(super) fn definition() -> crate::native::ComponentDefinition {
    __native_component_TimeField()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::test_support::Fixture;

    #[gpui::test]
    fn keyboard_edits_retain_identity_and_reject_stale_controlled_values(
        cx: &mut gpui::TestAppContext,
    ) {
        let props = TimeFieldProps {
            value: Some(CivilTime("09:30:15".into())),
            precision: TimePrecision::Second,
            ..Default::default()
        };
        let fixture = Fixture::<TimeField>::new(props.clone(), cx);
        let mut visual = gpui::VisualTestContext::from_window(fixture.window.into(), cx);
        let id = fixture.update(cx, |view, window, cx| {
            view.state.read(cx).focus_handle(cx).focus(window, cx);
            view.state.entity_id()
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("up");
        visual.run_until_parked();
        fixture.update(cx, |view, window, cx| {
            assert_eq!(
                view.state.read(cx).time().format("%H:%M:%S").to_string(),
                "10:30:15"
            );
            assert_eq!(view.edit_seq, 1);
            view.update(
                TimeFieldProps {
                    hour_cycle: HourCycle::H12,
                    ..props.clone()
                },
                window,
                cx,
            );
            assert_eq!(view.state.entity_id(), id);
            assert_eq!(
                view.state.read(cx).time().format("%H:%M:%S").to_string(),
                "10:30:15"
            );
            view.update(
                TimeFieldProps {
                    ack_edit_seq: 1,
                    disabled: true,
                    ..props
                },
                window,
                cx,
            );
            assert_eq!(
                view.state.read(cx).time().format("%H:%M:%S").to_string(),
                "09:30:15"
            );
        });
        visual.update(|window, cx| window.draw(cx).clear(cx));
        visual.simulate_keystrokes("up");
        visual.run_until_parked();
        fixture.update(cx, |view, _, cx| {
            assert_eq!(view.edit_seq, 1);
            assert_eq!(
                view.state.read(cx).time().format("%H:%M:%S").to_string(),
                "09:30:15"
            );
        });
        for value in ["24:00:00", "09:30:60", "9:30:00"] {
            assert!(serde_json::from_value::<CivilTime>(serde_json::json!(value)).is_err());
        }
    }
}
