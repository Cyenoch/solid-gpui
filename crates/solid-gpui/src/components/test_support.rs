use crate::{
    ExtensionChildren, ExtensionEventSink, InMemoryAdapter, Node, Snapshot, SolidRoot,
    native::{Event, NativeChildren, NativeView},
};
use gpui::{AppContext, Context, Entity, TestAppContext, Window, WindowHandle};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc, sync::Arc};
pub(super) struct Fixture<V: NativeView> {
    pub window: WindowHandle<gpui_component::Root>,
    pub view: Entity<V>,
    pub runtime: Arc<InMemoryAdapter>,
    _bridge: Entity<SolidRoot>,
}
impl<V: NativeView> Fixture<V> {
    pub fn new(props: V::Props, cx: &mut TestAppContext) -> Self {
        cx.update(gpui_component::init);
        let runtime = InMemoryAdapter::new();
        let captured = Rc::new(RefCell::new(None));
        let window = cx.open_window(gpui::size(gpui::px(500.), gpui::px(400.)), {
            let captured = captured.clone();
            let runtime = runtime.clone();
            move |window, cx| {
                let bridge = cx.new(|_| SolidRoot::new(runtime));
                bridge.update(cx, |root, cx| {
                    root.apply_decoded_message(
                        crate::protocol::DecodedMessage::Snapshot(Snapshot::new(
                            1,
                            1,
                            0,
                            1,
                            vec![Node::new(1, 0, 0, crate::KIND_VIEW)],
                        )),
                        cx,
                    )
                    .unwrap()
                });
                let mut names: BTreeMap<_, _> = if V::emits_primary_event() {
                    [(V::event_name().to_string(), 1)].into()
                } else {
                    BTreeMap::new()
                };
                let primary_count = names.len() as u32;
                let mut additional = V::additional_events();
                additional.sort_by_key(|e| e.name);
                for (i, event) in additional.into_iter().enumerate() {
                    names.insert(event.name.to_string(), i as u32 + primary_count + 1);
                }
                let mut ids: Vec<u32> = names.values().copied().collect();
                ids.sort_unstable();
                let ids: Arc<[u32]> = ids.into();
                let sink =
                    ExtensionEventSink::new(bridge.read(cx).extension_event_state(), 2, 10, ids);
                let view = cx.new(|cx| {
                    V::mount(
                        props,
                        Event::with_events(sink.clone(), primary_count, Rc::new(names)),
                        NativeChildren::new(
                            ExtensionChildren::new(bridge.downgrade(), sink),
                            V::slots(),
                        ),
                        window,
                        cx,
                    )
                });
                *captured.borrow_mut() = Some((view.clone(), bridge));
                gpui_component::Root::new(view, window, cx)
            }
        });
        let (view, bridge) = captured.borrow_mut().take().unwrap();
        Self {
            window,
            view,
            runtime,
            _bridge: bridge,
        }
    }
    pub fn update<R>(
        &self,
        cx: &mut TestAppContext,
        f: impl FnOnce(&mut V, &mut Window, &mut Context<V>) -> R,
    ) -> R {
        cx.update_window(self.window.into(), |_, window, cx| {
            self.view.update(cx, |view, cx| f(view, window, cx))
        })
        .unwrap()
    }
}
