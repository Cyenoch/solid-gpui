use gpui_kit::test::TestWindowExt;
use gpui_kit::{AppContext, Entity, TestAppContext, WindowHandle, px, size};
use serde_json::{Value, json};
use solid_gpui::native::ModuleDefinition;
use solid_gpui::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use solid_gpui::{
    DecodedMessage, EventPayload, HostProperties, InMemoryAdapter, KIND_EXTENSION, KIND_RAW_TEXT,
    KIND_TEXT, KIND_VIEW, Node, Patch, PatchOperation, Snapshot, SolidRoot, Style,
    UPDATE_PROPERTIES,
};
use std::{cell::RefCell, rc::Rc, sync::Arc};

struct Fixture {
    module: Rc<ModuleDefinition>,
    root: Entity<SolidRoot>,
    window: WindowHandle<gpui_kit::component::Root>,
    runtime: Arc<InMemoryAdapter>,
    revision: u32,
}
impl Fixture {
    fn new(cx: &mut TestAppContext) -> Self {
        cx.update(solid_gpui::components::initialize);
        let runtime = InMemoryAdapter::new();
        let module = Rc::new(solid_gpui::components::native_module());
        let captured = Rc::new(RefCell::new(None));
        let window = cx.open_window(size(px(480.), px(320.)), {
            let captured = captured.clone();
            let runtime = runtime.clone();
            let module = module.clone();
            move |window, cx| {
                let root = cx.new(|_| SolidRoot::with_extensions(runtime, module));
                *captured.borrow_mut() = Some(root.clone());
                gpui_kit::component::Root::new(root, window, cx)
            }
        });
        let root = captured.borrow_mut().take().unwrap();
        Self {
            module,
            root,
            window,
            runtime,
            revision: 1,
        }
    }
    fn props(&self, name: &str, value: Value, events: bool) -> HostProperties {
        HostProperties::Extension(ExtensionProperties {
            provider_id: self.module.id(),
            catalog_digest: self.module.digest(),
            entry_id: self.module.component_id(name).expect("native component"),
            entry_version: 1,
            fields: vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(serde_json::to_vec(&value).unwrap()),
            }],
            event_ids: if events {
                Arc::from([1])
            } else {
                Arc::from([])
            },
        })
    }
    fn node(
        &self,
        name: &str,
        id: u32,
        parent: u32,
        index: u32,
        value: Value,
        events: bool,
    ) -> Node {
        let mut node = Node::new(id, parent, index, KIND_EXTENSION);
        node.host_properties = Some(self.props(name, value, events));
        if events {
            node.listener_id = id + 100;
        }
        node
    }
    fn snapshot(&self, nodes: Vec<Node>, cx: &mut TestAppContext) {
        cx.update_window(self.window.into(), |_, window, cx| {
            self.root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    DecodedMessage::Snapshot(Snapshot::new(1, 1, 0, 1, nodes)),
                    window,
                    cx,
                )
                .unwrap();
            })
        })
        .unwrap();
        self.draw(cx);
    }
    fn patch(&mut self, ops: Vec<PatchOperation>, cx: &mut TestAppContext) {
        let revision = self.revision;
        cx.update_window(self.window.into(), |_, window, cx| {
            self.root.update(cx, |root, cx| {
                root.apply_decoded_message_in_window(
                    DecodedMessage::Patch(Patch::new(1, 1, revision, revision + 1, ops)),
                    window,
                    cx,
                )
                .unwrap();
            })
        })
        .unwrap();
        self.revision += 1;
        self.draw(cx);
    }
    fn draw(&self, cx: &mut TestAppContext) {
        cx.update_window(self.window.into(), |_, window, cx| window.render_frame(cx))
            .unwrap();
        cx.run_until_parked();
    }
    fn events(&self) -> Vec<Value> {
        let mut values = vec![];
        while let Some(event) = self.runtime.take_event().unwrap() {
            if let EventPayload::Extension { fields, .. } = event.payload {
                let ExtensionValue::Bytes(bytes) = &fields[0].value else {
                    panic!("native JSON event")
                };
                values.push(serde_json::from_slice(bytes).unwrap());
            }
        }
        values
    }
}
fn update(id: u32, props: HostProperties) -> PatchOperation {
    PatchOperation::Update {
        id,
        mask: UPDATE_PROPERTIES,
        host_properties: Some(props),
        style: None,
        text: None,
        listener_id: 0,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    }
}
fn root_node() -> Node {
    let mut node = Node::new(1, 0, 0, KIND_VIEW);
    node.style = Some(Style {
        width: Some(480.),
        height: Some(320.),
        ..Default::default()
    });
    node
}
fn text(id: u32, parent: u32, value: &str) -> [Node; 2] {
    let container = Node::new(id, parent, 0, KIND_TEXT);
    let mut text = Node::new(id + 10000, id, 0, KIND_RAW_TEXT);
    text.text = Some(value.into());
    [container, text]
}

