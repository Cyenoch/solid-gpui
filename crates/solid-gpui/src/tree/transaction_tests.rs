use super::*;

fn tree(unrelated: u32) -> NodeStore {
    let mut raw = Node::new(6, 4, 0, KIND_RAW_TEXT);
    raw.text = Some("before".into());
    let mut nodes = vec![
        Node::new(1, 0, 0, KIND_VIEW),
        Node::new(2, 1, 0, KIND_VIEW),
        Node::new(4, 2, 0, KIND_TEXT),
        raw,
        Node::new(3, 1, 1, KIND_VIEW),
        Node::new(5, 3, 0, KIND_TEXT),
    ];
    nodes.extend((0..unrelated).map(|i| Node::new(10 + i, 1, 2 + i, KIND_VIEW)));
    let mut store = NodeStore::empty();
    store
        .apply_snapshot(Snapshot::new(1, 1, 0, 1, nodes))
        .unwrap();
    store
}

fn text_update() -> PatchOperation {
    PatchOperation::Update {
        id: 6,
        mask: UPDATE_TEXT,
        text: Some("after".into()),
        style: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
        observes_layout: false,
    }
}

fn raw_text_update(id: u32, text: &str) -> PatchOperation {
    PatchOperation::Update {
        id,
        mask: UPDATE_TEXT,
        text: Some(text.to_owned()),
        style: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
        observes_layout: false,
    }
}

/// Root 1 holds paragraph 2 ("a" + run 4("b") + "c"), a plain sibling 7, and
/// paragraph 8 ("x") so text batches cross nested styled runs.
fn rich_tree() -> NodeStore {
    let mut raw_a = Node::new(3, 2, 0, KIND_RAW_TEXT);
    raw_a.text = Some("a".into());
    let mut run = Node::new(4, 2, 1, KIND_TEXT);
    run.style = Some(Style {
        color_rgba: Some(0x3366ccff),
        ..Style::default()
    });
    let mut raw_run = Node::new(5, 4, 0, KIND_RAW_TEXT);
    raw_run.text = Some("b".into());
    let mut raw_c = Node::new(6, 2, 2, KIND_RAW_TEXT);
    raw_c.text = Some("c".into());
    let mut raw_x = Node::new(9, 8, 0, KIND_RAW_TEXT);
    raw_x.text = Some("x".into());
    let nodes = vec![
        Node::new(1, 0, 0, KIND_VIEW),
        Node::new(2, 1, 0, KIND_TEXT),
        raw_a,
        run,
        raw_run,
        raw_c,
        Node::new(7, 1, 1, KIND_VIEW),
        Node::new(8, 1, 2, KIND_TEXT),
        raw_x,
    ];
    let mut store = NodeStore::empty();
    store
        .apply_snapshot(Snapshot::new(1, 1, 0, 1, nodes))
        .unwrap();
    store
}

fn child_order(store: &NodeStore, parent_id: u32) -> Vec<u32> {
    (0..)
        .map_while(|index| store.get_child_at(parent_id, index).map(|node| node.id))
        .collect()
}

#[test]
fn clearing_raw_text_is_rejected_at_the_operation_before_deferred_aggregation() {
    let mut store = rich_tree();
    let before = store.clone();
    // Clearing the text would leave an illegal RawText, and the later delete
    // removes the node before deferred aggregation could observe the illegal
    // intermediate state. The operation itself must still fail, exactly as
    // eager per-operation derivation did.
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![
            PatchOperation::Update {
                id: 6,
                mask: UPDATE_TEXT,
                text: None,
                style: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
                observes_layout: false,
            },
            PatchOperation::Delete { id: 6 },
        ],
    );
    assert_eq!(
        store.apply_patch(patch).unwrap_err(),
        TreeError::InvalidChild {
            node_id: 2,
            child_id: 6,
            reason: "Text child must carry text",
        }
    );
    assert_eq!(store, before);
}

