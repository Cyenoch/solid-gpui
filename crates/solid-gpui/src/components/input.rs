//! Retained text controls share the native editing engine and acknowledgement protocol.
use super::ControlSize;
use crate::native::{
    ComponentDefinition, ControlledBinding, Event, EventDefinition, NativeChildren, NativeView,
    ViewCommand,
};
use gpui::Window;
use gpui::{
    AnyElement, AppContext, Context, Entity, EntityInputHandler, IntoElement, Render, Subscription,
    Task,
};
use gpui_base::input::{
    EditorMode, InputBaseState, InputMode, InputModeKind, NumberStep, TabSize, TextareaMode,
};
use gpui_component::input::{Input as GpuiInput, InputEvent, InputState};
use gpui_component::{Disableable, Sizable};
use std::time::Duration;

trait TextProps: serde::de::DeserializeOwned + crate::native::TS + 'static {
    fn value(&self) -> &Option<String>;
    fn default_value(&self) -> &Option<String>;
    fn placeholder(&self) -> &Option<String>;
    fn disabled(&self) -> bool;
    fn readonly(&self) -> bool;
    fn ack_edit_seq(&self) -> u32;
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
            $(pub $field: $ty,)*
        }
        impl Default for $name {
            fn default() -> Self { Self { value: None, default_value: None, placeholder: None, disabled: false, readonly: false, ack_edit_seq: 0, appearance: true, $($field: $default,)* } }
        }
        impl TextProps for $name {
            fn value(&self) -> &Option<String> { &self.value }
            fn default_value(&self) -> &Option<String> { &self.default_value }
            fn placeholder(&self) -> &Option<String> { &self.placeholder }
            fn disabled(&self) -> bool { self.disabled }
            fn readonly(&self) -> bool { self.readonly }
            fn ack_edit_seq(&self) -> u32 { self.ack_edit_seq }
        }
    }
}
text_props!(InputProps { size: ControlSize = ControlSize::Medium, bordered: bool = true, aria_label: Option<String> = None, masked: bool = false, cleanable: bool = false, mask_toggle: bool = false });
text_props!(NumberInputProps { size: ControlSize = ControlSize::Medium, step: f64 = 1., min: Option<f64> = None, max: Option<f64> = None });
text_props!(TextareaProps { bordered: bool = true, aria_label: Option<String> = None, rows: usize = 2, auto_grow: Option<AutoGrow> = None, soft_wrap: bool = true, searchable: bool = false });
text_props!(EditorProps { bordered: bool = true, aria_label: Option<String> = None, language: String = String::new(), soft_wrap: bool = true, searchable: bool = true, line_numbers: bool = true, folding: bool = true, indent_guides: bool = true, tab_size: usize = 2, hard_tabs: bool = false, show_whitespaces: bool = false, scroll_beyond_last_line: Option<usize> = None, cursor_surrounding_lines: Option<usize> = None });

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
pub struct InputSelection {
    pub start_byte: usize,
    pub end_byte: usize,
    pub edit_seq: u32,
}

trait TextMode: Sized + 'static {
    type Mode: InputModeKind;
    type Props: TextProps;
    const SLOTS: &'static [&'static str] = &[];
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
    ) -> AnyElement;
    fn validate(_props: &Self::Props) -> Result<(), String> {
        Ok(())
    }
}
struct SingleLine;
struct Numeric;
struct MultiLine;
struct Code;
#[cfg(test)]
type Input = TextControl<SingleLine>;

struct PendingValue {
    value: String,
    ack_edit_seq: u32,
}