#[gpui_kit::test]
fn base_checkbox_routes_real_activation_and_rejects_disabled_activation(cx: &mut TestAppContext) {
    let mut fixture = Fixture::new(cx);
    let mut checkbox = fixture.node(
        "BaseCheckbox",
        2,
        1,
        0,
        json!({"accessibilityLabel":"Accept","state":"unchecked"}),
        true,
    );
    checkbox.style = Some(Style {
        width: Some(150.),
        height: Some(40.),
        ..Default::default()
    });
    fixture.snapshot(
        vec![root_node(), checkbox]
            .into_iter()
            .chain(text(3, 2, "Accept"))
            .collect(),
        cx,
    );
    cx.update_window(fixture.window.into(), |_, window, cx| {
        assert_eq!(window.find(2usize).checked(), Some(false));
        window.click(2usize, cx);
    })
    .unwrap();
    assert_eq!(fixture.events(), vec![json!("checked")]);
    fixture.patch(
        vec![update(
            2,
            fixture.props(
                "BaseCheckbox",
                json!({"accessibilityLabel":"Accept","state":"checked","disabled":true}),
                true,
            ),
        )],
        cx,
    );
    cx.update_window(fixture.window.into(), |_, window, cx| {
        assert_eq!(window.find(2usize).checked(), Some(true));
        window.click(2usize, cx);
    })
    .unwrap();
    assert!(fixture.events().is_empty());
    fixture.patch(vec![PatchOperation::Delete { id: 2 }], cx);
    cx.update_window(fixture.window.into(), |_, window, _| {
        assert!(window.try_find(2usize).is_none())
    })
    .unwrap();
}

#[gpui_kit::test]
fn carousel_keyboard_selection_follows_keyed_items_after_reorder(cx: &mut TestAppContext) {
    let mut fixture = Fixture::new(cx);
    cx.update(|cx| cx.set_reduce_motion(true));
    let mut carousel = fixture.node(
        "Carousel",
        2,
        1,
        0,
        json!({"viewportHeight":150,"controls":false}),
        true,
    );
    carousel.style = Some(Style {
        width: Some(400.),
        ..Default::default()
    });
    fixture.snapshot(
        vec![
            root_node(),
            carousel,
            Node::new(3, 2, 0, KIND_VIEW),
            fixture.node("CarouselItem", 4, 3, 0, json!({}), false),
            fixture.node("CarouselItem", 6, 3, 1, json!({}), false),
            Node::new(8, 2, 1, KIND_VIEW),
            Node::new(9, 2, 2, KIND_VIEW),
        ]
        .into_iter()
        .chain(text(5, 4, "First"))
        .chain(text(7, 6, "Second"))
        .collect(),
        cx,
    );
    cx.update_window(fixture.window.into(), |_, window, cx| {
        window.click("carousel", cx);
        window.press("right", cx);
    })
    .unwrap();
    assert_eq!(fixture.events(), vec![json!({"index":1,"editSeq":1})]);
    fixture.patch(
        vec![PatchOperation::Move {
            id: 6,
            parent_id: 3,
            index: 0,
        }],
        cx,
    );
    cx.update_window(fixture.window.into(), |_, window, cx| {
        window.press("right", cx)
    })
    .unwrap();
    assert_eq!(
        fixture.events(),
        vec![json!({"index":1,"editSeq":2})],
        "selected logical item moved to index zero"
    );
    fixture.patch(vec![PatchOperation::Delete { id: 2 }], cx);
    cx.update_window(fixture.window.into(), |_, window, cx| {
        assert!(window.try_find("carousel").is_none());
        window.press("right", cx);
    })
    .unwrap();
    assert!(fixture.events().is_empty());
}
