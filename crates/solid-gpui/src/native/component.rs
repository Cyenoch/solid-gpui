use super::{Serialize, TS, Types, decode_json, encode_json};
use crate::protocol::{ExtensionField, ExtensionProperties, ExtensionValue};
use crate::renderer::ExtensionInstance;
use crate::{
    ExtensionAdapter, ExtensionChildSummary, ExtensionError, ExtensionEventSink,
    ExtensionRenderContext,
};
use gpui::{AnyElement, App, AppContext, Context, Entity, IntoElement, Render, Window};
use serde::de::DeserializeOwned;
use std::{any::Any, collections::BTreeMap, marker::PhantomData};

pub struct Event<T> {
    sink: ExtensionEventSink,
    id: u32,
    marker: PhantomData<fn(T)>,
}
impl<T> Event<T> {
    pub fn is_subscribed(&self) -> bool {
        self.sink.is_subscribed(self.id)
    }
    pub(crate) fn new(sink: ExtensionEventSink, id: u32) -> Self {
        Self {
            sink,
            id,
            marker: PhantomData,
        }
    }
}
impl<T> Clone for Event<T> {
    fn clone(&self) -> Self {
        Self {
            sink: self.sink.clone(),
            id: self.id,
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
    event_ids: &'a BTreeMap<String, u32>,
    children: &'a mut dyn Iterator<Item = AnyElement>,
}
impl ElementContext<'_> {
    pub fn id(&self) -> gpui::ElementId {
        gpui::ElementId::Integer(self.node_id as u64)
    }
    pub fn children(&mut self) -> &mut dyn Iterator<Item = AnyElement> {
        self.children
    }
    pub fn event<T>(&self, name: &str) -> Event<T> {
        Event::new(
            self.sink.clone(),
            *self.event_ids.get(name).expect("declared native event"),
        )
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

/// A retained native view owns its GPUI state. Props synchronization is explicit.
pub trait NativeView: Render + Sized + 'static {
    type Props: DeserializeOwned + TS + 'static;
    type Event: Serialize + TS + 'static;
    fn mount(
        props: Self::Props,
        event: Event<Self::Event>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self;
    fn update(&mut self, props: Self::Props, window: &mut Window, cx: &mut Context<Self>);
    fn commands() -> Vec<ViewCommand<Self>> {
        Vec::new()
    }
    fn event_name() -> &'static str {
        "event"
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
type CommandSignature = fn(&mut Types) -> (String, String);
type Mount = dyn Fn(
    u32,
    Box<dyn Any>,
    ExtensionEventSink,
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
    pub(crate) prop_names: Option<&'static [&'static str]>,
    pub(crate) controlled: Option<ControlledBinding>,
    decode: Decoder,
    mount: Box<Mount>,
}
fn decode<P: DeserializeOwned + 'static>(bytes: &[u8]) -> Result<Box<dyn Any>, String> {
    Ok(Box::new(decode_json::<P>(bytes)?))
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
    pub fn with_contract(mut self, source: &'static str) -> Self {
        self.source = source;
        self
    }
    pub fn with_children(mut self, children: bool) -> Self {
        self.children = children;
        self
    }
    pub fn with_props(mut self, names: &'static [&'static str]) -> Self {
        self.prop_names = Some(names);
        self
    }
    pub fn element<P: DeserializeOwned + TS + 'static>(
        name: &'static str,
        mut events: Vec<EventDefinition>,
        render: fn(&P, &mut ElementContext<'_>) -> AnyElement,
    ) -> Self {
        events.sort_by_key(|event| event.name);
        let event_ids: BTreeMap<String, u32> = events
            .iter()
            .enumerate()
            .map(|(i, e)| (e.name.into(), i as u32 + 1))
            .collect();
        assert_eq!(event_ids.len(), events.len(), "duplicate native event");
        Self {
            name,
            props: Types::collect::<P>,
            events,
            commands: Vec::new(),
            source: "",
            children: true,
            prop_names: None,
            controlled: None,
            decode: decode::<P>,
            mount: Box::new(move |node_id, props, sink, _, _| {
                Box::new(ElementInstance::<P> {
                    node_id,
                    props: *props.downcast::<P>().unwrap(),
                    sink,
                    event_ids: event_ids.clone(),
                    render,
                })
            }),
        }
    }
    pub fn view<V: NativeView>(name: &'static str) -> Self {
        let mut methods = V::commands();
        methods.sort_by_key(|m| m.name);
        let commands = methods.iter().map(|m| (m.name, m.describe)).collect();
        Self {
            name,
            props: Types::collect::<V::Props>,
            events: vec![EventDefinition::new::<V::Event>(V::event_name())],
            commands,
            source: "",
            children: false,
            prop_names: None,
            controlled: V::controlled(),
            decode: decode::<V::Props>,
            mount: Box::new(|_, props, sink, window, cx| {
                let props = *props.downcast::<V::Props>().unwrap();
                let entity = cx.new(|cx| V::mount(props, Event::new(sink, 1), window, cx));
                let mut commands = V::commands();
                commands.sort_by_key(|m| m.name);
                Box::new(ViewInstance::<V> { entity, commands })
            }),
        }
    }
}
impl ExtensionAdapter for ComponentDefinition {
    fn validate(
        &self,
        node_id: u32,
        props: &ExtensionProperties,
        children: ExtensionChildSummary,
    ) -> Result<(), ExtensionError> {
        if !self.children && children.count != 0 {
            return Err(ExtensionError::InvalidChildren {
                node_id,
                reason: "retained native view does not accept JS children".into(),
            });
        }
        property_bytes(props)
            .and_then(|bytes| (self.decode)(bytes))
            .map_err(|reason| ExtensionError::InvalidProperties { node_id, reason })?;
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
        window: &mut Window,
        cx: &mut App,
    ) -> Option<Box<dyn ExtensionInstance>> {
        let props = (self.decode)(property_bytes(props).expect("validated props"))
            .expect("validated native props");
        Some((self.mount)(node_id, props, sink, window, cx))
    }
}
struct ElementInstance<P> {
    node_id: u32,
    props: P,
    sink: ExtensionEventSink,
    event_ids: BTreeMap<String, u32>,
    render: fn(&P, &mut ElementContext<'_>) -> AnyElement,
}
impl<P: DeserializeOwned + 'static> ExtensionInstance for ElementInstance<P> {
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
    fn render(&self, children: &mut dyn Iterator<Item = AnyElement>) -> AnyElement {
        (self.render)(
            &self.props,
            &mut ElementContext {
                node_id: self.node_id,
                sink: &self.sink,
                event_ids: &self.event_ids,
                children,
            },
        )
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
    fn render(&self, _: &mut dyn Iterator<Item = AnyElement>) -> AnyElement {
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
