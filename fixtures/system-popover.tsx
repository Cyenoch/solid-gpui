import { mountApplication, Pressable, SystemPopover, Text, TextInput, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

mountApplication<{ name: string; open: boolean }>({
  transport: () => new EmbeddedTransport(),
  setup(previous) {
    const [name, setName] = createSignal(previous?.name ?? "Ada");
    const [open, setOpen] = createSignal(previous?.open ?? false);
    const [nested, setNested] = createSignal(false);
    const [error, setError] = createSignal("");
    const [mounted, setMounted] = createSignal(true);
    const button = { padding: 12, borderRadius: 8, backgroundColor: "#2563eb" } as const;
    return {
      captureState: () => ({ name: name(), open: open() }),
      onMount: async (root) => {
        await root.setTitle("SystemPopover • native acceptance");
        await root.resize(560, 340);
      },
      render: () => (
        <View style={{ widthPercent: 100, heightPercent: 100, padding: 24, gap: 16, backgroundColor: "#101827" }}>
          <Text style={{ color: "#ffffff", fontSize: 22 }}>Owned native popovers</Text>
          <Text style={{ color: "#bac8de" }}>Move this window near a screen edge, then edit the shared name.</Text>
          <Text style={{ color: "#ffffff" }}>Saved name: {name()}</Text>
          <Pressable focusable onPress={() => setMounted((value) => !value)}>
            <Text style={{ color: "#93c5fd" }}>{mounted() ? "Unmount trigger" : "Mount trigger"}</Text>
          </Pressable>
          <View style={{ flexGrow: 1 }} />
          {mounted() && (
            <SystemPopover
              open={open()}
              onOpenChange={setOpen}
              width={340}
              height={236}
              gap={8}
              accessibilityLabel="Edit shared name"
              onError={(error) => setError(String(error))}
              slots={{
                trigger: (
                  <View style={button}>
                    <Text style={{ color: "#ffffff" }}>Edit name</Text>
                  </View>
                ),
              }}
              content={() => (
                <View
                  accessibilityRole="dialog"
                  accessibilityLabel="Edit name"
                  style={{
                    widthPercent: 100,
                    heightPercent: 100,
                    padding: 20,
                    gap: 12,
                    borderRadius: 12,
                    backgroundColor: "#eaf1fb",
                  }}
                >
                  <Text style={{ color: "#172033", fontSize: 18 }}>Content in a separate Surface</Text>
                  <TextInput
                    value={name()}
                    onChangeText={setName}
                    accessibilityLabel="Name"
                    style={{ widthPercent: 100, height: 36, backgroundColor: "#ffffff", color: "#172033", padding: 8 }}
                  />
                  <Pressable focusable onPress={() => setOpen(false)} style={button}>
                    <Text style={{ color: "#ffffff" }}>Save {name()}</Text>
                  </Pressable>
                  <SystemPopover
                    open={nested()}
                    onOpenChange={setNested}
                    width={230}
                    height={100}
                    slots={{ trigger: <Text style={{ color: "#2563eb" }}>Open nested popover</Text> }}
                    content={() => (
                      <View style={{ padding: 16, backgroundColor: "#ffffff", widthPercent: 100, heightPercent: 100 }}>
                        <Text>Shared name: {name()}</Text>
                      </View>
                    )}
                  />
                </View>
              )}
            />
          )}
          <Text style={{ color: "#fca5a5" }}>{error()}</Text>
        </View>
      ),
    };
  },
});
