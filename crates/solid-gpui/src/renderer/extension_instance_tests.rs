use super::*;
use crate::protocol::{
    CommandOperation, EventPayload, ExtensionField, ExtensionProperties, ExtensionValue, Node,
    Patch, Snapshot, UPDATE_LISTENER, UPDATE_PROPERTIES,
};
use crate::transport::InMemoryAdapter;
use crate::tree::KIND_EXTENSION;
use gpui::{AnyElement, App, AppContext, Entity, EntityId, TestAppContext, WindowHandle};

const PROVIDER: [u8; 16] = [7; 16];
const DIGEST: [u8; 32] = [9; 32];

#[derive(Default)]
struct Evidence {
    mounts: Cell<usize>,
    updates: Cell<usize>,
    drops: Cell<usize>,
    entities: RefCell<Vec<EntityId>>,
    sinks: RefCell<Vec<ExtensionEventSink>>,
    calls: RefCell<Vec<u32>>,
}
struct NativeView(u32);
impl Render for NativeView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size_full()
    }
}
struct Instance {
    entity: Entity<NativeView>,
    evidence: Rc<Evidence>,
}
impl Drop for Instance {
    fn drop(&mut self) {
        self.evidence.drops.set(self.evidence.drops.get() + 1);
    }
}
fn value(props: &ExtensionProperties) -> u32 {
    match props.fields.first().map(|field| &field.value) {
        Some(ExtensionValue::U32(value)) => *value,
        _ => 0,
    }
}
impl ExtensionInstance for Instance {
    fn update(
        &mut self,
        props: &ExtensionProperties,
        _: ExtensionEventSink,
        _: &mut Window,
        cx: &mut App,
    ) {
        self.evidence.updates.set(self.evidence.updates.get() + 1);
        self.entity.update(cx, |view, cx| {
            view.0 = value(props);
            cx.notify();
        });
    }
    fn render(&self, _: &mut dyn Iterator<Item = AnyElement>) -> AnyElement {
        self.entity.clone().into_any_element()
    }
    fn invoke(
        &mut self,
        function: u32,
        _: &[u8],
        _: &mut Window,
        cx: &mut App,
    ) -> Result<Vec<u8>, String> {
        if function != 1 {
            return Err("unknown method".into());
        }
        let value = self.entity.read(cx).0;
        self.evidence.calls.borrow_mut().push(value);
        Ok(value.to_le_bytes().to_vec())
    }
}
struct Registry(Rc<Evidence>);
impl ExtensionRegistry for Registry {
    fn resolve(
        &self,
        provider: [u8; 16],
        digest: [u8; 32],
        entry: u32,
        version: u32,
    ) -> Option<&dyn ExtensionAdapter> {
        (provider == PROVIDER && digest == DIGEST && (entry == 1 || entry == 2) && version == 1)
            .then_some(self)
    }
}
impl ExtensionAdapter for Registry {
    fn validate(
        &self,
        node_id: u32,
        properties: &ExtensionProperties,
        _: ExtensionChildSummary,
    ) -> Result<(), ExtensionError> {
        if value(properties) == 999 {
            return Err(ExtensionError::InvalidProperties {
                node_id,
                reason: "invalid test value".into(),
            });
        }
        Ok(())
    }
    fn render(&self, _: ExtensionRenderContext<'_>) -> AnyElement {
        panic!("native instance was not mounted")
    }
    fn mount(
        &self,
        _: u32,
        props: &ExtensionProperties,
        sink: ExtensionEventSink,
        _: &mut Window,
        cx: &mut App,
    ) -> Option<Box<dyn ExtensionInstance>> {
        self.0.mounts.set(self.0.mounts.get() + 1);
        let entity = cx.new(|_| NativeView(value(props)));
        self.0.entities.borrow_mut().push(entity.entity_id());
        self.0.sinks.borrow_mut().push(sink);
        Some(Box::new(Instance {
            entity,
            evidence: self.0.clone(),
        }))
    }
}
fn properties(number: u32, entry: u32) -> ExtensionProperties {
    ExtensionProperties {
        provider_id: PROVIDER,
        catalog_digest: DIGEST,
        entry_id: entry,
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::U32(number),
        }],
        event_ids: Arc::from([1]),
    }
}
fn node(number: u32) -> Node {
    let mut node = Node::new(2, 1, 0, KIND_EXTENSION);
    node.listener_id = 10;
    node.host_properties = Some(HostProperties::Extension(properties(number, 1)));
    node
}
fn snapshot(epoch: u32) -> DecodedMessage {
    DecodedMessage::Snapshot(Snapshot::new(
        1,
        epoch,
        0,
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            node(10),
            Node::new(3, 1, 1, KIND_VIEW),
        ],
    ))
}
fn update(number: u32, listener: u32, entry: u32) -> PatchOperation {
    PatchOperation::Update {
        id: 2,
        mask: UPDATE_PROPERTIES | UPDATE_LISTENER,
        style: None,
        text: None,
        listener_id: listener,
        host_properties: Some(HostProperties::Extension(properties(number, entry))),
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    }
}
fn patch(revision: u32, operations: Vec<PatchOperation>) -> DecodedMessage {
    DecodedMessage::Patch(Patch::new(1, 1, revision - 1, revision, operations))
}
fn setup(cx: &mut TestAppContext) -> (WindowHandle<SolidRoot>, Rc<Evidence>, Arc<InMemoryAdapter>) {
    let evidence = Rc::new(Evidence::default());
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(px(400.0), px(300.0)), {
        let evidence = evidence.clone();
        let runtime = runtime.clone();
        move |_, _| SolidRoot::with_extensions(runtime, Rc::new(Registry(evidence)))
    });
    window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(snapshot(1), window, cx)
        })
        .unwrap()
        .unwrap();
    (window, evidence, runtime)
}
fn apply(
    window: WindowHandle<SolidRoot>,
    cx: &mut TestAppContext,
    message: DecodedMessage,
) -> Result<(), RenderError> {
    window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(message, window, cx)
        })
        .unwrap()
}
fn drain(runtime: &InMemoryAdapter) -> Vec<Event> {
    let mut events = Vec::new();
    while let Some(event) = runtime.take_event().unwrap() {
        events.push(event);
    }
    events
}

