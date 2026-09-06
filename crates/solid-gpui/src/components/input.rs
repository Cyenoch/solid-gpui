//! Retained gpui-component input. Editing state stays in InputState; only values
//! and their acknowledgement sequence cross the native component boundary.
use crate::native::{ComponentDefinition, ControlledBinding, Event, NativeView, ViewCommand};
use gpui::{
    AppContext, Context, Entity, EntityInputHandler, IntoElement, Render, Subscription, Task,
    Window,
};
use gpui_component::input::{Input as GpuiInput, InputEvent, InputState};
use std::time::Duration;

#[crate::native_type]
#[derive(Clone, Debug, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct InputProps {
    /// Controlled value: onChange must synchronously accept or transform edits.
    /// Use defaultValue and replaceValue for asynchronous validation workflows.
    pub value: Option<String>,
    pub default_value: Option<String>,
    pub placeholder: Option<String>,
    pub disabled: bool,
    pub ack_edit_seq: u32,
}

#[crate::native_type]
#[derive(Clone, Debug, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct InputChange {
    pub value: String,
    pub edit_seq: u32,
}

struct PendingValue {
    value: String,
    ack_edit_seq: u32,
}

pub struct Input {
    state: Entity<InputState>,
    props: InputProps,
    event: Event<InputChange>,
    edit_seq: u32,
    committed_value: String,
    pending: Option<PendingValue>,
    retry_task: Option<Task<()>>,
    _subscription: Subscription,
}

#[crate::component]
impl NativeView for Input {
    type Props = InputProps;
    type Event = InputChange;

    fn mount(
        props: InputProps,
        event: Event<InputChange>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let initial = props
            .value
            .as_deref()
            .or(props.default_value.as_deref())
            .unwrap_or_default();
        let state = cx.new(|cx| {
            InputState::new(window, cx)
                .default_value(initial)
                .placeholder(props.placeholder.clone().unwrap_or_default())
        });
        let committed_value = state.read(cx).value().to_string();
        let subscription = cx.subscribe_in(&state, window, |this, state, event, _, cx| {
            if matches!(event, InputEvent::Change) {
                let value = state.read(cx).value().to_string();
                this.observe_committed_value(value);
                // A subsequent native edit supersedes a pending external write.
                this.pending = None;
                this.retry_task.take();
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
        }
    }

    fn update(&mut self, props: InputProps, window: &mut Window, cx: &mut Context<Self>) {
        self.retry_task.take();
        self.pending = None;
        let entering_controlled = self.props.value.is_none() && props.value.is_some();
        if self.props.placeholder != props.placeholder {
            self.state.update(cx, |state, cx| {
                state.set_placeholder(props.placeholder.clone().unwrap_or_default(), window, cx)
            });
        }
        self.props = props;
        if let Some(value) = &self.props.value {
            // Match the underlying single-line InputState normalization, so an
            // identical normalized echo never resets caret or undo history.
            self.pending = Some(PendingValue {
                value: value.replace(['\n', '\r'], ""),
                // Entering controlled mode is an explicit ownership change;
                // uncontrolled edits may never have had a JS listener to ack.
                ack_edit_seq: if entering_controlled {
                    self.props.ack_edit_seq.max(self.edit_seq)
                } else {
                    self.props.ack_edit_seq
                },
            });
            if self.reconcile_pending(window, cx) {
                self.start_retry(window, cx);
            }
        }
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
        ]
    }
}

impl Input {
    fn focus(&mut self, (): (), window: &mut Window, cx: &mut Context<Self>) -> Result<(), String> {
        if self.props.disabled {
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

impl Render for Input {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        GpuiInput::new(&self.state).disabled(self.props.disabled)
    }
}

pub(crate) fn definition() -> ComponentDefinition {
    __native_component_Input()
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
                        Input::mount(InputProps::default(), Event::new(sink, 1), window, cx)
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
}
