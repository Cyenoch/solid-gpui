//! Retained scalar controls with quiet property writes and semantic native events.
use super::ControlSize;
use crate::native::{
    ControlledBinding, Event, EventDefinition, NativeChildren, NativeView, ViewCommand,
};
use gpui::{AppContext, Context, Entity, IntoElement, Render, Subscription, Window};
use gpui_component::{Disableable, Sizable};

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum SliderValue {
    Single { value: f32 },
    Range { start: f32, end: f32 },
}
impl Default for SliderValue {
    fn default() -> Self {
        Self::Single { value: 0. }
    }
}
impl From<SliderValue> for gpui_component::slider::SliderValue {
    fn from(v: SliderValue) -> Self {
        match v {
            SliderValue::Single { value } => Self::Single(value),
            SliderValue::Range { start, end } => Self::Range(start, end),
        }
    }
}
impl From<gpui_component::slider::SliderValue> for SliderValue {
    fn from(v: gpui_component::slider::SliderValue) -> Self {
        match v {
            gpui_component::slider::SliderValue::Single(value) => Self::Single { value },
            gpui_component::slider::SliderValue::Range(start, end) => Self::Range { start, end },
        }
    }
}
#[crate::native_type]
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum SliderScale {
    #[default]
    Linear,
    Logarithmic,
}
impl From<SliderScale> for gpui_component::slider::SliderScale {
    fn from(v: SliderScale) -> Self {
        match v {
            SliderScale::Linear => Self::Linear,
            SliderScale::Logarithmic => Self::Logarithmic,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct SliderProps {
    pub value: Option<SliderValue>,
    pub default_value: SliderValue,
    pub ack_edit_seq: u32,
    pub min: f32,
    pub max: f32,
    pub step: f32,
    pub scale: SliderScale,
    pub vertical: bool,
    pub reverse: bool,
    pub disabled: bool,
}
impl Default for SliderProps {
    fn default() -> Self {
        Self {
            value: None,
            default_value: SliderValue::default(),
            ack_edit_seq: 0,
            min: 0.,
            max: 100.,
            step: 1.,
            scale: SliderScale::Linear,
            vertical: false,
            reverse: false,
            disabled: false,
        }
    }
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SliderChange {
    pub value: SliderValue,
    pub edit_seq: u32,
}
pub struct Slider {
    state: Entity<gpui_component::slider::SliderState>,
    props: SliderProps,
    event: Event<SliderChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for Slider {
    type Props = SliderProps;
    type Event = SliderChange;
    fn validate_props(p: &SliderProps) -> Result<(), String> {
        if !p.min.is_finite()
            || !p.max.is_finite()
            || p.min >= p.max
            || !p.step.is_finite()
            || p.step <= 0.
            || (p.scale == SliderScale::Logarithmic && p.min <= 0.)
        {
            return Err(
                "slider requires finite min < max, positive step, and positive logarithmic min"
                    .into(),
            );
        }
        let value: gpui_component::slider::SliderValue = p.value.unwrap_or(p.default_value).into();
        let start = if value.is_single() {
            value.end()
        } else {
            value.start()
        };
        if !start.is_finite()
            || !value.end().is_finite()
            || start < p.min
            || value.end() > p.max
            || start > value.end()
        {
            return Err("slider value must be finite, ordered and inside its bounds".into());
        }
        Ok(())
    }
    fn mount(
        props: SliderProps,
        event: Event<SliderChange>,
        _: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            let mut s = gpui_component::slider::SliderState::new();
            s.configure(props.min, props.max, props.step, props.scale.into(), cx)
                .expect("validated slider configuration");
            s.default_value(gpui_component::slider::SliderValue::from(
                props.value.unwrap_or(props.default_value),
            ))
        });
        let subscription = cx.subscribe(&state, |this, _, event, _| match event {
            gpui_component::slider::SliderEvent::Change(value) => {
                this.edit_seq = this
                    .edit_seq
                    .checked_add(1)
                    .expect("slider edit sequence exhausted");
                this.event.emit(SliderChange {
                    value: (*value).into(),
                    edit_seq: this.edit_seq,
                });
            }
            gpui_component::slider::SliderEvent::Release(value) => this
                .event
                .related("release")
                .emit(SliderValue::from(*value)),
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, props: SliderProps, window: &mut Window, cx: &mut Context<Self>) {
        let entering = self.props.value.is_none() && props.value.is_some();
        self.state.update(cx, |s, cx| {
            if (props.min, props.max, props.step, props.scale)
                != (
                    self.props.min,
                    self.props.max,
                    self.props.step,
                    self.props.scale,
                )
            {
                s.configure(props.min, props.max, props.step, props.scale.into(), cx)
                    .expect("validated slider configuration");
            }
            if let Some(value) = props.value
                && (entering || props.ack_edit_seq >= self.edit_seq)
                && SliderValue::from(s.value()) != value
            {
                s.set_value(gpui_component::slider::SliderValue::from(value), window, cx);
            }
        });
        self.props = props;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![EventDefinition::new::<SliderValue>("release")]
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "value",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
}
impl Render for Slider {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut v = gpui_component::slider::Slider::new(&self.state).disabled(self.props.disabled);
        if self.props.vertical {
            v = v.vertical();
        }
        if self.props.reverse {
            v = v.reverse();
        }
        v
    }
}

#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct OtpInputProps {
    pub length: usize,
    pub value: Option<String>,
    pub default_value: String,
    pub ack_edit_seq: u32,
    pub groups: usize,
    pub masked: bool,
    pub disabled: bool,
    pub size: ControlSize,
}
impl Default for OtpInputProps {
    fn default() -> Self {
        Self {
            length: 6,
            value: None,
            default_value: String::new(),
            ack_edit_seq: 0,
            groups: 2,
            masked: false,
            disabled: false,
            size: ControlSize::Medium,
        }
    }
}
pub struct OtpInput {
    state: Entity<gpui_component::input::OtpState>,
    props: OtpInputProps,
    event: Event<super::input::InputChange>,
    edit_seq: u32,
    _subscription: Subscription,
}
#[crate::component]
impl NativeView for OtpInput {
    type Props = OtpInputProps;
    type Event = super::input::InputChange;
    fn validate_props(p: &OtpInputProps) -> Result<(), String> {
        if !(1..=64).contains(&p.length) || p.groups == 0 || p.groups > p.length {
            return Err("OTP length must be 1..64 and groups 1..length".into());
        }
        let value = p.value.as_ref().unwrap_or(&p.default_value);
        if value.len() > p.length || !value.bytes().all(|c| c.is_ascii_digit()) {
            return Err("OTP value must contain at most length ASCII digits".into());
        }
        Ok(())
    }
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        _: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let state = cx.new(|cx| {
            gpui_component::input::OtpState::new(props.length, window, cx)
                .default_value(props.value.clone().unwrap_or(props.default_value.clone()))
                .masked(props.masked)
        });
        let subscription = cx.subscribe(&state, |this, state, event, cx| {
            use gpui_component::input::OtpEvent;
            match event {
                OtpEvent::Change => {
                    this.edit_seq = this
                        .edit_seq
                        .checked_add(1)
                        .expect("OTP edit sequence exhausted");
                    this.event.emit(super::input::InputChange {
                        value: state.read(cx).value().to_string(),
                        edit_seq: this.edit_seq,
                    });
                }
                OtpEvent::Complete => this
                    .event
                    .related("complete")
                    .emit(state.read(cx).value().to_string()),
                OtpEvent::Focus => this.event.related("focus").emit(()),
                OtpEvent::Blur => this.event.related("blur").emit(()),
            }
        });
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            _subscription: subscription,
        }
    }
    fn update(&mut self, p: Self::Props, window: &mut Window, cx: &mut Context<Self>) {
        let entering = self.props.value.is_none() && p.value.is_some();
        self.state.update(cx, |s, cx| {
            if p.length != self.props.length {
                s.set_length(p.length, cx);
            }
            if p.masked != self.props.masked {
                s.set_masked(p.masked, window, cx);
            }
            if let Some(value) = &p.value
                && (entering || p.ack_edit_seq >= self.edit_seq)
                && s.value().as_ref() != value
            {
                s.set_value(value.clone(), window, cx);
            }
        });
        self.props = p;
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<String>("complete"),
            EventDefinition::new::<()>("focus"),
            EventDefinition::new::<()>("blur"),
        ]
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            value_prop: "value",
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![ViewCommand::new("focus", |this, (): (), window, cx| {
            if this.props.disabled {
                return Err("disabled OTP cannot be focused".into());
            }
            this.state.update(cx, |s, cx| s.focus(window, cx));
            Ok(())
        })]
    }
}
impl Render for OtpInput {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui_component::input::OtpInput::new(&self.state)
            .groups(self.props.groups)
            .disabled(self.props.disabled)
            .with_size(self.props.size)
    }
}

pub(crate) fn definitions() -> Vec<crate::native::ComponentDefinition> {
    vec![__native_component_Slider(), __native_component_OtpInput()]
}