#[gpui::test]
fn retained_identity_listener_binding_and_child_updates(cx: &mut TestAppContext) {
    let (window, evidence, runtime) = setup(cx);
    let original = evidence.entities.borrow()[0];
    let subscription_sink = evidence.sinks.borrow()[0].clone();
    apply(window, cx, patch(2, vec![update(20, 11, 1)])).unwrap();
    assert_eq!(evidence.mounts.get(), 1);
    assert_eq!(evidence.updates.get(), 1);
    assert_eq!(evidence.entities.borrow()[0], original);
    drain(&runtime);
    subscription_sink.emit(1, vec![]).unwrap();
    let event = runtime.take_event().unwrap().unwrap();
    assert_eq!((event.meta.listener_id, event.meta.revision), (11, 2));
    let mut unsubscribe = update(20, 0, 1);
    if let PatchOperation::Update {
        host_properties: Some(HostProperties::Extension(props)),
        ..
    } = &mut unsubscribe
    {
        props.event_ids = Arc::from([]);
    }
    apply(window, cx, patch(3, vec![unsubscribe])).unwrap();
    assert!(matches!(
        subscription_sink.emit(1, vec![]),
        Err(ExtensionError::EventNotSubscribed { .. })
    ));
    apply(window, cx, patch(4, vec![update(20, 12, 1)])).unwrap();
    subscription_sink.emit(1, vec![]).unwrap();
    let event = runtime.take_event().unwrap().unwrap();
    assert_eq!((event.meta.listener_id, event.meta.revision), (12, 4));
    // Child changes must update a retained parent even with unchanged props.
    apply(
        window,
        cx,
        patch(
            5,
            vec![PatchOperation::Create(Node::new(4, 2, 0, KIND_VIEW))],
        ),
    )
    .unwrap();
    assert_eq!(evidence.updates.get(), 4);
    // Move the component under an existing container; native identity survives.
    apply(
        window,
        cx,
        patch(
            6,
            vec![PatchOperation::Move {
                id: 2,
                parent_id: 3,
                index: 0,
            }],
        ),
    )
    .unwrap();
    assert_eq!(evidence.mounts.get(), 1);
    assert_eq!(evidence.drops.get(), 0);
    let before = evidence.updates.get();
    cx.update_window(window.into(), |_, window, cx| {
        window.draw(cx).clear(cx);
        window.draw(cx).clear(cx);
    })
    .unwrap();
    assert_eq!(evidence.updates.get(), before);
}

