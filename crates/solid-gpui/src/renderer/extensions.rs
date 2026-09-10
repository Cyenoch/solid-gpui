use gpui::{AnyElement, IntoElement, ParentElement, RenderOnce};
use std::cell::{Cell, RefCell};
use std::collections::{HashMap, HashSet};
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
#[derive(Clone, Default)]
pub struct ExtensionChildSummary<'a> {
    pub count: usize,
    pub kinds: Vec<u32>,
    pub element_types: Vec<Option<std::any::TypeId>>,
    pub properties: Vec<Option<&'a ExtensionProperties>>,
    source: Option<(
        &'a StoredNode,
        &'a crate::tree::NodeStore,
        &'a dyn ExtensionRegistry,
    )>,
}
impl std::fmt::Debug for ExtensionChildSummary<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExtensionChildSummary")
            .field("count", &self.count)
            .field("kinds", &self.kinds)
            .field("element_types", &self.element_types)
            .finish_non_exhaustive()
    }
}

impl<'a> ExtensionChildSummary<'a> {
    /// Inspect one child's composition on demand, without materializing its subtree.
    pub fn child(&self, index: usize) -> Option<Self> {
        let (content, store, registry) = self.source?;
        let node = content.child_at(store, index)?;
        let group = match node.host_properties.as_ref() {
            Some(crate::protocol::HostProperties::Extension(p)) => registry
                .resolve(p.provider_id, p.catalog_digest, p.entry_id, p.entry_version)?
                .default_child_group(),
            _ => None,
        };
        Some(Self::from_node(node, store, registry, group))
    }
    /// Count the actual default content, excluding the generated named-slot groups.
    pub fn content_count(&self) -> usize {
        self.element_types.len()
    }

