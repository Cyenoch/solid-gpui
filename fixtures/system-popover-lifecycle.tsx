// Native lifecycle acceptance: keep the test application in the foreground.
import { mountApplication, SystemPopover, Text, View } from "@solid-gpui/core";
import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { createSignal } from "@solid-gpui/core/runtime";

const application = mountApplication({
  transport: () => new EmbeddedTransport(),
  setup() {
    const [mounted, setMounted] = createSignal(true);
    let mounts = 0;
    const [open, setOpen] = createSignal(true);
    const [nested, setNested] = createSignal(true);
    return {
      render: () => (
        <View style={{ padding: 30 }}>
          {mounted() && (
            <SystemPopover
              open={open()}
              onOpenChange={setOpen}
              width={320}
              height={160}
              slots={{ trigger: <Text>Outer</Text> }}
              content={() => (
                <View style={{ padding: 20, widthPercent: 100, heightPercent: 100, backgroundColor: "#ffffff" }}>
                  <Text>Parent stays open</Text>
                  <SystemPopover
                    open={nested()}
                    onOpenChange={setNested}
                    width={220}
                    height={100}
                    slots={{ trigger: <Text>Nested</Text> }}
                    content={() => {
                      mounts++;
                      setTimeout(async () => {
                        await application.root!.getWindowBounds();
                        if (!open() || !nested()) throw new Error("FAIL: nested activation dismissed its parent");
                        if (mounts < 3) {
                          console.log(`SystemPopover lifecycle: completed mount ${mounts}`);
                          setMounted(false);
                          setTimeout(() => setMounted(true), 150);
                        } else {
                          console.log("PASS: three native nested mounts preserve both Surfaces and the owner responds");
                          application.quit();
                        }
                      }, 500);
                      return (
                        <View
                          style={{ padding: 16, widthPercent: 100, heightPercent: 100, backgroundColor: "#eaf1fb" }}
                        >
                          <Text>Nested stays open</Text>
                        </View>
                      );
                    }}
                  />
                </View>
              )}
            />
          )}
        </View>
      ),
    };
  },
});
