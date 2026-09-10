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
    }
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
