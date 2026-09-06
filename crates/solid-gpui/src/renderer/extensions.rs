use gpui::{AnyElement, Element, InteractiveElement, ParentElement};
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use thiserror::Error;

use crate::protocol::{
    Event, ExtensionField, ExtensionProperties, ExtensionValue, MAX_EXTENSION_FIELDS,
    MAX_EXTENSION_TEXT_BYTES, MAX_EXTENSION_VALUE_BYTES,
};
use crate::transport::{RuntimeAdapter, send_event_or_exit};
use crate::tree::StoredNode;

/// A compact description of the children supplied to an extension adapter.
/// Children are ordered as they appear in the retained tree.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ExtensionChildSummary {
    pub count: usize,
    pub kinds: Vec<u32>,
}

impl ExtensionChildSummary {
    pub(crate) fn from_node(node: &StoredNode, store: &crate::tree::NodeStore) -> Self {
        let kinds = node
            .children(store)
            .map(|child| child.kind)
            .collect::<Vec<_>>();
        Self {
            count: kinds.len(),
            kinds,
        }
    }
}

/// Errors raised while resolving or validating a provider-neutral extension.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ExtensionError {
    #[error(
        "no extension adapter for provider {provider_id:?}, catalog {catalog_digest:?}, entry {entry_id} version {entry_version}"
    )]
    AdapterNotFound {
        provider_id: [u8; 16],
        catalog_digest: [u8; 32],
        entry_id: u32,
        entry_version: u32,
    },
    #[error("extension node {node_id} has invalid properties: {reason}")]
    InvalidProperties { node_id: u32, reason: String },
    #[error("extension node {node_id} has invalid children: {reason}")]
    InvalidChildren { node_id: u32, reason: String },
    #[error("extension event {event_id} is not subscribed by node {node_id}")]
    EventNotSubscribed { node_id: u32, event_id: u32 },
    #[error("extension node {node_id} event route has been retired")]
    EventRouteRetired { node_id: u32 },
    #[error("extension event {event_id} has invalid fields: {reason}")]
    InvalidEventFields { event_id: u32, reason: String },
}

/// A foreground-owned registry of provider-neutral extension adapters.
pub trait ExtensionRegistry {
    fn resolve(
        &self,
        provider_id: [u8; 16],
        catalog_digest: [u8; 32],
        entry_id: u32,
        entry_version: u32,
    ) -> Option<&dyn ExtensionAdapter>;

    /// Resolve a callable module on the foreground; only its owned, thread-safe
    /// implementation and immutable argument bytes move to the background.
    fn native_module(
        &self,
        _module_id: [u8; 16],
        _module_digest: [u8; 32],
    ) -> Option<Arc<dyn crate::native::NativeModule>> {
        None
    }
}

/// The default registry, which contains no provider implementations.
#[derive(Clone, Copy, Debug, Default)]
pub struct NoExtensions;

impl ExtensionRegistry for NoExtensions {
    fn resolve(
        &self,
        _provider_id: [u8; 16],
        _catalog_digest: [u8; 32],
        _entry_id: u32,
        _entry_version: u32,
    ) -> Option<&dyn ExtensionAdapter> {
        None
    }
}

/// An adapter validates wire properties before publication and renders a
/// validated node without a fallible fallback.
pub trait ExtensionAdapter {
    fn validate(
        &self,
        node_id: u32,
        properties: &ExtensionProperties,
        children: ExtensionChildSummary,
    ) -> Result<(), ExtensionError>;

