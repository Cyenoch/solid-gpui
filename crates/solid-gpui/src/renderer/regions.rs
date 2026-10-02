//! Native ownership for static, definite-size primitive regions. Eligibility is
//! derived from native capabilities; it is never an application cache hint.
use super::*;
use crate::protocol::OverflowCode;
use crate::tree::{KIND_RAW_TEXT, PatchChanges};
use gpui::{AnyElement, AppContext as _, Entity, StyleRefinement, WeakEntity};

#[derive(Clone, Copy, PartialEq, Eq)]
struct Subtree {
    static_content: bool,
}

#[derive(Default)]
pub(super) struct Regions {
    subtrees: HashMap<u32, Subtree>,
    candidates: HashSet<u32>,
    owners: HashMap<u32, Entity<Region>>,
    #[cfg(feature = "native-acceptance")]
    observation_states: HashMap<u32, Rc<RefCell<super::acceptance::RetainedObservations>>>,
    animated_ancestors: HashSet<u32>,
    animating: HashSet<u32>,
}

struct Region {
    root: WeakEntity<SolidRoot>,
    node_id: u32,
    #[cfg(feature = "native-acceptance")]
    observations: Rc<RefCell<super::acceptance::RetainedObservations>>,
}

impl Render for Region {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let _profile = profile::span(profile::Stage::Render);
        let Some(root) = self.root.upgrade() else {
            return div().into_any();
        };
        root.read_with(cx, |root_state, _| {
            let Some(node) = root_state.store.get(self.node_id) else {
                return div().into_any();
            };
            #[cfg(feature = "native-acceptance")]
            if root_state.acceptance.is_some() {
                let mut observations = self.observations.borrow_mut();
                observations.generation = observations.generation.wrapping_add(1);
                observations.ids.clear();
                let mut pending = vec![self.node_id];
                while let Some(id) = pending.pop() {
                    observations.ids.push(id);
                    pending.extend(
                        root_state
                            .store
                            .get(id)
                            .unwrap()
                            .children(&root_state.store)
                            .map(|node| node.id),
                    );
                }
            }
            // The parent-facing box owns margins and placement. The rendered
            // box fills that allocation and applies padding/paint exactly once.
            let mut content = node.clone();
            let style = Arc::make_mut(content.style.as_mut().expect("region has definite style"));
            style.width = None;
            style.height = None;
            style.width_percent = Some(100.0);
            style.height_percent = Some(100.0);
            style.margin_top = None;
            style.margin_right = None;
            style.margin_bottom = None;
            style.margin_left = None;
            style.position = None;
            style.left = None;
            style.top = None;
            style.right = None;
            style.bottom = None;
            style.align_self = None;
            style.grid_column_span = None;
            style.grid_row_span = None;
            root_state.render_node_direct(&content, &root)
        })
    }
}

fn static_node(node: &StoredNode) -> bool {
    matches!(node.kind, KIND_VIEW | KIND_TEXT | KIND_RAW_TEXT)
        && node.listener_id == 0
        && !node.focusable
        && !node.selectable
        && !node.observes_layout
        && !node.accepts_pointer_move
        && node.host_properties.is_none()
        && node.tooltip.is_none()
        && node.style.as_ref().is_none_or(|style| {
            style.transition.is_none()
                && style.overflow != Some(OverflowCode::Scroll)
                && style.position != Some(PositionCode::Overlay)
        })
}

fn definite_region(node: &StoredNode) -> bool {
    node.id != 1
        && node.kind == KIND_VIEW
        && node.has_children()
        && node.style.as_ref().is_some_and(|style| {
            style.width.is_some()
                && style.height.is_some()
                && style.width_percent.is_none()
                && style.height_percent.is_none()
                && style.overflow == Some(OverflowCode::Hidden)
                && style.flex_shrink == Some(0.0)
                && style.flex_grow.is_none_or(|grow| grow == 0.0)
                && style.min_width.is_none()
                && style.max_width.is_none()
                && style.min_height.is_none()
                && style.max_height.is_none()
        })
}

fn ancestors_allow_retention(store: &NodeStore, node: &StoredNode) -> bool {
    let mut parent = node.parent_id;
    while let Some(node) = store.get(parent) {
        // GPUI's scene cache keys inherited text and clipping, but not ancestor
        // opacity. Interactive ancestors may refine appearance without a Patch.
        if node.listener_id != 0
            || node.host_properties.is_some()
            || node.style.as_ref().is_some_and(|style| {
                style.transition.is_some()
                    || style.opacity.is_some_and(|opacity| opacity != 1.0)
                    || style.position == Some(PositionCode::Overlay)
            })
        {
            return false;
        }
        parent = node.parent_id;
    }
    true
}

impl Regions {
    pub(super) fn clear(&mut self) {
        self.owners.clear();
        #[cfg(feature = "native-acceptance")]
        self.observation_states.clear();
        self.subtrees.clear();
        self.candidates.clear();
        self.animated_ancestors.clear();
        self.animating.clear();
    }

