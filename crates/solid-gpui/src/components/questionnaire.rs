//! Native questionnaire flow: retained answers, validation, navigation and
//! keyboard handling stay in gpui-component's [`QuestionnaireState`]; the
//! questions arrive as typed children the way `Tree`'s items do.
//!
//! The root builds the state, keeps it across frames keyed by a schema
//! fingerprint, and renders the upstream default composition: progress, the
//! active question with its title, description, choices, freeform input and
//! error, then the navigation actions. `QuestionnaireProgress`, `Title`,
//! `Description`, `Choices`, `Choice`, `Input`, `Error`, `Actions` and the four
//! action buttons are native-internal parts of that composition; they are not
//! standalone JSX components. The native state is the authoritative editing
//! state: JS reads snapshots and events, and mutates only through commands.
//!
//! Data contracts and schema rules live in the sibling `questionnaire_types`
//! module; this file owns the retained view and framework glue.
#[path = "questionnaire_types.rs"]
mod types;

use std::rc::Rc;

use crate::ExtensionChildSummary;
use crate::native::{
    ComponentDefinition, ElementContext, Event, EventDefinition, NativeChildren, NativeView,
    ViewCommand, decode_json,
};
use gpui::{
    AnyElement, AppContext, Context, Entity, IntoElement, ParentElement, Render, StyleRefinement,
    Styled, Subscription, Window,
};
use gpui_component::Sizable;
use gpui_component::input::InputState;
use gpui_component::questionnaire as qn;
use gpui_component::questionnaire::{
    Questionnaire as NativeQuestionnaire, QuestionnaireActions,
    QuestionnaireChoice as NativeChoice, QuestionnaireChoiceDefinition, QuestionnaireChoices,
    QuestionnaireDescription, QuestionnaireError as NativeError, QuestionnaireEvent,
    QuestionnaireInput, QuestionnaireInputDefinition, QuestionnaireItem,
    QuestionnaireItemDefinition, QuestionnaireNext, QuestionnairePrevious,
    QuestionnaireProgress as NativeProgress, QuestionnaireSkip, QuestionnaireState,
    QuestionnaireSubmit, QuestionnaireTitle,
};
use types::{
    ItemChildProps, MAX_CHOICES_PER_ITEM, MAX_ITEMS, QuestionnaireAnswerChange,
    QuestionnaireChange, QuestionnaireChoiceProps, QuestionnaireFocusChoice,
    QuestionnaireInputProps, QuestionnaireItemProps, QuestionnaireItemSnapshot,
    QuestionnaireItemStatus, QuestionnaireProps, QuestionnaireSetAnswer,
    QuestionnaireSetChoiceDisabled, QuestionnaireSetExternalError, QuestionnaireSetInputValue,
    QuestionnaireSetItemDisabled, QuestionnaireShortcutMode, QuestionnaireSnapshot,
    QuestionnaireSubmission, question_shape_errors,
};

/// One choice of a committed question: schema data plus the JS-painted card
/// content, which replaces the default label/description block when present.
pub(crate) struct QuestionnaireChoiceDef {
    props: QuestionnaireChoiceProps,
    style: StyleRefinement,
    content: Vec<AnyElement>,
}

/// The child a `QuestionnaireItem` holds. Both descriptors build this one type
/// so the framework can type-check the whole child list.
pub(crate) enum QuestionnaireItemChild {
    Choice(QuestionnaireChoiceDef),
    Input {
        props: QuestionnaireInputProps,
        style: StyleRefinement,
    },
}
impl Styled for QuestionnaireItemChild {
    fn style(&mut self) -> &mut StyleRefinement {
        match self {
            Self::Choice(choice) => &mut choice.style,
            // Upstream rejects style on the freeform input data node; keeping
            // the refinement here costs nothing and never reaches the skin.
            Self::Input { style, .. } => style,
        }
    }
}

/// A committed question: schema data with its typed children.
pub(crate) struct QuestionnaireItemDef {
    props: QuestionnaireItemProps,
    choices: Vec<QuestionnaireChoiceDef>,
    input: Option<QuestionnaireInputProps>,
}
impl QuestionnaireItemDef {
    fn name(&self) -> &str {
        &self.props.name
    }