#[test]
fn batched_text_updates_derive_each_text_ancestor_once() {
    let mut store = rich_tree();
    let mut created = Node::new(20, 2, 3, KIND_RAW_TEXT);
    created.text = Some("D".into());
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![
            raw_text_update(3, "A"),
            raw_text_update(5, "B"),
            raw_text_update(6, "C"),
            PatchOperation::Create(created),
        ],
    );
    let changes = store
        .apply_patch_validated(patch, |candidate, _| {
            assert_eq!(
                candidate.get(2).unwrap().text_content.as_deref(),
                Some("ABCD")
            );
            assert_eq!(candidate.get(4).unwrap().text_content.as_deref(), Some("B"));
            assert_eq!(candidate.get(8).unwrap().text_content.as_deref(), Some("x"));
            Ok::<_, TreeError>(())
        })
        .unwrap();
    assert_eq!(store.get(2).unwrap().text_content.as_deref(), Some("ABCD"));
    assert_eq!(store.get(4).unwrap().text_content.as_deref(), Some("B"));
    assert_eq!(store.get(8).unwrap().text_content.as_deref(), Some("x"));
    assert_eq!(changes.changed, HashSet::from([2, 3, 4, 5, 6, 20]));
    assert!(changes.removed.is_empty());
    // Paragraph 2 and nested run 4 rederive once for the whole batch; the
    // untouched paragraph 8 never rederives.
    assert_eq!(changes.work.text_derivations, 2);
}

#[test]
fn invalid_operation_after_derived_work_rolls_back_the_whole_transaction() {
    let mut store = rich_tree();
    let before = store.clone();
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![
            raw_text_update(3, "A"),
            raw_text_update(5, "B"),
            PatchOperation::Move {
                id: 6,
                parent_id: 2,
                index: 0,
            },
            PatchOperation::Move {
                id: 5,
                parent_id: 8,
                index: 1,
            },
            PatchOperation::Create(Node::new(6, 2, 0, KIND_RAW_TEXT)),
        ],
    );
    let error = store.apply_patch(patch).unwrap_err();
    assert_eq!(
        error,
        TreeError::PatchConflict {
            operation: 4,
            node_id: 6
        }
    );
    assert_eq!(store, before);
    store
        .apply_patch(Patch::new(1, 1, 1, 2, vec![raw_text_update(3, "A")]))
        .unwrap();
    assert_eq!(store.get(2).unwrap().text_content.as_deref(), Some("Abc"));
    assert_eq!(store.revision(), 2);
}

#[test]
fn moved_and_deleted_text_rederives_only_surviving_ancestors() {
    let mut store = rich_tree();
    let changes = store
        .apply_patch_validated(
            Patch::new(
                1,
                1,
                1,
                2,
                vec![
                    PatchOperation::Move {
                        id: 6,
                        parent_id: 8,
                        index: 0,
                    },
                    PatchOperation::Delete { id: 3 },
                ],
            ),
            |candidate, _| {
                assert_eq!(candidate.get(2).unwrap().text_content.as_deref(), Some("b"));
                assert_eq!(
                    candidate.get(8).unwrap().text_content.as_deref(),
                    Some("cx")
                );
                Ok::<_, TreeError>(())
            },
        )
        .unwrap();
    assert_eq!(store.get(2).unwrap().text_content.as_deref(), Some("b"));
    assert_eq!(store.get(8).unwrap().text_content.as_deref(), Some("cx"));
    assert_eq!(store.get(4).unwrap().text_content.as_deref(), Some("b"));
    assert!(store.get(3).is_none());
    assert_eq!(changes.removed, HashSet::from([3]));
    assert_eq!(changes.changed, HashSet::from([2, 3, 4, 6, 8, 9]));
    assert_eq!(changes.work.text_derivations, 2);
}

#[test]
fn validation_failure_and_unwind_restore_structure_text_and_revision() {
    for unwind in [false, true] {
        let mut store = tree(0);
        let before = store.clone();
        let patch = Patch::new(
            1,
            1,
            1,
            2,
            vec![
                text_update(),
                PatchOperation::Move {
                    id: 6,
                    parent_id: 5,
                    index: 0,
                },
                PatchOperation::Delete { id: 4 },
                PatchOperation::Create(Node::new(7, 2, 0, KIND_TEXT)),
            ],
        );
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            store.apply_patch_validated(patch, |candidate, changes| {
                assert_eq!(candidate.revision(), 1);
                assert_eq!(
                    candidate.get(5).unwrap().text_content.as_deref(),
                    Some("after")
                );
                assert!(changes.affected.contains(&2) && changes.affected.contains(&3));
                assert!(changes.removed.contains(&4));
                assert!(changes.changed.contains(&7));
                if unwind {
                    panic!("adapter validation unwound");
                }
                Err(TreeError::InvalidProperties {
                    node_id: 5,
                    reason: "rejected contract",
                })
            })
        }));
        if unwind {
            assert!(result.is_err());
        } else {
            assert!(result.unwrap().is_err());
        }
        assert_eq!(store, before);
        store
            .apply_patch(Patch::new(1, 1, 1, 2, vec![text_update()]))
            .unwrap();
        assert_eq!(store.revision(), 2);
    }
}

