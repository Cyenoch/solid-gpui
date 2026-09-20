//! VirtualList reconciliation: one [`ListState`] per committed VirtualList node.
//!
//! Data changes arrive as revision-chained replacement spans
//! ([`VirtualListDataEdit`]): the tree validates every transition against the
//! previously published properties and rejects malformed metadata, so this
//! module consumes the validated invariants and splices only the replaced
//! region of each `ListState`. Rows outside the edited span keep their measured
//! heights and identity, the logical scroll anchor adjusts incrementally, and a
//! count change never rebuilds the whole backend tree.
//!
//! Snapshots are re-basing boundaries, never deltas: their DTO may carry any
//! revision/edit pair, so a full reconciliation resets the retained state's
//! bookkeeping, re-hints heights onto the snapshot's item space (measured
//! heights survive as hints), and clamps the logical scroll position onto the
//! new item count.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::Ordering;

use gpui::{ListAlignment, ListState, px};

use crate::protocol::{Event, HostProperties, VirtualListDataEdit, VirtualListProperties};
use crate::transport::send_event_or_exit;
use crate::tree::{KIND_VIRTUAL_LIST, NodeStore};

use super::SolidRoot;

pub(super) fn virtual_list_ancestor(store: &NodeStore, mut node_id: u32) -> Option<u32> {
    loop {
        let node = store.get(node_id)?;
        if node.kind == KIND_VIRTUAL_LIST {
            return Some(node.id);
        }
        if node.parent_id == 0 {
            return None;
        }
        node_id = node.parent_id;
    }
}

fn committed_virtual_list_properties(
    store: &NodeStore,
    node_id: u32,
) -> Option<VirtualListProperties> {
    let node = store.get(node_id)?;
    match node.host_properties.as_ref()? {
        HostProperties::VirtualList(list) => Some(list.clone()),
        _ => None,
    }
}

/// Splice a tree-validated data edit into `state` without touching rows outside
/// the replaced span.
///
/// The tree guarantees `edit.base_revision == applied` and the count equation
/// against the previously published properties, so the invariants are consumed
/// with assertions instead of silent fallbacks. Rows seeded into the span carry
/// the published estimate as a height hint so scrollbar geometry stays valid
/// before they are measured.
fn apply_virtual_list_data_edit(
    state: &ListState,
    list: &VirtualListProperties,
    applied: Option<u32>,
) {
    let edit: &VirtualListDataEdit = list.data_edit.as_ref().expect(
        "VirtualList dataRevision advanced without dataEdit; the tree validates this transition",
    );
    assert_eq!(
        applied,
        Some(edit.base_revision),
        "VirtualList dataEdit must chain from the applied data revision"
    );
    let previous_count = state.item_count();
    let item_count = list.item_count as usize;
    assert_eq!(
        previous_count
            .checked_sub(edit.old_count as usize)
            .and_then(|rest| rest.checked_add(edit.new_count as usize)),
        Some(item_count),
        "VirtualList dataEdit count equation must hold against the list state"
    );
    let start = edit.start as usize;
    let old_count = edit.old_count as usize;
    let new_count = edit.new_count as usize;
    let scroll_top = state.logical_scroll_top();
    let was_at_end = previous_count > 0 && scroll_top.item_ix >= previous_count;
    state.splice_with_size_hint(
        start..start + old_count,
        new_count,
        Some(px(list.estimated_item_size)),
    );
    if was_at_end && new_count > old_count && item_count > 0 {
        state.scroll_to_end();
    }
}

/// How a properties update relates to the state a node already holds.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum VirtualListReset {
    /// Patch delta: chain the revision edit onto the retained state.
    None,
    /// Snapshot boundary: re-base the retained state (measured heights survive
    /// as hints) and carry the scroll anchor, clamped onto the new item space.
    Snapshot,
    /// Node creation: drop any retained incarnation entirely and seed fresh
    /// state with no inherited anchor.
    Create,
}