    fn shape(&self) -> types::QuestionnaireItemShape<'_> {
        types::QuestionnaireItemShape::new(
            &self.props,
            self.choices.iter().map(|c| &c.props).collect(),
            self.input.as_ref(),
        )
    }
}

fn choice_def(p: &QuestionnaireChoiceProps, cx: &mut ElementContext) -> QuestionnaireItemChild {
    QuestionnaireItemChild::Choice(QuestionnaireChoiceDef {
        props: p.clone(),
        style: StyleRefinement::default(),
        content: cx.children().collect(),
    })
}
fn input_def(p: &QuestionnaireInputProps, _: &mut ElementContext) -> QuestionnaireItemChild {
    QuestionnaireItemChild::Input {
        props: p.clone(),
        style: StyleRefinement::default(),
    }
}
fn item_def(p: &QuestionnaireItemProps, cx: &mut ElementContext) -> QuestionnaireItemDef {
    let mut choices = Vec::new();
    let mut input = None;
    for child in cx.typed_children::<QuestionnaireItemChild>() {
        match child.native {
            QuestionnaireItemChild::Choice(choice) => choices.push(choice),
            QuestionnaireItemChild::Input { props, .. } => {
                // Publication validation admits at most one input; extras are
                // dropped here so the schema stays deterministic.
                input.replace(props);
            }
        }
    }
    QuestionnaireItemDef {
        props: p.clone(),
        choices,
        input,
    }
}

/// Decode one committed question's typed children: every child must be a
/// `QuestionnaireItemChild`, at most one of them the freeform input.
fn decode_item_children(
    summary: &ExtensionChildSummary,
) -> Result<
    (
        Vec<QuestionnaireChoiceProps>,
        Option<QuestionnaireInputProps>,
    ),
    String,
> {
    let child_type = std::any::TypeId::of::<QuestionnaireItemChild>();
    if summary
        .element_types
        .iter()
        .any(|actual| *actual != Some(child_type))
    {
        return Err(
            "QuestionnaireItem requires QuestionnaireChoice and QuestionnaireInput children".into(),
        );
    }
    let mut choices = Vec::new();
    let mut input = None;
    for index in 0..summary.count {
        let props = summary
            .properties
            .get(index)
            .copied()
            .flatten()
            .ok_or("expected native child props")?;
        match decode_json::<ItemChildProps>(crate::native::property_bytes(props)?)? {
            ItemChildProps::Choice(choice) => choices.push(choice),
            ItemChildProps::Input(child_input) => {
                if input.replace(child_input).is_some() {
                    return Err("QuestionnaireItem accepts at most one QuestionnaireInput".into());
                }
            }
        }
    }
    if choices.len() > MAX_CHOICES_PER_ITEM {
        return Err(format!(
            "a question supports at most {MAX_CHOICES_PER_ITEM} choices"
        ));
    }
    Ok((choices, input))
}

fn build_definitions(
    schema: &[QuestionnaireItemDef],
    inputs: &[(String, Entity<InputState>)],
) -> Vec<QuestionnaireItemDefinition> {
    schema
        .iter()
        .map(|item| {
            let mut definition = QuestionnaireItemDefinition::new(
                item.props.name.clone(),
                item.props.accessibility_label.clone(),
            )
            .with_required(item.props.required)
            .with_multiple(item.props.multiple)
            .with_disabled(item.props.disabled);
            if let Some(description) = &item.props.description {
                definition = definition.with_description(description.clone());
            }
            for choice in &item.choices {
                let mut answer = QuestionnaireChoiceDefinition::new(
                    choice.props.value.clone(),
                    choice.props.accessibility_label.clone(),
                )
                .with_disabled(choice.props.disabled)
                .with_default_selected(choice.props.default_selected);
                if let Some(description) = &choice.props.description {
                    answer = answer.with_description(description.clone());
                }
                definition = definition.with_choice(answer);
            }
            if let (Some(input), Some((_, state))) = (
                &item.input,
                inputs.iter().find(|(name, _)| name == item.name()),
            ) {
                definition = definition.with_input(
                    QuestionnaireInputDefinition::new(
                        state.clone(),
                        input.accessibility_label.clone(),
                    )
                    .with_disabled(input.disabled),
                );
            }
            definition
        })
        .collect()
}

