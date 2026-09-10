import type { ComponentVariant } from "../component-variants";

export const kitRecipes: ComponentVariant[] = [
  {
    component: "TextView",
    id: "TextView--frontmatter",
    title: "Document metadata",
    description: "Render opt-in Markdown metadata as a native description list.",
    source:
      'import { TextView } from "@solid-gpui/core/components";\nexport default function Example() {\n  return <TextView format="markdown" frontmatter text={"---\\ntitle: Project notes\\nauthor: Alex\\n---\\n# Release plan\\n\\nReady for review."} selectable />;\n}',
  },
  {
    component: "Icon",
    id: "Icon--svg",
    title: "Explicit SVG",
    description: "Use a bounded monochrome SVG in an application icon slot.",
    source:
      'import { Button } from "@solid-gpui/core/components";\nconst svg = \'<svg viewBox="0 0 24 24"><path d="M5 12l4 4L19 6" fill="none" stroke="currentColor" stroke-width="2"/></svg>\';\nexport default function Example() { return <Button label="Approved" icon={{ svg }} />; }',
  },
  {
    component: "Editor",
    id: "Editor--native-editing",
    title: "Native editing",
    description: "Try native bracket completion, indentation and multiple cursors.",
    source:
      'import { Editor } from "@solid-gpui/core/components";\nexport default function Example() {\n  return <Editor language="javascript" autoClose smartIndent defaultValue={"function greet() {\\n  return \'Hello\';\\n}"} style={{ height: 220 }} />;\n}',
  },
  {
    component: "Motion",
    id: "Motion--keyframes",
    title: "Replay keyframes",
    description: "Replay native keyframes with a short stagger delay.",
    source:
      'import { Button, Motion, Label } from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nimport { createSignal } from "@solid-gpui/core/runtime";\nexport default function Example() {\n  const [playbackId, replay] = createSignal(0);\n  return <View style={{ gap: 12 }}><Button label="Replay" onPress={() => replay(playbackId() + 1)} />\n    <Motion playbackId={playbackId()} stagger={{ index: 1, count: 2, intervalMs: 80 }} animation={{ type: "keyframes", durationMs: 500, frames: [{ offset: 0, value: { x: 0, opacity: 0 } }, { offset: 1, value: { x: 80, opacity: 1 } }] }}><Label text="Native keyframes" /></Motion>\n  </View>;\n}',
  },
  {
    component: "NativePresence",
    id: "NativePresence--lifecycle",
    title: "Presence lifecycle",
    description: "Keep child resources alive until native exit completes.",
    source:
      'import { Presence } from "@solid-gpui/core/motion";\nimport { Button, Label } from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nimport { createSignal } from "@solid-gpui/core/runtime";\nexport default function Example() {\n  const [open, setOpen] = createSignal(true);\n  return <View style={{ gap: 12 }}><Button label="Toggle" onPress={() => setOpen(!open())} />\n    <Presence show={open()} reveal>{() => <Label text="Retained through native exit" />}</Presence>\n  </View>;\n}',
  },
  {
    component: "Carousel",
    id: "Carousel--vertical",
    title: "Vertical carousel",
    description: "Navigate a vertically bounded native viewport.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.Carousel orientation="vertical" viewportHeight={160} pagination><N.CarouselItem><N.Label text="First" /></N.CarouselItem><N.CarouselItem><N.Label text="Second" /></N.CarouselItem></N.Carousel>;\n}',
  },
  {
    component: "Carousel",
    id: "Carousel--looping",
    title: "Looping carousel",
    description: "Continue from the last slide to the first.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.Carousel looping viewportHeight={140}><N.CarouselItem><N.Label text="Overview" /></N.CarouselItem><N.CarouselItem><N.Label text="Details" /></N.CarouselItem></N.Carousel>;\n}',
  },
  {
    component: "CarouselItem",
    id: "CarouselItem--content",
    title: "Slide content",
    description: "Compose native controls inside each slide.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.Carousel viewportHeight={160}><N.CarouselItem style={{ padding: 24 }}><N.Button label="Open overview" /></N.CarouselItem><N.CarouselItem style={{ padding: 24 }}><N.Label text="Complete" /></N.CarouselItem></N.Carousel>;\n}',
  },
  {
    component: "CarouselItem",
    id: "CarouselItem--labels",
    title: "Slide labels",
    description: "Give slides meaningful accessible names.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.Carousel viewportHeight={140} pagination><N.CarouselItem accessibilityLabel="Release overview"><N.Label text="Release overview" /></N.CarouselItem><N.CarouselItem accessibilityLabel="Release checklist"><N.Label text="Release checklist" /></N.CarouselItem></N.Carousel>;\n}',
  },
  {
    component: "BaseButton",
    id: "BaseButton--disabled",
    title: "BaseButton disabled",
    description: "Disable native activation while keeping the custom visual.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseButton accessibilityLabel="Unavailable" disabled style={{ padding: 12, opacity: 0.5 }}><N.Label text="Unavailable" /></N.BaseButton>;\n}',
  },
  {
    component: "BaseButton",
    id: "BaseButton--state",
    title: "BaseButton state",
    description: "Render application-owned content for a native control state.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseButton accessibilityLabel="Selected action" selected={true} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text="Selected action" /></N.BaseButton>;\n}',
  },
  {
    component: "BaseCheckbox",
    id: "BaseCheckbox--disabled",
    title: "BaseCheckbox disabled",
    description: "Disable native activation while keeping the custom visual.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseCheckbox accessibilityLabel="Unavailable" disabled style={{ padding: 12, opacity: 0.5 }}><N.Label text="Unavailable" /></N.BaseCheckbox>;\n}',
  },
  {
    component: "BaseCheckbox",
    id: "BaseCheckbox--state",
    title: "BaseCheckbox state",
    description: "Render application-owned content for a native control state.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseCheckbox accessibilityLabel="Mixed selection" state={"indeterminate"} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text="Mixed selection" /></N.BaseCheckbox>;\n}',
  },
  {
    component: "BaseSwitch",
    id: "BaseSwitch--disabled",
    title: "BaseSwitch disabled",
    description: "Disable native activation while keeping the custom visual.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseSwitch accessibilityLabel="Unavailable" disabled style={{ padding: 12, opacity: 0.5 }}><N.Label text="Unavailable" /></N.BaseSwitch>;\n}',
  },
  {
    component: "BaseSwitch",
    id: "BaseSwitch--state",
    title: "BaseSwitch state",
    description: "Render application-owned content for a native control state.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseSwitch accessibilityLabel="Enabled setting" checked={true} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text="Enabled setting" /></N.BaseSwitch>;\n}',
  },
  {
    component: "BaseToggle",
    id: "BaseToggle--disabled",
    title: "BaseToggle disabled",
    description: "Disable native activation while keeping the custom visual.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseToggle accessibilityLabel="Unavailable" disabled style={{ padding: 12, opacity: 0.5 }}><N.Label text="Unavailable" /></N.BaseToggle>;\n}',
  },
  {
    component: "BaseToggle",
    id: "BaseToggle--state",
    title: "BaseToggle state",
    description: "Render application-owned content for a native control state.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nexport default function Example() {\n\nreturn <N.BaseToggle accessibilityLabel="Pinned item" pressed={true} style={{ padding: 12, borderWidth: 1, borderColor: "#64748b", borderRadius: 8 }}><N.Label text="Pinned item" /></N.BaseToggle>;\n}',
  },
  {
    component: "Motion",
    id: "Motion--transition",
    title: "Delayed transition",
    description: "Animate a changed target after a bounded delay.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nimport { createSignal } from "@solid-gpui/core/runtime";\nexport default function Example() {\nconst [moved, setMoved] = createSignal(false);\nreturn <View style={{ gap: 12 }}><N.Button label="Toggle target" onPress={() => setMoved(!moved())} /><N.Motion target={{ x: moved() ? 90 : 0, opacity: moved() ? 0.5 : 1 }} animation={{ type: "transition", durationMs: 250, delayMs: 100 }}><N.Label text="Native transition" /></N.Motion></View>;\n}',
  },
  {
    component: "NativePresence",
    id: "NativePresence--offset",
    title: "Slide and fade",
    description: "Combine opacity with native vertical entry and exit motion.",
    source:
      'import * as N from "@solid-gpui/core/components";\nimport { View } from "@solid-gpui/core";\nimport { createSignal } from "@solid-gpui/core/runtime";\nexport default function Example() {\nconst [open, setOpen] = createSignal(true);\nreturn <View style={{ gap: 12 }}><N.Button label="Toggle" onPress={() => setOpen(!open())} /><N.NativePresence present={open()} offsetY={24} durationMs={300}><N.Label text="Slide and fade" /></N.NativePresence></View>;\n}',
  },
];