impl SolidRoot {
    /// Bring this node's list state in line with one validated properties
    /// update.
    ///
    /// A [`VirtualListReset::Snapshot`] boundary wipes the per-list maps and
    /// re-bases a retained state with `reset_with_uniform_height`: measured
    /// rows keep their measured heights as hints, never-measured rows take the
    /// published estimate, and the previous logical scroll anchor is carried
    /// over and clamped onto the new item space. A
    /// [`VirtualListReset::Create`] boundary drops the retained incarnation
    /// entirely: the recreated node seeds fresh state with no inherited
    /// anchor. Without a reset, a fresh state records the incoming revision
    /// and an existing state consumes the revision-chained edit without
    /// touching the estimate map, so the final reconciliation still sees the
    /// previous estimate for estimate-only invalidation.
    pub(super) fn reconcile_virtual_list_properties(
        &mut self,
        node_id: u32,
        list: &VirtualListProperties,
        reset: VirtualListReset,
    ) {
        let item_count = list.item_count as usize;
        let estimated = px(list.estimated_item_size);
        let (replaced_scroll, replaced_state) = match reset {
            VirtualListReset::None => (None, None),
            VirtualListReset::Snapshot => {
                let scroll = self
                    .virtual_lists
                    .get(&node_id)
                    .map(|state| state.logical_scroll_top());
                (scroll, self.virtual_lists.remove(&node_id))
            }
            VirtualListReset::Create => {
                self.virtual_lists.remove(&node_id);
                (None, None)
            }
        };
        if reset != VirtualListReset::None {
            self.virtual_ranges.remove(&node_id);
            self.virtual_item_sizes.remove(&node_id);
            self.virtual_data_revisions.remove(&node_id);
            self.reported_visible_ranges.remove(&node_id);
            self.pending_visible_ranges.borrow_mut().remove(&node_id);
        }
        if let Some(previous) = replaced_state {
            // Re-base the retained state on the boundary properties instead of
            // discarding it: measured rows keep their measured heights (as
            // hints), never-measured rows take the published estimate, and the
            // carried scroll anchor clamps onto the new item space.
            previous.reset_with_uniform_height(item_count, estimated);
            if let Some(scroll_top) = replaced_scroll {
                previous.scroll_to(scroll_top);
            }
            self.virtual_lists.insert(node_id, previous);
            self.virtual_item_sizes
                .insert(node_id, list.estimated_item_size);
            self.virtual_data_revisions
                .insert(node_id, list.data_revision);
            return;
        }
        let created = !self.virtual_lists.contains_key(&node_id);
        let state = self.virtual_lists.entry(node_id).or_insert_with(|| {
            ListState::new(item_count, ListAlignment::Top, px(2048.0))
                .with_uniform_item_height(estimated)
        });
        if created {
            self.virtual_item_sizes
                .insert(node_id, list.estimated_item_size);
            self.virtual_data_revisions
                .insert(node_id, list.data_revision);
        } else {
            let applied_revision = self.virtual_data_revisions.get(&node_id).copied();
            if applied_revision != Some(list.data_revision) {
                apply_virtual_list_data_edit(state, list, applied_revision);
                self.virtual_data_revisions
                    .insert(node_id, list.data_revision);
            }
        }
    }

    /// Consume the ordered `VirtualListProperties` updates collected from a
    /// patch before it is applied: one entry per `Create` row and per list
    /// `UPDATE_PROPERTIES` row, in operation order.
    ///
    /// A legal patch may carry several updates for one node (revision 0→1 with
    /// base 0, then 1→2 with base 1); consuming only the final properties would
    /// observe an unapplied base, so every validated step is replayed onto the
    /// list state before the final reconciliation. Only the final generation of
    /// each node survives: entries preceding the node's last `Create` belong to
    /// a deleted incarnation and are skipped, so replaying
    /// update→delete→create→update seeds the recreated node fresh instead of
    /// re-using the dead generation's state. Entries whose final node is no
    /// longer a VirtualList are skipped; their state is pruned with the regular
    /// deleted-node cleanup.
    pub(super) fn replay_virtual_list_updates(
        &mut self,
        updates: &[(u32, VirtualListProperties, bool)],
    ) {
        let mut last_create: HashMap<u32, usize> = HashMap::new();
        for (index, (node_id, _, is_create)) in updates.iter().enumerate() {
            if *is_create {
                last_create.insert(*node_id, index);
            }
        }
        for (index, (node_id, list, is_create)) in updates.iter().enumerate() {
            if last_create
                .get(node_id)
                .is_some_and(|&create_index| index < create_index)
            {
                continue;
            }
            if self
                .store
                .get(*node_id)
                .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            {
                let reset = if *is_create {
                    VirtualListReset::Create
                } else {
                    VirtualListReset::None
                };
                self.reconcile_virtual_list_properties(*node_id, list, reset);
            }
        }
    }