    pub(crate) fn from_node(
        node: &'a StoredNode,
        store: &'a crate::tree::NodeStore,
        registry: &'a dyn ExtensionRegistry,
        group: Option<usize>,
    ) -> Self {
        let kinds = node
            .children(store)
            .map(|child| child.kind)
            .collect::<Vec<_>>();
        let content = group
            .and_then(|index| node.child_at(store, index))
            .unwrap_or(node);
        let element_types = content
            .children(store)
            .map(|child| {
                let crate::protocol::HostProperties::Extension(props) =
                    child.host_properties.as_ref()?
                else {
                    return None;
                };
                registry
                    .resolve(
                        props.provider_id,
                        props.catalog_digest,
                        props.entry_id,
                        props.entry_version,
                    )?
                    .element_type()
            })
            .collect();
        Self {
            source: Some((content, store, registry)),
            count: kinds.len(),
            kinds,
            element_types,
            properties: content
                .children(store)
                .map(|child| match child.host_properties.as_ref() {
                    Some(crate::protocol::HostProperties::Extension(props)) => Some(props),
                    _ => None,
                })
                .collect(),
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
    /// A contract may group named content under direct child containers.
    fn default_child_group(&self) -> Option<usize> {
        None
    }
    fn element_type(&self) -> Option<std::any::TypeId> {
        None
    }
    fn native_style(&self) -> bool {
        false
    }
    fn child_type(&self) -> Option<std::any::TypeId> {
        None
    }
    fn requires_typed_parent(&self) -> bool {
        false
    }

    /// Called once after the complete candidate tree has passed validation.
    fn mount(
        &self,
        _node_id: u32,
        _properties: &ExtensionProperties,
        _sink: ExtensionEventSink,
        _children: ExtensionChildren,
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
    fn render(&self, context: ExtensionRenderContext<'_>) -> AnyElement;
    fn build_native(&self, _context: ExtensionRenderContext<'_>) -> Option<Box<dyn std::any::Any>> {
        None
    }
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
    child_nodes: RefCell<Vec<Rc<Vec<u32>>>>,
    changed_children: RefCell<Vec<Rc<Vec<usize>>>>,
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
                    child_nodes: RefCell::new(Vec::new()),
                    changed_children: RefCell::new(Vec::new()),
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

    pub(crate) fn is_active(&self) -> bool {
        self.route.active.get()
            && self.surface_id == self.state.surface_id.get()
            && self.epoch == self.state.epoch.get()
    }

    pub(crate) fn set_child_nodes(
        &self,
        groups: Vec<Vec<u32>>,
        dirty: &std::collections::HashSet<u32>,
    ) {
        *self.route.changed_children.borrow_mut() = groups
            .iter()
            .map(|ids| {
                Rc::new(
                    ids.iter()
                        .enumerate()
                        .filter_map(|(i, id)| dirty.contains(id).then_some(i))
                        .collect(),
                )
            })
            .collect();
        let mut current = self.route.child_nodes.borrow_mut();
        current.truncate(groups.len());
        for (index, ids) in groups.into_iter().enumerate() {
            match current.get_mut(index) {
                Some(old) if old.as_ref() != &ids => *old = Rc::new(ids),
                Some(_) => {}
                None => current.push(Rc::new(ids)),
            }
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

/// A repeatable native projection of committed JS children. The weak owner and
/// instance route prevent retained overlays from resurrecting retired content.
#[derive(Clone)]
pub struct ExtensionChildren {
    root: gpui::WeakEntity<crate::SolidRoot>,
    sink: ExtensionEventSink,
}
impl ExtensionChildren {
    pub(crate) fn new(root: gpui::WeakEntity<crate::SolidRoot>, sink: ExtensionEventSink) -> Self {
        Self { root, sink }
    }
    /// All direct children, or one named-slot group selected by its contract index.
    pub fn content(&self, group: Option<usize>) -> ExtensionContent {
        ExtensionContent {
            children: self.clone(),
            group,
            item: None,
        }
    }
}
#[derive(Clone, IntoElement)]
pub struct ExtensionContent {
    children: ExtensionChildren,
    group: Option<usize>,
    item: Option<usize>,
}
impl ExtensionContent {
    pub fn len(&self) -> usize {
        let sink = &self.children.sink;
        if !sink.route.active.get()
            || sink.epoch != sink.state.epoch.get()
            || sink.surface_id != sink.state.surface_id.get()
        {
            return 0;
        }
        let count = sink
            .route
            .child_nodes
            .borrow()
            .get(self.group.map_or(0, |i| i + 1))
            .map_or(0, |ids| ids.len());
        self.item.map_or(count, |i| usize::from(i < count))
    }
    /// Stable host identities in committed order. The shared snapshot changes only
    /// when composition changes; obtaining it does not scan or build the children.
    pub fn node_ids(&self) -> Rc<Vec<u32>> {
        if !self.children.sink.is_active() {
            return Rc::default();
        }
        let groups = self.children.sink.route.child_nodes.borrow();
        let Some(ids) = groups.get(self.group.map_or(0, |i| i + 1)) else {
            return Rc::default();
        };
        self.item.map_or_else(
            || ids.clone(),
            |i| Rc::new(ids.get(i).copied().into_iter().collect()),
        )
    }
    /// Indices whose subtree changed in the commit currently being reconciled.
    /// Retained views consume this during update to invalidate measured rows locally.
    pub fn changed_indices(&self) -> Rc<Vec<usize>> {
        if !self.children.sink.is_active() {
            return Rc::default();
        }
        let groups = self.children.sink.route.changed_children.borrow();
        let Some(indices) = groups.get(self.group.map_or(0, |i| i + 1)) else {
            return Rc::default();
        };
        self.item.map_or_else(
            || indices.clone(),
            |i| Rc::new(indices.contains(&i).then_some(0).into_iter().collect()),
        )
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
    pub fn item(&self, index: usize) -> Self {
        let mut content = self.clone();
        content.item = Some(index);
        content
    }
    pub(crate) fn native_items<T: 'static>(&self, cx: &gpui::App) -> crate::native::NativeItems<T> {
        let mut items = Vec::new();
        let sink = &self.children.sink;
        if !sink.is_active() {
            return crate::native::NativeItems(items);
        }
        let Some(entity) = self.children.root.upgrade() else {
            return crate::native::NativeItems(items);
        };
        let root = entity.read(cx);
        let Some(mut parent) = root.store().get(sink.node_id) else {
            return crate::native::NativeItems(items);
        };
        if let Some(group) = self.group {
            parent = parent
                .child_at(root.store(), group)
                .expect("validated content group");
        }
        let nodes: Box<dyn Iterator<Item = &StoredNode>> = match self.item {
            Some(i) => Box::new(parent.child_at(root.store(), i).into_iter()),
            None => Box::new(parent.children(root.store())),
        };
        let mut iterator = ChildIterator {
            root,
            entity: &entity,
            nodes,
        };
        while let Some(child) = iterator.next_native() {
            items.push(
                child.map_native(|child| {
                    *child.downcast::<T>().expect("declared retained child type")
                }),
            );
        }
        crate::native::NativeItems(items)
    }
    pub fn elements(&self, cx: &gpui::App) -> Vec<AnyElement> {
        let sink = &self.children.sink;
        if !sink.route.active.get()
            || sink.surface_id != sink.state.surface_id.get()
            || sink.epoch != sink.state.epoch.get()
        {
            return Vec::new();
        }
        let Some(entity) = self.children.root.upgrade() else {
            return Vec::new();
        };
        let root = entity.read(cx);
        let Some(parent) = root.store().get(sink.node_id) else {
            return Vec::new();
        };
        let parent = match self.group {
            Some(index) => match parent.child_at(root.store(), index) {
                Some(group) => group,
                None => return Vec::new(),
            },
            None => parent,
        };
        match self.item {
            Some(index) => parent
                .child_at(root.store(), index)
                .map(|node| root.render_node_for_extension(node, &entity))
                .into_iter()
                .collect(),
            None => parent
                .children(root.store())
                .map(|node| root.render_node_for_extension(node, &entity))
                .collect(),
        }
    }
}
impl RenderOnce for ExtensionContent {
    fn render(self, _: &mut gpui::Window, cx: &mut gpui::App) -> impl IntoElement {
        gpui::div().children(self.elements(cx))
    }
}

/// One ordered iterator supports ordinary rendered children and native typed
/// children. Parents consume either form once; no child is materialized twice.
pub trait ExtensionChildIterator: Iterator<Item = AnyElement> {
    fn next_native(&mut self) -> Option<crate::native::NativeChild<Box<dyn std::any::Any>>>;
}
struct ChildIterator<'a> {
    root: &'a crate::SolidRoot,
    entity: &'a gpui::Entity<crate::SolidRoot>,
    nodes: Box<dyn Iterator<Item = &'a StoredNode> + 'a>,
}
impl Iterator for ChildIterator<'_> {
    type Item = AnyElement;
    fn next(&mut self) -> Option<AnyElement> {
        self.nodes
            .next()
            .map(|node| self.root.render_node_for_extension(node, self.entity))
    }
}
impl ExtensionChildIterator for ChildIterator<'_> {
    fn next_native(&mut self) -> Option<crate::native::NativeChild<Box<dyn std::any::Any>>> {
        let node = self.nodes.next()?;
        let crate::protocol::HostProperties::Extension(props) = node
            .host_properties
            .as_ref()
            .expect("validated typed child")
        else {
            unreachable!("validated typed child")
        };
        let adapter = self
            .root
            .extension_registry()
            .resolve(
                props.provider_id,
                props.catalog_digest,
                props.entry_id,
                props.entry_version,
            )
            .expect("validated typed child adapter");
        let content = adapter
            .default_child_group()
            .map(|index| {
                node.child_at(self.root.store(), index)
                    .expect("validated slot group")
            })
            .unwrap_or(node);
        let mut children = ChildIterator {
            root: self.root,
            entity: self.entity,
            nodes: Box::new(content.children(self.root.store())),
        };
        let sink = ExtensionEventSink::new(
            self.root.extension_event_state(),
            node.id,
            node.listener_id,
            props.event_ids.clone(),
        );
        let boundary = native_boundary(
            node.id,
            node.listener_id,
            adapter.native_style(),
            self.root.style_for_node(node),
            self.entity,
            sink.clone(),
        );
        let context = ExtensionRenderContext::new(
            node.id,
            node.listener_id,
            props,
            &mut children,
            self.root.style_for_node(node),
            sink,
        );
        self.root
            .extension_instances
            .get(&node.id)
            .and_then(|m| m.instance.as_ref())
            .expect("mounted typed child")
            .build_native(context)
            .map(|native| crate::native::NativeChild { native, boundary })
    }
}

/// Data handed to an adapter's infallible render method.
pub struct ExtensionRenderContext<'a> {
    pub node_id: u32,
    pub listener_id: u32,
    pub properties: &'a ExtensionProperties,
    pub children: &'a mut dyn ExtensionChildIterator,
    pub style: Option<&'a crate::protocol::Style>,
    pub event_sink: ExtensionEventSink,
}

impl<'a> ExtensionRenderContext<'a> {
    pub(crate) fn new(
        node_id: u32,
        listener_id: u32,
        properties: &'a ExtensionProperties,
        children: &'a mut dyn ExtensionChildIterator,
        style: Option<&'a crate::protocol::Style>,
        event_sink: ExtensionEventSink,
    ) -> Self {
        Self {
            node_id,
            listener_id,
            properties,
            children,
            style,
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
    changed: Option<&HashSet<u32>>,
) {
    let ids: HashSet<u32> = match changed {
        Some(ids) => ids.clone(),
        None => state
            .routes
            .borrow()
            .keys()
            .copied()
            .chain(
                store
                    .iter()
                    .filter(|node| node.kind == crate::tree::KIND_EXTENSION)
                    .map(|node| node.id),
            )
            .collect(),
    };
    for id in ids {
        let properties = store.get(id).and_then(|node| {
            if let Some(crate::protocol::HostProperties::Extension(props)) = &node.host_properties {
                Some((node.listener_id, props))
            } else {
                None
            }
        });
        let keep = state.routes.borrow().get(&id).is_some_and(|route| {
            properties.is_some_and(|(_, props)| {
                route
                    .contract
                    .get()
                    .is_none_or(|old| old == ExtensionContract::from(props))
            })
        });
        if !keep {
            revoke_node_events(state, id);
        }
        if let Some((listener, props)) = properties {
            let _sink =
                ExtensionEventSink::new(state.clone(), id, listener, props.event_ids.clone());
            state
                .routes
                .borrow()
                .get(&id)
                .expect("seeded event route")
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
    let content_node = adapter
        .default_child_group()
        .map(|index| {
            node.child_at(root.store(), index)
                .expect("validated native slot group")
        })
        .unwrap_or(node);
    let mut children = ChildIterator {
        root,
        entity,
        nodes: Box::new(content_node.children(root.store())),
    };
    let sink = ExtensionEventSink::new(
        root.extension_event_state(),
        node.id,
        node.listener_id,
        properties.event_ids.clone(),
    );
    let boundary = native_boundary(
        node.id,
        node.listener_id,
        adapter.native_style(),
        style,
        entity,
        sink.clone(),
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
            instance.render(ExtensionRenderContext::new(
                node.id,
                node.listener_id,
                properties,
                &mut children,
                style,
                sink,
            ))
        }
        None => adapter.render(ExtensionRenderContext::new(
            node.id,
            node.listener_id,
            properties,
            &mut children,
            style,
            sink,
        )),
    };
    boundary(rendered)
}

fn native_boundary(
    node_id: u32,
    listener_id: u32,
    native_style: bool,
    style: Option<&crate::protocol::Style>,
    entity: &gpui::Entity<crate::SolidRoot>,
    sink: ExtensionEventSink,
) -> Rc<dyn Fn(AnyElement) -> AnyElement> {
    let style = (!native_style).then(|| style.cloned()).flatten();
    let entity = entity.downgrade();
    Rc::new(move |element| {
        let Some(entity) = entity.upgrade() else {
            return gpui::Empty.into_any_element();
        };
        // A scope alone must not insert a layout parent: percentage-sized
        // native descendants (e.g. a settings page's virtual list) need their
        // original parent's definite constraints.
        let element = if native_style || style.is_none() {
            element
        } else {
            crate::renderer::paint::apply_style_to_extension(
                gpui::div().child(element),
                style.as_ref(),
            )
            .into_any_element()
        };
        crate::renderer::paint::scope_native_element(
            node_id,
            listener_id != 0,
            element,
            &entity,
            sink.clone(),
        )
    })
}
