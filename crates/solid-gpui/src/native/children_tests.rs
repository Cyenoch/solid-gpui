use super::*;
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::{
    DecodedMessage, HostProperties, InMemoryAdapter, KIND_EXTENSION, KIND_TEXT, KIND_VIEW, Node,
    Patch, PatchOperation, Snapshot, SolidRoot, UPDATE_PROPERTIES,
};
use gpui::{
    App, AppContext, Context, IntoElement, ParentElement, Render, RenderOnce, Styled,
    TestAppContext, Window,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

#[crate::native_type]
#[derive(Default)]
struct Props {
    #[serde(default)]
    text: String,
}
#[derive(Clone, Default)]
struct Probe(Rc<RefCell<Evidence>>);
impl gpui::Global for Probe {}
#[derive(Default)]
struct Evidence {
    mounts: usize,
    values: Vec<String>,
    content: Option<NativeChildren>,
    states: std::collections::HashMap<String, gpui::EntityId>,
}
#[derive(IntoElement)]
struct Item(String);
impl RenderOnce for Item {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state("same-local-id", cx, |_, _| ());
        cx.global::<Probe>()
            .0
            .borrow_mut()
            .states
            .insert(self.0.clone(), state.entity_id());
        cx.global::<Probe>()
            .0
            .borrow_mut()
            .values
            .push(self.0.clone());
        gpui::div().w(gpui::px(100.)).h(gpui::px(20.)).child(self.0)
    }
}
struct Panel(NativeChildren);
impl NativeView for Panel {
    type Props = Props;
    type Event = ();
    fn mount(
        _: Props,
        _: Event<()>,
        children: NativeChildren,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let probe = &mut *cx.global::<Probe>().0.borrow_mut();
        probe.mounts += 1;
        probe.content = Some(children.clone());
        Self(children)
    }
    fn update(&mut self, _: Props, _: &mut Window, _: &mut Context<Self>) {}
    fn accepts_children() -> bool {
        true
    }
    fn slots() -> &'static [&'static str] {
        &["header"]
    }
}
impl Render for Panel {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
            .child(self.0.slot("header"))
            .child(self.0.content())
    }
}
struct TypedLeaf(String);
struct TypedGroup(NativeChildren);
impl NativeView for TypedGroup {
    type Props = Props;
    type Event = ();
    fn accepts_children() -> bool {
        true
    }
    fn mount(
        _: Props,
        _: Event<()>,
        children: NativeChildren,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Self {
        Self(children)
    }
    fn update(&mut self, _: Props, _: &mut Window, _: &mut Context<Self>) {}
}
impl Render for TypedGroup {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui::div().children(
            self.0
                .typed_children::<TypedLeaf>(cx)
                .into_iter()
                .map(|child| child.map_native(|leaf| gpui::div().child(leaf.0))),
        )
    }
}
fn module() -> ModuleDefinition {
    ModuleDefinition::new(
        "children-test",
        vec![
            ComponentDefinition::element::<Props, _>("Group", vec![], |_, cx| {
                gpui::div().children(cx.typed_children::<Item>())
            })
            .with_child_type::<Item>(),
            ComponentDefinition::element::<Props, _>("Item", vec![], |p, _| Item(p.text.clone())),
            ComponentDefinition::view::<Panel>("Panel"),
            ComponentDefinition::view::<TypedGroup>("TypedGroup").with_child_type::<TypedLeaf>(),
            ComponentDefinition::descriptor::<Props, _>("TypedLeaf", vec![], |p, _| {
                TypedLeaf(p.text.clone())
            })
            .with_typed_parent_required(),
        ],
        vec![],
    )
}
fn props(module: &ModuleDefinition, entry_id: u32, text: &str) -> HostProperties {
    HostProperties::Extension(ExtensionProperties {
        provider_id: module.id(),
        catalog_digest: module.digest(),
        entry_id,
        entry_version: 1,
        fields: vec![ExtensionField {
            id: 1,
            value: ExtensionValue::Bytes(encode_json(&Props { text: text.into() }).unwrap()),
        }],
        event_ids: Arc::from([]),
    })
}
fn extension(
    module: &ModuleDefinition,
    id: u32,
    parent: u32,
    index: u32,
    entry: u32,
    text: &str,
) -> Node {
    let mut node = Node::new(id, parent, index, KIND_EXTENSION);
    node.host_properties = Some(props(module, entry, text));
    if entry == 2 {
        node.listener_id = id + 100;
        node.style = Some(crate::Style {
            width: Some(if id == 8 { 180. } else { 120. }),
            height: Some(30.),
            ..Default::default()
        });
    }
    node
}
#[gpui::test]
fn typed_children_and_slots_render_updates_without_remount_and_retire_with_owner(
    cx: &mut TestAppContext,
) {
    let probe = Probe::default();
    cx.update(|cx| cx.set_global(probe.clone()));
    let module = Rc::new(module());
    let runtime = InMemoryAdapter::new();
    let window = cx.open_window(gpui::size(gpui::px(400.), gpui::px(240.)), {
        let module = module.clone();
        let runtime = runtime.clone();
        move |_, _| SolidRoot::with_extensions(runtime, module)
    });
    let snapshot = Snapshot::new(
        1,
        1,
        0,
        1,
        vec![
            Node::new(1, 0, 0, KIND_VIEW),
            extension(&module, 2, 1, 0, 3, ""),
            Node::new(3, 2, 0, KIND_VIEW),
            extension(&module, 5, 3, 0, 1, ""),
            extension(&module, 6, 5, 0, 2, "first"),
            extension(&module, 8, 5, 1, 2, "second"),
            extension(&module, 9, 3, 1, 4, ""),
            extension(&module, 10, 9, 0, 5, "retained typed content"),
            Node::new(4, 2, 1, KIND_VIEW),
            extension(&module, 7, 4, 0, 2, "header"),
        ],
    );
    window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(DecodedMessage::Snapshot(snapshot), window, cx)
        })
        .unwrap()
        .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    cx.update_window(window.into(), |_, window, cx| {
        window.simulate_next_frame(cx)
    })
    .unwrap();
    cx.executor().run_until_parked();
    assert_ne!(
        probe.0.borrow().states["first"],
        probe.0.borrow().states["second"],
        "typed children must scope identical local keys independently"
    );
    let first_state = probe.0.borrow().states["first"];
    let mut frames = std::collections::HashMap::new();
    while let Some(event) = runtime.take_event().unwrap() {
        if let crate::EventPayload::Layout { width, height, .. } = event.payload {
            frames.insert(event.meta.node_id, (width, height));
        }
    }
    assert_eq!(
        frames.get(&6),
        Some(&(120., 30.)),
        "typed non-Styled child must retain wrapper style and onLayout"
    );
    assert_eq!(frames.get(&8), Some(&(180., 30.)));
    assert_eq!(
        frames.get(&7),
        Some(&(120., 30.)),
        "ordinary path must agree"
    );
    assert_eq!(probe.0.borrow().mounts, 1);
    assert!(probe.0.borrow().values.contains(&"first".to_string()));
    assert!(probe.0.borrow().values.contains(&"header".to_string()));
    let content = probe.0.borrow().content.clone().unwrap();
    assert_eq!(content.content().len(), 2);
    let ids = content.content().node_ids();
    assert_eq!(&*ids, &[5, 9]);
    assert_eq!(content.slot("header").len(), 1);
    let update = PatchOperation::Update {
        id: 6,
        mask: UPDATE_PROPERTIES,
        style: None,
        text: None,
        listener_id: 0,
        host_properties: Some(props(&module, 2, "updated")),
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    };
    window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(
                DecodedMessage::Patch(Patch::new(1, 1, 1, 2, vec![update])),
                window,
                cx,
            )
        })
        .unwrap()
        .unwrap();
    cx.update_window(window.into(), |_, window, cx| window.draw(cx).clear(cx))
        .unwrap();
    assert_eq!(probe.0.borrow().mounts, 1);
    assert!(probe.0.borrow().values.contains(&"updated".to_string()));
    assert_eq!(probe.0.borrow().states["updated"], first_state);
    assert!(
        Rc::ptr_eq(&ids, &content.content().node_ids()),
        "content identities are shared across an in-place child update"
    );
    assert_eq!(&*content.content().changed_indices(), &[0]);
    let error = window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(
                DecodedMessage::Patch(Patch::new(
                    1,
                    1,
                    2,
                    3,
                    vec![PatchOperation::Move {
                        id: 10,
                        parent_id: 4,
                        index: 1,
                    }],
                )),
                window,
                cx,
            )
        })
        .unwrap()
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("requires its declared typed parent"),
        "{error}"
    );
    window
        .read_with(cx, |root, _| {
            assert_eq!(root.store().revision(), 2);
            assert_eq!(root.store().get(10).unwrap().parent_id, 9);
        })
        .unwrap();
    // A wrong concrete child fails atomically, before calling any native builder.
    let bad = Node::new(11, 5, 2, KIND_TEXT);
    let error = window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(
                DecodedMessage::Patch(Patch::new(1, 1, 2, 3, vec![PatchOperation::Create(bad)])),
                window,
                cx,
            )
        })
        .unwrap()
        .unwrap_err();
    assert!(
        error.to_string().contains("declared native type"),
        "{error}"
    );
    window
        .read_with(cx, |root, _| assert_eq!(root.store().revision(), 2))
        .unwrap();
    window
        .update(cx, |root, window, cx| {
            root.apply_decoded_message_in_window(
                DecodedMessage::Patch(Patch::new(
                    1,
                    1,
                    2,
                    3,
                    vec![PatchOperation::Delete { id: 2 }],
                )),
                window,
                cx,
            )
        })
        .unwrap()
        .unwrap();
    assert!(content.slot("header").is_empty());
    cx.update(|cx| assert!(content.content().elements(cx).is_empty()));
}
