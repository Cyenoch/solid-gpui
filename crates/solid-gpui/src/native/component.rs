use super::{Serialize, TS, Types, decode_json, encode_json};
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::renderer::ExtensionInstance;
use crate::{
    ExtensionAdapter, ExtensionChildIterator, ExtensionChildSummary, ExtensionChildren,
    ExtensionContent, ExtensionError, ExtensionEventSink, ExtensionRenderContext,
};
use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, Render, Window};
use serde::de::DeserializeOwned;
use std::{any::Any, collections::BTreeMap, marker::PhantomData, rc::Rc};

pub struct Event<T> {
    sink: ExtensionEventSink,
    id: u32,
    names: Rc<BTreeMap<String, u32>>,
    marker: PhantomData<fn(T)>,
}
impl<T> Event<T> {
    pub fn is_subscribed(&self) -> bool {
        self.sink.is_subscribed(self.id)
    }
    pub fn related<U>(&self, name: &str) -> Event<U> {
        Event {
            sink: self.sink.clone(),
            id: *self.names.get(name).expect("declared native view event"),
            names: self.names.clone(),
            marker: PhantomData,
        }
    }
    pub(crate) fn with_events(
        sink: ExtensionEventSink,
        id: u32,
        names: Rc<BTreeMap<String, u32>>,
    ) -> Self {
        Self {
            sink,
            id,
            names,
            marker: PhantomData,
        }
    }
}
impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self {
            sink: self.sink.clone(),
            id: self.id,
            names: self.names.clone(),
            marker: PhantomData,
        }
    }
}
impl<T: Serialize> Event<T> {
    pub fn emit(&self, value: T) {
        if !self.is_subscribed() {
            return;
        }
        let bytes =
            encode_json(&value).expect("native event must satisfy its declared DTO contract");
        // A revoked/unsubscribed endpoint has no consumer. The sink enforces lifetime and ordering.
        match self.sink.emit(
            self.id,
            vec![ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(bytes),
            }],
        ) {
            Ok(())
            | Err(
                ExtensionError::EventRouteRetired { .. }
                | ExtensionError::EventNotSubscribed { .. },
            ) => {}
            Err(error) => panic!("native event violated its contract: {error}"),
        }
    }
}
pub struct ElementContext<'a> {
    node_id: u32,
    sink: &'a ExtensionEventSink,
    event_ids: &'a Rc<BTreeMap<String, u32>>,
    children: &'a mut dyn ExtensionChildIterator,
    content: &'a NativeChildren,
}
impl ElementContext<'_> {
    pub fn id(&self) -> gpui::ElementId {
        gpui::ElementId::Integer(self.node_id as u64)
    }
    pub fn children(&mut self) -> &mut dyn Iterator<Item = AnyElement> {
        self.children
    }
    pub fn typed_children<T: 'static>(&mut self) -> NativeItems<T> {
        let mut items = Vec::new();
        while let Some(item) = self.children.next_native() {
            items.push(
                item.map_native(|item| *item.downcast::<T>().expect("validated native child type")),
            );
        }
        NativeItems(items)
    }
    pub fn slot(&self, name: &str) -> ExtensionContent {
        self.content.slot(name)
    }
    pub fn content(&self) -> ExtensionContent {
        self.content.content()
    }
    pub fn event<T>(&self, name: &str) -> Event<T> {
        Event::with_events(
            self.sink.clone(),
            *self.event_ids.get(name).expect("declared native event"),
            self.event_ids.clone(),
        )
    }
}