#[test]
fn leaf_update_dependency_work_does_not_grow_with_unrelated_nodes() {
    for unrelated in [1_000, 20_000] {
        let mut store = tree(unrelated);
        let changes = store
            .apply_patch_validated(Patch::new(1, 1, 1, 2, vec![text_update()]), |_, changes| {
                assert_eq!(changes.changed, HashSet::from([4, 6]));
                assert_eq!(changes.validation, HashSet::from([1, 2, 4, 6]));
                Ok::<_, TreeError>(())
            })
            .unwrap();
        assert_eq!(changes.affected, HashSet::from([1, 2, 4, 6]));
        assert!(changes.removed.is_empty());
        assert_eq!(store.get(4).unwrap().text_content.as_deref(), Some("after"));
    }
}

#[test]
fn moved_slot_revalidates_children_and_both_original_and_final_parents() {
    let mut store = tree(0);
    let changes = store
        .apply_patch_validated(
            Patch::new(
                1,
                1,
                1,
                2,
                vec![PatchOperation::Move {
                    id: 4,
                    parent_id: 3,
                    index: 1,
                }],
            ),
            |_, changes| {
                assert!(
                    changes.validation.contains(&6),
                    "a moved child's ownership reads its grandparent"
                );
                assert!(changes.validation.contains(&2) && changes.validation.contains(&3));
                Ok::<_, TreeError>(())
            },
        )
        .unwrap();
    assert_eq!(store.get(4).unwrap().parent_id, 3);
    assert!(changes.affected.contains(&2));
}

fn indexed_tree() -> NodeStore {
    let nodes = vec![
        Node::new(1, 0, 0, KIND_VIEW),
        Node::new(9, 1, 0, KIND_VIEW),
        Node::new(10, 9, 0, KIND_VIEW),
        Node::new(11, 9, 1, KIND_VIEW),
        Node::new(12, 9, 2, KIND_VIEW),
        Node::new(13, 9, 3, KIND_VIEW),
        Node::new(14, 9, 4, KIND_VIEW),
    ];
    let mut store = NodeStore::empty();
    store
        .apply_snapshot(Snapshot::new(1, 1, 0, 1, nodes))
        .unwrap();
    store
}

#[test]
fn mixed_create_move_delete_keeps_sequential_wire_indexes() {
    let mut store = indexed_tree();
    let patch = Patch::new(
        1,
        1,
        1,
        2,
        vec![
            PatchOperation::Create(Node::new(15, 9, 0, KIND_VIEW)),
            PatchOperation::Move {
                id: 12,
                parent_id: 9,
                index: 0,
            },
            PatchOperation::Delete { id: 11 },
            PatchOperation::Move {
                id: 14,
                parent_id: 9,
                index: 1,
            },
            PatchOperation::Move {
                id: 10,
                parent_id: 1,
                index: 1,
            },
        ],
    );
    // The validation callback observes derived state already settled: each
    // operation's wire index refers to the result of the previous one.
    let changes = store
        .apply_patch_validated(patch, |candidate, _| {
            assert_eq!(child_order(candidate, 9), vec![12, 14, 15, 13]);
            assert_eq!(candidate.get(12).unwrap().index, 0);
            Ok::<_, TreeError>(())
        })
        .unwrap();
    assert_eq!(child_order(&store, 9), vec![12, 14, 15, 13]);
    assert_eq!(child_order(&store, 1), vec![9, 10]);
    for parent in [1, 9] {
        for (position, id) in child_order(&store, parent).into_iter().enumerate() {
            assert_eq!(store.get(id).unwrap().index as usize, position);
        }
    }
    assert_eq!(changes.removed, HashSet::from([11]));
    assert_eq!(changes.work.index_writes, 4);
}