    pub(super) fn reconcile_virtual_lists_for(&mut self, affected: Option<&HashSet<u32>>) {
        if let Some(ids) = affected {
            for id in ids {
                if self
                    .store
                    .get(*id)
                    .is_none_or(|node| node.kind != KIND_VIRTUAL_LIST)
                {
                    self.virtual_lists.remove(id);
                    self.virtual_ranges.remove(id);
                    self.virtual_item_sizes.remove(id);
                    self.virtual_data_revisions.remove(id);
                    self.reported_visible_ranges.remove(id);
                    self.pending_visible_ranges.borrow_mut().remove(id);
                }
            }
        } else {
            self.virtual_lists.retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
            self.virtual_ranges.retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
            self.virtual_item_sizes.retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
            self.virtual_data_revisions.retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
            self.reported_visible_ranges.retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
            self.pending_visible_ranges.borrow_mut().retain(|id, _| {
                self.store
                    .get(*id)
                    .is_some_and(|node| node.kind == KIND_VIRTUAL_LIST)
            });
        }
        let affected_virtual_lists: HashSet<u32> = affected
            .into_iter()
            .flat_map(|ids| ids.iter().copied())
            .filter_map(|id| virtual_list_ancestor(&self.store, id))
            .collect();
        let ids: Vec<u32> = match affected {
            Some(ids) => ids.iter().copied().collect(),
            None => self
                .store
                .iter()
                .filter(|node| node.kind == KIND_VIRTUAL_LIST)
                .map(|node| node.id)
                .collect(),
        };
        for id in ids {
            let Some(list) = committed_virtual_list_properties(&self.store, id) else {
                continue;
            };
            let next_range = (list.range_start, list.range_end);
            let previous_range = self.virtual_ranges.insert(id, next_range);
            let remeasure = (previous_range.is_some_and(|previous| previous != next_range)
                || affected_virtual_lists.contains(&id))
                && next_range.0 < next_range.1;
            if affected.is_none() {
                // Snapshot/full pass: every list is a re-basing boundary.
                self.reconcile_virtual_list_properties(id, &list, VirtualListReset::Snapshot);
                self.virtual_ranges.insert(id, next_range);
                if remeasure && let Some(state) = self.virtual_lists.get(&id) {
                    state.remeasure_items(next_range.0 as usize..next_range.1 as usize);
                }
                continue;
            }
            // Patch pass: replay_virtual_list_updates already consumed the
            // collected updates; these guards cover estimate invalidation,
            // late state creation, and committed-range remeasurement.
            let item_count = list.item_count as usize;
            let estimated = px(list.estimated_item_size);
            let previous_estimate = self.virtual_item_sizes.insert(id, list.estimated_item_size);
            let created = !self.virtual_lists.contains_key(&id);
            let state = self.virtual_lists.entry(id).or_insert_with(|| {
                ListState::new(item_count, ListAlignment::Top, px(2048.0))
                    .with_uniform_item_height(estimated)
            });
            if created {
                self.virtual_data_revisions.insert(id, list.data_revision);
            }
            let applied_revision = self.virtual_data_revisions.get(&id).copied();
            if applied_revision != Some(list.data_revision) {
                apply_virtual_list_data_edit(state, &list, applied_revision);
                self.virtual_data_revisions.insert(id, list.data_revision);
            } else if previous_estimate.is_some_and(|previous| previous != list.estimated_item_size)
            {
                // Explicit estimated-size invalidation: unmeasured rows re-hint
                // so the scrollbar reflects the new estimate, measured rows keep
                // their measured heights as hints, and the scroll anchor is
                // restored unchanged.
                let scroll_top = state.logical_scroll_top();
                state.reset_with_uniform_height(item_count, estimated);
                state.scroll_to(scroll_top);
            }
            if remeasure {
                state.remeasure_items(next_range.0 as usize..next_range.1 as usize);
            }
        }
    }

    #[cfg(test)]
    pub(super) fn reconcile_virtual_lists(&mut self) {
        // Test-facing "reconcile current state": chain edits for every current
        // list AND prune maps for ids no longer in the store, mirroring the
        // patch path. The production snapshot boundary is
        // reconcile_virtual_lists_for(None).
        let mut candidates: HashSet<u32> = self
            .store
            .iter()
            .filter(|node| node.kind == KIND_VIRTUAL_LIST)
            .map(|node| node.id)
            .collect();
        candidates.extend(self.virtual_lists.keys().copied());
        candidates.extend(self.virtual_ranges.keys().copied());
        candidates.extend(self.virtual_item_sizes.keys().copied());
        candidates.extend(self.virtual_data_revisions.keys().copied());
        candidates.extend(self.reported_visible_ranges.keys().copied());
        candidates.extend(self.pending_visible_ranges.borrow().keys().copied());
        self.reconcile_virtual_lists_for(Some(&candidates));
    }