#[gpui::test]
fn rejected_batch_has_no_native_effects_and_replacements_revoke_sinks(cx: &mut TestAppContext) {
    let (window, evidence, _) = setup(cx);
    let sink = evidence.sinks.borrow()[0].clone();
    let mut bad = node(999);
    bad.id = 4;
    bad.index = 2;
    assert!(
        apply(
            window,
            cx,
            patch(2, vec![update(20, 11, 1), PatchOperation::Create(bad)])
        )
        .is_err()
    );
    assert_eq!(
        (
            evidence.mounts.get(),
            evidence.updates.get(),
            evidence.drops.get()
        ),
        (1, 0, 0)
    );
    window
        .read_with(cx, |root, _| assert_eq!(root.store.revision(), 1))
        .unwrap();
    assert_eq!(sink.listener_id(), 10);
    // A different component contract at the same node creates a new instance.
    apply(window, cx, patch(2, vec![update(20, 10, 2)])).unwrap();
    assert_eq!((evidence.mounts.get(), evidence.drops.get()), (2, 1));
    assert!(matches!(
        sink.emit(1, vec![]),
        Err(ExtensionError::EventRouteRetired { .. })
    ));
    let replacement_sink = evidence.sinks.borrow()[1].clone();
    apply(window, cx, snapshot(2)).unwrap();
    assert_eq!((evidence.mounts.get(), evidence.drops.get()), (3, 2));
    assert!(replacement_sink.emit(1, vec![]).is_err());
    let current_sink = evidence.sinks.borrow()[2].clone();
    apply(
        window,
        cx,
        DecodedMessage::Patch(Patch::new(
            1,
            2,
            1,
            2,
            vec![PatchOperation::Delete { id: 2 }],
        )),
    )
    .unwrap();
    assert_eq!(evidence.drops.get(), 3);
    assert!(current_sink.emit(1, vec![]).is_err());
}

#[gpui::test]
fn native_node_methods_execute_before_next_commit_and_reject_bad_targets(cx: &mut TestAppContext) {
    let (window, evidence, runtime) = setup(cx);
    drain(&runtime);
    let command = |revision, provider| {
        DecodedMessage::Command(Command::new(
            CommandMeta {
                surface_id: 1,
                epoch: 1,
                after_revision: revision,
                request_id: revision + 10,
                node_id: 2,
            },
            CommandOperation::InvokeNative {
                module_id: provider,
                module_digest: DIGEST,
                function_id: 1,
                args: vec![],
            },
        ))
    };
    apply(window, cx, command(1, PROVIDER)).unwrap();
    // No draw between the method and the next props commit.
    apply(window, cx, patch(2, vec![update(20, 10, 1)])).unwrap();
    apply(window, cx, command(2, [0; 16])).unwrap();
    apply(window, cx, command(1, PROVIDER)).unwrap();
    assert_eq!(&*evidence.calls.borrow(), &[10]);
    let results: Vec<_> = drain(&runtime)
        .into_iter()
        .filter_map(|event| match event.payload {
            EventPayload::CommandResult(result) => Some(result),
            _ => None,
        })
        .collect();
    assert_eq!(results.len(), 3);
    assert!(results[0].success);
    assert_eq!(
        results[0].value,
        Some(CommandValue::Bytes(10u32.to_le_bytes().to_vec()))
    );
    assert!(!results[1].success);
    assert!(!results[2].success);
}

#[test]
fn root_drop_revokes_a_retained_external_event_sink() {
    let runtime = InMemoryAdapter::new();
    let root = SolidRoot::new(runtime);
    let sink = ExtensionEventSink::new(root.extension_event_state.clone(), 2, 10, Arc::from([1]));
    drop(root);
    assert!(matches!(
        sink.emit(1, vec![]),
        Err(ExtensionError::EventRouteRetired { .. })
    ));
}