#[test]
fn structural_batch_keeps_sibling_work_bounded_by_parent_width() {
    let width: u32 = 120;
    let moves: u32 = 48;
    let mut nodes = vec![Node::new(1, 0, 0, KIND_VIEW), Node::new(9, 1, 0, KIND_VIEW)];
    nodes.extend((0..width).map(|i| Node::new(100 + i, 9, i, KIND_VIEW)));
    let mut store = NodeStore::empty();
    store
        .apply_snapshot(Snapshot::new(1, 1, 0, 1, nodes))
        .unwrap();
    let operations: Vec<PatchOperation> = (0..moves)
        .map(|i| PatchOperation::Move {
            id: 100 + i,
            parent_id: 9,
            index: 0,
        })
        .collect();
    let changes = store
        .apply_patch_validated(Patch::new(1, 1, 1, 2, operations), |_, _| {
            Ok::<_, TreeError>(())
        })
        .unwrap();
    // Machine-independent bounds: journaling is per moved identity plus one
    // settled resync over the parent, never per moved suffix.
    assert!(changes.work.journal_captures <= 2 * moves + width + 16);
    assert!(changes.work.index_writes <= width);
    assert_eq!(changes.work.text_derivations, 0);
    let order = child_order(&store, 9);
    let expected_first: Vec<u32> = (0..moves).rev().map(|i| 100 + i).collect();
    assert_eq!(order[..moves as usize], expected_first[..]);
    for (position, id) in order.into_iter().enumerate() {
        assert_eq!(store.get(id).unwrap().index as usize, position);
    }
    let stats = store.last_patch_stats();
    assert_eq!(stats.operation_count, moves);
    assert_eq!(stats.affected_nodes, moves);
    assert_eq!(stats.affected_parents, 1);
}

fn layout_update(id: u32, observes_layout: bool) -> PatchOperation {
    PatchOperation::Update {
        id,
        mask: UPDATE_LAYOUT,
        observes_layout,
        style: None,
        text: None,
        listener_id: 0,
        host_properties: None,
        accessibility: None,
        focusable: false,
        selectable: false,
        tooltip: None,
        accepts_pointer_move: false,
    }
}

#[test]
fn observes_layout_binds_updates_and_rolls_back_with_failed_transactions() {
    let mut store = rich_tree();
    assert!(!store.get(2).unwrap().observes_layout);

    store
        .apply_patch(Patch::new(1, 1, 1, 2, vec![layout_update(2, true)]))
        .unwrap();
    assert!(store.get(2).unwrap().observes_layout);

    store
        .apply_patch(Patch::new(1, 1, 2, 3, vec![layout_update(2, false)]))
        .unwrap();
    assert!(!store.get(2).unwrap().observes_layout);

    let error = store
        .apply_patch(Patch::new(
            1,
            1,
            3,
            4,
            vec![PatchOperation::Update {
                id: 2,
                mask: UPDATE_LAYOUT | 1024,
                observes_layout: true,
                style: None,
                text: None,
                listener_id: 0,
                host_properties: None,
                accessibility: None,
                focusable: false,
                selectable: false,
                tooltip: None,
                accepts_pointer_move: false,
            }],
        ))
        .unwrap_err();
    assert!(matches!(
        error,
        TreeError::InvalidPatchOperation {
            operation: 0,
            reason: "invalid update mask"
        }
    ));

    // A later invalid operation must not leak an earlier subscription flip.
    let before = store.clone();
    let patch = Patch::new(
        1,
        1,
        3,
        4,
        vec![layout_update(7, true), PatchOperation::Delete { id: 404 }],
    );
    assert!(matches!(
        store.apply_patch(patch),
        Err(TreeError::MissingPatchNode { .. })
    ));
    assert_eq!(store, before);

    // A combined mask binds both fields, and created nodes carry the flag.
    store
        .apply_patch(Patch::new(
            1,
            1,
            3,
            4,
            vec![
                PatchOperation::Update {
                    id: 2,
                    mask: UPDATE_STYLE | UPDATE_LAYOUT,
                    observes_layout: true,
                    style: Some(Style {
                        color_rgba: Some(0xff0000ff),
                        ..Style::default()
                    }),
                    text: None,
                    listener_id: 0,
                    host_properties: None,
                    accessibility: None,
                    focusable: false,
                    selectable: false,
                    tooltip: None,
                    accepts_pointer_move: false,
                },
                PatchOperation::Create({
                    let mut view = Node::new(21, 1, 3, KIND_VIEW);
                    view.observes_layout = true;
                    view
                }),
            ],
        ))
        .unwrap();
    assert!(store.get(2).unwrap().observes_layout);
    assert_eq!(
        store.get(2).unwrap().style.as_ref().unwrap().color_rgba,
        Some(0xff0000ff)
    );
    assert!(store.get(21).unwrap().observes_layout);
}