/// Typed compound components consume their native children before type erasure.
pub struct NativeItems<T>(pub(crate) Vec<NativeChild<T>>);
/// Carries a host node's rendering scope through native parent configuration.
#[derive(Clone)]
pub struct NativeChild<T> {
    pub(crate) native: T,
    pub(crate) boundary: Rc<dyn Fn(AnyElement) -> AnyElement>,
}
impl<T> NativeChild<T> {
    pub fn map_native<U>(self, f: impl FnOnce(T) -> U) -> NativeChild<U> {
        NativeChild {
            native: f(self.native),
            boundary: self.boundary,
        }
    }
}
impl<T: IntoElement> IntoElement for NativeChild<T> {
    type Element = AnyElement;
    fn into_element(self) -> AnyElement {
        (self.boundary)(self.native.into_any_element())
    }
}
impl<T: gpui::Styled> gpui::Styled for NativeChild<T> {
    fn style(&mut self) -> &mut gpui::StyleRefinement {
        self.native.style()
    }
}
#[cfg(feature = "gpui-component")]
impl<T> From<NativeChild<T>> for gpui_component::ComponentChild<T> {
    fn from(child: NativeChild<T>) -> Self {
        Self::with_boundary(child.native, child.boundary)
    }
}
#[cfg(feature = "gpui-component")]
impl<T: gpui_component::Sizable> gpui_component::Sizable for NativeChild<T> {
    fn with_size(self, size: impl Into<gpui_component::Size>) -> Self {
        self.map_native(|v| v.with_size(size))
    }
}
#[cfg(feature = "gpui-component")]
impl<T: gpui_component::ChildElement> gpui_component::ChildElement for NativeChild<T> {
    fn with_ix(self, ix: usize) -> Self {
        self.map_native(|v| v.with_ix(ix))
    }
}
#[cfg(feature = "gpui-component")]
impl<T: gpui_component::Collapsible> gpui_component::Collapsible for NativeChild<T> {
    fn is_collapsed(&self) -> bool {
        self.native.is_collapsed()
    }
    fn collapsed(self, value: bool) -> Self {
        self.map_native(|v| v.collapsed(value))
    }
}
#[cfg(feature = "gpui-component")]
impl<T: gpui_component::sidebar::SidebarItem + 'static> gpui_component::sidebar::SidebarItem
    for NativeChild<T>
{
    fn render(
        self,
        id: impl Into<gpui::ElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> impl IntoElement {
        gpui_component::ComponentChild::<T>::from(self).render(id, window, cx)
    }
}
impl<T> IntoIterator for NativeItems<T> {
    type Item = NativeChild<T>;
    type IntoIter = std::vec::IntoIter<NativeChild<T>>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

pub struct EventDefinition {
    pub(crate) name: &'static str,
    pub(crate) describe: fn(&mut Types) -> String,
}
impl EventDefinition {
    pub fn new<T: Serialize + TS + 'static>(name: &'static str) -> Self {
        Self {
            name,
            describe: Types::collect::<T>,
        }
    }
}

/// Child content is evaluated only from the committed Rust tree. Cloning a
/// handle neither clones a Solid owner nor retains a removed component.
#[derive(Clone)]
pub struct NativeChildren {
    source: ExtensionChildren,
    slots: &'static [&'static str],
}
pub type NativeSlot = ExtensionContent;
impl NativeChildren {
    pub fn new(source: ExtensionChildren, slots: &'static [&'static str]) -> Self {
        Self { source, slots }
    }
    pub fn content(&self) -> ExtensionContent {
        self.source.content((!self.slots.is_empty()).then_some(0))
    }
    /// Consume native children from a retained view after its declared child type
    /// has been validated at publication.
    pub fn typed_children<T: 'static>(&self, cx: &App) -> NativeItems<T> {
        self.content().native_items(cx)
    }

    pub fn slot(&self, name: &str) -> ExtensionContent {
        let index = self
            .slots
            .iter()
            .position(|slot| *slot == name)
            .expect("declared native slot");
        self.source.content(Some(index + 1))
    }
}