    pub(super) fn emit_visible_range(&mut self, node_id: u32, start: u32, end: u32) {
        if self.reported_visible_ranges.get(&node_id) == Some(&(start, end)) {
            return;
        }
        let Some(node) = self.store.get(node_id) else {
            return;
        };
        if node.kind != KIND_VIRTUAL_LIST || node.listener_id == 0 {
            return;
        }
        self.reported_visible_ranges.insert(node_id, (start, end));
        let event = Event::visible_range(
            self.store.surface_id(),
            self.store.epoch(),
            self.store.revision(),
            self.next_sequence.fetch_add(1, Ordering::Relaxed),
            node_id,
            node.listener_id,
            start,
            end,
        );
        send_event_or_exit(
            self.runtime.as_ref(),
            "VirtualList visible range event",
            event,
        );
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use gpui::ListOffset;

    use crate::protocol::{
        Node, Patch, PatchOperation, Snapshot, UPDATE_PROPERTIES, VirtualListDataEdit,
    };
    use crate::transport::InMemoryAdapter;
    use crate::tree::KIND_VIEW;

    use super::*;

    fn list_properties(
        item_count: u32,
        data_revision: u32,
        data_edit: Option<VirtualListDataEdit>,
    ) -> VirtualListProperties {
        VirtualListProperties {
            item_count,
            range_start: 0,
            range_end: 4,
            estimated_item_size: 20.0,
            overscan: 2,
            data_revision,
            data_edit,
        }
    }

    fn mount_list(item_count: u32) -> SolidRoot {
        let runtime = InMemoryAdapter::new();
        let mut root = SolidRoot::new(runtime);
        let mut list = Node::new(2, 1, 0, crate::tree::KIND_VIRTUAL_LIST);
        list.listener_id = 12;
        list.host_properties = Some(HostProperties::VirtualList(list_properties(
            item_count, 0, None,
        )));
        root.store
            .apply_snapshot(Snapshot::new(
                7,
                3,
                0,
                1,
                vec![Node::new(1, 0, 0, KIND_VIEW), list],
            ))
            .expect("VirtualList snapshot");
        root.reconcile_virtual_lists();
        root
    }

    /// Mirror the integration wiring: collect the ordered patch update, replay
    /// it onto the list state, then reconcile the affected set.
    fn apply_list_update(root: &mut SolidRoot, list: VirtualListProperties) {
        let base = root.store.revision();
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                base,
                base + 1,
                vec![PatchOperation::Update {
                    id: 2,
                    mask: UPDATE_PROPERTIES,
                    style: None,
                    text: None,
                    listener_id: 12,
                    host_properties: Some(HostProperties::VirtualList(list.clone())),
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                    observes_layout: false,
                }],
            ))
            .expect("VirtualList patch");
        let affected: HashSet<u32> = [2].into_iter().collect();
        root.replay_virtual_list_updates(&[(2, list, false)]);
        root.reconcile_virtual_lists_for(Some(&affected));
    }

    #[test]
    fn prepend_edit_shifts_anchor_to_the_same_content() {
        let mut root = mount_list(100);
        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 40,
                offset_in_item: px(4.0),
            });

        apply_list_update(
            &mut root,
            list_properties(
                110,
                1,
                Some(VirtualListDataEdit {
                    base_revision: 0,
                    start: 0,
                    old_count: 0,
                    new_count: 10,
                }),
            ),
        );

        let state = root.virtual_lists.get(&2).expect("prepended list state");
        assert_eq!(state.item_count(), 110);
        assert_eq!(state.logical_scroll_top().item_ix, 50);
        assert_eq!(state.logical_scroll_top().offset_in_item, px(4.0));
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&1));
    }

    #[test]
    fn equal_count_replace_keeps_count_and_anchor_outside_span() {
        let mut root = mount_list(100);
        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 60,
                offset_in_item: px(0.0),
            });

        apply_list_update(
            &mut root,
            list_properties(
                100,
                1,
                Some(VirtualListDataEdit {
                    base_revision: 0,
                    start: 30,
                    old_count: 20,
                    new_count: 20,
                }),
            ),
        );

        let state = root.virtual_lists.get(&2).expect("replaced list state");
        assert_eq!(state.item_count(), 100);
        assert_eq!(state.logical_scroll_top().item_ix, 60);
        assert_eq!(state.logical_scroll_top().offset_in_item, px(0.0));
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&1));
    }

    #[test]
    fn sequential_edits_in_one_transaction_replay_in_order() {
        let mut root = mount_list(100);
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![
                    PatchOperation::Update {
                        id: 2,
                        mask: UPDATE_PROPERTIES,
                        style: None,
                        text: None,
                        listener_id: 12,
                        host_properties: Some(HostProperties::VirtualList(list_properties(
                            110,
                            1,
                            Some(VirtualListDataEdit {
                                base_revision: 0,
                                start: 100,
                                old_count: 0,
                                new_count: 10,
                            }),
                        ))),
                        accessibility: None,
                        focusable: false,
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
                        observes_layout: false,
                    },
                    PatchOperation::Update {
                        id: 2,
                        mask: UPDATE_PROPERTIES,
                        style: None,
                        text: None,
                        listener_id: 12,
                        host_properties: Some(HostProperties::VirtualList(list_properties(
                            105,
                            2,
                            Some(VirtualListDataEdit {
                                base_revision: 1,
                                start: 105,
                                old_count: 5,
                                new_count: 0,
                            }),
                        ))),
                        accessibility: None,
                        focusable: false,
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
                        observes_layout: false,
                    },
                ],
            ))
            .expect("two-list-update patch");

        // The store publishes only the final properties (revision 2, base 1);
        // ordered replay must consume both validated steps.
        root.replay_virtual_list_updates(&[
            (
                2,
                list_properties(
                    110,
                    1,
                    Some(VirtualListDataEdit {
                        base_revision: 0,
                        start: 100,
                        old_count: 0,
                        new_count: 10,
                    }),
                ),
                false,
            ),
            (
                2,
                list_properties(
                    105,
                    2,
                    Some(VirtualListDataEdit {
                        base_revision: 1,
                        start: 105,
                        old_count: 5,
                        new_count: 0,
                    }),
                ),
                false,
            ),
        ]);
        let affected: HashSet<u32> = [2].into_iter().collect();
        root.reconcile_virtual_lists_for(Some(&affected));

        let state = root.virtual_lists.get(&2).expect("replayed list state");
        assert_eq!(state.item_count(), 105);
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&2));
    }

    #[test]
    fn retained_edit_republish_does_not_reapply() {
        let mut root = mount_list(100);
        apply_list_update(
            &mut root,
            list_properties(
                101,
                1,
                Some(VirtualListDataEdit {
                    base_revision: 0,
                    start: 100,
                    old_count: 0,
                    new_count: 1,
                }),
            ),
        );
        assert_eq!(
            root.virtual_lists
                .get(&2)
                .expect("appended list state")
                .item_count(),
            101
        );

        // Viewport-only commit republishes identical properties, including the
        // retained edit; applying it again would corrupt the count.
        apply_list_update(
            &mut root,
            list_properties(
                101,
                1,
                Some(VirtualListDataEdit {
                    base_revision: 0,
                    start: 100,
                    old_count: 0,
                    new_count: 1,
                }),
            ),
        );

        assert_eq!(
            root.virtual_lists
                .get(&2)
                .expect("stable list state")
                .item_count(),
            101
        );
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&1));
    }

    #[test]
    fn recreate_generation_replays_only_the_final_incarnation() {
        let mut root = mount_list(100);
        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 99,
                offset_in_item: px(0.0),
            });

        // Legal sequential ops in one payload: update the old incarnation,
        // delete it, recreate the same id, then update the new incarnation.
        let mut created = Node::new(2, 1, 0, crate::tree::KIND_VIRTUAL_LIST);
        created.listener_id = 12;
        created.host_properties = Some(HostProperties::VirtualList(list_properties(30, 0, None)));
        root.store
            .apply_patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![
                    PatchOperation::Update {
                        id: 2,
                        mask: UPDATE_PROPERTIES,
                        style: None,
                        text: None,
                        listener_id: 12,
                        host_properties: Some(HostProperties::VirtualList(list_properties(
                            110,
                            1,
                            Some(VirtualListDataEdit {
                                base_revision: 0,
                                start: 100,
                                old_count: 0,
                                new_count: 10,
                            }),
                        ))),
                        accessibility: None,
                        focusable: false,
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
                        observes_layout: false,
                    },
                    PatchOperation::Delete { id: 2 },
                    PatchOperation::Create(created),
                    PatchOperation::Update {
                        id: 2,
                        mask: UPDATE_PROPERTIES,
                        style: None,
                        text: None,
                        listener_id: 12,
                        host_properties: Some(HostProperties::VirtualList(list_properties(
                            40,
                            1,
                            Some(VirtualListDataEdit {
                                base_revision: 0,
                                start: 0,
                                old_count: 0,
                                new_count: 10,
                            }),
                        ))),
                        accessibility: None,
                        focusable: false,
                        selectable: false,
                        tooltip: None,
                        accepts_pointer_move: false,
                        observes_layout: false,
                    },
                ],
            ))
            .expect("update-delete-create-update patch");

        // Collected updates in operation order; only the final generation may
        // seed state, and the create must drop the dead incarnation wholesale.
        root.replay_virtual_list_updates(&[
            (
                2,
                list_properties(
                    110,
                    1,
                    Some(VirtualListDataEdit {
                        base_revision: 0,
                        start: 100,
                        old_count: 0,
                        new_count: 10,
                    }),
                ),
                false,
            ),
            (2, list_properties(30, 0, None), true),
            (
                2,
                list_properties(
                    40,
                    1,
                    Some(VirtualListDataEdit {
                        base_revision: 0,
                        start: 0,
                        old_count: 0,
                        new_count: 10,
                    }),
                ),
                false,
            ),
        ]);
        let affected: HashSet<u32> = [2].into_iter().collect();
        root.reconcile_virtual_lists_for(Some(&affected));

        let state = root.virtual_lists.get(&2).expect("recreated list state");
        assert_eq!(state.item_count(), 40);
        assert_eq!(
            state.logical_scroll_top().item_ix,
            0,
            "a recreated node must not inherit the dead incarnation's scroll anchor"
        );
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&1));
    }

    #[test]
    fn deleted_list_releases_revision_state() {
        let mut root = mount_list(100);
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&0));

        root.store
            .apply_patch(Patch::new(
                7,
                3,
                1,
                2,
                vec![PatchOperation::Delete { id: 2 }],
            ))
            .expect("VirtualList delete");
        root.reconcile_virtual_lists_for(None);

        assert!(!root.virtual_data_revisions.contains_key(&2));
        assert!(!root.virtual_lists.contains_key(&2));
    }

    #[test]
    fn snapshot_reseed_replaces_retained_state_and_keeps_scroll() {
        let mut root = mount_list(100);
        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 40,
                offset_in_item: px(0.0),
            });

        // Same-epoch replacement snapshot carries an arbitrary revision/edit
        // pair that must never be consumed as a delta.
        let mut list = Node::new(2, 1, 0, crate::tree::KIND_VIRTUAL_LIST);
        list.listener_id = 12;
        list.host_properties = Some(HostProperties::VirtualList(VirtualListProperties {
            item_count: 80,
            range_start: 0,
            range_end: 4,
            estimated_item_size: 20.0,
            overscan: 2,
            data_revision: 9,
            data_edit: Some(VirtualListDataEdit {
                base_revision: 8,
                start: 0,
                old_count: 0,
                new_count: 80,
            }),
        }));
        root.store
            .apply_snapshot(Snapshot::new(
                7,
                3,
                1,
                2,
                vec![Node::new(1, 0, 0, KIND_VIEW), list],
            ))
            .expect("replacement snapshot");
        root.reconcile_virtual_lists_for(None);

        let state = root.virtual_lists.get(&2).expect("reseeded list state");
        assert_eq!(state.item_count(), 80);
        assert_eq!(state.logical_scroll_top().item_ix, 40);
        assert_eq!(root.virtual_data_revisions.get(&2), Some(&9));
    }

    #[test]
    fn tail_append_while_parked_at_end_follows_the_new_end() {
        let mut root = mount_list(100);
        root.virtual_lists
            .get(&2)
            .expect("initial list state")
            .scroll_to(ListOffset {
                item_ix: 100,
                offset_in_item: px(0.0),
            });

        apply_list_update(
            &mut root,
            list_properties(
                105,
                1,
                Some(VirtualListDataEdit {
                    base_revision: 0,
                    start: 100,
                    old_count: 0,
                    new_count: 5,
                }),
            ),
        );

        let state = root.virtual_lists.get(&2).expect("grown list state");
        assert_eq!(state.item_count(), 105);
        assert_eq!(
            state.logical_scroll_top().item_ix,
            105,
            "a list parked at the end stays at the end across tail appends"
        );
    }
}