fn create_inputs(
    schema: &[QuestionnaireItemDef],
    window: &mut Window,
    cx: &mut Context<Questionnaire>,
) -> Vec<(String, Entity<InputState>)> {
    schema
        .iter()
        .filter_map(|item| {
            item.input.as_ref().map(|input| {
                let entity = cx.new(|cx| {
                    // The freeform capacity is part of the serialized response
                    // reserve, so the engine rejects longer drafts while typing.
                    let mut state = InputState::new(window, cx)
                        .validate(|value, _| value.len() <= types::MAX_INPUT_TEXT_BYTES);
                    if let Some(placeholder) = &input.placeholder {
                        state = state.placeholder(placeholder.clone());
                    }
                    if let Some(default_value) = &input.default_value {
                        state = state.default_value(default_value.clone());
                    }
                    state
                });
                (item.props.name.clone(), entity)
            })
        })
        .collect()
}

/// What the retained state was built from. A question or choice that changed
/// semantically means a different questionnaire, and `QuestionnaireState`
/// fixes its schema at construction, so the state rebuilds rather than patches.
fn fingerprint(
    schema: &[QuestionnaireItemDef],
    shortcuts: Option<QuestionnaireShortcutMode>,
) -> u64 {
    use std::hash::{Hash, Hasher};
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    shortcuts.map(|mode| mode as u8).hash(&mut hasher);
    for item in schema {
        item.props.name.hash(&mut hasher);
        item.props.accessibility_label.hash(&mut hasher);
        item.props.description.hash(&mut hasher);
        (item.props.required, item.props.multiple).hash(&mut hasher);
        if let Some(input) = &item.input {
            input.accessibility_label.hash(&mut hasher);
            input.disabled.hash(&mut hasher);
            input.placeholder.hash(&mut hasher);
            input.default_value.hash(&mut hasher);
        }
        for choice in &item.choices {
            choice.props.value.hash(&mut hasher);
            choice.props.accessibility_label.hash(&mut hasher);
            choice.props.description.hash(&mut hasher);
            (choice.props.disabled, choice.props.default_selected).hash(&mut hasher);
        }
    }
    hasher.finish()
}

#[derive(Clone)]
struct QuestionnaireEvents {
    change: Event<QuestionnaireChange>,
    answer: Event<QuestionnaireAnswerChange>,
    complete: Event<QuestionnaireSubmission>,
    submit: Event<QuestionnaireSubmission>,
}

fn subscribe_events(
    events: QuestionnaireEvents,
    state: &Entity<QuestionnaireState>,
    cx: &mut Context<Questionnaire>,
) -> Subscription {
    cx.subscribe(state, move |_, _, event, cx| {
        match event {
            QuestionnaireEvent::CurrentItemChanged { previous, current } => {
                events.change.emit(QuestionnaireChange {
                    previous: previous.as_ref().map(|n| n.to_string()),
                    current: current.as_ref().map(|n| n.to_string()),
                });
            }
            QuestionnaireEvent::AnswerChanged(change) => {
                events.answer.emit(change.clone().into());
            }
            QuestionnaireEvent::Completed(submission) => {
                events.complete.emit(submission.clone().into());
            }
            QuestionnaireEvent::Submit(submission) => {
                events.submit.emit(submission.clone().into());
            }
            _ => {}
        }
        cx.notify();
    })
}

/// Construct the retained state from a validated schema. Free-standing so
/// mount can call it before the view entity exists.
fn build_state(
    shortcuts: Option<QuestionnaireShortcutMode>,
    initial_item: Option<&str>,
    schema: &[QuestionnaireItemDef],
    window: &mut Window,
    cx: &mut Context<Questionnaire>,
) -> Entity<QuestionnaireState> {
    let inputs = create_inputs(schema, window, cx);
    let definitions = build_definitions(schema, &inputs);
    cx.new(|cx| {
        let state = QuestionnaireState::new(definitions, cx)
            .expect("validated questionnaire schema must construct");
        let state = match shortcuts {
            Some(mode) => state.with_shortcuts(mode.into()),
            None => state,
        };
        match initial_item {
            Some(name) => state
                .with_current_item(name)
                .expect("validated initial item must exist"),
            None => state,
        }
    })
}

