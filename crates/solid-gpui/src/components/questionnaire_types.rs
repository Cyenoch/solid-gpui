//! Bounded data contracts and schema rules for the native questionnaire flow.
//! Pure values and rules only: the retained view in `questionnaire.rs` owns
//! elements, state entities and framework glue.
use std::collections::HashSet;

use crate::native::Deserialize;
use gpui_component::questionnaire as qn;

pub(crate) const MAX_ITEMS: usize = 256;
pub(crate) const MAX_CHOICES_PER_ITEM: usize = 128;
pub(crate) const MAX_NAME_BYTES: usize = 256;
pub(crate) const MAX_TEXT_BYTES: usize = 4096;
pub(crate) const MAX_INPUT_TEXT_BYTES: usize = 8192;
/// Aggregate budget for every schema text (names, labels, descriptions, input
/// seeds). Bounds keep a full snapshot or submission far inside the 1 MiB
/// native-call wire budget before the tree is published.
pub(crate) const MAX_SCHEMA_TEXT_BYTES: usize = 512 * 1024;

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum QuestionnaireShortcutMode {
    Letters,
    Numbers,
}
impl From<QuestionnaireShortcutMode> for qn::QuestionnaireShortcutMode {
    fn from(value: QuestionnaireShortcutMode) -> Self {
        match value {
            QuestionnaireShortcutMode::Letters => Self::Letters,
            QuestionnaireShortcutMode::Numbers => Self::Numbers,
        }
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub(crate) enum QuestionnaireItemStatus {
    Unanswered,
    Answered,
    Skipped,
}
impl From<qn::QuestionnaireItemStatus> for QuestionnaireItemStatus {
    fn from(value: qn::QuestionnaireItemStatus) -> Self {
        match value {
            qn::QuestionnaireItemStatus::Unanswered => Self::Unanswered,
            qn::QuestionnaireItemStatus::Answered => Self::Answered,
            qn::QuestionnaireItemStatus::Skipped => Self::Skipped,
        }
    }
}

/// One answer: also a command payload, so missing optional fields decode.
#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireAnswer {
    pub choices: Vec<String>,
    pub freeform: Option<String>,
}
impl QuestionnaireAnswer {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if self.choices.len() > MAX_CHOICES_PER_ITEM {
            return Err(format!(
                "an answer selects at most {MAX_CHOICES_PER_ITEM} choices"
            ));
        }
        for choice in &self.choices {
            valid_name(choice)
                .map_err(|_| "answer choice values must be valid names".to_owned())?;
        }
        if let Some(freeform) = &self.freeform
            && freeform.len() > MAX_INPUT_TEXT_BYTES
        {
            return Err(format!(
                "freeform answers may contain at most {MAX_INPUT_TEXT_BYTES} bytes"
            ));
        }
        Ok(())
    }
}
impl From<qn::QuestionnaireAnswer> for QuestionnaireAnswer {
    fn from(value: qn::QuestionnaireAnswer) -> Self {
        Self {
            choices: value.choices().iter().map(|v| v.to_string()).collect(),
            freeform: value.freeform().map(|v| v.to_string()),
        }
    }
}
impl From<QuestionnaireAnswer> for qn::QuestionnaireAnswer {
    fn from(value: QuestionnaireAnswer) -> Self {
        let mut answer = qn::QuestionnaireAnswer::new().with_choices(value.choices);
        if let Some(freeform) = value.freeform {
            answer = answer.with_freeform(freeform);
        }
        answer
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireProgress {
    pub current: usize,
    pub total: usize,
}
impl From<qn::QuestionnaireProgressState> for QuestionnaireProgress {
    fn from(value: qn::QuestionnaireProgressState) -> Self {
        Self {
            current: value.current(),
            total: value.total(),
        }
    }
}

#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireNavigation {
    pub previous_visible: bool,
    pub next_visible: bool,
    pub skip_visible: bool,
    pub submit_visible: bool,
    pub confirmable: bool,
}
impl From<qn::QuestionnaireNavigationState> for QuestionnaireNavigation {
    fn from(value: qn::QuestionnaireNavigationState) -> Self {
        Self {
            previous_visible: value.is_previous_visible(),
            next_visible: value.is_next_visible(),
            skip_visible: value.is_skip_visible(),
            submit_visible: value.is_submit_visible(),
            confirmable: value.is_confirmable(),
        }
    }
}

/// Why an item currently fails validation; built-in reasons carry no text, so
/// the host supplies the sentence it wants to show.
#[crate::native_type]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum QuestionnaireErrorReason {
    Required,
    Unanswered,
    Message,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireError {
    pub reason: QuestionnaireErrorReason,
    pub message: Option<String>,
}
impl From<qn::QuestionnaireValidationError> for QuestionnaireError {
    fn from(value: qn::QuestionnaireValidationError) -> Self {
        match value {
            qn::QuestionnaireValidationError::Required => Self {
                reason: QuestionnaireErrorReason::Required,
                message: None,
            },
            qn::QuestionnaireValidationError::Unanswered => Self {
                reason: QuestionnaireErrorReason::Unanswered,
                message: None,
            },
            _ => Self {
                reason: QuestionnaireErrorReason::Message,
                message: value.message().map(|m| m.to_string()),
            },
        }
    }
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireChoiceSnapshot {
    pub value: String,
    pub selected: bool,
    pub disabled: bool,
    pub invalid: bool,
    pub shortcut: Option<String>,
}
impl From<qn::QuestionnaireChoiceState> for QuestionnaireChoiceSnapshot {
    fn from(value: qn::QuestionnaireChoiceState) -> Self {
        Self {
            value: value.value().to_string(),
            selected: value.is_selected(),
            disabled: value.is_disabled(),
            invalid: value.is_invalid(),
            shortcut: value.shortcut().map(|s| s.to_string()),
        }
    }
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireItemSnapshot {
    pub name: String,
    pub status: QuestionnaireItemStatus,
    pub required: bool,
    pub multiple: bool,
    pub disabled: bool,
    pub invalid: bool,
    pub has_input: bool,
    pub answer: QuestionnaireAnswer,
    pub error: Option<QuestionnaireError>,
    pub choices: Vec<QuestionnaireChoiceSnapshot>,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireSnapshot {
    pub current: Option<String>,
    pub progress: QuestionnaireProgress,
    pub navigation: QuestionnaireNavigation,
    pub complete: bool,
    pub items: Vec<QuestionnaireItemSnapshot>,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireSubmissionItem {
    pub name: String,
    pub status: QuestionnaireItemStatus,
    pub answer: QuestionnaireAnswer,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireSubmission {
    pub items: Vec<QuestionnaireSubmissionItem>,
}
impl From<qn::QuestionnaireSubmission> for QuestionnaireSubmission {
    fn from(value: qn::QuestionnaireSubmission) -> Self {
        Self {
            items: value
                .items()
                .iter()
                .map(|item| QuestionnaireSubmissionItem {
                    name: item.name().to_string(),
                    status: item.status().into(),
                    answer: item.answer().clone().into(),
                })
                .collect(),
        }
    }
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireAnswerChange {
    pub item: String,
    pub answer: QuestionnaireAnswer,
    pub status: QuestionnaireItemStatus,
}
impl From<qn::QuestionnaireAnswerChange> for QuestionnaireAnswerChange {
    fn from(value: qn::QuestionnaireAnswerChange) -> Self {
        Self {
            item: value.item().to_string(),
            answer: value.answer().clone().into(),
            status: value.status().into(),
        }
    }
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireChange {
    pub previous: Option<String>,
    pub current: Option<String>,
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireItemProps {
    pub name: String,
    pub accessibility_label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub multiple: bool,
    #[serde(default)]
    pub disabled: bool,
}
impl QuestionnaireItemProps {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_name(&self.name)?;
        valid_text("accessibilityLabel", &self.accessibility_label)?;
        if let Some(description) = &self.description {
            valid_optional_text("description", description)?;
        }
        Ok(())
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireChoiceProps {
    pub value: String,
    pub accessibility_label: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub default_selected: bool,
}
impl QuestionnaireChoiceProps {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_name(&self.value)?;
        valid_text("accessibilityLabel", &self.accessibility_label)?;
        if let Some(description) = &self.description {
            valid_optional_text("description", description)?;
        }
        Ok(())
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct QuestionnaireInputProps {
    pub accessibility_label: String,
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub placeholder: Option<String>,
    #[serde(default)]
    pub default_value: Option<String>,
}
impl QuestionnaireInputProps {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_text("accessibilityLabel", &self.accessibility_label)?;
        if let Some(placeholder) = &self.placeholder {
            valid_optional_text("placeholder", placeholder)?;
        }
        if let Some(default_value) = &self.default_value
            && default_value.len() > MAX_INPUT_TEXT_BYTES
        {
            return Err(format!(
                "defaultValue may contain at most {MAX_INPUT_TEXT_BYTES} bytes"
            ));
        }
        Ok(())
    }
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireProps {
    pub shortcuts: Option<QuestionnaireShortcutMode>,
    pub size: crate::components::ControlSize,
    pub show_progress: bool,
    pub show_actions: bool,
    pub initial_item: Option<String>,
    pub previous_label: Option<String>,
    pub skip_label: Option<String>,
    pub next_label: Option<String>,
    pub submit_label: Option<String>,
}
impl Default for QuestionnaireProps {
    fn default() -> Self {
        Self {
            shortcuts: None,
            size: crate::components::ControlSize::default(),
            show_progress: true,
            show_actions: true,
            initial_item: None,
            previous_label: None,
            skip_label: None,
            next_label: None,
            submit_label: None,
        }
    }
}
impl QuestionnaireProps {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if let Some(initial) = &self.initial_item {
            valid_name(initial)?;
        }
        for (field, label) in [
            ("previousLabel", &self.previous_label),
            ("skipLabel", &self.skip_label),
            ("nextLabel", &self.next_label),
            ("submitLabel", &self.submit_label),
        ] {
            if let Some(label) = label {
                valid_optional_text(field, label)?;
            }
        }
        Ok(())
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireSetAnswer {
    pub item: String,
    pub answer: QuestionnaireAnswer,
}
impl QuestionnaireSetAnswer {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_name(&self.item)?;
        self.answer.validate()
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireSetInputValue {
    pub item: String,
    pub value: String,
}
impl QuestionnaireSetInputValue {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_name(&self.item)?;
        if self.value.len() > MAX_INPUT_TEXT_BYTES {
            return Err(format!(
                "value may contain at most {MAX_INPUT_TEXT_BYTES} bytes"
            ));
        }
        Ok(())
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireSetItemDisabled {
    pub item: String,
    pub disabled: bool,
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireSetChoiceDisabled {
    pub item: String,
    pub value: String,
    pub disabled: bool,
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireSetExternalError {
    pub item: String,
    pub message: String,
}
impl QuestionnaireSetExternalError {
    pub(crate) fn validate(&self) -> Result<(), String> {
        valid_name(&self.item)?;
        valid_optional_text("message", &self.message)
    }
}

#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(default, rename_all = "camelCase")]
pub struct QuestionnaireFocusChoice {
    pub item: String,
    pub value: String,
}

pub(crate) fn valid_name(value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err("names must not be empty".into())
    } else if value.len() > MAX_NAME_BYTES {
        Err(format!("names may contain at most {MAX_NAME_BYTES} bytes"))
    } else {
        Ok(())
    }
}
pub(crate) fn valid_text(field: &str, value: &str) -> Result<(), String> {
    if value.trim().is_empty() {
        Err(format!("{field} must not be empty"))
    } else if value.len() > MAX_TEXT_BYTES {
        Err(format!(
            "{field} may contain at most {MAX_TEXT_BYTES} bytes"
        ))
    } else {
        Ok(())
    }
}
pub(crate) fn valid_optional_text(field: &str, value: &str) -> Result<(), String> {
    if value.len() > MAX_TEXT_BYTES {
        Err(format!(
            "{field} may contain at most {MAX_TEXT_BYTES} bytes"
        ))
    } else {
        Ok(())
    }
}

/// What one committed question's shape means for keyed answer preservation.
pub(crate) struct QuestionnaireItemShape<'a> {
    props: &'a QuestionnaireItemProps,
    choices: Vec<&'a QuestionnaireChoiceProps>,
    input: Option<&'a QuestionnaireInputProps>,
}
impl<'a> QuestionnaireItemShape<'a> {
    pub(crate) fn new(
        props: &'a QuestionnaireItemProps,
        choices: Vec<&'a QuestionnaireChoiceProps>,
        input: Option<&'a QuestionnaireInputProps>,
    ) -> Self {
        Self {
            props,
            choices,
            input,
        }
    }

    /// Answers port across a schema edit only when the answer's meaning is
    /// unchanged: same toggles and same choice values, in the same order.
    /// Accessibility text, descriptions and per-choice default selection may
    /// change freely. Disabled flags are ignored: the state's effective answer
    /// already excludes disabled choices, and `set_answer` re-validates.
    pub(crate) fn answer_compatible(&self, other: &QuestionnaireItemShape<'_>) -> bool {
        self.props.required == other.props.required
            && self.props.multiple == other.props.multiple
            && self.choices.len() == other.choices.len()
            && self
                .choices
                .iter()
                .zip(other.choices.iter())
                .all(|(old, new)| old.value == new.value)
            && self.input.is_some() == other.input.is_some()
    }
}

/// Validation-time discrimination of the two child kinds by required props: a
/// choice requires `value` and `accessibilityLabel`, an input only the label.
#[derive(Deserialize)]
#[serde(untagged)]
pub(crate) enum ItemChildProps {
    Choice(QuestionnaireChoiceProps),
    Input(QuestionnaireInputProps),
}

/// Schema rules shared by publication validation and the retained-state
/// constructor: per-question shape, unique names and values, single-choice
/// defaults, the aggregate text budget and the serialized response reserve.
pub(crate) fn question_shape_errors(
    items: &[(
        QuestionnaireItemProps,
        Vec<QuestionnaireChoiceProps>,
        Option<QuestionnaireInputProps>,
    )],
) -> Result<(), String> {
    if items.len() > MAX_ITEMS {
        return Err(format!(
            "a questionnaire supports at most {MAX_ITEMS} questions"
        ));
    }
    let mut names = HashSet::new();
    let mut text_bytes = 0usize;
    // Reserve worst-case JSON escaping, response structure, external errors and
    // the full editable capacity of every input before accepting the schema.
    let mut response_budget = 1024usize;
    for (item, choices, input) in items {
        item.validate()?;
        if !names.insert(item.name.as_str()) {
            return Err(format!("duplicate question name `{}`", item.name));
        }
        text_bytes += item.name.len() + item.accessibility_label.len();
        text_bytes += item.description.as_ref().map_or(0, |d| d.len());
        response_budget += 1024 + 6 * (item.name.len() + MAX_TEXT_BYTES);
        let mut values = HashSet::new();
        let mut defaults = 0;
        if choices.len() > MAX_CHOICES_PER_ITEM {
            return Err(format!(
                "a question supports at most {MAX_CHOICES_PER_ITEM} choices"
            ));
        }
        for choice in choices {
            choice.validate()?;
            text_bytes += choice.value.len() + choice.accessibility_label.len();
            text_bytes += choice.description.as_ref().map_or(0, |d| d.len());
            response_budget += 256 + 12 * choice.value.len();
            if !values.insert(choice.value.as_str()) {
                return Err(format!(
                    "duplicate choice value `{}` in question `{}`",
                    choice.value, item.name
                ));
            }
            defaults += usize::from(choice.default_selected);
        }
        if !item.multiple && defaults > 1 {
            return Err(format!(
                "single-choice question `{}` has more than one defaultSelected",
                item.name
            ));
        }
        if let Some(input) = input {
            response_budget += 6 * MAX_INPUT_TEXT_BYTES;
            input.validate()?;
            text_bytes += input.accessibility_label.len();
            text_bytes += input.placeholder.as_ref().map_or(0, |p| p.len());
            text_bytes += input.default_value.as_ref().map_or(0, |d| d.len());
        }
    }
    if text_bytes > MAX_SCHEMA_TEXT_BYTES {
        return Err(format!(
            "questionnaire text may total at most {MAX_SCHEMA_TEXT_BYTES} bytes"
        ));
    }
    if response_budget > crate::native::MAX_NATIVE_CALL_BYTES {
        return Err("questionnaire state exceeds the 1 MiB response budget".into());
    }
    Ok(())
}

/// Per-question rules over borrowed choices, for the item descriptor's own
/// publication validation.
pub(crate) fn validate_item_shape(
    item: &QuestionnaireItemProps,
    choices: &[&QuestionnaireChoiceProps],
    input: Option<&QuestionnaireInputProps>,
) -> Result<(), String> {
    item.validate()?;
    let mut values = HashSet::new();
    let mut defaults = 0;
    for choice in choices {
        choice.validate()?;
        if !values.insert(choice.value.as_str()) {
            return Err(format!(
                "duplicate choice value `{}` in question `{}`",
                choice.value, item.name
            ));
        }
        defaults += usize::from(choice.default_selected);
    }
    if !item.multiple && defaults > 1 {
        return Err(format!(
            "single-choice question `{}` has more than one defaultSelected",
            item.name
        ));
    }
    if let Some(input) = input {
        input.validate()?;
    }
    Ok(())
}