    pub(super) fn reconcile(
        &mut self,
        store: &NodeStore,
        changes: Option<&PatchChanges>,
        cx: &mut Context<SolidRoot>,
    ) {
        if let Some(changes) = changes {
            for id in &changes.removed {
                self.subtrees.remove(id);
                self.owners.remove(id);
                #[cfg(feature = "native-acceptance")]
                self.observation_states.remove(id);
                self.candidates.remove(id);
            }
            // A semantic content edit with unchanged capabilities stops summary
            // propagation. Structural/capability changes read affected child
            // lists, rather than rescanning unrelated descendants every frame.
            let mut affected: Vec<_> = changes.affected.iter().copied().collect();
            affected.sort_unstable_by_key(|id| std::cmp::Reverse(depth(store, *id)));
            let mut dirty = changes.changed.clone();
            for &id in &changes.changed {
                if store.get(id).is_some_and(definite_region) {
                    self.candidates.insert(id);
                } else {
                    self.candidates.remove(&id);
                }
            }
            for &id in &affected {
                if !dirty.contains(&id) {
                    continue;
                }
                let Some(node) = store.get(id) else { continue };
                let summary = self.summary(store, node);
                if self.subtrees.insert(id, summary) != Some(summary) {
                    dirty.insert(node.parent_id);
                }
            }
            // Content changes notify its owner even when eligibility is equal.
            // An actual ancestor edit also invalidates inherited appearance.
            for (&id, owner) in &self.owners {
                if changes.affected.contains(&id) || has_changed_ancestor(store, id, changes) {
                    owner.update(cx, |_, cx| cx.notify());
                }
            }
            // Ancestor capabilities can invalidate any previously retained
            // region. Check owners, not every primitive in the tree.
            let invalid: Vec<_> = self
                .owners
                .keys()
                .filter(|id| !self.eligible(store, **id))
                .copied()
                .collect();
            for id in invalid {
                self.owners.remove(&id);
                #[cfg(feature = "native-acceptance")]
                self.observation_states.remove(&id);
            }
        } else {
            self.clear();
            let mut order = Vec::with_capacity(store.len());
            if let Some(root) = store.root() {
                let mut pending = vec![root.id];
                while let Some(id) = pending.pop() {
                    order.push(id);
                    pending.extend(store.get(id).unwrap().children(store).map(|child| child.id));
                }
            }
            for &id in order.iter().rev() {
                let node = store.get(id).unwrap();
                self.subtrees.insert(id, self.summary(store, node));
                if definite_region(node) {
                    self.candidates.insert(id);
                }
            }
        }
        let root = cx.weak_entity();
        for &id in &self.candidates {
            if self.eligible(store, id) && !self.owners.contains_key(&id) {
                #[cfg(feature = "native-acceptance")]
                let observations = Rc::default();
                #[cfg(feature = "native-acceptance")]
                self.observation_states.insert(id, Rc::clone(&observations));
                self.owners.insert(
                    id,
                    cx.new(|_| Region {
                        root: root.clone(),
                        node_id: id,
                        #[cfg(feature = "native-acceptance")]
                        observations,
                    }),
                );
            }
        }
    }

    #[cfg(test)]
    pub(super) fn contains(&self, id: u32) -> bool {
        self.owners.contains_key(&id)
    }

    fn summary(&self, store: &NodeStore, node: &StoredNode) -> Subtree {
        Subtree {
            static_content: static_node(node)
                && node.children(store).all(|child| {
                    self.subtrees
                        .get(&child.id)
                        .is_some_and(|summary| summary.static_content)
                }),
        }
    }

    fn eligible(&self, store: &NodeStore, id: u32) -> bool {
        store.get(id).is_some_and(|node| {
            definite_region(node)
                && self
                    .subtrees
                    .get(&id)
                    .is_some_and(|summary| summary.static_content)
                && ancestors_allow_retention(store, node)
        })
    }

    pub(super) fn element(&self, node: &StoredNode, root: &SolidRoot) -> Option<AnyElement> {
        #[cfg(not(feature = "native-acceptance"))]
        let _ = root;
        if self.animated_ancestors.contains(&node.id) {
            return None;
        }
        let owner = self.owners.get(&node.id)?;
        let style = node.style.as_deref()?;
        let full = paint::apply_style_to_extension(StyleRefinement::default(), Some(style));
        let outer = StyleRefinement {
            size: full.size,
            margin: full.margin,
            position: full.position,
            inset: full.inset,
            flex_shrink: Some(0.0),
            align_self: full.align_self,
            grid_location: full.grid_location,
            ..StyleRefinement::default()
        };
        let element = owner.clone().cached(outer).into_any();
        #[cfg(feature = "native-acceptance")]
        let element = if let Some(observations) = &root.acceptance {
            super::acceptance::retained(
                element,
                observations.clone(),
                self.observation_states[&node.id].clone(),
            )
        } else {
            element
        };
        Some(element)
    }

    pub(super) fn prepare_frame(&mut self, store: &NodeStore, animation: &AnimationBook) {
        self.animated_ancestors.clear();
        self.animating.clear();
        self.animating.extend(animation.in_flight());
        if self.animating.is_empty() {
            return;
        }
        for &id in &self.animating {
            let mut current = id;
            while let Some(node) = store.get(current) {
                self.animated_ancestors.insert(current);
                current = node.parent_id;
            }
        }
        // An in-flight ancestor remains a dependency even if its transition
        // declaration was removed while it finishes the sampled animation.
        for &id in self.owners.keys() {
            let mut current = id;
            while let Some(node) = store.get(current) {
                if self.animating.contains(&current) {
                    self.animated_ancestors.insert(id);
                    break;
                }
                current = node.parent_id;
            }
        }
    }

    #[cfg(test)]
    pub(super) fn len(&self) -> usize {
        self.owners.len()
    }
}

fn depth(store: &NodeStore, id: u32) -> usize {
    let mut depth = 0;
    let mut current = id;
    while let Some(node) = store.get(current) {
        depth += 1;
        current = node.parent_id;
    }
    depth
}

fn has_changed_ancestor(store: &NodeStore, id: u32, changes: &PatchChanges) -> bool {
    let mut parent = store.get(id).map_or(0, |node| node.parent_id);
    while let Some(node) = store.get(parent) {
        if changes.changed.contains(&parent) {
            return true;
        }
        parent = node.parent_id;
    }
    false
}