struct TextControl<M: TextMode> {
    state: Entity<InputBaseState<M::Mode>>,
    props: M::Props,
    event: Event<InputChange>,
    edit_seq: u32,
    committed_value: String,
    pending: Option<PendingValue>,
    retry_task: Option<Task<()>>,
    _subscription: Subscription,
    children: NativeChildren,
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
        let initial = props
            .value()
            .as_deref()
            .or(props.default_value().as_deref())
            .unwrap_or_default();
        let state = cx.new(|cx| {
            let mut state = M::new(window, cx).default_value(initial);
            state.set_placeholder(props.placeholder().clone().unwrap_or_default(), window, cx);
            state.set_disabled(props.disabled(), cx);
            state.set_readonly(props.readonly(), cx);
            M::sync(&mut state, &props, None, window, cx);
            state
        });
        let committed_value = state.read(cx).value().to_string();
        let subscription =
            cx.subscribe_in(&state, window, |this, state, event, _, cx| match event {
                InputEvent::Change => {
                    let value = state.read(cx).value().to_string();
                    this.observe_committed_value(value);
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
        Self {
            state,
            props,
            event,
            edit_seq: 0,
            committed_value,
            pending: None,
            retry_task: None,
            _subscription: subscription,
            children,
        }
    }

    fn update(&mut self, props: M::Props, window: &mut Window, cx: &mut Context<Self>) {
        self.retry_task.take();
        self.pending = None;
        let entering_controlled = self.props.value().is_none() && props.value().is_some();
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
        if let Some(value) = self.props.value() {
            // Match the underlying single-line InputState normalization, so an
            // identical normalized echo never resets caret or undo history.
            self.pending = Some(PendingValue {
                value: if M::Mode::MULTI_LINE {
                    value.clone()
                } else {
                    value.replace(['\n', '\r'], "")
                },
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
        vec![
            EventDefinition::new::<()>("blur"),
            EventDefinition::new::<()>("focus"),
            EventDefinition::new::<InputSubmit>("submit"),
        ]
    }
    fn event_name() -> &'static str {
        "change"
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
        vec![
            ViewCommand::new("focus", Self::focus),
            ViewCommand::new("replaceValue", Self::replace_value),
            ViewCommand::new("getSelection", Self::get_selection),
            ViewCommand::new("setSelection", Self::set_selection),
            ViewCommand::new("insert", Self::insert),
        ]
    }
}

impl<M: TextMode> TextControl<M> {
    fn get_selection(
        &mut self,
        (): (),
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<InputSelection, String> {
        let range = self.state.read(cx).selected_range();
        Ok(InputSelection {
            start_byte: range.start,
            end_byte: range.end,
            edit_seq: self.edit_seq,
        })
    }
    fn set_selection(
        &mut self,
        selection: InputSelection,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if selection.edit_seq != self.edit_seq {
            return Err("selection targets an obsolete edit sequence".into());
        }
        let value = self.state.read(cx).value().to_string();
        if selection.start_byte > selection.end_byte
            || !value.is_char_boundary(selection.start_byte)
            || !value.is_char_boundary(selection.end_byte)
        {
            return Err("selection must contain ordered UTF-8 byte boundaries".into());
        }
        self.state.update(cx, |state, cx| {
            state.set_selected_range(selection.start_byte..selection.end_byte, cx)
        });
        Ok(())
    }
    fn insert(
        &mut self,
        value: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Result<(), String> {
        if self.props.disabled() || self.props.readonly() {
            return Err("input is not editable".into());
        }
        self.state
            .update(cx, |state, cx| state.insert(value, window, cx));
        Ok(())
    }

    fn focus(&mut self, (): (), window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.props.disabled() {
            return Err("disabled input cannot be focused".into());
        }
        self.state.update(cx, |state, cx| state.focus(window, cx));
        Ok(())
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

    fn observe_committed_value(&mut self, value: String) {
        if value == self.committed_value {
            return;
        }
        self.edit_seq = self
            .edit_seq
            .checked_add(1)
            .expect("native input edit sequence exhausted");
        self.committed_value = value.clone();
        self.pending = None;
        self.event.emit(InputChange {
            value,
            edit_seq: self.edit_seq,
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
            self.observe_committed_value(value);
            return false;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.value == value)
        {
            self.pending = None;
            return false;
        }
        if composing {
            return true;
        }
        let pending = self.pending.take().expect("pending value was checked");
        self.state
            .update(cx, |state, cx| state.set_value(pending.value, window, cx));
        self.committed_value = self.state.read(cx).value().to_string();
        false
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
        M::render(&self.state, &self.props, &self.children)
    }
}

impl TextMode for SingleLine {
    type Mode = InputMode;
    type Props = InputProps;
    const SLOTS: &'static [&'static str] = &["prefix", "suffix"];
    fn new(window: &mut Window, cx: &mut Context<InputState>) -> InputState {
        InputState::new(window, cx)
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
    fn render(s: &Entity<InputState>, p: &InputProps, children: &NativeChildren) -> AnyElement {
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
        v.into_any_element()
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
    fn new(
        window: &mut Window,
        cx: &mut Context<gpui_base::input::TextareaState>,
    ) -> gpui_base::input::TextareaState {
        gpui_base::input::TextareaState::new(window, cx)
    }
    fn validate(p: &TextareaProps) -> Result<(), String> {
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
    fn render(
        s: &Entity<gpui_base::input::TextareaState>,
        p: &TextareaProps,
        _: &NativeChildren,
    ) -> AnyElement {
        let mut v = gpui_component::input::Textarea::new(s)
            .disabled(p.disabled)
            .readonly(p.readonly)
            .appearance(p.appearance)
            .bordered(p.bordered);
        if let Some(label) = &p.aria_label {
            v = v.aria_label(label.clone());
        }
        v.into_any_element()
    }
}
impl TextMode for Code {
    type Mode = EditorMode;
    type Props = EditorProps;
    fn new(
        window: &mut Window,
        cx: &mut Context<gpui_base::input::EditorState>,
    ) -> gpui_base::input::EditorState {
        gpui_base::input::EditorState::new(window, cx)
    }
    fn validate(p: &EditorProps) -> Result<(), String> {
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
        _: &NativeChildren,
    ) -> AnyElement {
        let mut v = gpui_component::input::Editor::new(s)
            .disabled(p.disabled)
            .readonly(p.readonly)
            .appearance(p.appearance)
            .bordered(p.bordered);
        if let Some(label) = &p.aria_label {
            v = v.aria_label(label.clone());
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
    fn controlled_echo_preserves_real_selection_and_undo_and_rejects_stale_ack(
        cx: &mut TestAppContext,
    ) {
        let fixture = Fixture::new(cx);
        fixture.commit(cx, "abc");
        assert_eq!(
            fixture.changes().last(),
            Some(&InputChange {
                value: "abc".into(),
                edit_seq: 1
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
                edit_seq: 1
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
                    code.set_selection(
                        InputSelection {
                            start_byte: 7,
                            end_byte: 15,
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
}