/// The retained questionnaire view. Native state is the authoritative editing
/// state; JS reads snapshots and events and mutates through commands.
struct Questionnaire {
    props: QuestionnaireProps,
    children: NativeChildren,
    schema: Vec<QuestionnaireItemDef>,
    state: Entity<QuestionnaireState>,
    fingerprint: u64,
    initialized: bool,
    events: Rc<QuestionnaireEvents>,
    _subscription: Subscription,
}

impl Questionnaire {
    /// Child descriptors mount in nondeterministic order within one commit, so
    /// the schema is read on the first paint, update or command instead of in
    /// `mount`. Until then the state is a valid empty questionnaire.
    fn ensure_initialized(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.initialized {
            return;
        }
        let schema = self.committed_schema(cx);
        self.rebuild(schema, window, cx);
        self.initialized = true;
    }

    fn committed_schema(&self, cx: &gpui::App) -> Vec<QuestionnaireItemDef> {
        self.children
            .typed_children::<QuestionnaireItemDef>(cx)
            .into_iter()
            .map(|child| child.native)
            .collect()
    }

    fn rebuild(
        &mut self,
        schema: Vec<QuestionnaireItemDef>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Keyed port: answers move across a schema edit when the question's
        // meaning is unchanged, keyed by question name.
        let previous: Vec<(String, qn::QuestionnaireAnswer)> = {
            let state = self.state.read(cx);
            self.schema
                .iter()
                .filter_map(|old| {
                    schema
                        .iter()
                        .find(|new| new.name() == old.name())
                        .is_some_and(|new| old.shape().answer_compatible(&new.shape()))
                        .then(|| {
                            state
                                .answer(old.name())
                                .filter(|answer| !answer.is_empty())
                                .map(|answer| (old.props.name.clone(), answer))
                        })
                        .flatten()
                })
                .collect()
        };
        let previous_current = self.state.read(cx).current_item().map(|n| n.to_string());

        self.state = build_state(
            self.props.shortcuts,
            self.props.initial_item.as_deref(),
            &schema,
            window,
            cx,
        );
        self.state.update(cx, |state, cx| {
            for (name, answer) in previous {
                let _ = state.set_answer(&name, answer, window, cx);
            }
            if let Some(current) = previous_current {
                let _ = state.set_current_item(&current, window, cx);
            }
        });
        self._subscription = subscribe_events((*self.events).clone(), &self.state, cx);
        self.fingerprint = fingerprint(&schema, self.props.shortcuts);
        self.schema = schema;
    }

    fn snapshot(&self, cx: &gpui::App) -> QuestionnaireSnapshot {
        let state = self.state.read(cx);
        QuestionnaireSnapshot {
            current: state.current_item().map(|n| n.to_string()),
            progress: state.progress().into(),
            navigation: state.navigation_state().into(),
            complete: state.is_complete(),
            items: self
                .schema
                .iter()
                .map(|item| {
                    let name = item.props.name.as_str();
                    let snapshot = state.item_state(name);
                    QuestionnaireItemSnapshot {
                        name: item.props.name.clone(),
                        status: snapshot
                            .as_ref()
                            .map(|s| s.status().into())
                            .unwrap_or(QuestionnaireItemStatus::Unanswered),
                        required: item.props.required,
                        multiple: item.props.multiple,
                        disabled: snapshot.as_ref().is_some_and(|s| s.is_disabled()),
                        invalid: snapshot.as_ref().is_some_and(|s| s.is_invalid()),
                        has_input: item.input.is_some(),
                        answer: state.answer(name).map(Into::into).unwrap_or_default(),
                        error: state.error(name).cloned().map(Into::into),
                        choices: item
                            .choices
                            .iter()
                            .filter_map(|choice| {
                                state
                                    .choice_state(name, &choice.props.value)
                                    .map(Into::into)
                            })
                            .collect(),
                    }
                })
                .collect(),
        }
    }
}

