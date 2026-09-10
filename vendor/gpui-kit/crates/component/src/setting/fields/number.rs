use std::rc::Rc;

use gpui::{
    AnyElement, App, AppContext as _, Entity, IntoElement, SharedString, StyleRefinement, Styled,
    Subscription, Window, prelude::FluentBuilder as _,
};

use crate::{
    AxisExt, Disableable, Sizable, StyledExt,
    input::{InputEvent, InputState, NumberInput, NumberInputEvent, StepAction},
    setting::{
        AnySettingField, RenderOptions,
        fields::{SettingFieldRender, get_value, set_value},
    },
};

#[derive(Clone, Debug)]
pub struct NumberFieldOptions {
    /// The minimum value for the number input, default is `f64::MIN`.
    pub min: f64,
    /// The maximum value for the number input, default is `f64::MAX`.
    pub max: f64,
    /// The step value for the number input, default is `1.0`.
    pub step: f64,
}

impl Default for NumberFieldOptions {
    fn default() -> Self {
        Self {
            min: f64::MIN,
            max: f64::MAX,
            step: 1.0,
        }
    }
}

pub(crate) struct NumberField {
    options: NumberFieldOptions,
}

impl NumberField {
    pub(crate) fn new(options: Option<&NumberFieldOptions>) -> Self {
        Self {
            options: options.cloned().unwrap_or_default(),
        }
    }
}

struct State {
    set_value: Rc<dyn Fn(f64, &mut App)>,
    options: NumberFieldOptions,
    input: Entity<InputState>,
    initial_value: f64,
    _subscriptions: Vec<Subscription>,
}

impl SettingFieldRender for NumberField {
    fn render(
        &self,
        field: Rc<dyn AnySettingField>,
        options: &RenderOptions,
        style: &StyleRefinement,
        window: &mut Window,
        cx: &mut App,
    ) -> AnyElement {
        let value = get_value::<f64>(&field, cx);
        let set_value = set_value::<f64>(&field, cx);

        let state_entity = window.use_keyed_state("number-state", cx, |window, cx| {
            let input = cx.new(|cx| InputState::new(window, cx).default_value(value.to_string()));
            let _subscriptions = vec![
                cx.subscribe_in(&input, window, {
                    move |state: &mut State, input, event: &NumberInputEvent, window, cx| {
                        match event {
                            NumberInputEvent::Step(action) => {
                                let value = input.read(cx).value();
                                if let Ok(value) = value.parse::<f64>() {
                                    let new_value = if *action == StepAction::Increment {
                                        value + state.options.step
                                    } else {
                                        value - state.options.step
                                    };
                                    let clamp_value =
                                        new_value.clamp(state.options.min, state.options.max);

                                    input.update(cx, |input, cx| {
                                        input.set_value(
                                            SharedString::from(clamp_value.to_string()),
                                            window,
                                            cx,
                                        );
                                    });
                                    (state.set_value)(clamp_value, cx);
                                    state.initial_value = clamp_value;
                                }
                            }
                        }
                    }
                }),
                cx.subscribe_in(&input, window, {
                    move |state: &mut State, input, event: &InputEvent, window, cx| match event {
                        InputEvent::Change => {
                            input.update(cx, |input, cx| {
                                let value = input.value();
                                if value == state.initial_value.to_string() {
                                    return;
                                }

                                if let Ok(value) = value.parse::<f64>() {
                                    let clamp_value =
                                        value.clamp(state.options.min, state.options.max);

                                    (state.set_value)(clamp_value, cx);
                                    state.initial_value = clamp_value;
                                    if clamp_value != value {
                                        input.set_value(
                                            SharedString::from(clamp_value.to_string()),
                                            window,
                                            cx,
                                        );
                                    }
                                }
                            });
                        }
                        _ => {}
                    }
                }),
            ];

            State {
                set_value: set_value.clone(),
                options: self.options.clone(),
                input,
                initial_value: value,
                _subscriptions,
            }
        });

        // Sync the displayed value when the underlying setting changed externally
        state_entity.update(cx, |state, cx| {
            state.set_value = set_value;
            state.options = self.options.clone();
            if state.initial_value != value {
                state.initial_value = value;
                state.input.update(cx, |input, cx| {
                    input.set_value(SharedString::from(value.to_string()), window, cx);
                });
            }
        });

        let state = state_entity.read(cx);

        NumberInput::new(&state.input)
            .disabled(options.is_disabled())
            .with_size(options.size())
            .map(|this| {
                if options.layout().is_horizontal() {
                    this.w_32()
                } else {
                    this.w_full()
                }
            })
            .refine_style(style)
            .into_any_element()
    }
}