/// A retained native view owns its GPUI state. Props synchronization is explicit.
pub trait NativeView: Render + Sized + 'static {
    type Props: DeserializeOwned + TS + 'static;
    type Event: Serialize + TS + 'static;
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        children: NativeChildren,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self;
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>);
    fn validate_props(_props: &Self::Props) -> Result<(), String> {
        Ok(())
    }
    fn validate_children(
        _props: &Self::Props,
        _children: &ExtensionChildSummary,
    ) -> Result<(), String> {
        Ok(())
    }
    fn unmount(&mut self, _window: &mut Window, _cx: &mut App) {}
    fn additional_events() -> Vec<EventDefinition> {
        Vec::new()
    }
    fn accepts_children() -> bool {
        false
    }
    fn slots() -> &'static [&'static str] {
        &[]
    }
    fn commands() -> Vec<ViewCommand<Self>> {
        Vec::new()
    }
    fn event_name() -> &'static str {
        "event"
    }
    /// Pure retained rendering components have no synthetic event in their JS API.
    fn emits_primary_event() -> bool {
        true
    }
    fn controlled() -> Option<ControlledBinding> {
        None
    }
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ControlledBinding {
    pub value_prop: &'static str,
    pub event_id: u32,
    pub sequence_field: &'static str,
    pub ack_prop: &'static str,
}

type ViewHandler<V> =
    dyn Fn(&mut V, &[u8], &mut Window, &mut Context<V>) -> Result<Vec<u8>, String>;
pub struct ViewCommand<V: 'static> {
    pub(crate) name: &'static str,
    pub(crate) describe: fn(&mut Types) -> (String, String),
    handler: Box<ViewHandler<V>>,
}
impl<V: 'static> ViewCommand<V> {
    pub fn new<I: DeserializeOwned + TS + 'static, O: Serialize + TS + 'static>(
        name: &'static str,
        handler: fn(&mut V, I, &mut Window, &mut Context<V>) -> Result<O, String>,
    ) -> Self {
        Self {
            name,
            describe: |types| (types.collect::<I>(), types.collect::<O>()),
            handler: Box::new(move |view, bytes, window, cx| {
                encode_json(&handler(view, decode_json(bytes)?, window, cx)?)
            }),
        }
    }
}