    fn render(&self, context: ExtensionRenderContext<'_>) -> AnyElement;

    /// Called once after the complete candidate tree has passed validation.
    fn mount(
        &self,
        _node_id: u32,
        _properties: &ExtensionProperties,
        _sink: ExtensionEventSink,
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> Option<Box<dyn ExtensionInstance>> {
        None
    }
}

/// A foreground-owned component instance. Owned entities, subscriptions and tasks
/// are released when the host node is removed or its contract/epoch is replaced.
pub trait ExtensionInstance {
    fn update(
        &mut self,
        properties: &ExtensionProperties,
        sink: ExtensionEventSink,
        window: &mut gpui::Window,
        cx: &mut gpui::App,
    );
    fn render(&self, children: &mut dyn Iterator<Item = AnyElement>) -> AnyElement;
    fn invoke(
        &mut self,
        _function_id: u32,
        _args: &[u8],
        _window: &mut gpui::Window,
        _cx: &mut gpui::App,
    ) -> Result<Vec<u8>, String> {
        Err("this native component has no callable methods".to_owned())
    }
}

pub(super) struct MountedExtension {
    pub properties: ExtensionProperties,
    pub listener_id: u32,
    pub instance: Option<Box<dyn ExtensionInstance>>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct ExtensionContract {
    provider: [u8; 16],
    digest: [u8; 32],
    entry: u32,
    version: u32,
}
impl From<&ExtensionProperties> for ExtensionContract {
    fn from(props: &ExtensionProperties) -> Self {
        Self {
            provider: props.provider_id,
            digest: props.catalog_digest,
            entry: props.entry_id,
            version: props.entry_version,
        }
    }
}

pub(super) fn same_contract(a: &ExtensionProperties, b: &ExtensionProperties) -> bool {
    ExtensionContract::from(a) == ExtensionContract::from(b)
}

struct EventRoute {
    listener_id: Cell<u32>,
    event_ids: RefCell<Arc<[u32]>>,
    active: Cell<bool>,
    contract: Cell<Option<ExtensionContract>>,
}

pub(crate) struct ExtensionEventState {
    pub(crate) runtime: Arc<dyn RuntimeAdapter>,
    pub(crate) sequence: Arc<AtomicU32>,
    pub(crate) surface_id: Cell<u32>,
    pub(crate) epoch: Cell<u32>,
    pub(crate) revision: Cell<u32>,
    routes: RefCell<HashMap<u32, Rc<EventRoute>>>,
}

/// A cloneable event emitter scoped to one retained extension node.
#[derive(Clone)]
pub struct ExtensionEventSink {
    state: Rc<ExtensionEventState>,
    node_id: u32,
    route: Rc<EventRoute>,
    surface_id: u32,
    epoch: u32,
}

impl ExtensionEventSink {
    pub(crate) fn new(
        state: Rc<ExtensionEventState>,
        node_id: u32,
        listener_id: u32,
        event_ids: Arc<[u32]>,
    ) -> Self {
        let route = {
            let mut routes = state.routes.borrow_mut();
            let route = routes.entry(node_id).or_insert_with(|| {
                Rc::new(EventRoute {
                    listener_id: Cell::new(listener_id),
                    event_ids: RefCell::new(event_ids.clone()),
                    active: Cell::new(true),
                    contract: Cell::new(None),
                })
            });
            route.listener_id.set(listener_id);
            *route.event_ids.borrow_mut() = event_ids;
            route.clone()
        };
        Self {
            surface_id: state.surface_id.get(),
            epoch: state.epoch.get(),
            state,
            node_id,
            route,
        }
    }

    /// Whether this live instance currently has a consumer for the event.
    pub fn is_subscribed(&self, event_id: u32) -> bool {
        self.route.active.get()
            && self.surface_id == self.state.surface_id.get()
            && self.epoch == self.state.epoch.get()
            && self.route.listener_id.get() != 0
            && event_id != 0
            && self
                .route
                .event_ids
                .borrow()
                .binary_search(&event_id)
                .is_ok()
    }

    /// Emit one subscribed extension event. Invalid IDs or fields do not
    /// consume a sequence number or publish an event.
    pub fn emit(&self, event_id: u32, fields: Vec<ExtensionField>) -> Result<(), ExtensionError> {
        if !self.route.active.get()
            || self.surface_id != self.state.surface_id.get()
            || self.epoch != self.state.epoch.get()
        {
            return Err(ExtensionError::EventRouteRetired {
                node_id: self.node_id,
            });
        }
        if !self.is_subscribed(event_id) {
            return Err(ExtensionError::EventNotSubscribed {
                node_id: self.node_id,
                event_id,
            });
        }
        validate_extension_fields(&fields)
            .map_err(|reason| ExtensionError::InvalidEventFields { event_id, reason })?;
        let event = Event::extension(
            self.surface_id,
            self.epoch,
            self.state.revision.get(),
            self.state.sequence.fetch_add(1, Ordering::Relaxed),
            self.node_id,
            self.route.listener_id.get(),
            event_id,
            fields,
        );
        let _ = send_event_or_exit(self.state.runtime.as_ref(), "extension event", event);
        Ok(())
    }

    pub fn node_id(&self) -> u32 {
        self.node_id
    }

    pub fn listener_id(&self) -> u32 {
        self.route.listener_id.get()
    }
}

/// Data handed to an adapter's infallible render method.
pub struct ExtensionRenderContext<'a> {
    pub node_id: u32,
    pub listener_id: u32,
    pub properties: &'a ExtensionProperties,
    pub children: &'a mut dyn Iterator<Item = AnyElement>,
    pub event_sink: ExtensionEventSink,
}

impl<'a> ExtensionRenderContext<'a> {
    pub(crate) fn new(
        node_id: u32,
        listener_id: u32,
        properties: &'a ExtensionProperties,
        children: &'a mut dyn Iterator<Item = AnyElement>,
        event_sink: ExtensionEventSink,
    ) -> Self {
        Self {
            node_id,
            listener_id,
            properties,
            children,
            event_sink,
        }
    }

    pub fn event_sink(&self) -> ExtensionEventSink {
        self.event_sink.clone()
    }

    pub fn children(&mut self) -> &mut dyn Iterator<Item = AnyElement> {
        self.children
    }
}

pub(crate) fn new_event_state(
    runtime: Arc<dyn RuntimeAdapter>,
    sequence: Arc<AtomicU32>,
) -> Rc<ExtensionEventState> {
    Rc::new(ExtensionEventState {
        runtime,
        sequence,
        surface_id: Cell::new(0),
        epoch: Cell::new(0),
        revision: Cell::new(0),
        routes: RefCell::new(HashMap::new()),
    })
}

pub(crate) fn update_event_state(
    state: &Rc<ExtensionEventState>,
    surface_id: u32,
    epoch: u32,
    revision: u32,
) {
    if state.surface_id.get() != surface_id || state.epoch.get() != epoch {
        revoke_all_events(state);
    }
    state.surface_id.set(surface_id);
    state.epoch.set(epoch);
    state.revision.set(revision);
}

pub(super) fn revoke_all_events(state: &ExtensionEventState) {
    for (_, route) in state.routes.borrow_mut().drain() {
        route.active.set(false);
    }
}

pub(super) fn revoke_node_events(state: &ExtensionEventState, id: u32) {
    if let Some(route) = state.routes.borrow_mut().remove(&id) {
        route.active.set(false);
    }
}

pub(super) fn reconcile_event_routes(
    state: &Rc<ExtensionEventState>,
    store: &crate::tree::NodeStore,
) {
    state.routes.borrow_mut().retain(|id, route| {
        let keep = store.get(*id).is_some_and(|node| {
            let Some(crate::protocol::HostProperties::Extension(props)) = &node.host_properties
            else {
                return false;
            };
            route
                .contract
                .get()
                .is_none_or(|old| old == ExtensionContract::from(props))
        });
        if !keep {
            route.active.set(false);
        }
        keep
    });
    // Seed routes before mount/render so their contract is known even when a
    // retained subscription outlives the element that originally created it.
    for node in store.iter() {
        if let Some(crate::protocol::HostProperties::Extension(props)) = &node.host_properties {
            let _sink = ExtensionEventSink::new(
                state.clone(),
                node.id,
                node.listener_id,
                props.event_ids.clone(),
            );
            state
                .routes
                .borrow()
                .get(&node.id)
                .unwrap()
                .contract
                .set(Some(ExtensionContract::from(props)));
        }
    }
}

pub(crate) fn validate_extension_fields(fields: &[ExtensionField]) -> Result<(), String> {
    if fields.len() > MAX_EXTENSION_FIELDS {
        return Err(format!("at most {MAX_EXTENSION_FIELDS} fields are allowed"));
    }
    let mut previous = None;
    let mut text_bytes = 0usize;
    let mut binary_bytes = 0usize;
    for field in fields {
        if field.id == 0 {
            return Err("field IDs must be non-zero".to_owned());
        }
        if let Some(previous) = previous
            && field.id <= previous
        {
            return Err("fields must be sorted and unique".to_owned());
        }
        previous = Some(field.id);
        match &field.value {
            ExtensionValue::Bool(_) | ExtensionValue::Int32(_) | ExtensionValue::U32(_) => {}
            ExtensionValue::F32(value) => {
                if !value.is_finite() {
                    return Err("f32 value must be finite".to_owned());
                }
            }
            ExtensionValue::Text(value) => {
                if value.len() > MAX_EXTENSION_VALUE_BYTES {
                    return Err(format!(
                        "text values may be at most {MAX_EXTENSION_VALUE_BYTES} bytes"
                    ));
                }
                text_bytes = text_bytes
                    .checked_add(value.len())
                    .ok_or_else(|| "text size overflow".to_owned())?;
            }
            ExtensionValue::Bytes(value) => {
                if value.len() > MAX_EXTENSION_VALUE_BYTES {
                    return Err(format!(
                        "bytes values may be at most {MAX_EXTENSION_VALUE_BYTES} bytes"
                    ));
                }
                binary_bytes = binary_bytes
                    .checked_add(value.len())
                    .ok_or_else(|| "bytes size overflow".to_owned())?;
            }
        }
    }
    if text_bytes > MAX_EXTENSION_TEXT_BYTES {
        return Err(format!(
            "text values may total at most {MAX_EXTENSION_TEXT_BYTES} bytes"
        ));
    }
    if binary_bytes > MAX_EXTENSION_VALUE_BYTES {
        return Err(format!(
            "bytes values may total at most {MAX_EXTENSION_VALUE_BYTES} bytes"
        ));
    }
    Ok(())
}

pub(crate) fn render(
    root: &crate::renderer::SolidRoot,
    node: &StoredNode,
    entity: &gpui::Entity<crate::renderer::SolidRoot>,
    style: Option<&crate::protocol::Style>,
) -> AnyElement {
    let properties = match node.host_properties.as_ref() {
        Some(crate::protocol::HostProperties::Extension(properties)) => properties,
        _ => panic!("validated Extension node is missing ExtensionProperties"),
    };
    let adapter = root
        .extension_registry()
        .resolve(
            properties.provider_id,
            properties.catalog_digest,
            properties.entry_id,
            properties.entry_version,
        )
        .unwrap_or_else(|| panic!("validated Extension node has no matching adapter"));
    let mut children = node
        .children(root.store())
        .map(|child| root.render_node_for_extension(child, entity));
    let sink = ExtensionEventSink::new(
        root.extension_event_state(),
        node.id,
        node.listener_id,
        properties.event_ids.clone(),
    );
    let rendered = match root
        .extension_instances
        .get(&node.id)
        .and_then(|mounted| mounted.instance.as_ref())
    {
        Some(instance) => {
            assert!(
                !root.extension_dirty.contains(&node.id),
                "retained native component requires apply_decoded_message_in_window before rendering"
            );
            instance.render(&mut children)
        }
        None => adapter.render(ExtensionRenderContext::new(
            node.id,
            node.listener_id,
            properties,
            &mut children,
            sink,
        )),
    };
    let wrapper =
        crate::renderer::paint::apply_style_to_extension(gpui::div().child(rendered), style)
            .id(gpui::ElementId::Integer(node.id as u64));
    crate::renderer::paint::measure_node_for_extension(node, wrapper.into_any(), entity)
}
