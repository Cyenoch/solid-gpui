import { Text, TextInput, View, type TextInputHandle } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { useGallery } from "../context";
import { Badge, Button, Card, DenseRow, Divider, PropTable, ResponsiveRow, SectionHeader } from "../components/ui";

export function TextInputShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  let inputHandle: TextInputHandle | undefined;

  const [textValue, setTextValue] = createSignal("Hello Solid GPUI");
  const [multilineValue, setMultilineValue] = createSignal(
    "Solid GPUI provides native text input with hardware accelerated cursor painting and selection handling.",
  );
  const [passwordValue, setPasswordValue] = createSignal("secretPassword123");
  const [selectionInfo, setSelectionInfo] = createSignal<{ start: number; end: number }>({ start: 0, end: 0 });
  const [isFocused, setIsFocused] = createSignal(false);
  const [submittedText, setSubmittedText] = createSignal<string | null>(null);

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="TextInput Component"
        description="TextInput binds native editable text fields to reactive signals. It supports single-line inputs, multi-line text areas, controlled and uncontrolled values, selection tracking, submit editing, and focus commands."
        tag="Host Component"
      />

      {/* Interactive Inputs */}
      <Card
        title="Interactive Text Fields"
        description="Test different input variants and inspect real-time state synchronization."
      >
        <View style={{ gap: 16 }}>
          {/* Single-line Controlled Input */}
          <View style={{ gap: 8 }}>
            <ResponsiveRow gap={8} grow={[1, 0]}>
              <Text style={{ color: theme().textPrimary, fontSize: 14, fontWeight: "semibold" }}>
                1. Standard Single-line Input (Controlled)
              </Text>
              <Badge
                label={isFocused() ? "Focused" : "Blurred"}
                variant={isFocused() ? "success" : "default"}
                size="sm"
              />
            </ResponsiveRow>
            <TextInput
              ref={(handle: TextInputHandle) => {
                inputHandle = handle;
              }}
              value={textValue()}
              placeholder="Type something here..."
              onChangeText={(val) => setTextValue(val)}
              onFocus={() => setIsFocused(true)}
              onBlur={() => setIsFocused(false)}
              onSelectionChange={(sel) => setSelectionInfo({ start: sel.start, end: sel.end })}
              onSubmitEditing={(val) => {
                setSubmittedText(val);
                showStatus(`Submitted text: "${val}"`);
              }}
              style={{
                backgroundColor: theme().bgInput,
                borderWidth: 1,
                borderColor: isFocused() ? theme().borderFocus : theme().border,
                borderRadius: 6,
                padding: 10,
                color: theme().textPrimary,
                fontSize: 14,
                minWidth: 0,
                flexShrink: 1,
              }}
            />

            {/* Input Toolbar (Programmatic Focus & Selection) */}
            <DenseRow gap={8} style={{ alignItems: "center", marginTop: 4 }}>
              <Button size="sm" variant="secondary" onPress={() => inputHandle?.focus()}>
                Focus Input
              </Button>
              <Button size="sm" variant="secondary" onPress={() => inputHandle?.blur()}>
                Blur Input
              </Button>
              <Button size="sm" variant="secondary" onPress={() => inputHandle?.setSelection(0, textValue().length)}>
                Select All
              </Button>
              <Button size="sm" variant="outline" onPress={() => setTextValue("")}>
                Clear
              </Button>
              <Text style={{ color: theme().textMuted, fontSize: 12, marginLeft: 8 }}>
                {() => `Selection: [${selectionInfo().start}, ${selectionInfo().end}] | Length: ${textValue().length}`}
              </Text>
            </DenseRow>
          </View>

          <Divider margin={4} />

          {/* Multi-line Text Area */}
          <View style={{ gap: 8 }}>
            <Text style={{ color: theme().textPrimary, fontSize: 14, fontWeight: "semibold" }}>
              2. Multi-line TextArea (multiline: true)
            </Text>
            <TextInput
              multiline
              value={multilineValue()}
              placeholder="Enter multi-line text notes..."
              onChangeText={(val) => setMultilineValue(val)}
              style={{
                backgroundColor: theme().bgInput,
                borderWidth: 1,
                borderColor: theme().border,
                borderRadius: 6,
                padding: 12,
                color: theme().textPrimary,
                fontSize: 13,
                lineHeight: 18,
                minHeight: 80,
                minWidth: 0,
                flexShrink: 1,
              }}
            />
          </View>

          <Divider margin={4} />

          {/* Password & Disabled Inputs Row */}
          <ResponsiveRow gap={16}>
            {/* Password input */}
            <View style={{ flexGrow: 1, gap: 6 }}>
              <Text style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "medium" }}>3. Password Field</Text>
              <TextInput
                value={passwordValue()}
                placeholder="Enter password..."
                onChangeText={(val) => setPasswordValue(val)}
                style={{
                  backgroundColor: theme().bgInput,
                  borderWidth: 1,
                  borderColor: theme().border,
                  borderRadius: 6,
                  padding: 10,
                  color: theme().textPrimary,
                  fontSize: 14,
                  minWidth: 0,
                  flexShrink: 1,
                }}
              />
            </View>

            {/* Disabled input */}
            <View style={{ flexGrow: 1, gap: 6 }}>
              <Text style={{ color: theme().textMuted, fontSize: 13, fontWeight: "medium" }}>4. Disabled Input</Text>
              <TextInput
                disabled
                value="System configuration (read-only)"
                style={{
                  backgroundColor: theme().bgActive,
                  borderWidth: 1,
                  borderColor: theme().border,
                  borderRadius: 6,
                  padding: 10,
                  color: theme().textMuted,
                  fontSize: 14,
                  opacity: 0.6,
                  minWidth: 0,
                  flexShrink: 1,
                }}
              />
            </View>
          </ResponsiveRow>
        </View>
      </Card>

      {/* Props Reference */}
      <Card
        title="TextInput Props Reference"
        description="All supported properties and callbacks on the native TextInput component."
      >
        <PropTable
          props={[
            { name: "value", type: "string", default: "undefined", description: "Controlled text content" },
            {
              name: "defaultValue",
              type: "string",
              default: "undefined",
              description: "Initial text value when uncontrolled",
            },
            {
              name: "placeholder",
              type: "string",
              default: "undefined",
              description: "Placeholder text shown when field is empty",
            },
            {
              name: "onChangeText",
              type: "(text: string) => void",
              default: "undefined",
              description: "Invoked on each keystroke/edit with current text",
            },
            {
              name: "onSelectionChange",
              type: "(sel: Selection) => void",
              default: "undefined",
              description: "Fired when caret or highlight selection changes",
            },
            {
              name: "onSubmitEditing",
              type: "(text: string) => void",
              default: "undefined",
              description: "Fired when user presses Return / Enter key",
            },
            {
              name: "multiline",
              type: "boolean",
              default: "false",
              description: "Allows multiple lines and line-breaking",
            },
            { name: "disabled", type: "boolean", default: "false", description: "Disables editing and focus" },
            { name: "maxLength", type: "number", default: "undefined", description: "Maximum allowed character count" },
            {
              name: "onFocus",
              type: "() => void",
              default: "undefined",
              description: "Fired when input gains keyboard focus",
            },
            { name: "onBlur", type: "() => void", default: "undefined", description: "Fired when input loses focus" },
          ]}
        />
      </Card>
    </View>
  );
}