impl NativeView for Questionnaire {
    type Props = QuestionnaireProps;
    type Event = QuestionnaireChange;
    fn accepts_children() -> bool {
        true
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn additional_events() -> Vec<EventDefinition> {
        vec![
            EventDefinition::new::<QuestionnaireAnswerChange>("answerChange"),
            EventDefinition::new::<QuestionnaireSubmission>("complete"),
            EventDefinition::new::<QuestionnaireSubmission>("submit"),
        ]
    }
    fn validate_props(props: &QuestionnaireProps) -> Result<(), String> {
        props.validate()
    }
    fn validate_children(
        props: &QuestionnaireProps,
        children: &ExtensionChildSummary,
    ) -> Result<(), String> {
        props.validate()?;
        if children.count > MAX_ITEMS {
            return Err(format!(
                "a questionnaire supports at most {MAX_ITEMS} questions"
            ));
        }
        let items: Vec<QuestionnaireItemProps> = children.native_props()?;
        let mut shapes = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            let inner = children
                .child(index)
                .ok_or("missing QuestionnaireItem composition")?;
            let (choices, input) = decode_item_children(&inner)?;
            shapes.push((item.clone(), choices, input));
        }
        question_shape_errors(&shapes)?;
        if let Some(initial) = &props.initial_item
            && !items.iter().any(|item| &item.name == initial)
        {
            return Err("initialItem must address a committed question".into());
        }
        Ok(())
    }
    fn mount(
        props: QuestionnaireProps,
        event: Event<QuestionnaireChange>,
        children: NativeChildren,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let events = Rc::new(QuestionnaireEvents {
            answer: event.related("answerChange"),
            complete: event.related("complete"),
            submit: event.related("submit"),
            change: event,
        });
        // An empty schema is valid, so the seed state constructs without
        // touching the children; `ensure_initialized` swaps in the real flow.
        let shortcuts = props.shortcuts;
        let state = cx.new(|cx| {
            let state =
                QuestionnaireState::new(Vec::new(), cx).expect("an empty questionnaire is valid");
            match shortcuts {
                Some(mode) => state.with_shortcuts(mode.into()),
                None => state,
            }
        });
        let subscription = subscribe_events((*events).clone(), &state, cx);
        Self {
            props,
            children,
            schema: Vec::new(),
            state,
            fingerprint: fingerprint(&[], shortcuts),
            initialized: false,
            events,
            _subscription: subscription,
        }
    }
    fn update(&mut self, props: QuestionnaireProps, window: &mut Window, cx: &mut Context<Self>) {
        self.props = props;
        self.ensure_initialized(window, cx);
        let schema = self.committed_schema(cx);
        if fingerprint(&schema, self.props.shortcuts) != self.fingerprint {
            self.rebuild(schema, window, cx);
        } else {
            self.schema = schema;
        }
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        vec![
            ViewCommand::new("getState", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this.snapshot(cx))
            }),
            ViewCommand::new("setCurrentItem", |this, item: String, window, cx| {
                this.ensure_initialized(window, cx);
                types::valid_name(&item)?;
                this.state.update(cx, |state, cx| {
                    state
                        .set_current_item(&item, window, cx)
                        .map_err(|error| error.to_string())
                })?;
                Ok(())
            }),
            ViewCommand::new(
                "setAnswer",
                |this, args: QuestionnaireSetAnswer, window, cx| {
                    this.ensure_initialized(window, cx);
                    args.validate()?;
                    this.state.update(cx, |state, cx| {
                        state
                            .set_answer(&args.item, args.answer.into(), window, cx)
                            .map_err(|error| error.to_string())
                    })?;
                    Ok(())
                },
            ),
            ViewCommand::new(
                "setInputValue",
                |this, args: QuestionnaireSetInputValue, window, cx| {
                    this.ensure_initialized(window, cx);
                    args.validate()?;
                    this.state.update(cx, |state, cx| {
                        state
                            .set_input_value(&args.item, args.value, window, cx)
                            .map_err(|error| error.to_string())
                    })?;
                    Ok(())
                },
            ),
            ViewCommand::new(
                "setItemDisabled",
                |this, args: QuestionnaireSetItemDisabled, window, cx| {
                    this.ensure_initialized(window, cx);
                    types::valid_name(&args.item)?;
                    this.state.update(cx, |state, cx| {
                        state
                            .set_item_disabled(&args.item, args.disabled, window, cx)
                            .map_err(|error| error.to_string())
                    })?;
                    Ok(())
                },
            ),
            ViewCommand::new(
                "setChoiceDisabled",
                |this, args: QuestionnaireSetChoiceDisabled, window, cx| {
                    this.ensure_initialized(window, cx);
                    types::valid_name(&args.item)?;
                    types::valid_name(&args.value)?;
                    this.state.update(cx, |state, cx| {
                        state
                            .set_choice_disabled(&args.item, &args.value, args.disabled, cx)
                            .map_err(|error| error.to_string())
                    })?;
                    Ok(())
                },
            ),
            ViewCommand::new(
                "setExternalError",
                |this, args: QuestionnaireSetExternalError, window, cx| {
                    this.ensure_initialized(window, cx);
                    args.validate()?;
                    this.state.update(cx, |state, cx| {
                        state
                            .set_external_error(&args.item, args.message, cx)
                            .map_err(|error| error.to_string())
                    })?;
                    Ok(())
                },
            ),
            ViewCommand::new("clearExternalError", |this, item: String, window, cx| {
                this.ensure_initialized(window, cx);
                types::valid_name(&item)?;
                this.state.update(cx, |state, cx| {
                    state
                        .clear_external_error(&item, cx)
                        .map_err(|error| error.to_string())
                })?;
                Ok(())
            }),
            ViewCommand::new("goPrevious", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this
                    .state
                    .update(cx, |state, cx| state.go_previous(window, cx)))
            }),
            ViewCommand::new("goNext", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this.state.update(cx, |state, cx| state.go_next(window, cx)))
            }),
            ViewCommand::new("skip", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this
                    .state
                    .update(cx, |state, cx| state.skip_current(window, cx)))
            }),
            ViewCommand::new("confirm", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this
                    .state
                    .update(cx, |state, cx| state.confirm_current(window, cx)))
            }),
            ViewCommand::new("submit", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this.state.update(cx, |state, cx| state.submit(window, cx)))
            }),
            ViewCommand::new("reset", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                this.state.update(cx, |state, cx| state.reset(window, cx));
                Ok(())
            }),
            ViewCommand::new("focusCurrent", |this, (): (), window, cx| {
                this.ensure_initialized(window, cx);
                Ok(this
                    .state
                    .update(cx, |state, cx| state.focus_current_item(window, cx)))
            }),
            ViewCommand::new(
                "focusChoice",
                |this, args: QuestionnaireFocusChoice, window, cx| {
                    this.ensure_initialized(window, cx);
                    types::valid_name(&args.item)?;
                    types::valid_name(&args.value)?;
                    Ok(this.state.update(cx, |state, cx| {
                        state.focus_choice(&args.item, &args.value, window, cx)
                    }))
                },
            ),
            ViewCommand::new("focusInput", |this, item: String, window, cx| {
                this.ensure_initialized(window, cx);
                types::valid_name(&item)?;
                Ok(this
                    .state
                    .update(cx, |state, cx| state.focus_input(&item, window, cx)))
            }),
        ]
    }
}