type Decoder = fn(&[u8]) -> Result<Box<dyn Any>, String>;
type CompositionValidator = dyn Fn(&dyn Any, &ExtensionChildSummary) -> Result<(), String>;
type CommandSignature = fn(&mut Types) -> (String, String);
type Mount = dyn Fn(
    u32,
    Box<dyn Any>,
    ExtensionEventSink,
    NativeChildren,
    &mut Window,
    &mut App,
) -> Box<dyn ExtensionInstance>;
pub struct ComponentDefinition {
    pub(crate) name: &'static str,
    pub(crate) props: fn(&mut Types) -> String,
    pub(crate) events: Vec<EventDefinition>,
    pub(crate) commands: Vec<(&'static str, CommandSignature)>,
    pub(crate) source: &'static str,
    pub(crate) children: bool,
    pub(crate) slots: &'static [&'static str],
    pub(crate) element_type: Option<std::any::TypeId>,
    pub(crate) child_type: Option<std::any::TypeId>,
    pub(crate) native_style: bool,
    pub(crate) requires_typed_parent: bool,
    pub(crate) prop_names: Option<&'static [&'static str]>,
    pub(crate) controlled: Option<ControlledBinding>,
    decode: Decoder,
    validate_composition: Box<CompositionValidator>,
    mount: Box<Mount>,
}
fn decode_view<V: NativeView>(bytes: &[u8]) -> Result<Box<dyn Any>, String> {
    let props: V::Props = decode_json(bytes)?;
    V::validate_props(&props)?;
    Ok(Box::new(props))
}
fn decode<P: DeserializeOwned + 'static>(bytes: &[u8]) -> Result<Box<dyn Any>, String> {
    Ok(Box::new(decode_json::<P>(bytes)?))
}
impl ExtensionChildSummary<'_> {
    /// Decode direct native child props for cross-child composition validation.
    pub fn native_props<P: DeserializeOwned>(&self) -> Result<Vec<P>, String> {
        self.properties
            .iter()
            .map(|props| decode_json(property_bytes(props.ok_or("expected native child props")?)?))
            .collect()
    }
}
pub(crate) fn property_bytes(props: &ExtensionProperties) -> Result<&[u8], String> {
    match props.fields.as_slice() {
        [
            ExtensionField {
                id: 1,
                value: ExtensionValue::Bytes(bytes),
            },
        ] => Ok(bytes),
        _ => Err("native props require one typed JSON byte field".into()),
    }
}
impl ComponentDefinition {
    pub fn with_validation<P: 'static>(
        mut self,
        validate: fn(&P, &ExtensionChildSummary) -> Result<(), String>,
    ) -> Self {
        self.validate_composition = Box::new(move |props, children| {
            validate(
                props.downcast_ref::<P>().expect("validator props type"),
                children,
            )
        });
        self
    }
    pub fn with_contract(mut self, source: &'static str) -> Self {
        self.source = source;
        self
    }
    pub fn with_children(mut self, children: bool) -> Self {
        self.children = children;
        self
    }
    pub fn with_slots(mut self, slots: &'static [&'static str]) -> Self {
        self.slots = slots;
        self
    }
    pub fn with_typed_parent_required(mut self) -> Self {
        self.requires_typed_parent = true;
        self
    }
    pub fn with_child_type<T: 'static>(mut self) -> Self {
        self.child_type = Some(std::any::TypeId::of::<T>());
        self
    }
    pub fn with_props(mut self, names: &'static [&'static str]) -> Self {
        self.prop_names = Some(names);
        self
    }
    pub fn element<P: DeserializeOwned + TS + 'static, E: IntoElement + 'static>(
        name: &'static str,
        events: Vec<EventDefinition>,
        render: fn(&P, &mut ElementContext<'_>) -> E,
    ) -> Self {
        Self::element_with_style(
            name,
            events,
            render,
            |element, _| element,
            false,
            Some(E::into_any_element),
        )
    }
    pub fn styled_element<
        P: DeserializeOwned + TS + 'static,
        E: IntoElement + gpui::Styled + 'static,
    >(
        name: &'static str,
        events: Vec<EventDefinition>,
        render: fn(&P, &mut ElementContext<'_>) -> E,
    ) -> Self {
        Self::element_with_style(
            name,
            events,
            render,
            crate::renderer::paint::apply_style_to_extension,
            true,
            Some(E::into_any_element),
        )
    }
    /// Build a native configuration object consumed by its typed parent.
    /// Descriptors never enter the render tree as standalone elements.
    pub fn descriptor<P: DeserializeOwned + TS + 'static, E: 'static>(
        name: &'static str,
        events: Vec<EventDefinition>,
        build: fn(&P, &mut ElementContext<'_>) -> E,
    ) -> Self {
        Self::element_with_style(name, events, build, |element, _| element, false, None)
            .with_typed_parent_required()
    }
    pub fn styled_descriptor<P: DeserializeOwned + TS + 'static, E: gpui::Styled + 'static>(
        name: &'static str,
        events: Vec<EventDefinition>,
        build: fn(&P, &mut ElementContext<'_>) -> E,
    ) -> Self {
        Self::element_with_style(
            name,
            events,
            build,
            crate::renderer::paint::apply_style_to_extension,
            true,
            None,
        )
        .with_typed_parent_required()
    }
    fn element_with_style<P: DeserializeOwned + TS + 'static, E: 'static>(
        name: &'static str,
        mut events: Vec<EventDefinition>,
        render: fn(&P, &mut ElementContext<'_>) -> E,
        apply_style: fn(E, Option<&crate::protocol::Style>) -> E,
        native_style: bool,
        into_element: Option<fn(E) -> AnyElement>,
    ) -> Self {
        events.sort_by_key(|event| event.name);
        let event_ids: BTreeMap<String, u32> = events
            .iter()
            .enumerate()
            .map(|(i, e)| (e.name.into(), i as u32 + 1))
            .collect();
        assert_eq!(event_ids.len(), events.len(), "duplicate native event");
        let event_ids = Rc::new(event_ids);
        Self {
            name,
            props: Types::collect::<P>,
            events,
            commands: Vec::new(),
            source: "",
            children: true,
            slots: &[],
            element_type: Some(std::any::TypeId::of::<E>()),
            child_type: None,
            requires_typed_parent: false,
            validate_composition: Box::new(|_, _| Ok(())),
            native_style,
            prop_names: None,
            controlled: None,
            decode: decode::<P>,
            mount: Box::new(move |node_id, props, sink, content, _, _| {
                Box::new(ElementInstance::<P, E> {
                    node_id,
                    props: *props.downcast::<P>().unwrap(),
                    sink,
                    event_ids: event_ids.clone(),
                    render,
                    apply_style,
                    into_element,
                    content,
                })
            }),
        }
    }
    pub fn view<V: NativeView>(name: &'static str) -> Self {
        let mut methods = V::commands();
        methods.sort_by_key(|m| m.name);
        let commands = methods.iter().map(|m| (m.name, m.describe)).collect();
        let mut additional = V::additional_events();
        additional.sort_by_key(|event| event.name);
        let mut events = if V::emits_primary_event() {
            vec![EventDefinition::new::<V::Event>(V::event_name())]
        } else {
            vec![]
        };
        events.extend(additional);
        let event_ids = events
            .iter()
            .enumerate()
            .map(|(index, event)| (event.name.to_owned(), index as u32 + 1))
            .collect::<BTreeMap<_, _>>();
        assert_eq!(event_ids.len(), events.len(), "duplicate native view event");
        let event_ids = Rc::new(event_ids);
        Self {
            name,
            props: Types::collect::<V::Props>,
            events,
            commands,
            source: "",
            children: V::accepts_children(),
            slots: V::slots(),
            element_type: None,
            child_type: None,
            requires_typed_parent: false,
            validate_composition: Box::new(|props, children| {
                V::validate_children(
                    props
                        .downcast_ref::<V::Props>()
                        .expect("decoded view props"),
                    children,
                )
            }),
            native_style: false,
            prop_names: None,
            controlled: V::controlled(),
            decode: decode_view::<V>,
            mount: Box::new(move |_, props, sink, children, window, cx| {
                let props = *props.downcast::<V::Props>().unwrap();
                let entity = cx.new(|cx| {
                    cx.on_release_in(window, |view: &mut V, window, cx| view.unmount(window, cx))
                        .detach();
                    V::mount(
                        props,
                        Event::with_events(
                            sink,
                            if V::emits_primary_event() { 1 } else { 0 },
                            event_ids.clone(),
                        ),
                        children,
                        window,
                        cx,
                    )
                });
                let mut commands = V::commands();
                commands.sort_by_key(|m| m.name);
                Box::new(ViewInstance::<V> { entity, commands })
            }),
        }
    }
}
impl ExtensionAdapter for ComponentDefinition {
    fn element_type(&self) -> Option<std::any::TypeId> {
        self.element_type
    }
    fn native_style(&self) -> bool {
        self.native_style
    }
    fn child_type(&self) -> Option<std::any::TypeId> {
        self.child_type
    }
    fn requires_typed_parent(&self) -> bool {
        self.requires_typed_parent
    }
    fn default_child_group(&self) -> Option<usize> {
        (!self.slots.is_empty()).then_some(0)
    }
    fn validate(
        &self,
        node_id: u32,
        props: &ExtensionProperties,
        children: ExtensionChildSummary,
    ) -> Result<(), ExtensionError> {
        if self.slots.is_empty() && !self.children && children.count != 0 {
            return Err(ExtensionError::InvalidChildren {
                node_id,
                reason: "retained native view does not accept JS children".into(),
            });
        }
        if !self.slots.is_empty()
            && (children.count != self.slots.len() + 1
                || children.kinds.iter().any(|kind| *kind != crate::KIND_VIEW))
        {
            return Err(ExtensionError::InvalidChildren { node_id, reason: "native slots require one ordered View group per declared slot and default content".into() });
        }
        if let Some(expected) = self.child_type
            && children
                .element_types
                .iter()
                .any(|actual| *actual != Some(expected))
        {
            return Err(ExtensionError::InvalidChildren {
                node_id,
                reason: "this compound component requires children of its declared native type"
                    .into(),
            });
        }
        let decoded = property_bytes(props)
            .and_then(|bytes| (self.decode)(bytes))
            .map_err(|reason| ExtensionError::InvalidProperties { node_id, reason })?;
        (self.validate_composition)(decoded.as_ref(), &children)
            .map_err(|reason| ExtensionError::InvalidChildren { node_id, reason })?;
        if props
            .event_ids
            .iter()
            .any(|id| *id == 0 || *id as usize > self.events.len())
        {
            return Err(ExtensionError::InvalidProperties {
                node_id,
                reason: "unknown native event".into(),
            });
        }
        Ok(())
    }
    fn render(&self, _: ExtensionRenderContext<'_>) -> AnyElement {
        panic!("native component must be mounted by the window commit entry")
    }
    fn mount(
        &self,
        node_id: u32,
        props: &ExtensionProperties,
        sink: ExtensionEventSink,
        children: ExtensionChildren,
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Box<dyn ExtensionInstance>> {
        let props = (self.decode)(property_bytes(props).expect("validated props"))
            .expect("validated native props");
        Some((self.mount)(
            node_id,
            props,
            sink,
            NativeChildren {
                source: children,
                slots: self.slots,
            },
            window,
            cx,
        ))
    }
}
struct ElementInstance<P, E> {
    content: NativeChildren,
    node_id: u32,
    props: P,
    sink: ExtensionEventSink,
    event_ids: Rc<BTreeMap<String, u32>>,
    render: fn(&P, &mut ElementContext<'_>) -> E,
    apply_style: fn(E, Option<&crate::protocol::Style>) -> E,
    into_element: Option<fn(E) -> AnyElement>,
}
impl<P: DeserializeOwned + 'static, E: 'static> ExtensionInstance for ElementInstance<P, E> {
    fn update(
        &mut self,
        props: &ExtensionProperties,
        sink: ExtensionEventSink,
        _: &mut Window,
        _: &mut App,
    ) {
        self.props = decode_json(property_bytes(props).unwrap()).expect("validated native props");
        self.sink = sink;
    }
    fn render(&self, context: ExtensionRenderContext<'_>) -> AnyElement {
        self.into_element
            .expect("descriptor requires its validated typed parent")(self.build(context))
    }
    fn build_native(&self, context: ExtensionRenderContext<'_>) -> Option<Box<dyn Any>> {
        Some(Box::new(self.build(context)))
    }
}
impl<P, E> ElementInstance<P, E> {
    fn build(&self, context: ExtensionRenderContext<'_>) -> E {
        let element = (self.render)(
            &self.props,
            &mut ElementContext {
                node_id: self.node_id,
                sink: &self.sink,
                event_ids: &self.event_ids,
                children: context.children,
                content: &self.content,
            },
        );
        (self.apply_style)(element, context.style)
    }
}

struct ViewInstance<V: NativeView> {
    entity: Entity<V>,
    commands: Vec<ViewCommand<V>>,
}
impl<V: NativeView> ExtensionInstance for ViewInstance<V> {
    fn update(
        &mut self,
        props: &ExtensionProperties,
        _: ExtensionEventSink,
        window: &mut Window,
        cx: &mut App,
    ) {
        let props = decode_json(property_bytes(props).unwrap()).expect("validated native props");
        self.entity.update(cx, |view, cx| {
            view.update(props, window, cx);
            cx.notify();
        });
    }
    fn render(&self, _: ExtensionRenderContext<'_>) -> AnyElement {
        self.entity.clone().into_any_element()
    }
    fn invoke(
        &mut self,
        id: u32,
        args: &[u8],
        window: &mut Window,
        cx: &mut App,
    ) -> Result<Vec<u8>, String> {
        let command = self
            .commands
            .get(id.wrapping_sub(1) as usize)
            .ok_or("unknown native view command")?;
        self.entity
            .update(cx, |view, cx| (command.handler)(view, args, window, cx))
    }
}
