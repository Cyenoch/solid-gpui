//! Retained text controls share the native editing engine and acknowledgement protocol.
use super::ControlSize;
use crate::native::{
    ComponentDefinition, ControlledBinding, Event, EventDefinition, NativeChildren, NativeView,
    ViewCommand,
};
use gpui::Window;
use gpui::{
    AnyElement, App, AppContext, Context, Entity, EntityInputHandler, IntoElement, Render,
    Subscription, Task,
};
use gpui_base::input::{
    EditorMode, InputBaseState, InputMode, InputModeKind, NumberStep, TabSize, TextareaMode,
};
use gpui_component::input::{
    InlineToken, InlineTokenSpan, Input as GpuiInput, InputContent, InputEvent, InputGroupControl,
    InputState,
};
use gpui_component::{Disableable, Sizable};
use std::time::Duration;

pub(crate) trait TextProps:
    serde::de::DeserializeOwned + crate::native::TS + 'static
{
    fn value(&self) -> &Option<String>;
    fn default_value(&self) -> &Option<String>;
    fn placeholder(&self) -> &Option<String>;
    fn disabled(&self) -> bool;
    fn readonly(&self) -> bool;
    fn ack_edit_seq(&self) -> u32;
    fn content(&self) -> Option<&InputContentSnapshot>;
}
macro_rules! text_props {
    ($name:ident { $($field:ident : $ty:ty = $default:expr),* $(,)? }) => {
        #[crate::native_type]
        #[derive(Clone, Debug)]
        #[serde(default, rename_all = "camelCase")]
        pub struct $name {
            pub value: Option<String>, pub default_value: Option<String>,
            pub placeholder: Option<String>, pub disabled: bool, pub readonly: bool,
            pub ack_edit_seq: u32, pub appearance: bool,
            pub content: Option<InputContentSnapshot>,
            $(pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> Self { Self { value: None, default_value: None, placeholder: None, disabled: false, readonly: false, ack_edit_seq: 0, appearance: true, content: None, $($field: $default,)* } }
        }
        impl TextProps for $name {
            fn value(&self) -> &Option<String> { &self.value }
            fn default_value(&self) -> &Option<String> { &self.default_value }
            fn placeholder(&self) -> &Option<String> { &self.placeholder }
            fn disabled(&self) -> bool { self.disabled }
            fn readonly(&self) -> bool { self.readonly }
            fn ack_edit_seq(&self) -> u32 { self.ack_edit_seq }
            fn content(&self) -> Option<&InputContentSnapshot> { self.content.as_ref() }
        }
    }
}
text_props!(InputProps { size: ControlSize = ControlSize::Medium, bordered: bool = true, aria_label: Option<String> = None, masked: bool = false, cleanable: bool = false, mask_toggle: bool = false });
text_props!(NumberInputProps { size: ControlSize = ControlSize::Medium, step: f64 = 1., min: Option<f64> = None, max: Option<f64> = None });
text_props!(TextareaProps { bordered: bool = true, aria_label: Option<String> = None, rows: usize = 2, auto_grow: Option<AutoGrow> = None, soft_wrap: bool = true, searchable: bool = false });
text_props!(EditorProps { bordered: bool = true, aria_label: Option<String> = None, language: String = String::new(), soft_wrap: bool = true, searchable: bool = true, line_numbers: bool = true, folding: bool = true, indent_guides: bool = true, tab_size: usize = 2, hard_tabs: bool = false, show_whitespaces: bool = false, scroll_beyond_last_line: Option<usize> = None, cursor_surrounding_lines: Option<usize> = None, auto_close: bool = true, smart_indent: bool = true });

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AutoGrow {
    pub min_rows: usize,
    pub max_rows: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputChange {
    pub value: String,
    pub edit_seq: u32,
    /// The full text-plus-token snapshot at commit time, present only for
    /// controls that carry atomic inline tokens, so a controlled consumer can
    /// round-trip `content` without a racing `getContent` call.
    #[serde(default)]
    pub content: Option<InputContentSnapshot>,
}
#[crate::native_type]
#[derive(Clone, Debug)]
pub struct InputSubmit {
    pub secondary: bool,
    pub shift: bool,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSelections {
    pub ranges: Vec<InputSelectionRange>,
    pub edit_seq: u32,
}
#[crate::native_type]
#[derive(Clone, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InputSelectionRange {
    pub anchor_byte: usize,
    pub head_byte: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputSearchRange {
    pub anchor_byte: usize,
    pub head_byte: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputSearchQuery {
    pub query: String,
    pub case_insensitive: bool,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputSearchReplacement {
    pub replacement: String,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputSearchReplaceAll {
    pub count: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputSearchSession {
    pub open: bool,
    pub active: bool,
    pub replace_mode: bool,
    pub case_insensitive: bool,
    pub query: String,
    pub replacement: String,
    pub match_count: usize,
    pub current_match_index: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputTokenSpec {
    pub id: String,
    pub text: String,
    #[serde(default)]
    pub label: Option<String>,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputTokenRangeSpec {
    pub token: InputTokenSpec,
    pub anchor_byte: usize,
    pub head_byte: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputTokenSpan {
    pub id: String,
    pub text: String,
    pub label: String,
    pub anchor_byte: usize,
    pub head_byte: usize,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct InputContentSnapshot {
    pub text: String,
    pub tokens: Vec<InputTokenSpan>,
}
#[crate::native_type]
#[derive(Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputPasteImage {
    pub format: String,
    pub byte_length: usize,
    pub data: Option<String>,
}
#[crate::native_type]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InputPaste {
    pub text: Option<String>,
    pub text_truncated: bool,
    pub image: Option<InputPasteImage>,
    pub files: Vec<String>,
}

/// A paste payload never exceeds one encoded extension event. Text, file
/// names, and base64 image data are bounded so the total stays far below the
/// transport limit; oversized images arrive as metadata only.
const PASTE_TEXT_LIMIT: usize = 64 * 1024;
const PASTE_IMAGE_DATA_LIMIT: usize = 192 * 1024;
const PASTE_FILE_COUNT_LIMIT: usize = 32;
const PASTE_FILE_BYTES_LIMIT: usize = 32 * 1024;
const TOKEN_COUNT_LIMIT: usize = 256;
/// A token-carrying document reserves 32KiB of text plus 32KiB of token
/// id/text/label metadata. Doubled text (value and content.text), 6x JSON
/// escaping, and at most 256 token structures stay far below one encoded
/// native event, so user typing can never overflow `Event::emit`.
const TOKEN_TEXT_LIMIT: usize = 32 * 1024;
const TOKEN_METADATA_LIMIT: usize = 32 * 1024;

pub(crate) trait TextMode: Sized + 'static {
    type Mode: InputModeKind;
    type Props: TextProps;
    const SLOTS: &'static [&'static str] = &[];
    /// The engine's paste hook exists for every mode except the numeric
    /// field, whose engine rejects pasted text anyway.
    const SUPPORTS_PASTE: bool = false;
    /// Atomic inline tokens exist only where the upstream engine defines
    /// them: the single-line and multi-line plain-text modes.
    const SUPPORTS_TOKENS: bool = false;
    fn new(
        window: &mut Window,
        cx: &mut Context<InputBaseState<Self::Mode>>,
    ) -> InputBaseState<Self::Mode>;
    fn sync(
        state: &mut InputBaseState<Self::Mode>,
        props: &Self::Props,
        old: Option<&Self::Props>,
        window: &mut Window,
        cx: &mut Context<InputBaseState<Self::Mode>>,
    );
    fn render(
        state: &Entity<InputBaseState<Self::Mode>>,
        props: &Self::Props,
        children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> AnyElement;
    fn validate(_props: &Self::Props) -> Result<(), String> {
        Ok(())
    }
    /// Commands for modes whose engine carries atomic inline tokens.
    fn token_commands() -> Vec<ViewCommand<TextControl<Self>>>
    where
        Self: Sized,
    {
        Vec::new()
    }
    /// The text-plus-token snapshot for change events; `None` where the mode
    /// carries no tokens.
    fn content_snapshot(
        _state: &Entity<InputBaseState<Self::Mode>>,
        _cx: &App,
    ) -> Option<InputContentSnapshot>
    where
        Self: Sized,
    {
        None
    }
    /// The typed upstream control this mode contributes to an input group,
    /// carrying every configured capability. `None` where the upstream group
    /// does not accept the mode.
    fn group_control(
        _state: &Entity<InputBaseState<Self::Mode>>,
        _props: &Self::Props,
        _children: &NativeChildren,
        _events: &Event<InputChange>,
    ) -> Option<InputGroupControl>
    where
        Self: Sized,
    {
        None
    }
}
pub(crate) struct SingleLine;
pub(crate) struct Numeric;
pub(crate) struct MultiLine;
pub(crate) struct Code;
#[cfg(test)]
type Input = TextControl<SingleLine>;

struct PendingDocument {
    content: InputContent,
    ack_edit_seq: u32,
}

pub(crate) struct TextControl<M: TextMode> {
    state: Entity<InputBaseState<M::Mode>>,
    props: M::Props,
    event: Event<InputChange>,
    edit_seq: u32,
    committed_value: String,
    /// The last emitted token snapshot; token-only edits with identical text
    /// compare against it before suppressing a change event.
    committed_content: Option<InputContentSnapshot>,
    pending: Option<PendingDocument>,
    retry_task: Option<Task<()>>,
    _subscription: Subscription,
    children: NativeChildren,
    /// The frame editing overlay a surrounding input group applies, keyed by
    /// the owning group's entity id. The group's readonly/disabled joins the
    /// control's own props; lifting the overlay restores exactly the
    /// requested values.
    group_editing: Option<(gpui::EntityId, bool, bool)>,
    applied_editing: (bool, bool),
    token_limits_installed: bool,
}

impl<M: TextMode> NativeView for TextControl<M> {
    type Props = M::Props;
    type Event = InputChange;

    fn mount(
        props: M::Props,
        event: Event<InputChange>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let content = props
            .content()
            .map(|content| build_content(content).expect("validated controlled content"));
        let state = cx.new(|cx| {
            let mut state = match &content {
                // A controlled snapshot restores text and its tokens together.
                Some(content) => {
                    let mut state = M::new(window, cx);
                    state.set_value(content.clone(), window, cx);
                    state
                }
                None => {
                    let initial = props
                        .value()
                        .as_deref()
                        .or(props.default_value().as_deref())
                        .unwrap_or_default();
                    M::new(window, cx).default_value(initial)
                }
            };
            state.set_placeholder(props.placeholder().clone().unwrap_or_default(), window, cx);
            state.set_disabled(props.disabled(), cx);
            state.set_readonly(props.readonly(), cx);
            M::sync(&mut state, &props, None, window, cx);
            state
        });
        let committed_value = state.read(cx).value().to_string();
        let committed_content = M::content_snapshot(&state, cx);
        let subscription =
            cx.subscribe_in(&state, window, |this, state, event, _, cx| match event {
                InputEvent::Change => {
                    let value = state.read(cx).value().to_string();
                    let content = M::content_snapshot(state, cx);
                    this.observe_committed_value(value, content);
                    this.pending = None;
                    this.retry_task.take();
                }
                InputEvent::Focus => this.event.related("focus").emit(()),
                InputEvent::Blur => this.event.related("blur").emit(()),
                InputEvent::PressEnter { secondary, shift } => {
                    this.event.related("submit").emit(InputSubmit {
                        secondary: *secondary,
                        shift: *shift,
                    })
                }
            });
        let applied_editing = (props.disabled(), props.readonly());
        let token_limits_installed = props
            .content()
            .is_some_and(|content| !content.tokens.is_empty());
        if token_limits_installed {
            state.update(cx, |state, cx| state.set_validator(live_token_limit(), cx));
        }
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            committed_value,
            committed_content,
            pending: None,
            retry_task: None,
            _subscription: subscription,
            children,
            group_editing: None,
            applied_editing,
            token_limits_installed,
        }
    }

    fn update(&mut self, props: M::Props, window: &mut Window, cx: &mut Context<Self>) {
        self.retry_task.take();
        self.pending = None;
        let entering_controlled = self.props.value().is_none()
            && self.props.content().is_none()
            && (props.value().is_some() || props.content().is_some());
        if self.props.placeholder() != props.placeholder() {
            self.state.update(cx, |state, cx| {
                state.set_placeholder(props.placeholder().clone().unwrap_or_default(), window, cx)
            });
        }
        self.state.update(cx, |state, cx| {
            if self.props.disabled() != props.disabled() {
                state.set_disabled(props.disabled(), cx);
            }
            if self.props.readonly() != props.readonly() {
                state.set_readonly(props.readonly(), cx);
            }
            M::sync(state, &props, Some(&self.props), window, cx);
        });
        self.props = props;
        // The group overlay joins after the control's own values, so a lifted
        // overlay always falls back to exactly the requested values.
        self.sync_group_editing(cx);
        let controlled = self
            .props
            .content()
            .map(|content| build_content(content).expect("validated controlled content"))
            .or_else(|| {
                self.props.value().as_ref().map(|value| {
                    // Match the underlying single-line InputState normalization,
                    // so an identical normalized echo never resets caret or
                    // undo history.
                    InputContent::new(if M::Mode::MULTI_LINE {
                        value.clone()
                    } else {
                        value.replace(['\n', '\r'], "")
                    })
                })
            });
        if let Some(content) = controlled {
            self.pending = Some(PendingDocument {
                content,
                // Entering controlled mode is an explicit ownership change;
                // uncontrolled edits may never have had a JS listener to ack.
                ack_edit_seq: if entering_controlled {
                    self.props.ack_edit_seq().max(self.edit_seq)
                } else {
                    self.props.ack_edit_seq()
                },
            });
            if self.reconcile_pending(window, cx) {
                self.start_retry(window, cx);
            }
        }
    }

    fn validate_props(props: &Self::Props) -> Result<(), String> {
        M::validate(props)
    }
    fn slots() -> &'static [&'static str] {
        M::SLOTS
    }
    fn additional_events() -> Vec<EventDefinition> {
        let mut events = vec![
            EventDefinition::new::<()>("blur"),
            EventDefinition::new::<()>("focus"),
            EventDefinition::new::<InputSubmit>("submit"),
        ];
        if M::SUPPORTS_PASTE {
            events.push(EventDefinition::new::<InputPaste>("paste"));
        }
        if M::SUPPORTS_TOKENS {
            events.push(EventDefinition::new::<InputTokenSpan>("tokenClick"));
        }
        events
    }
    fn event_name() -> &'static str {
        "change"
    }
    fn controlled() -> Option<ControlledBinding> {
        Some(ControlledBinding {
            // Token-carrying controls accept `content` (text plus its tokens)
            // as an alternate carrier of the same controlled document; the
            // change event and its editSeq acknowledgement stay identical.
            value_props: if M::SUPPORTS_TOKENS {
                &["value", "content"]
            } else {
                &["value"]
            },
            event_id: 1,
            sequence_field: "editSeq",
            ack_prop: "ackEditSeq",
        })
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        let mut commands = vec![
            ViewCommand::new("focus", Self::focus),
            ViewCommand::new("replaceValue", Self::replace_value),
            ViewCommand::new("getSelections", Self::get_selections),
            ViewCommand::new("setSelections", Self::set_selections),
            ViewCommand::new("insert", Self::insert),
            // A custom search UI drives the session without the built-in
            // panel; every text mode carries the engine for it.
            ViewCommand::new("setSearchQuery", Self::set_search_query),
            ViewCommand::new("closeSearch", Self::close_search),
            ViewCommand::new("nextSearchMatch", Self::next_search_match),
            ViewCommand::new("previousSearchMatch", Self::previous_search_match),
            ViewCommand::new(
                "replaceCurrentSearchMatch",
                Self::replace_current_search_match,
            ),
            ViewCommand::new("replaceAllSearchMatches", Self::replace_all_search_matches),
            ViewCommand::new("getSearchSession", Self::get_search_session),
        ];
        commands.extend(M::token_commands());
        commands
    }
}

impl<M: TextMode> TextControl<M> {
    fn set_search_query(
        &mut self,
        query: InputSearchQuery,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.state.update(cx, |state, cx| {
            state.set_search_query(query.query, query.case_insensitive, cx)
        });
        Ok(())
    }
    fn close_search(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.state.update(cx, |state, cx| state.close_search(cx));
        Ok(())
    }
    fn next_search_match(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<Option<InputSearchRange>, String> {
        let range = self
            .state
            .update(cx, |state, cx| state.next_search_match(cx))
            .map(|range| InputSearchRange {
                anchor_byte: range.start,
                head_byte: range.end,
            });
        Ok(range)
    }
    fn previous_search_match(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<Option<InputSearchRange>, String> {
        let range = self
            .state
            .update(cx, |state, cx| state.previous_search_match(cx))
            .map(|range| InputSearchRange {
                anchor_byte: range.start,
                head_byte: range.end,
            });
        Ok(range)
    }
    fn replace_current_search_match(
        &mut self,
        replacement: InputSearchReplacement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<bool, String> {
        let replaced = self.state.update(cx, |state, cx| {
            state.replace_current_search_match(&replacement.replacement, window, cx)
        });
        Ok(replaced)
    }
    fn replace_all_search_matches(
        &mut self,
        replacement: InputSearchReplacement,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<InputSearchReplaceAll, String> {
        let count = self.state.update(cx, |state, cx| {
            state.replace_all_search_matches(&replacement.replacement, window, cx)
        });
        Ok(InputSearchReplaceAll { count })
    }
    fn get_search_session(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<InputSearchSession, String> {
        let session = self.state.read(cx).search_session();
        Ok(InputSearchSession {
            open: session.open,
            active: session.is_active(),
            replace_mode: session.replace_mode,
            case_insensitive: session.case_insensitive,
            query: session.query.clone(),
            replacement: session.replacement.clone(),
            match_count: session.matcher.matched_ranges().len(),
            current_match_index: session.matcher.current_match_index(),
        })
    }
    fn get_selections(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<InputSelections, String> {
        Ok(InputSelections {
            ranges: self
                .state
                .read(cx)
                .cursor_selections()
                .map(|selection| InputSelectionRange {
                    anchor_byte: selection.anchor,
                    head_byte: selection.head,
                })
                .collect(),
            edit_seq: self.edit_seq,
        })
    }
    fn set_selections(
        &mut self,
        selections: InputSelections,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if selections.edit_seq != self.edit_seq {
            return Err("selection targets an obsolete edit sequence".into());
        }
        if selections.ranges.is_empty() || selections.ranges.len() > 1024 {
            return Err("selections must contain between 1 and 1024 ranges".into());
        }
        let native = selections
            .ranges
            .iter()
            .map(|range| gpui_base::input::DirectedSelection {
                anchor: range.anchor_byte,
                head: range.head_byte,
            })
            .collect::<Vec<_>>();
        self.state
            .update(cx, |state, cx| state.set_cursor_selections(&native, cx))
    }
    fn insert(
        &mut self,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if !self.state.read(cx).is_editable() {
            return Err("input is not editable".into());
        }
        self.state
            .update(cx, |state, cx| state.insert(value, window, cx));
        Ok(())
    }

    fn focus(&mut self, (): (), window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.props.disabled() || self.group_editing.is_some_and(|(_, disabled, _)| disabled) {
            return Err("disabled input cannot be focused".into());
        }
        self.state.update(cx, |state, cx| state.focus(window, cx));
        Ok(())
    }

    /// Join the surrounding group's editing overlay with the control's own
    /// props. Called with a change-guard so repeated group renders converge
    /// instead of looping notifications.
    fn sync_group_editing(&mut self, cx: &mut impl AppContext) {
        let (group_disabled, group_readonly) = self
            .group_editing
            .map_or((false, false), |(_, disabled, readonly)| {
                (disabled, readonly)
            });
        let effective = (
            self.props.disabled() || group_disabled,
            self.props.readonly() || group_readonly,
        );
        if effective == self.applied_editing {
            return;
        }
        self.state.update(cx, |state, cx| {
            state.set_disabled(effective.0, cx);
            state.set_readonly(effective.1, cx);
        });
        self.applied_editing = effective;
    }

    /// The surrounding input group applies its disabled/readonly policy to
    /// the retained engine, not just the frame. The group's entity id owns
    /// the overlay, so a stale clear from a previous owner can never erase a
    /// newer group's policy; lifting it restores exactly the control's
    /// requested values.
    pub(crate) fn apply_group_editing(
        &mut self,
        owner: gpui::EntityId,
        disabled: bool,
        readonly: bool,
        cx: &mut Context<Self>,
    ) {
        if self.group_editing == Some((owner, disabled, readonly)) {
            return;
        }
        self.group_editing = Some((owner, disabled, readonly));
        self.sync_group_editing(cx);
        cx.notify();
    }

    /// Drop the overlay when its owner unmounts or the control moves away;
    /// an owner mismatch is a no-op, protecting a newer group's overlay.
    pub(crate) fn clear_group_editing(&mut self, owner: gpui::EntityId, cx: &mut impl AppContext) {
        if !self
            .group_editing
            .is_some_and(|(holder, _, _)| holder == owner)
        {
            return;
        }
        self.group_editing = None;
        self.sync_group_editing(cx);
    }

    /// The retained engine entity. An input group reads it for freshness and
    /// writes to it only through [`Self::apply_group_editing`].
    pub(crate) fn state(&self) -> &Entity<InputBaseState<M::Mode>> {
        &self.state
    }

    /// The typed upstream control this view contributes to an input group,
    /// with every configured capability; the group restyles it and keeps this
    /// view's entity observed for fresh rendering.
    pub(crate) fn group_representation(&self) -> Option<InputGroupControl> {
        M::group_control(&self.state, &self.props, &self.children, &self.event)
    }

    fn replace_value(
        &mut self,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        self.pending = None;
        self.retry_task.take();
        // An explicit replacement ends composition and records an undo entry.
        // Upstream replace_all intentionally resets caret and scroll.
        self.state.update(cx, |state, cx| {
            state.unmark_text(window, cx);
            state.replace_all(value, window, cx);
        });
        Ok(())
    }

    fn observe_committed_value(&mut self, value: String, content: Option<InputContentSnapshot>) {
        // Token edits can change spans, labels, or removals while the text
        // stays identical; only a full content comparison may suppress the
        // change event for token-carrying controls.
        if value == self.committed_value && content == self.committed_content {
            return;
        }
        self.edit_seq = self
            .edit_seq
            .checked_add(1)
            .expect("native input edit sequence exhausted");
        self.committed_value = value.clone();
        self.committed_content = content.clone();
        self.pending = None;
        self.event.emit(InputChange {
            value,
            edit_seq: self.edit_seq,
            content,
        });
    }

    /// Returns true only while an acknowledged replacement is waiting for IME.
    fn reconcile_pending(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let Some(pending) = self.pending.as_ref() else {
            return false;
        };
        if pending.ack_edit_seq < self.edit_seq {
            self.pending = None;
            return false;
        }
        let (value, composing) = self.state.update(cx, |state, cx| {
            (
                state.value().to_string(),
                state.marked_text_range(window, cx).is_some(),
            )
        });
        // Upstream unmark_text may commit preedit without a Change event. Detect
        // that native edit before applying an older external value.
        if !composing && value != self.committed_value {
            let content = M::content_snapshot(&self.state, cx);
            self.observe_committed_value(value, content);
            return false;
        }
        // A pending document is settled only when the native document matches
        // it completely; token-only updates with identical text must still
        // apply (labels, ids, removals).
        let settled = self.pending.as_ref().is_some_and(|pending| {
            let pending_snapshot = InputContentSnapshot {
                text: pending.content.text().to_string(),
                tokens: pending.content.tokens().iter().map(token_span).collect(),
            };
            match M::content_snapshot(&self.state, cx) {
                Some(current) => current == pending_snapshot,
                None => pending.content.text().as_ref() == value.as_str(),
            }
        });
        if settled {
            self.pending = None;
            return false;
        }
        if composing {
            return true;
        }
        let pending = self.pending.take().expect("pending value was checked");
        let carries_tokens = !pending.content.tokens().is_empty();
        self.state
            .update(cx, |state, cx| state.set_value(pending.content, window, cx));
        self.committed_value = self.state.read(cx).value().to_string();
        self.committed_content = M::content_snapshot(&self.state, cx);
        if carries_tokens {
            self.ensure_token_limits(cx);
        }
        false
    }

    /// Once a document carries tokens, every later change event encodes the
    /// text plus their metadata; the live validator holds typing to the
    /// reserved budget so `Event::emit` can never overflow. Installed once
    /// and kept: removing tokens leaves the safer cap in place.
    fn ensure_token_limits(&mut self, cx: &mut Context<Self>) {
        if self.token_limits_installed {
            return;
        }
        self.state
            .update(cx, |state, cx| state.set_validator(live_token_limit(), cx));
        self.token_limits_installed = true;
    }

    fn start_retry(&mut self, window: &Window, cx: &mut Context<Self>) {
        let executor = cx.background_executor().clone();
        self.retry_task = Some(cx.spawn_in(window, async move |view, cx| {
            loop {
                executor.timer(Duration::from_millis(16)).await;
                match cx.update(|window, cx| {
                    view.update(cx, |view, cx| view.reconcile_pending(window, cx))
                }) {
                    Ok(Ok(true)) => {}
                    // No pending write, owner dropped, or window closed.
                    _ => break,
                }
            }
        }));
    }
}

impl<M: TextMode> Render for TextControl<M> {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        M::render(&self.state, &self.props, &self.children, &self.event)
    }
}

/// A controlled content snapshot must survive one encoded change event: the
/// candidate echoes the text twice (value and content.text) with the token
/// metadata, so the exact encoding is checked before any mutation.
fn content_event_budget(snapshot: &InputContentSnapshot) -> Result<(), String> {
    if snapshot.tokens.len() > TOKEN_COUNT_LIMIT {
        return Err("an input accepts at most 256 inline tokens".into());
    }
    if snapshot.text.len() > TOKEN_TEXT_LIMIT {
        return Err(format!(
            "content text must not exceed {TOKEN_TEXT_LIMIT} bytes so later edits stay encodable"
        ));
    }
    let metadata: usize = snapshot
        .tokens
        .iter()
        .map(|token| token.id.len() + token.text.len() + token.label.len())
        .sum();
    if metadata > TOKEN_METADATA_LIMIT {
        return Err(format!(
            "token metadata must not exceed {TOKEN_METADATA_LIMIT} bytes so later edits stay encodable"
        ));
    }
    // The heaviest event this document can ever produce carries the maximum
    // sequence digits.
    let candidate = InputChange {
        value: snapshot.text.clone(),
        edit_seq: u32::MAX,
        content: Some(snapshot.clone()),
    };
    crate::native::encode_json(&candidate)
        .map(|_: Vec<u8>| ())
        .map_err(|_| "controlled content exceeds the encoded native event budget".to_owned())
}

/// Decode a controlled snapshot into upstream content. Validation is
/// upstream's own, so a rejected snapshot never partially describes anything.
fn build_content(snapshot: &InputContentSnapshot) -> Result<InputContent, String> {
    if snapshot.tokens.len() > TOKEN_COUNT_LIMIT {
        return Err("an input accepts at most 256 inline tokens".into());
    }
    let mut content = InputContent::new(snapshot.text.clone());
    for token in &snapshot.tokens {
        let spec = bounded_token(&InputTokenSpec {
            id: token.id.clone(),
            text: token.text.clone(),
            label: Some(token.label.clone()),
        })?;
        content = content
            .with_token(token.anchor_byte..token.head_byte, spec)
            .map_err(|error| format!("invalid controlled content: {error}"))?;
    }
    Ok(content)
}

fn bounded_token(spec: &InputTokenSpec) -> Result<InlineToken, String> {
    if spec.id.is_empty() || spec.id.len() > 256 {
        return Err("token ids must contain 1..256 bytes".into());
    }
    if spec.text.is_empty() || spec.text.len() > 4096 {
        return Err("token text must contain 1..4096 bytes".into());
    }
    if spec
        .label
        .as_ref()
        .is_some_and(|label| label.is_empty() || label.len() > 4096)
    {
        return Err("token labels must contain 1..4096 bytes".into());
    }
    let mut token = InlineToken::new(spec.id.clone(), spec.text.clone());
    if let Some(label) = &spec.label {
        token = token.with_label(label.clone());
    }
    Ok(token)
}

fn token_parts(token: &InlineToken, range: std::ops::Range<usize>) -> InputTokenSpan {
    InputTokenSpan {
        id: token.id().to_string(),
        text: token.text().to_string(),
        label: token.label().to_string(),
        anchor_byte: range.start,
        head_byte: range.end,
    }
}

fn token_span(span: &InlineTokenSpan) -> InputTokenSpan {
    token_parts(span.token(), span.range())
}

fn exclusive_value_and_content(props: &impl TextProps) -> Result<(), String> {
    if props.value().is_some() && props.content().is_some() {
        return Err(
            "value and content are mutually exclusive; content is the value plus its tokens".into(),
        );
    }
    Ok(())
}

fn reject_content() -> Result<(), String> {
    Err("inline token content is only supported by Input and Textarea".into())
}

/// Paste is intercepted only for subscribed controls; the subscriber then owns
/// paste semantics entirely (upstream handler-returns-true) and applies text
/// itself through the controlled value or the insert command.
fn paste_intercepted(events: &Event<InputChange>) -> bool {
    events.related::<InputPaste>("paste").is_subscribed()
}

fn paste_payload(item: &gpui::ClipboardItem) -> InputPaste {
    let mut payload = InputPaste::default();
    for entry in item.entries() {
        match entry {
            gpui::ClipboardEntry::String(text) => {
                if payload.text.is_none() {
                    payload.text_truncated = text.text.len() > PASTE_TEXT_LIMIT;
                    payload.text = Some(truncate_utf8(&text.text, PASTE_TEXT_LIMIT).to_string());
                }
            }
            gpui::ClipboardEntry::Image(image) => {
                if payload.image.is_none() {
                    payload.image = Some(InputPasteImage {
                        format: image_format_name(image.format).into(),
                        byte_length: image.bytes.len(),
                        // Base64 must keep the whole encoded event bounded;
                        // larger images arrive as metadata only.
                        data: (image.bytes.len() <= PASTE_IMAGE_DATA_LIMIT)
                            .then(|| base64_encode(&image.bytes)),
                    });
                }
            }
            gpui::ClipboardEntry::ExternalPaths(paths) => {
                let mut total = payload.files.iter().map(|f| f.len()).sum::<usize>();
                for path in &paths.0 {
                    if payload.files.len() >= PASTE_FILE_COUNT_LIMIT
                        || total >= PASTE_FILE_BYTES_LIMIT
                    {
                        break;
                    }
                    let name = path.display().to_string();
                    if name.len() > PASTE_FILE_BYTES_LIMIT
                        || total + name.len() > PASTE_FILE_BYTES_LIMIT
                    {
                        break;
                    }
                    total += name.len();
                    payload.files.push(name);
                }
            }
        }
    }
    payload
}

fn truncate_utf8(text: &str, limit: usize) -> &str {
    let mut end = limit.min(text.len());
    while end > 0 && !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn image_format_name(format: gpui::ImageFormat) -> &'static str {
    match format {
        gpui::ImageFormat::Png => "png",
        gpui::ImageFormat::Jpeg => "jpeg",
        gpui::ImageFormat::Webp => "webp",
        gpui::ImageFormat::Gif => "gif",
        gpui::ImageFormat::Svg => "svg",
        gpui::ImageFormat::Bmp => "bmp",
        gpui::ImageFormat::Tiff => "tiff",
        gpui::ImageFormat::Ico => "ico",
        _ => "unknown",
    }
}

const BASE64_ALPHABET: &[u8; 64] =
    b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn base64_encode(data: &[u8]) -> String {
    let mut encoded = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let word = (chunk[0] as u32) << 16
            | (chunk.get(1).copied().unwrap_or(0) as u32) << 8
            | chunk.get(2).copied().unwrap_or(0) as u32;
        encoded.push(BASE64_ALPHABET[(word >> 18) as usize & 63] as char);
        encoded.push(BASE64_ALPHABET[(word >> 12) as usize & 63] as char);
        encoded.push(if chunk.len() > 1 {
            BASE64_ALPHABET[(word >> 6) as usize & 63] as char
        } else {
            '='
        });
        encoded.push(if chunk.len() > 2 {
            BASE64_ALPHABET[word as usize & 63] as char
        } else {
            '='
        });
    }
    encoded
}

/// The live edit validator for token-carrying documents: typing, pasting,
/// and every other engine edit path consult it before mutating, holding the
/// text inside the reserved encoded-event budget.
fn live_token_limit() -> impl Fn(&str, &mut gpui::App) -> bool + 'static {
    move |text: &str, _: &mut gpui::App| text.len() <= TOKEN_TEXT_LIMIT
}

/// The post-edit change event must stay inside one encoded native event. The
/// candidate over-approximates: the full current token set plus the incoming
/// token, so the true document is never larger than what is admitted here.
fn budget_edit(
    current: &InputContentSnapshot,
    token: &InlineToken,
    replace: Option<std::ops::Range<usize>>,
) -> Result<(), String> {
    let text = &current.text;
    let (start, end) = replace.map_or((text.len(), text.len()), |range| (range.start, range.end));
    if end > text.len()
        || start > end
        || !text.is_char_boundary(start)
        || !text.is_char_boundary(end)
    {
        return Err("token range must lie on UTF-8 boundaries inside the document".into());
    }
    let mut new_text = String::with_capacity(text.len() - (end - start) + token.text().len());
    new_text.push_str(&text[..start]);
    new_text.push_str(token.text());
    new_text.push_str(&text[end..]);
    let mut tokens = current.tokens.clone();
    tokens.push(InputTokenSpan {
        id: token.id().to_string(),
        text: token.text().to_string(),
        label: token.label().to_string(),
        anchor_byte: 0,
        head_byte: token.text().len(),
    });
    content_event_budget(&InputContentSnapshot {
        text: new_text,
        tokens,
    })
}

/// Commands for modes whose engine carries atomic inline tokens; token and
/// content edits are engine-validated and atomic, so a rejected command
/// leaves the document exactly as it was.
macro_rules! token_commands {
    () => {
        fn token_commands() -> Vec<ViewCommand<TextControl<Self>>> {
            vec![
                ViewCommand::new(
                    "insertToken",
                    |this: &mut TextControl<Self>,
                     spec: InputTokenSpec,
                     window: &mut Window,
                     cx: &mut Context<TextControl<Self>>| {
                        let token = bounded_token(&spec)?;
                        let current = Self::content_snapshot(&this.state, cx)
                            .expect("token mode carries a content snapshot");
                        budget_edit(&current, &token, None)?;
                        let applied = this.state.update(cx, |state, cx| {
                            state
                                .replace_with_token(token, window, cx)
                                .map_err(|error| format!("token edit rejected: {error}"))
                        });
                        applied?;
                        this.ensure_token_limits(cx);
                        Ok(())
                    },
                ),
                ViewCommand::new(
                    "replaceRangeWithToken",
                    |this: &mut TextControl<Self>,
                     spec: InputTokenRangeSpec,
                     window: &mut Window,
                     cx: &mut Context<TextControl<Self>>| {
                        let token = bounded_token(&spec.token)?;
                        let current = Self::content_snapshot(&this.state, cx)
                            .expect("token mode carries a content snapshot");
                        budget_edit(&current, &token, Some(spec.anchor_byte..spec.head_byte))?;
                        let applied = this.state.update(cx, |state, cx| {
                            state
                                .replace_range_with_token(
                                    spec.anchor_byte..spec.head_byte,
                                    token,
                                    window,
                                    cx,
                                )
                                .map_err(|error| format!("token edit rejected: {error}"))
                        });
                        applied?;
                        this.ensure_token_limits(cx);
                        Ok(())
                    },
                ),
                ViewCommand::new(
                    "getTokens",
                    |this: &mut TextControl<Self>,
                     (): (),
                     _: &mut Window,
                     cx: &mut Context<TextControl<Self>>| {
                        Ok(this
                            .state
                            .read(cx)
                            .tokens()
                            .iter()
                            .map(token_span)
                            .collect::<Vec<_>>())
                    },
                ),
                ViewCommand::new(
                    "getContent",
                    |this: &mut TextControl<Self>,
                     (): (),
                     _: &mut Window,
                     cx: &mut Context<TextControl<Self>>| {
                        let content = this.state.read(cx).content();
                        Ok(InputContentSnapshot {
                            text: content.text().to_string(),
                            tokens: content.tokens().iter().map(token_span).collect(),
                        })
                    },
                ),
            ]
        }
    };
}

impl TextMode for SingleLine {
    type Mode = InputMode;
    type Props = InputProps;
    const SLOTS: &'static [&'static str] = &["prefix", "suffix"];
    const SUPPORTS_PASTE: bool = true;
    const SUPPORTS_TOKENS: bool = true;
    fn new(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
        InputState::new(window, cx)
    }
    fn validate(p: &InputProps) -> Result<(), String> {
        exclusive_value_and_content(p)?;
        if let Some(content) = p.content() {
            build_content(content)?;
            content_event_budget(content)?;
        }
        Ok(())
    }
    fn sync(
        s: &mut InputState,
        p: &InputProps,
        old: Option<&InputProps>,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        if old.is_none_or(|old| old.masked != p.masked) {
            s.set_masked(p.masked, window, cx);
        }
    }
    fn content_snapshot(state: &Entity<InputState>, cx: &App) -> Option<InputContentSnapshot> {
        let content = state.read(cx).content();
        Some(InputContentSnapshot {
            text: content.text().to_string(),
            tokens: content.tokens().iter().map(token_span).collect(),
        })
    }
    fn render(
        s: &Entity<InputState>,
        p: &InputProps,
        children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> AnyElement {
        SingleLine::build(s, p, children, events).into_any_element()
    }
    fn group_control(
        s: &Entity<InputState>,
        p: &InputProps,
        children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> Option<InputGroupControl> {
        Some(SingleLine::build(s, p, children, events).into())
    }
    token_commands!();
}
impl SingleLine {
    fn build(
        s: &Entity<InputState>,
        p: &InputProps,
        children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> GpuiInput {
        let mut v = GpuiInput::new(s)
            .disabled(p.disabled)
            .readonly(p.readonly)
            .appearance(p.appearance)
            .bordered(p.bordered)
            .cleanable(p.cleanable)
            .with_size(p.size);
        if p.mask_toggle {
            v = v.mask_toggle();
        }
        if let Some(label) = &p.aria_label {
            v = v.aria_label(label.clone());
        }
        for (name, prefix) in [("prefix", true), ("suffix", false)] {
            let slot = children.slot(name);
            if !slot.is_empty() {
                v = if prefix {
                    v.prefix(slot)
                } else {
                    v.suffix(slot)
                };
            }
        }
        let token_click = events.related::<InputTokenSpan>("tokenClick");
        if token_click.is_subscribed() {
            v = v.on_token_click(move |event, _, _| {
                token_click.emit(token_parts(event.token(), event.range()))
            });
        }
        if paste_intercepted(events) {
            let sink = events.related::<InputPaste>("paste");
            v = v.on_paste(move |item, _window, _cx| {
                sink.emit(paste_payload(item));
                // A subscribed control owns paste semantics; nothing inserts
                // natively, matching upstream handler-returns-true.
                true
            });
        }
        v
    }
}
impl TextMode for Numeric {
    type Mode = InputMode;
    type Props = NumberInputProps;
    const SLOTS: &'static [&'static str] = &["prefix", "suffix"];
    fn new(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
        InputState::new(window, cx)
    }
    fn validate(p: &NumberInputProps) -> Result<(), String> {
        exclusive_value_and_content(p)?;
        if p.content().is_some() {
            reject_content()?;
        }
        if !p.step.is_finite()
            || p.step <= 0.
            || p.min.is_some_and(|n| !n.is_finite())
            || p.max.is_some_and(|n| !n.is_finite())
            || p.min.zip(p.max).is_some_and(|(a, b)| a > b)
        {
            return Err("number bounds must be finite and ordered, with a positive step".into());
        }
        Ok(())
    }
    fn sync(
        s: &mut InputState,
        p: &NumberInputProps,
        old: Option<&NumberInputProps>,
        window: &mut Window,
        cx: &mut Context<InputState>,
    ) {
        if old.is_none_or(|o| o.step != p.step) {
            s.set_step(Some(NumberStep::Fixed(p.step)), window, cx);
        }
        if old.is_none_or(|o| o.min != p.min) {
            s.set_min(p.min, window, cx);
        }
        if old.is_none_or(|o| o.max != p.max) {
            s.set_max(p.max, window, cx);
        }
    }
    fn render(
        s: &Entity<InputState>,
        p: &NumberInputProps,
        children: &NativeChildren,
        _events: &Event<InputChange>,
    ) -> AnyElement {
        let mut v = gpui_component::input::NumberInput::new(s)
            .disabled(p.disabled || p.readonly)
            .appearance(p.appearance)
            .with_size(p.size);
        if let Some(placeholder) = &p.placeholder {
            v = v.placeholder(placeholder.clone());
        }
        for (name, prefix) in [("prefix", true), ("suffix", false)] {
            let slot = children.slot(name);
            if !slot.is_empty() {
                v = if prefix {
                    v.prefix(slot)
                } else {
                    v.suffix(slot)
                };
            }
        }
        v.into_any_element()
    }
}
impl TextMode for MultiLine {
    type Mode = TextareaMode;
    type Props = TextareaProps;
    const SUPPORTS_PASTE: bool = true;
    const SUPPORTS_TOKENS: bool = true;
    fn new(
        window: &mut Window,
        cx: &mut Context<gpui_base::input::TextareaState>,
    ) -> gpui_base::input::TextareaState {
        gpui_base::input::TextareaState::new(window, cx)
    }
    fn validate(p: &TextareaProps) -> Result<(), String> {
        exclusive_value_and_content(p)?;
        if let Some(content) = p.content() {
            build_content(content)?;
            content_event_budget(content)?;
        }
        if !(1..=10000).contains(&p.rows)
            || p.auto_grow
                .as_ref()
                .is_some_and(|a| a.min_rows == 0 || a.max_rows < a.min_rows || a.max_rows > 10000)
        {
            return Err(
                "textarea rows must be 1..10000 and autoGrow bounds must be ordered".into(),
            );
        }
        Ok(())
    }
    fn sync(
        s: &mut gpui_base::input::TextareaState,
        p: &TextareaProps,
        old: Option<&TextareaProps>,
        window: &mut Window,
        cx: &mut Context<gpui_base::input::TextareaState>,
    ) {
        if old.is_none_or(|o| o.rows != p.rows || o.auto_grow != p.auto_grow) {
            if let Some(a) = &p.auto_grow {
                s.set_auto_grow(a.min_rows, a.max_rows, cx);
            } else {
                s.set_rows(p.rows, cx);
            }
        }
        if old.is_none_or(|o| o.soft_wrap != p.soft_wrap) {
            s.set_soft_wrap(p.soft_wrap, window, cx);
        }
        if old.is_none_or(|o| o.searchable != p.searchable) {
            s.set_searchable(p.searchable, cx);
        }
    }
    fn content_snapshot(
        state: &Entity<gpui_base::input::TextareaState>,
        cx: &App,
    ) -> Option<InputContentSnapshot> {
        let content = state.read(cx).content();
        Some(InputContentSnapshot {
            text: content.text().to_string(),
            tokens: content.tokens().iter().map(token_span).collect(),
        })
    }
    fn render(
        s: &Entity<gpui_base::input::TextareaState>,
        p: &TextareaProps,
        _children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> AnyElement {
        MultiLine::build(s, p, events).into_any_element()
    }
    fn group_control(
        s: &Entity<gpui_base::input::TextareaState>,
        p: &TextareaProps,
        _children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> Option<InputGroupControl> {
        Some(MultiLine::build(s, p, events).into())
    }
    token_commands!();
}
impl MultiLine {
    fn build(
        s: &Entity<gpui_base::input::TextareaState>,
        p: &TextareaProps,
        events: &Event<InputChange>,
    ) -> gpui_component::input::Textarea {
        let mut v = gpui_component::input::Textarea::new(s)
            .disabled(p.disabled)
            .readonly(p.readonly)
            .appearance(p.appearance)
            .bordered(p.bordered);
        if let Some(label) = &p.aria_label {
            v = v.aria_label(label.clone());
        }
        let token_click = events.related::<InputTokenSpan>("tokenClick");
        if token_click.is_subscribed() {
            v = v.on_token_click(move |event, _, _| {
                token_click.emit(token_parts(event.token(), event.range()))
            });
        }
        if paste_intercepted(events) {
            let sink = events.related::<InputPaste>("paste");
            v = v.on_paste(move |item, _window, _cx| {
                sink.emit(paste_payload(item));
                // A subscribed control owns paste semantics; nothing inserts
                // natively, matching upstream handler-returns-true.
                true
            });
        }
        v
    }
}
impl TextMode for Code {
    type Mode = EditorMode;
    type Props = EditorProps;
    const SUPPORTS_PASTE: bool = true;
    fn new(
        window: &mut Window,
        cx: &mut Context<gpui_base::input::EditorState>,
    ) -> gpui_base::input::EditorState {
        gpui_base::input::EditorState::new(window, cx)
    }
    fn validate(p: &EditorProps) -> Result<(), String> {
        exclusive_value_and_content(p)?;
        if p.content().is_some() {
            reject_content()?;
        }
        if !(1..=16).contains(&p.tab_size)
            || p.cursor_surrounding_lines.is_some_and(|v| v > 10000)
            || p.scroll_beyond_last_line.is_some_and(|v| v > 10000)
        {
            return Err(
                "editor tabSize must be 1..16 and cursorSurroundingLines at most 10000".into(),
            );
        }
        Ok(())
    }
    fn sync(
        s: &mut gpui_base::input::EditorState,
        p: &EditorProps,
        old: Option<&EditorProps>,
        window: &mut Window,
        cx: &mut Context<gpui_base::input::EditorState>,
    ) {
        if old.is_none_or(|o| o.auto_close != p.auto_close) {
            s.set_auto_close(p.auto_close, window, cx);
        }
        if old.is_none_or(|o| o.smart_indent != p.smart_indent) {
            s.set_smart_indent(p.smart_indent, window, cx);
        }
        if old.is_none_or(|o| o.language != p.language) {
            s.set_highlighter(p.language.clone(), cx);
        }
        if old.is_none_or(|o| o.soft_wrap != p.soft_wrap) {
            s.set_soft_wrap(p.soft_wrap, window, cx);
        }
        if old.is_none_or(|o| o.searchable != p.searchable) {
            s.set_searchable(p.searchable, cx);
        }
        if old.is_none_or(|o| o.line_numbers != p.line_numbers) {
            s.set_line_number(p.line_numbers, window, cx);
        }
        if old.is_none_or(|o| o.folding != p.folding) {
            s.set_folding(p.folding, window, cx);
        }
        if old.is_none_or(|o| o.indent_guides != p.indent_guides) {
            s.set_indent_guides(p.indent_guides, window, cx);
        }
        if old.is_none_or(|o| o.tab_size != p.tab_size || o.hard_tabs != p.hard_tabs) {
            s.set_tab_size(
                TabSize {
                    tab_size: p.tab_size,
                    hard_tabs: p.hard_tabs,
                },
                cx,
            );
        }
        if old.is_none_or(|o| o.show_whitespaces != p.show_whitespaces) {
            s.set_show_whitespaces(p.show_whitespaces, window, cx);
        }
        if old.is_none_or(|o| o.scroll_beyond_last_line != p.scroll_beyond_last_line) {
            s.set_scroll_beyond_last_line(p.scroll_beyond_last_line, window, cx);
        }
        if old.is_none_or(|o| o.cursor_surrounding_lines != p.cursor_surrounding_lines) {
            s.set_cursor_surrounding_lines(p.cursor_surrounding_lines, window, cx);
        }
    }
    fn render(
        s: &Entity<gpui_base::input::EditorState>,
        p: &EditorProps,
        _children: &NativeChildren,
        events: &Event<InputChange>,
    ) -> AnyElement {
        let mut v = gpui_component::input::Editor::new(s)
            .h(gpui::relative(1.))
            .disabled(p.disabled)
            .readonly(p.readonly)
            .appearance(p.appearance)
            .bordered(p.bordered);
        if let Some(label) = &p.aria_label {
            v = v.aria_label(label.clone());
        }
        if paste_intercepted(events) {
            let sink = events.related::<InputPaste>("paste");
            v = v.on_paste(move |item, _window, _cx| {
                sink.emit(paste_payload(item));
                // A subscribed control owns paste semantics; nothing inserts
                // natively, matching upstream handler-returns-true.
                true
            });
        }
        v.into_any_element()
    }
}
pub(crate) fn definitions() -> Vec<ComponentDefinition> {
    vec![
        ComponentDefinition::view::<TextControl<SingleLine>>("Input"),
        ComponentDefinition::view::<TextControl<Numeric>>("NumberInput"),
        ComponentDefinition::view::<TextControl<MultiLine>>("Textarea"),
        ComponentDefinition::view::<TextControl<Code>>("Editor"),
    ]
    .into_iter()
    .map(|d| d.with_contract(include_str!("input.rs")))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{DecodedMessage, EventPayload};
    use crate::{ExtensionEventSink, InMemoryAdapter, Node, Snapshot, SolidRoot};
    use gpui::{AppContext, TestAppContext, WindowHandle};
    use std::{cell::RefCell, rc::Rc, sync::Arc};

    struct Fixture {
        window: WindowHandle<gpui_component::Root>,
        input: Entity<Input>,
        _bridge: Entity<SolidRoot>,
        runtime: Arc<InMemoryAdapter>,
    }
    impl Fixture {
        fn new(cx: &mut TestAppContext) -> Self {
            cx.update(gpui_component::init);
            let runtime = InMemoryAdapter::new();
            let captured = Rc::new(RefCell::new(None));
            let window = cx.open_window(gpui::size(gpui::px(400.), gpui::px(200.)), {
                let captured = captured.clone();
                let runtime = runtime.clone();
                move |window, cx| {
                    let bridge = cx.new(|_| SolidRoot::new(runtime));
                    bridge.update(cx, |bridge, cx| {
                        bridge
                            .apply_decoded_message(
                                DecodedMessage::Snapshot(Snapshot::new(
                                    1,
                                    1,
                                    0,
                                    1,
                                    vec![Node::new(1, 0, 0, crate::KIND_VIEW)],
                                )),
                                cx,
                            )
                            .unwrap();
                    });
                    let sink = ExtensionEventSink::new(
                        bridge.read(cx).extension_event_state(),
                        2,
                        10,
                        Arc::from([1]),
                    );
                    let input = cx.new(|cx| {
                        Input::mount(
                            InputProps::default(),
                            Event::with_events(
                                sink.clone(),
                                1,
                                Rc::new(
                                    [
                                        (String::from("change"), 1),
                                        (String::from("blur"), 2),
                                        (String::from("focus"), 3),
                                        (String::from("submit"), 4),
                                        (String::from("paste"), 5),
                                        (String::from("tokenClick"), 6),
                                    ]
                                    .into(),
                                ),
                            ),
                            NativeChildren::new(
                                crate::ExtensionChildren::new(bridge.downgrade(), sink),
                                SingleLine::SLOTS,
                            ),
                            window,
                            cx,
                        )
                    });
                    *captured.borrow_mut() = Some((input.clone(), bridge));
                    gpui_component::Root::new(input, window, cx)
                }
            });
            let (input, bridge) = captured.borrow_mut().take().unwrap();
            Self {
                window,
                input,
                _bridge: bridge,
                runtime,
            }
        }
        fn update<R>(
            &self,
            cx: &mut TestAppContext,
            f: impl FnOnce(&mut Input, &mut Window, &mut Context<Input>) -> R,
        ) -> R {
            cx.update_window(self.window.into(), |_, window, cx| {
                self.input.update(cx, |input, cx| f(input, window, cx))
            })
            .unwrap()
        }
        fn value(&self, cx: &TestAppContext) -> String {
            self.input
                .read_with(cx, |input, cx| input.state.read(cx).value().to_string())
        }
        fn props(&self, cx: &mut TestAppContext, value: &str, ack_edit_seq: u32) {
            self.update(cx, |input, window, cx| {
                input.update(
                    InputProps {
                        value: Some(value.into()),
                        ack_edit_seq,
                        ..InputProps::default()
                    },
                    window,
                    cx,
                )
            });
        }
        fn mark(&self, cx: &mut TestAppContext, value: &str) {
            self.update(cx, |input, window, cx| {
                input.state.update(cx, |state, cx| {
                    state.replace_and_mark_text_in_range(None, value, None, window, cx);
                })
            });
        }
        fn commit(&self, cx: &mut TestAppContext, value: &str) {
            self.update(cx, |input, window, cx| {
                input.state.update(cx, |state, cx| {
                    state.replace_text_in_range(None, value, window, cx);
                })
            });
            cx.run_until_parked();
        }
        fn changes(&self) -> Vec<InputChange> {
            let mut changes = Vec::new();
            while let Some(event) = self.runtime.take_event().unwrap() {
                if let EventPayload::Extension { fields, .. } = event.payload
                    && let crate::protocol::ExtensionValue::Bytes(bytes) = &fields[0].value
                {
                    changes.push(crate::native::decode_json(bytes).unwrap());
                }
            }
            changes
        }
    }
    fn tick(cx: &mut TestAppContext) {
        cx.run_until_parked();
        cx.executor().advance_clock(Duration::from_millis(32));
        cx.run_until_parked();
    }

    #[gpui::test]
    fn editor_multiple_selections_validate_atomically_and_undo_one_native_edit(
        cx: &mut TestAppContext,
    ) {
        let f = crate::components::test_support::Fixture::<TextControl<Code>>::new(
            EditorProps {
                default_value: Some("aé\r\nz".into()),
                ..Default::default()
            },
            cx,
        );
        f.update(cx, |code, window, cx| {
            let selections = || InputSelections {
                ranges: vec![
                    InputSelectionRange {
                        anchor_byte: 1,
                        head_byte: 0,
                    },
                    InputSelectionRange {
                        anchor_byte: 5,
                        head_byte: 5,
                    },
                ],
                edit_seq: 0,
            };
            code.set_selections(selections(), window, cx).unwrap();
            for invalid in [2, 4, 99] {
                let mut bad = selections();
                bad.ranges[1].head_byte = invalid;
                assert!(code.set_selections(bad, window, cx).is_err());
                let actual = code.get_selections((), window, cx).unwrap();
                assert_eq!(
                    actual
                        .ranges
                        .iter()
                        .map(|r| (r.anchor_byte, r.head_byte))
                        .collect::<Vec<_>>(),
                    vec![(1, 0), (5, 5)]
                );
            }
            let mut stale = selections();
            stale.edit_seq = 99;
            assert!(code.set_selections(stale, window, cx).is_err());
            code.state.update(cx, |state, cx| {
                state.replace_text_in_range(None, "X", window, cx)
            });
            assert_eq!(code.state.read(cx).value().as_ref(), "Xé\r\nXz");
            code.focus((), window, cx).unwrap();
        });
        cx.update_window(f.window.into(), |_, window, cx| window.draw(cx).clear(cx))
            .unwrap();
        cx.dispatch_action(f.window.into(), gpui_component::input::Undo);
        f.update(cx, |code, _, cx| {
            assert_eq!(code.state.read(cx).value().as_ref(), "aé\r\nz")
        });
    }

    #[gpui::test]
    fn controlled_echo_preserves_real_selection_and_undo_and_rejects_stale_ack(
        cx: &mut TestAppContext,
    ) {
        let fixture = Fixture::new(cx);
        fixture.commit(cx, "abc");
        assert_eq!(
            fixture.changes().last(),
            Some(&InputChange {
                value: "abc".into(),
                edit_seq: 1,
                content: Some(InputContentSnapshot {
                    text: "abc".into(),
                    tokens: vec![]
                })
            })
        );
        let identity = fixture
            .input
            .read_with(cx, |input, _| input.state.entity_id());
        fixture.update(cx, |input, _, cx| {
            input
                .state
                .update(cx, |state, cx| state.set_selected_range(1..2, cx))
        });
        fixture.props(cx, "abc", 1);
        fixture.input.read_with(cx, |input, cx| {
            assert_eq!(input.state.entity_id(), identity);
            assert_eq!(input.state.read(cx).selected_range(), 1..2);
        });
        fixture.update(cx, |input, window, cx| input.focus((), window, cx).unwrap());
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        cx.dispatch_action(fixture.window.into(), gpui_component::input::Undo);
        assert_eq!(fixture.value(cx), "");
        fixture.commit(cx, "latest");
        fixture.props(cx, "abc", 1);
        assert_eq!(fixture.value(cx), "latest");
    }

    #[gpui::test]
    fn pending_value_waits_for_ime_cancellation_and_unmark_without_change(cx: &mut TestAppContext) {
        let fixture = Fixture::new(cx);
        fixture.mark(cx, "ni");
        fixture.props(cx, "server", 0);
        tick(cx);
        assert_eq!(fixture.value(cx), "ni");
        assert!(
            fixture
                .input
                .read_with(cx, |input, _| input.pending.is_some())
        );
        // Cancellation has no Change event; the bounded pending-only task retries.
        fixture.mark(cx, "");
        tick(cx);
        assert_eq!(fixture.value(cx), "server");
        assert!(
            fixture
                .input
                .read_with(cx, |input, _| input.pending.is_none())
        );
        assert!(fixture.changes().is_empty());
        fixture.update(cx, |input, _, cx| {
            input
                .state
                .update(cx, |state, cx| state.set_selected_range(6..6, cx))
        });
        fixture.mark(cx, "x");
        fixture.props(cx, "old response", 0);
        fixture.update(cx, |input, window, cx| {
            input
                .state
                .update(cx, |state, cx| state.unmark_text(window, cx))
        });
        tick(cx);
        assert_eq!(fixture.value(cx), "serverx");
        assert!(
            fixture
                .input
                .read_with(cx, |input, _| input.pending.is_none())
        );
        assert_eq!(
            fixture.changes().last(),
            Some(&InputChange {
                value: "serverx".into(),
                edit_seq: 1,
                content: Some(InputContentSnapshot {
                    text: "serverx".into(),
                    tokens: vec![]
                })
            })
        );
    }

    #[gpui::test]
    fn native_commit_and_explicit_replace_cancel_pending_work_and_owner_drop_releases_it(
        cx: &mut TestAppContext,
    ) {
        let fixture = Fixture::new(cx);
        fixture.mark(cx, "ni");
        fixture.props(cx, "server", 0);
        fixture.commit(cx, "你");
        assert!(
            fixture
                .input
                .read_with(cx, |input, _| input.pending.is_none()
                    && input.retry_task.is_none())
        );
        tick(cx);
        assert_eq!(fixture.value(cx), "你");
        fixture.update(cx, |input, window, cx| {
            input.replace_value("program".into(), window, cx).unwrap()
        });
        cx.run_until_parked();
        assert_eq!(fixture.value(cx), "program");
        fixture.update(cx, |input, window, cx| {
            input.update(InputProps::default(), window, cx)
        });
        fixture.update(cx, |input, window, cx| {
            input
                .replace_value("native-only".into(), window, cx)
                .unwrap()
        });
        cx.run_until_parked();
        fixture.props(cx, "controlled", 0);
        assert_eq!(
            fixture.value(cx),
            "controlled",
            "entering controlled mode must not require an acknowledgement of unseen uncontrolled edits"
        );
        let seq = fixture.input.read_with(cx, |input, _| input.edit_seq);
        fixture.mark(cx, "x");
        fixture.props(cx, "pending", seq);
        assert!(
            fixture
                .input
                .read_with(cx, |input, _| input.pending.is_some())
        );
        let weak = fixture.input.downgrade();
        fixture
            .window
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        drop(fixture);
        tick(cx);
        assert!(
            weak.upgrade().is_none(),
            "pending retry must not retain the native view"
        );
    }
    #[gpui::test]
    fn editor_fills_its_native_viewport(cx: &mut TestAppContext) {
        let f = super::super::test_support::Fixture::<TextControl<Code>>::new(
            EditorProps {
                default_value: Some("first\nsecond\nthird".into()),
                ..Default::default()
            },
            cx,
        );
        cx.update_window(f.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx);
            let state = f.view.read(cx).state.read(cx);
            assert!(state.input_bounds().size.height > gpui::px(200.));
        })
        .unwrap();
    }

    #[gpui::test]
    fn fixed_textarea_rows_set_the_native_drawn_height(cx: &mut TestAppContext) {
        let f = super::super::test_support::Fixture::<TextControl<MultiLine>>::new(
            TextareaProps {
                rows: 3,
                ..Default::default()
            },
            cx,
        );
        let measure = |cx: &mut TestAppContext| {
            cx.update_window(f.window.into(), |_, window, cx| {
                window.draw(cx).clear(cx);
                let state = f.view.read(cx).state.read(cx);
                (
                    state.input_bounds().size.height,
                    state.line_height().unwrap(),
                )
            })
            .unwrap()
        };
        let (three, line_height) = measure(cx);
        assert!(
            three >= 3. * line_height,
            "three rows: {three:?}, line: {line_height:?}"
        );
        f.update(cx, |input, window, cx| {
            input.update(
                TextareaProps {
                    rows: 6,
                    ..Default::default()
                },
                window,
                cx,
            )
        });
        let (six, _) = measure(cx);
        assert!((six - three - 3. * line_height).abs() < gpui::px(1.));
    }

    #[gpui::test]
    fn multiline_controls_keep_text_selection_and_entity_across_configuration_changes(
        cx: &mut TestAppContext,
    ) {
        let f = Fixture::new(cx);
        f.update(cx, |input, window, cx| {
            let value = "first\n第二行\nthird".to_string();
            let text = cx.new(|cx| {
                TextControl::<MultiLine>::mount(
                    TextareaProps {
                        value: Some(value.clone()),
                        auto_grow: Some(AutoGrow {
                            min_rows: 2,
                            max_rows: 5,
                        }),
                        ..Default::default()
                    },
                    input.event.clone(),
                    input.children.clone(),
                    window,
                    cx,
                )
            });
            let code = cx.new(|cx| {
                TextControl::<Code>::mount(
                    EditorProps {
                        value: Some(value.clone()),
                        ..Default::default()
                    },
                    input.event.clone(),
                    input.children.clone(),
                    window,
                    cx,
                )
            });
            let text_id = text.read(cx).state.entity_id();
            let code_id = code.read(cx).state.entity_id();
            text.update(cx, |text, cx| {
                text.state
                    .update(cx, |state, cx| state.set_selected_range(6..15, cx));
                text.update(
                    TextareaProps {
                        value: Some(value.clone()),
                        rows: 4,
                        ..Default::default()
                    },
                    window,
                    cx,
                );
                assert_eq!(text.state.entity_id(), text_id);
                assert_eq!(text.state.read(cx).value().as_ref(), value);
                assert_eq!(text.state.read(cx).selected_range(), 6..15);
            });
            code.update(cx, |code, cx| {
                code.state
                    .update(cx, |state, cx| state.set_selected_range(6..15, cx));
                code.update(
                    EditorProps {
                        value: Some(value.clone()),
                        line_numbers: false,
                        language: "rust".into(),
                        tab_size: 4,
                        ..Default::default()
                    },
                    window,
                    cx,
                );
                assert_eq!(code.state.entity_id(), code_id);
                assert_eq!(code.state.read(cx).value().as_ref(), value);
                assert_eq!(code.state.read(cx).selected_range(), 6..15);
                assert!(
                    code.set_selections(
                        InputSelections {
                            ranges: vec![InputSelectionRange {
                                anchor_byte: 7,
                                head_byte: 15
                            }],
                            edit_seq: 0
                        },
                        window,
                        cx
                    )
                    .is_err()
                );
            });
        });
    }
    #[gpui::test]
    fn large_editor_composition_survives_parent_updates_resize_delayed_echo_and_undo(
        cx: &mut TestAppContext,
    ) {
        let original = "start e\u{301} 👩🏽‍💻 🇸🇬 العربية אבג\n".repeat(1_000);
        let fixture = crate::components::test_support::Fixture::<TextControl<Code>>::new(
            EditorProps {
                value: Some(original.clone()),
                ..Default::default()
            },
            cx,
        );
        let identity = fixture.update(cx, |editor, window, cx| {
            editor.focus((), window, cx).unwrap();
            editor.state.update(cx, |state, cx| {
                state.set_selected_range(0..5, cx);
                state.replace_and_mark_text_in_range(None, "ni", None, window, cx);
            });
            editor.update(
                EditorProps {
                    value: Some("delayed server result".into()),
                    line_numbers: false,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert!(editor.pending.is_some());
            editor.state.entity_id()
        });
        cx.simulate_window_resize(
            fixture.window.into(),
            gpui::size(gpui::px(300.0), gpui::px(240.0)),
        );
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        fixture.update(cx, |editor, window, cx| {
            editor.state.update(cx, |state, cx| {
                state.replace_text_in_range(None, "你", window, cx)
            })
        });
        cx.run_until_parked();
        tick(cx);
        let expected = format!("你{}", &original[5..]);
        fixture.update(cx, |editor, window, cx| {
            assert_eq!(editor.state.entity_id(), identity);
            assert_eq!(editor.state.read(cx).value().as_ref(), expected);
            assert!(editor.pending.is_none() && editor.retry_task.is_none());
            editor.update(
                EditorProps {
                    value: Some(original.clone()),
                    ack_edit_seq: 0,
                    ..Default::default()
                },
                window,
                cx,
            );
            assert_eq!(editor.state.read(cx).value().as_ref(), expected);
        });
        cx.update_window(fixture.window.into(), |_, window, cx| {
            window.draw(cx).clear(cx)
        })
        .unwrap();
        cx.dispatch_action(fixture.window.into(), gpui_component::input::Undo);
        fixture.update(cx, |editor, _, cx| {
            assert_eq!(editor.state.read(cx).value().as_ref(), original);
            assert_eq!(editor.state.read(cx).selected_range(), 0..5);
        });
    }

    #[gpui::test]
    fn group_editing_overlay_blocks_focused_edits_and_restores_requested_values(
        cx: &mut TestAppContext,
    ) {
        let fixture = Fixture::new(cx);
        fixture.update(cx, |input, window, cx| {
            input.focus((), window, cx).unwrap();
            // A readonly group overlay blocks editing even while focused...
            input.apply_group_editing(cx.entity_id(), false, true, cx);
            assert!(!input.state.read(cx).is_editable());
            assert!(input.insert("x".into(), window, cx).is_err());
            // ...and a disabled overlay additionally refuses focus.
            input.apply_group_editing(cx.entity_id(), true, false, cx);
            assert!(input.focus((), window, cx).is_err());
            // Lifting the overlay restores exactly the control's own values.
            input.apply_group_editing(cx.entity_id(), false, false, cx);
            input.focus((), window, cx).unwrap();
            input.insert("typed".into(), window, cx).unwrap();
        });
        assert_eq!(fixture.value(cx), "typed");
        // A props commit while the overlay is up rejoins the overlay; after
        // it lifts, the control's own readonly prop still wins.
        fixture.update(cx, |input, window, cx| {
            input.apply_group_editing(cx.entity_id(), true, false, cx);
            input.update(
                InputProps {
                    readonly: true,
                    ..InputProps::default()
                },
                window,
                cx,
            );
            assert!(!input.state.read(cx).is_editable());
            input.apply_group_editing(cx.entity_id(), false, false, cx);
            assert!(
                !input.state.read(cx).is_editable(),
                "the control's own readonly prop must win after the overlay lifts"
            );
        });
    }

    #[gpui::test]
    fn controlled_content_restores_tokens_atomically_and_rejects_invalid_updates(
        cx: &mut TestAppContext,
    ) {
        let fixture = crate::components::test_support::Fixture::<TextControl<SingleLine>>::new(
            InputProps {
                content: Some(InputContentSnapshot {
                    text: "see @ada".into(),
                    tokens: vec![InputTokenSpan {
                        id: "user-1".into(),
                        text: "@ada".into(),
                        label: "Ada".into(),
                        anchor_byte: 4,
                        head_byte: 8,
                    }],
                }),
                ..Default::default()
            },
            cx,
        );
        fixture.update(cx, |input, _, cx| {
            assert_eq!(input.state.read(cx).value().as_ref(), "see @ada");
            let content = input.state.read(cx).content();
            assert_eq!(content.tokens().len(), 1);
            assert_eq!(content.tokens()[0].token().id().as_ref(), "user-1");
            assert_eq!(content.tokens()[0].range(), 4..8);
        });
        // The controlled snapshot round-trips: text and tokens return
        // together through the same pending/acknowledgement channel.
        fixture.update(cx, |input, window, cx| {
            input.update(
                InputProps {
                    content: Some(InputContentSnapshot {
                        text: "cc @bob".into(),
                        tokens: vec![InputTokenSpan {
                            id: "user-2".into(),
                            text: "@bob".into(),
                            label: "Bob".into(),
                            anchor_byte: 3,
                            head_byte: 7,
                        }],
                    }),
                    ack_edit_seq: 0,
                    ..Default::default()
                },
                window,
                cx,
            );
        });
        tick(cx);
        fixture.update(cx, |input, _, cx| {
            assert_eq!(input.state.read(cx).value().as_ref(), "cc @bob");
            let content = input.state.read(cx).content();
            assert_eq!(content.tokens().len(), 1);
            assert_eq!(content.tokens()[0].token().id().as_ref(), "user-2");
        });
        // A snapshot whose tokens do not match its text is rejected before
        // any mutation: value and tokens both stay untouched.
        let valid = <TextControl<SingleLine> as NativeView>::validate_props(&InputProps {
            content: Some(InputContentSnapshot {
                text: "cc ".into(),
                tokens: vec![InputTokenSpan {
                    id: "user-3".into(),
                    text: "@carol".into(),
                    label: "Carol".into(),
                    anchor_byte: 0,
                    head_byte: 6,
                }],
            }),
            ..Default::default()
        });
        assert!(valid.is_err());
        fixture.update(cx, |input, _, cx| {
            assert_eq!(input.state.read(cx).value().as_ref(), "cc @bob");
            assert_eq!(input.state.read(cx).content().tokens().len(), 1);
        });
        // `value` and `content` never carry the document together.
        assert!(
            <TextControl<SingleLine> as NativeView>::validate_props(&InputProps {
                value: Some("plain".into()),
                content: Some(InputContentSnapshot::default()),
                ..Default::default()
            })
            .is_err()
        );
    }
}