impl Render for Questionnaire {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.ensure_initialized(window, cx);
        let schema = self.committed_schema(cx);
        let mut root = NativeQuestionnaire::new(&self.state).with_size(self.props.size);
        if self.props.show_progress {
            root = root.child(NativeProgress::new(&self.state));
        }
        for item in schema {
            let name = item.props.name.clone();
            let mut question = QuestionnaireItem::new(&self.state, name.clone())
                .child(QuestionnaireTitle::new(&self.state, name.clone()))
                .child(QuestionnaireDescription::new(&self.state, name.clone()));
            let mut answers = QuestionnaireChoices::new(&self.state, name.clone());
            for choice in item.choices {
                let mut element =
                    NativeChoice::new(&self.state, name.clone(), choice.props.value.clone());
                gpui::Refineable::refine(element.style(), &choice.style);
                if !choice.content.is_empty() {
                    // Custom card content keeps the native control; it replaces
                    // the default label/description block, as upstream.
                    element = element.children(choice.content);
                }
                answers = answers.child(element);
            }
            if item.input.is_some() {
                answers = answers.child(QuestionnaireInput::new(&self.state, name.clone()));
            }
            question = question
                .child(answers)
                .child(NativeError::new(&self.state, name.clone()));
            root = root.child(question);
        }
        if self.props.show_actions {
            let mut previous = QuestionnairePrevious::new(&self.state);
            if let Some(label) = &self.props.previous_label {
                previous = previous.child(label.clone());
            }
            let mut skip = QuestionnaireSkip::new(&self.state);
            if let Some(label) = &self.props.skip_label {
                skip = skip.child(label.clone());
            }
            let mut next = QuestionnaireNext::new(&self.state);
            if let Some(label) = &self.props.next_label {
                next = next.child(label.clone());
            }
            let mut submit = QuestionnaireSubmit::new(&self.state);
            if let Some(label) = &self.props.submit_label {
                submit = submit.child(label.clone());
            }
            root = root.child(
                QuestionnaireActions::new(&self.state)
                    .child(previous)
                    .child(skip)
                    .child(next)
                    .child(submit),
            );
        }
        root
    }
}

