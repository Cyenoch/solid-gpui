import { MemoryTransport, createRoot } from '../../packages/solid-gpui/dist/index.js';
import { createComponent } from '../../packages/solid-gpui/dist/runtime.js';
import { RouterProvider } from '../../packages/solid-gpui-router/dist/index.js';
import { createGalleryState } from '../../examples/gallery/src/gallery/context';
import { createGalleryRouter } from '../../examples/gallery/src/gallery/routes';
import { Envelope } from '../../packages/solid-gpui/src/protocol/generated/protocol';

const transport = new MemoryTransport();
const root = createRoot(transport);
const gallery = createGalleryState('dark');
gallery.setRoot(root);
const router = createGalleryRouter();
let cursor = 0;
const nodes = new Map<number, number>();
function validate(label: string) {
  for (; cursor < transport.submitted.length; cursor++) {
    const frame = Envelope.decode(transport.submitted[cursor]!.subarray(4)).body!;
    if (frame.tag === 1) {
      nodes.clear();
      for (const node of frame.value.nodes!) nodes.set(node.id!, node.parentId!);
    }
    if (frame.tag !== 3) continue;
    const ops = frame.value.operations!;
    const before = new Map(nodes);
    for (let i = 0; i < ops.length; i++) {
      const op = ops[i]!.operation!;
      const id = op.tag === 1 ? op.value.node!.id! : op.value.id!;
      if (op.tag !== 1 && !nodes.has(id)) {
        const ancestors: number[] = []; for(let parent = before.get(id); parent; parent=before.get(parent)) ancestors.push(parent);
        console.error(JSON.stringify({label, cursor, i, id, ancestors, operations: ops.map(({operation: op})=>({tag: op!.tag, id: op!.tag === 1 ? op!.value.node!.id : op!.value.id, parent: op!.tag === 1 ? op!.value.node!.parentId : op!.tag === 3 ? op!.value.parentId : null}))},null,2));
        throw new Error(`missing node ${id}`);
      }
      if (op.tag === 1) {
        if (nodes.has(id) || !nodes.has(op.value.node!.parentId!)) throw new Error(`bad create ${id}`);
        nodes.set(id, op.value.node!.parentId!);
      }
      if (op.tag === 3) {
        if (!nodes.has(op.value.parentId!)) throw new Error(`bad parent ${id}`);
        nodes.set(id, op.value.parentId!);
      }
      if (op.tag === 4) {
        const remove = (id: number) => {
          for (const [child, parent] of nodes) if (parent === id) remove(child);
          nodes.delete(id);
        };
        remove(id);
      }
    }
  }
}
try {
  root.render(() => createComponent(RouterProvider, {router}));
  await router.load(); await Promise.resolve(); validate('initial');
  for (const query of ['Native','Overlays','Settings','','Controls','Native','S','Se','Set','Settings','','xxxxxxxx','','Dock','Charts']) {
    gallery.setSearchQuery(query);
    await Promise.resolve(); validate(query);
  }
  gallery.showStatus('transient status', 1);
  await Promise.resolve(); validate('status shown');
  await Bun.sleep(5); validate('status expired');
  const queries = ['Overlays', 'Settings', 'Native', '', 'Controls', 'Dock', 'Charts', 'xxx'];
  const paths = ['/native-controls', '/native-overlays', '/native-settings', '/native-dock', '/native-charts'];
  for (let turn = 0; turn < 80; turn++) {
    await router.navigate({to: paths[turn % paths.length]!});
    await Promise.resolve(); validate(`route ${turn}`);
    gallery.toggleTheme();
    const query = queries[turn % queries.length]!;
    for(let i = 0; i <= query.length; i++) {
      gallery.setSearchQuery(query.slice(0,i));
      if (turn % 2 === 0) { await Promise.resolve(); validate(`query ${turn}/${i}`); }
    }
    gallery.windowSizeStore.set(turn % 2 ? 800 : 1280, 800, 1);
    await Promise.resolve(); validate(`cycle ${turn}`);
  }
  console.log('VALID', cursor, nodes.size);
} finally { root.unmount(); }
