import { Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { useGallery } from "../context";
import type { SolidChild } from "../types";
import {
  Button,
  Card,
  CodeSnippet,
  DenseRow,
  Divider,
  Input,
  PropTable,
  ResponsiveRow,
  SectionHeader,
} from "../components/ui";

export function NativePlatformShowcase(): SolidChild {
  const { theme, root, showStatus } = useGallery();

  const [windowTitle, setWindowTitle] = createSignal("Solid GPUI Gallery");
  const [clipboardContent, setClipboardContent] = createSignal<string>("");
  const [lastNotification, setLastNotification] = createSignal("Not requested");
  const [filePickResult, setFilePickResult] = createSignal<string | null>(null);

  const handleSetTitle = async () => {
    const r = root();
    if (r) {
      await r.setTitle(windowTitle());
      showStatus(`Window title updated to "${windowTitle()}"`, "success");
    }
  };

  const handleReadClipboard = async () => {
    const r = root();
    if (r) {
      const text = await r.getClipboardText();
      setClipboardContent(text);
      showStatus("Read text from system clipboard", "info");
    }
  };

  const handleWriteClipboard = async () => {
    const r = root();
    if (r) {
      await r.setClipboardText(clipboardContent());
      showStatus("Copied text to system clipboard", "success");
    }
  };

  const handleShowNotification = async () => {
    const r = root();
    if (!r) {
      setLastNotification("Native host is not connected");
      showStatus("Cannot request notification: native host is not connected", "warning");
      return;
    }
    try {
      await r.showNotification({
        title: "Solid GPUI Notification",
        body: "Native OS notification sent from the SolidJS renderer.",
      });
      setLastNotification(`Accepted by host at ${new Date().toLocaleTimeString()}`);
      showStatus("Notification request accepted by host", "info");
    } catch (error) {
      const message = error instanceof Error ? error.message : String(error);
      setLastNotification(`Unavailable: ${message}`);
      showStatus(`Notification unavailable: ${message}`, "warning");
    }
  };

  const handlePickFile = async () => {
    const r = root();
    if (r) {
      const paths = await r.pickFiles({ multiple: false });
      if (paths && paths.length > 0) {
        setFilePickResult(paths[0]);
        showStatus(`Selected file: ${paths[0]}`, "success");
      }
    }
  };

  return (
    <View style={{ gap: 20, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Native & Platform APIs"
        tag="Desktop Integration"
        description="Direct bidirectional communication with the GPUI native host: window management, system clipboard, notifications, file pickers, and menus."
      />

      {/* Window Controls Card */}
      <Card
        title="Window Title & Dimensions"
        description="Programmatically adjust desktop window attributes via root methods."
      >
        <ResponsiveRow gap={12} grow={[1, 0]} style={{ alignItems: "flex-end" }}>
          <View style={{ alignSelf: "stretch", maxWidth: 280, minWidth: 0 }}>
            <Input label="Window Title" value={windowTitle()} onInput={setWindowTitle} />
          </View>
          <Button variant="primary" onPress={handleSetTitle}>
            Apply Title
          </Button>
        </ResponsiveRow>
        <Divider margin={8} />
        <DenseRow gap={10} style={{ alignItems: "center" }}>
          <Button
            variant="outline"
            onPress={async () => {
              const r = root();
              if (r) await r.toggleFullscreen();
              showStatus("Toggled Fullscreen", "info");
            }}
          >
            Toggle Fullscreen
          </Button>
          <Button
            variant="outline"
            onPress={async () => {
              const r = root();
              if (r) await r.minimizeWindow();
              showStatus("Window Minimized", "info");
            }}
          >
            Minimize Window
          </Button>
          <Button
            variant="outline"
            onPress={async () => {
              const r = root();
              if (r) await r.zoom();
              showStatus("Window Zoomed", "info");
            }}
          >
            Zoom / Maximize
          </Button>
        </DenseRow>
      </Card>

      {/* System Clipboard Integration */}
      <Card
        title="System Clipboard Integration"
        description="Read and write arbitrary text to and from the host OS clipboard."
      >
        <Input
          label="Clipboard Payload"
          value={clipboardContent()}
          onInput={setClipboardContent}
          placeholder="Type text here to write to system clipboard..."
          multiline
        />
        <DenseRow gap={10}>
          <Button variant="primary" onPress={handleWriteClipboard}>
            Copy to Clipboard
          </Button>
          <Button variant="secondary" onPress={handleReadClipboard}>
            Paste from Clipboard
          </Button>
        </DenseRow>
      </Card>

      {/* Native Notifications & File Pickers */}
      <Card
        title="Native Dialogs & Notifications"
        description="Native file selection is available. OS notifications are currently unavailable in this provider host because of the upstream callback singleton limitation."
      >
        <DenseRow gap={12} style={{ alignItems: "center" }}>
          <Button variant="primary" onPress={handleShowNotification}>
            Try Notification API
          </Button>
          <Button variant="secondary" onPress={handlePickFile}>
            Open Native File Picker
          </Button>
        </DenseRow>
        <Text style={{ color: theme().textSecondary, fontSize: 12, minWidth: 0 }}>
          {`Notification status: ${lastNotification()}`}
        </Text>
        {filePickResult() ? (
          <View
            style={{
              backgroundColor: theme().bgMuted,
              borderRadius: 6,
              padding: 12,
              borderWidth: 1,
              borderColor: theme().border,
              gap: 4,
            }}
          >
            <Text style={{ color: theme().success, fontSize: 11, fontWeight: "bold" }}>SELECTED NATIVE PATH:</Text>
            <Text
              style={{
                color: theme().textPrimary,
                fontSize: 13,
                minWidth: 0,
                flexShrink: 1,
                overflow: "hidden",
                textOverflow: "ellipsis",
              }}
            >
              {filePickResult()!}
            </Text>
          </View>
        ) : null}
      </Card>

      {/* Root Methods Reference */}
      <Card
        title="Root API Methods Reference"
        description="Commands supported on the Root handle instance returned by createRoot(transport)."
      >
        <PropTable
          props={[
            {
              name: "setTitle(title)",
              type: "Promise<void>",
              default: "-",
              description: "Updates the host desktop window title",
            },
            {
              name: "resize(w, h)",
              type: "Promise<void>",
              default: "-",
              description: "Resizes the native window bounds",
            },
            {
              name: "minimizeWindow()",
              type: "Promise<void>",
              default: "-",
              description: "Minimizes window to OS dock / taskbar",
            },
            {
              name: "toggleFullscreen()",
              type: "Promise<void>",
              default: "-",
              description: "Toggles native fullscreen mode",
            },
            {
              name: "setClipboardText(text)",
              type: "Promise<void>",
              default: "-",
              description: "Copies UTF-8 string to host clipboard",
            },
            {
              name: "getClipboardText()",
              type: "Promise<string>",
              default: "-",
              description: "Reads current plain text from clipboard",
            },
            {
              name: "showNotification(opts)",
              type: "Promise<void>",
              default: "-",
              description: "Dispatches native OS notification bubble",
            },
            {
              name: "pickFiles(opts)",
              type: "Promise<string[]|null>",
              default: "-",
              description: "Opens native OS file selection modal",
            },
            {
              name: "pickSavePath(opts)",
              type: "Promise<string|null>",
              default: "-",
              description: "Opens native OS file save destination modal",
            },
            {
              name: "setMenus(menus)",
              type: "Promise<void>",
              default: "-",
              description: "Configures native application menu bar",
            },
            {
              name: "setKeybindings(keys)",
              type: "Promise<void>",
              default: "-",
              description: "Registers global native keystroke handlers",
            },
          ]}
        />
      </Card>

      {/* Code Example */}
      <Card title="Code Example: Root Native APIs">
        <CodeSnippet
          code={`import { createRoot, StdioTransport } from "@solid-gpui/core";

const root = createRoot(new StdioTransport());

// Native platform commands
await r.setTitle("My Native App");
await r.setClipboardText("Copied string");
const files = await r.pickFiles({ multiple: true });`}
        />
      </Card>
    </View>
  );
}
