import { hostConfig, withRoot } from "../../packages/solid-gpui/src/renderer/host-config";
import { RootContainer } from "../../packages/solid-gpui/src/renderer/root-container";

// This measures host-tree construction, not GPUI layout or display cadence.
for (const count of [1_000, 5_000, 10_000]) {
  const samples: number[] = [];
  for (let run = 0; run < 6; run++) {
    const container = new RootContainer({
      surfaceId: 1, epoch: 1, scheduleDispatch: (dispatch) => dispatch(),
      submitFrame: () => true,
    });
    const start = performance.now();
    withRoot(container.tree, () => {
      const parent = hostConfig.createElement("View");
      for (let index = 0; index < count; index++) {
        hostConfig.insertNode(parent, hostConfig.createElement("View"));
      }
      hostConfig.insertNode(container.tree.syntheticRoot, parent);
      container.tree.commit();
      if (parent.children.length !== count || parent.children.some((node, index) => node.index !== index)) {
        throw new Error("construction lost child identity or order");
      }
    });
    const elapsed = performance.now() - start;
    if (run > 0) samples.push(elapsed);
    container.dispose();
    await Promise.resolve();
  }
  console.log(JSON.stringify({ count, samplesMs: samples, medianMs: [...samples].sort((a, b) => a - b)[2] }));
}