pub(super) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::descriptor::<QuestionnaireItemProps, _>(
            "QuestionnaireItem",
            vec![],
            item_def,
        )
        .with_child_type::<QuestionnaireItemChild>()
        .with_validation::<QuestionnaireItemProps>(|props, children| {
            props.validate()?;
            let (choices, input) = decode_item_children(children)?;
            let choice_refs: Vec<_> = choices.iter().collect();
            types::validate_item_shape(props, &choice_refs, input.as_ref())
        }),
        ComponentDefinition::styled_descriptor::<QuestionnaireChoiceProps, _>(
            "QuestionnaireChoice",
            vec![],
            choice_def,
        ),
        ComponentDefinition::styled_descriptor::<QuestionnaireInputProps, _>(
            "QuestionnaireInput",
            vec![],
            input_def,
        )
        .with_children(false),
        ComponentDefinition::view::<Questionnaire>("Questionnaire")
            .with_child_type::<QuestionnaireItemDef>(),
    ]
    .into_iter()
    .map(|definition| {
        definition.with_contract(concat!(
            include_str!("questionnaire.rs"),
            include_str!("questionnaire_types.rs")
        ))
    })
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use types::{
        MAX_INPUT_TEXT_BYTES, MAX_TEXT_BYTES, QuestionnaireAnswer, QuestionnaireInputProps,
    };

    fn item(name: &str) -> QuestionnaireItemProps {
        QuestionnaireItemProps {
            name: name.into(),
            accessibility_label: format!("{name} label"),
            ..Default::default()
        }
    }
    fn choice(value: &str) -> QuestionnaireChoiceProps {
        QuestionnaireChoiceProps {
            value: value.into(),
            accessibility_label: format!("{value} label"),
            ..Default::default()
        }
    }
    fn schema(
        items: &[(
            QuestionnaireItemProps,
            Vec<QuestionnaireChoiceProps>,
            Option<QuestionnaireInputProps>,
        )],
    ) -> Vec<QuestionnaireItemDef> {
        items
            .iter()
            .map(|(props, choices, input)| QuestionnaireItemDef {
                props: props.clone(),
                choices: choices
                    .iter()
                    .map(|props| QuestionnaireChoiceDef {
                        props: props.clone(),
                        style: StyleRefinement::default(),
                        content: Vec::new(),
                    })
                    .collect(),
                input: input.clone(),
            })
            .collect()
    }

    #[test]
    fn duplicate_names_and_choices_are_rejected_before_publication() {
        let duplicate_names = vec![item("same"), item("same")];
        let shapes: Vec<_> = duplicate_names
            .into_iter()
            .map(|item| (item, Vec::new(), None))
            .collect();
        assert_eq!(
            question_shape_errors(&shapes).unwrap_err(),
            "duplicate question name `same`"
        );

        let duplicate_choices = vec![(item("q"), vec![choice("a"), choice("a")], None)];
        assert_eq!(
            question_shape_errors(&duplicate_choices).unwrap_err(),
            "duplicate choice value `a` in question `q`"
        );

        let single_choice_multiple_defaults = vec![(
            item("q"),
            vec![
                QuestionnaireChoiceProps {
                    default_selected: true,
                    ..choice("a")
                },
                QuestionnaireChoiceProps {
                    default_selected: true,
                    ..choice("b")
                },
            ],
            None,
        )];
        assert_eq!(
            question_shape_errors(&single_choice_multiple_defaults).unwrap_err(),
            "single-choice question `q` has more than one defaultSelected"
        );
    }

    #[test]
    fn multiple_choice_items_admit_any_number_of_defaults() {
        let multiple_defaults = vec![(
            QuestionnaireItemProps {
                multiple: true,
                ..item("q")
            },
            vec![
                QuestionnaireChoiceProps {
                    default_selected: true,
                    ..choice("a")
                },
                QuestionnaireChoiceProps {
                    default_selected: true,
                    ..choice("b")
                },
            ],
            None,
        )];
        assert!(question_shape_errors(&multiple_defaults).is_ok());
    }

    #[test]
    fn response_budget_bounds_the_whole_schema() {
        // Every field stays inside its own bound; only the serialized reserve
        // for a full response trips.
        let label = "x".repeat(4000);
        let mut shapes = Vec::new();
        for index in 0..64 {
            shapes.push((
                QuestionnaireItemProps {
                    accessibility_label: label.clone(),
                    ..item(&index.to_string())
                },
                Vec::new(),
                None,
            ));
        }
        assert!(shapes.len() <= MAX_ITEMS);
        assert!(question_shape_errors(&shapes).is_err());
    }

    #[test]
    fn answers_port_only_across_meaning_preserving_edits() {
        let base = schema(&[(
            item("q"),
            vec![choice("a"), choice("b")],
            Some(QuestionnaireInputProps::default()),
        )]);

        // A label edit keeps the answer meaningful.
        let relabeled = schema(&[(
            QuestionnaireItemProps {
                accessibility_label: "New title".into(),
                ..item("q")
            },
            vec![choice("a"), choice("b")],
            Some(QuestionnaireInputProps::default()),
        )]);
        assert!(base[0].shape().answer_compatible(&relabeled[0].shape()));

        // Toggling required or multiple changes what the answer means.
        let required = schema(&[(
            QuestionnaireItemProps {
                required: true,
                ..item("q")
            },
            vec![choice("a"), choice("b")],
            Some(QuestionnaireInputProps::default()),
        )]);
        assert!(!base[0].shape().answer_compatible(&required[0].shape()));

        // Replacing the answer set invalidates the keyed answer.
        let revalued = schema(&[(
            item("q"),
            vec![choice("a"), choice("c")],
            Some(QuestionnaireInputProps::default()),
        )]);
        assert!(!base[0].shape().answer_compatible(&revalued[0].shape()));

        // Dropping the freeform input invalidates it too.
        let inputless = schema(&[(item("q"), vec![choice("a"), choice("b")], None)]);
        assert!(!base[0].shape().answer_compatible(&inputless[0].shape()));
    }

    #[test]
    fn bounded_answers_and_text_reject_oversized_payloads() {
        let oversized = QuestionnaireAnswer {
            freeform: Some("x".repeat(MAX_INPUT_TEXT_BYTES + 1)),
            choices: Vec::new(),
        };
        assert!(oversized.validate().is_err());

        let too_many = QuestionnaireAnswer {
            freeform: None,
            choices: (0..=MAX_CHOICES_PER_ITEM).map(|i| i.to_string()).collect(),
        };
        assert!(too_many.validate().is_err());

        let valid = QuestionnaireAnswer {
            freeform: Some("draft".into()),
            choices: vec!["a".into()],
        };
        assert!(valid.validate().is_ok());

        let oversized_label = QuestionnaireItemProps {
            accessibility_label: "x".repeat(MAX_TEXT_BYTES + 1),
            ..item("q")
        };
        assert!(oversized_label.validate().is_err());
    }
}
