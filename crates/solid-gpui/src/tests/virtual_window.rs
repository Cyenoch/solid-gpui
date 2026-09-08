use crate::protocol::{HostProperties, Patch, Snapshot};
use crate::tree::{KIND_VIRTUAL_LIST, NodeStore};

#[test]
fn solid_window_patches_preserve_row_order_in_the_native_tree() {
    let output = std::process::Command::new("bun")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args(["--conditions=browser", "-e", r#"
import { MemoryTransport, VirtualList, Text, createRoot } from './packages/solid-gpui/src/index.ts';
import { createComponent } from './packages/solid-gpui/src/runtime.ts';
import { encodeFrame } from './packages/solid-gpui/src/protocol/index.ts';
const transport = new MemoryTransport();
const root = createRoot(transport, { surfaceId: 32 });
root.render(() => createComponent(VirtualList, {
  data: Array.from({ length: 100 }, (_, i) => i), itemKey: i => i,
  initialNumToRender: 34, estimatedItemSize: 32,
  renderItem: i => createComponent(Text, { children: String(i) }),
}));
const { Envelope } = await import('./packages/solid-gpui/src/protocol/generated/protocol.ts');
const snapshot = Envelope.decode(transport.submitted[0].subarray(4)).body.value;
const list = snapshot.nodes.find(n => n.kind === 6);
let sequence = 0;
for (const [start, end] of [[17, 53], [1, 35], [80, 100]]) {
  transport.push(encodeFrame({ type: 'event', surfaceId: 32, epoch: 1, revision: transport.submitted.length,
    sequence: ++sequence, nodeId: list.id, listenerId: list.listenerId,
    payload: { type: 'visible-range', start, end } }));
  await Promise.resolve();
}
for (const frame of transport.submitted) process.stdout.write(frame);
"#])
        .output().expect("run Solid window fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut frames = output.stdout.as_slice();
    let mut store = NodeStore::default();
    let mut revision = 0;
    while !frames.is_empty() {
        let len = u32::from_le_bytes(frames[..4].try_into().unwrap()) as usize;
        let payload = &frames[4..4 + len];
        if revision == 0 {
            store
                .apply_snapshot(Snapshot::decode(payload).unwrap())
                .unwrap();
        } else {
            store.apply_patch(Patch::decode(payload).unwrap()).unwrap();
        }
        revision += 1;
        let list = store
            .iter()
            .find(|node| node.kind == KIND_VIRTUAL_LIST)
            .unwrap();
        let Some(HostProperties::VirtualList(range)) = &list.host_properties else {
            panic!("list properties")
        };
        for (offset, index) in (range.range_start..range.range_end).enumerate() {
            let row = store.get_child_at(list.id, offset as u32).unwrap();
            let text = store.get_child_at(row.id, 0).unwrap();
            let raw = store.get_child_at(text.id, 0).unwrap();
            assert_eq!(
                raw.text.as_deref(),
                Some(index.to_string().as_str()),
                "row {offset} at revision {revision}"
            );
        }
        frames = &frames[4 + len..];
    }
    assert_eq!(revision, 4);
}

#[test]
fn structural_patch_rescues_retained_children_before_removing_their_parent() {
    let output = std::process::Command::new("bun")
        .current_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
        .args(["--conditions=browser", "-e", r#"
import { RootContainer } from './packages/solid-gpui/src/renderer/root-container.ts';
import { hostConfig as h, withRoot } from './packages/solid-gpui/src/renderer/host-config.ts';
const frames = [];
const label = text => { const node = h.createElement('Text'); h.insertNode(node, h.createTextNode(text)); return node; };
const root = new RootContainer({surfaceId: 8, epoch: 1, scheduleDispatch: f => f(), submitFrame: f => { frames.push(f); return true; }});
withRoot(root.tree, () => {
  root.tree.beginRender();
  const main = h.createElement('View'), left = h.createElement('View'), right = h.createElement('View');
  const a = label('A'), b = label('B');
  h.insertNode(root.tree.syntheticRoot, main);
  h.insertNode(main, left); h.insertNode(main, right);
  h.insertNode(left, a); h.insertNode(right, b);
  root.tree.commit();
  root.tree.beginRender();
  const next = h.createElement('View'), c = label('C');
  h.insertNode(main, next, right);
  h.insertNode(next, a); h.removeNode(main, left);
  h.insertNode(next, c, a); h.insertNode(main, right, next);
  root.tree.commit();
});
for (const frame of frames) process.stdout.write(frame);
"#])
        .output().expect("run retained-child fixture");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let first_len = u32::from_le_bytes(output.stdout[..4].try_into().unwrap()) as usize;
    let mut store = NodeStore::default();
    store
        .apply_snapshot(Snapshot::decode(&output.stdout[4..4 + first_len]).unwrap())
        .unwrap();
    let retained_id = store
        .iter()
        .find(|node| {
            node.kind == crate::tree::KIND_TEXT && node.text_content.as_deref() == Some("A")
        })
        .unwrap()
        .id;
    store
        .apply_patch(Patch::decode(&output.stdout[8 + first_len..]).unwrap())
        .unwrap();
    let main = store.get_child_at(1, 0).unwrap();
    let right = store.get_child_at(main.id, 0).unwrap();
    let next = store.get_child_at(main.id, 1).unwrap();
    assert_eq!(
        store
            .get_child_at(right.id, 0)
            .unwrap()
            .text_content
            .as_deref(),
        Some("B")
    );
    assert_eq!(
        store
            .get_child_at(next.id, 0)
            .unwrap()
            .text_content
            .as_deref(),
        Some("C")
    );
    assert_eq!(store.get_child_at(next.id, 1).unwrap().id, retained_id);
    assert!(store.get_child_at(main.id, 2).is_none());
}
