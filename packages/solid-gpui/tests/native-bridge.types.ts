import type { Root } from "@solid-gpui/core";
import { Button, Checkbox } from "@solid-gpui/core/components";
import { createClient } from "../../../examples/gallery/src/gallery/generated/native";

// Compiled by package-typecheck; not executed as a UI or runtime test.
export async function generatedContractsAreTypeSafe(root: Root) {
  const client = createClient(root);
  const report = await client.analyzeWorkspace({ name: "Typed workspace", readiness: true, builds: 2 });
  const completed: number = report.progress.completed;
  const status: "blocked" | "ready" | "established" = report.status;
  void [completed, status];

  // @ts-expect-error Rust u32 inputs are numbers.
  await client.analyzeWorkspace({ name: "Typed workspace", readiness: true, builds: "2" });
  // @ts-expect-error Request fields are generated from Rust.
  await client.analyzeWorkspace({ name: "Typed workspace", readiness: true, builds: 2, missingField: 1 });
  // @ts-expect-error Nested Rust result fields retain their types.
  const invalid: string = report.progress.completed;
  void invalid;

  Button({ variant: "primary", label: "Build" });
  // @ts-expect-error Provider enum values come from the Rust catalog.
  Button({ variant: "unsupported" });
  Checkbox({ checked: true, onChange: (checked: boolean) => void checked });
  // @ts-expect-error Provider events preserve their boolean payload.
  Checkbox({ onChange: (checked: string) => void checked });
}
