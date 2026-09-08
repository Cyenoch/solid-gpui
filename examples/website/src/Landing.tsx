import { landingCode } from "./snippets";
import { CodeBlock } from "./CodeBlock";
import { HeroVisual } from "./HeroVisual";
import { locale, t } from "./i18n";
import { View, TextInput, Pressable } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import { Copy, Button, Counter, LayoutDemo, colors, panel } from "./ui";

function Inputs() {
  const [name, setName] = createSignal("");
  const [saved, setSaved] = createSignal(false);
  return (
    <View style={panel}>
      <Copy size={15} style={{ fontWeight: "semibold" }}>
        Make yourself at home
      </Copy>
      <Copy color={colors.muted}>Small details. Familiar interactions.</Copy>
      <Copy size={13}>Display name</Copy>
      <TextInput
        value={name()}
        onChangeText={(value) => {
          setName(value);
          setSaved(false);
        }}
        placeholder={t("Your name")}
        accessibilityLabel="Display name"
        style={{
          height: 44,
          fontSize: 14,
          lineHeight: 20,
          padding: 10,
          color: colors.text,
          backgroundColor: colors.bg,
          borderWidth: 1,
          borderColor: colors.line,
          borderRadius: 8,
        }}
      />
      <Button primary icon={saved() ? "lucide:check" : "lucide:check"} onPress={() => setSaved(true)}>
        {saved() ? "Saved" : "Save changes"}
      </Button>
      <Copy size={12} color={colors.muted}>
        {saved()
          ? locale() === "zh-CN"
            ? `欢迎${name() ? `，${name()}` : ""}，已准备就绪。`
            : `Welcome${name() ? `, ${name()}` : ""}. You're all set.`
          : "Your profile, just the way you like it."}
      </Copy>
    </View>
  );
}
function Preferences() {
  const [enabled, setEnabled] = createSignal(true);
  const [compact, setCompact] = createSignal(false);
  return (
    <View style={panel}>
      <Copy size={15} style={{ fontWeight: "semibold" }}>
        Your workspace, your way
      </Copy>
      <Copy color={colors.muted}>Settings that respond instantly.</Copy>
      {[
        { name: "Notifications", note: "Keep up with your workspace.", value: enabled, set: setEnabled },
        { name: "Compact layout", note: "A little more room to focus.", value: compact, set: setCompact },
      ].map((setting) => (
        <View style={{ flexDirection: "row", alignItems: "center", gap: 16, padding: 6 }}>
          <View style={{ flexGrow: 1, minWidth: 0, gap: 3 }}>
            <Copy>{setting.name}</Copy>
            <Copy size={12} color={colors.muted}>
              {setting.note}
            </Copy>
          </View>
          <Pressable
            accessibilityLabel={setting.name}
            onPress={() => setting.set(!setting.value())}
            style={{
              width: 36,
              height: 22,
              padding: 3,
              flexDirection: "row",
              justifyContent: setting.value() ? "flex-end" : "flex-start",
              borderRadius: 12,
              backgroundColor: setting.value() ? colors.text : colors.line,
            }}
          >
            <View
              style={{
                width: 16,
                height: 16,
                borderRadius: 8,
                backgroundColor: setting.value() ? colors.bg : colors.muted,
              }}
            />
          </Pressable>
        </View>
      ))}
    </View>
  );
}
function CodeCard() {
  return <CodeBlock label="counter.tsx" source={landingCode} />;
}
export function Landing(props: { width: number; height: number; navigate: (path: string) => void }) {
  const narrow = () => props.width < 1100;
  const contentWidth = () => Math.min(narrow() ? 700 : 1200, props.width - (narrow() ? 32 : 64));
  const column = () => (narrow() ? contentWidth() : (contentWidth() - 40) / 3);
  return (
    <View style={{ height: props.height, overflow: "scroll", gap: 48 }}>
      <HeroVisual width={props.width} height={props.width < 600 ? 520 : 600}>
        <Button
          ghost
          onPress={() => props.navigate("/docs/start")}
          style={{ borderRadius: 20, backgroundColor: "#1c1c1cbb" }}
        >
          Your next app starts here
        </Button>
        <Copy
          size={props.width < 360 ? 30 : narrow() ? 34 : 48}
          style={{
            maxWidth: contentWidth() - (narrow() ? 32 : 80),
            fontWeight: "bold",
            textAlign: "center",
            lineHeight: props.width < 360 ? 38 : narrow() ? 42 : 58,
          }}
        >
          Build native apps. Stay with Solid.
        </Copy>
        <Copy
          size={narrow() ? 16 : 18}
          color={colors.muted}
          style={{ width: Math.min(620, contentWidth() - 32), textAlign: "center" }}
        >
          Bring your Solid skills to native apps. Familiar components, thoughtful interactions, and room to make it
          yours.
        </Copy>
        <View
          style={{ flexDirection: props.width < 400 ? "column" : "row", alignItems: "center", gap: 10, marginTop: 8 }}
        >
          <Button
            primary
            icon="lucide:chevron-right"
            iconAfter
            style={{ minHeight: 44, width: props.width < 400 ? 190 : 150 }}
            onPress={() => props.navigate("/docs/start")}
          >
            Get Started
          </Button>
          <Button
            icon="lucide:layers"
            style={{ minHeight: 44, width: 190 }}
            onPress={() => props.navigate("/docs/layout")}
          >
            {locale() === "en" ? "Explore components" : "探索组件"}
          </Button>
        </View>
      </HeroVisual>
      <View style={{ width: contentWidth(), alignSelf: "center", flexShrink: 0, gap: 16 }}>
        <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center" }}>
          <View style={{ width: 260 }}>
            <Copy size={13}>Explore the building blocks</Copy>
          </View>
          {() =>
            !narrow() ? (
              <View style={{ width: 280 }}>
                <Copy size={12} color={colors.muted}>
                  Try a few things. Make yourself at home.
                </Copy>
              </View>
            ) : null
          }
        </View>
        <View style={{ flexShrink: 0, flexDirection: narrow() ? "column" : "row", gap: 20 }}>
          <View style={{ width: column(), flexShrink: 0, gap: 20 }}>
            <Inputs />
            <Preferences />
          </View>
          <View style={{ width: column(), flexShrink: 0, gap: 20 }}>
            <Counter />
            <CodeCard />
          </View>
          <View style={{ width: column(), flexShrink: 0, gap: 20 }}>
            <LayoutDemo />
            <View style={panel}>
              <Copy size={15} style={{ fontWeight: "semibold" }}>
                Start small. Make it yours.
              </Copy>
              <Copy color={colors.muted}>Build a simple screen, connect your data, and grow from there.</Copy>
              <Button icon="lucide:book-open" onPress={() => props.navigate("/docs/start")}>
                Read the documentation →
              </Button>
            </View>
          </View>
        </View>
      </View>
      <View style={{ width: contentWidth(), alignSelf: "center", flexShrink: 0, padding: 12 }}>
        <Copy size={12} color={colors.muted}>
          Solid GPUI · Open source. Built for your next idea.
        </Copy>
      </View>
    </View>
  );
}
