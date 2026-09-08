import { mountApplication, Pressable, Text } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

type State = { session: { userId: string | null | undefined } };

mountApplication<State>({
  transport: () => new EmbeddedTransport(),
  setup(previous) {
    const [userId, setUserId] = createSignal(previous?.session.userId);
    const [active, setActive] = createSignal(false);
    return {
      onMount: () => setActive(true),
      captureState: () => ({ session: { userId: userId() } }),
      render: () => (
        <Pressable onPress={() => setUserId(null)}>
          <Text>{userId() === null ? "Signed out" : active() ? "Uninitialized" : "Staged"}</Text>
        </Pressable>
      ),
    };
  },
});
