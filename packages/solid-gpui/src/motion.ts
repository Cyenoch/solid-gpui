import { createRenderEffect, createRoot, createSignal, getOwner, mergeProps, onCleanup, splitProps } from "solid-js";
import { NativePresence } from "@solid-gpui/core/components";
import type { SolidElement } from "./renderer";

export { Motion, type MotionProps, type MotionTarget, type MotionAnimation } from "@solid-gpui/core/components";

export type PresenceProps = Omit<
  Parameters<typeof NativePresence>[0],
  "present" | "generation" | "children" | "onComplete" | "ref"
> & {
  readonly show: boolean;
  /** Construct the child once per mounted interval, under the current Solid context. */
  readonly children: () => SolidElement;
  readonly onExitComplete?: () => void;
};

/** Retain the child owner through native exit motion, then dispose its state and resources. */
export function Presence(props: PresenceProps): SolidElement {
  const owner = getOwner();
  if (!owner) throw new Error("Presence requires a Solid owner");
  const [local, native] = splitProps(props, ["show", "children", "onExitComplete"]);
  const [content, setContent] = createSignal<SolidElement>();
  const [generation, setGeneration] = createSignal(0);
  let previous = local.show;
  let release: (() => void) | undefined;
  const disposeContent = () => {
    const dispose = release;
    release = undefined;
    dispose?.();
    setContent(undefined);
  };
  createRenderEffect(() => {
    const show = local.show;
    if (show !== previous) {
      previous = show;
      setGeneration((value) => {
        if (value === 0xffff_ffff) throw new Error("Presence generation exhausted");
        return value + 1;
      });
    }
    if (show && !release) {
      const child = createRoot((dispose) => {
        release = dispose;
        return local.children();
      }, owner);
      setContent(() => child);
    }
  });
  onCleanup(disposeContent);
  return NativePresence(
    mergeProps(native, {
      get present() {
        return local.show;
      },
      get generation() {
        return generation();
      },
      get children() {
        return content();
      },
      onComplete: (event: { present: boolean; generation: number }) => {
        if (event.present || local.show || event.generation !== generation() || !release) return;
        disposeContent();
        local.onExitComplete?.();
      },
    }),
  );
}
