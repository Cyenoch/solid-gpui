import { Icon, Pressable, Text, TextInput, View, useWindowSize } from "@solid-gpui/core";
import type { IconProps, Style } from "@solid-gpui/core";
import { createMemo, createSignal, onCleanup } from "@solid-gpui/core/runtime";
import { contrastingText } from "../theme";
import { useGallery } from "../context";
import type { SolidChild } from "../types";

export interface ButtonProps {
  readonly children?: string;
  readonly icon?: IconProps["name"];
  readonly iconPosition?: "start" | "end";
  readonly accessibilityLabel?: string;
  readonly onPress?: () => void;
  readonly variant?: "primary" | "secondary" | "outline" | "ghost" | "danger" | "success";
  readonly size?: "sm" | "md" | "lg";
  readonly disabled?: boolean;
  readonly active?: boolean;
  readonly tooltip?: string;
}

export function Button(props: ButtonProps) {
  const { theme } = useGallery();
  const [hovered, setHovered] = createSignal(false);
  const lineHeight = () => (props.size === "sm" ? 16 : props.size === "lg" ? 22 : 18);

  const style = () => {
    const t = theme();
    const variant = props.variant ?? "primary";
    const size = props.size ?? "md";
    const isHovered = hovered();
    const isDisabled = props.disabled ?? false;
    const isActive = props.active ?? false;

    let bg = t.primary;
    let border = "#00000000";
    let borderWidth = 0;

    if (variant === "primary") {
      bg = isActive || isHovered ? t.primaryHover : t.primary;
    } else if (variant === "secondary") {
      bg = isActive || isHovered ? t.bgCardHover : t.bgCard;
      border = t.border;
      borderWidth = 1;
    } else if (variant === "outline") {
      bg = isHovered ? t.accentMuted : "#00000000";
      border = isHovered ? t.accent : t.border;
      borderWidth = 1;
    } else if (variant === "ghost") {
      bg = isActive || isHovered ? t.bgActive : "#00000000";
    } else if (variant === "danger") {
      bg = isHovered ? "#DC2626" : t.danger;
    } else if (variant === "success") {
      bg = isHovered ? "#059669" : t.success;
    }

    let p = 8;
    let borderRadius = 6;
    if (size === "sm") {
      p = 5;
      borderRadius = 4;
    } else if (size === "lg") {
      p = 14;
      borderRadius = 8;
    }

    return {
      // Single-line controls own their height; intrinsic height compounds
      // measurement through nested flex rows (notably the Kanban actions).
      height: lineHeight() + p * 2 + borderWidth * 2,
      padding: p,
      backgroundColor: bg,
      borderRadius,
      borderWidth,
      borderColor: border,
      opacity: isDisabled ? 0.5 : 1,
      flexDirection: "row" as const,
      flexShrink: 0,
      alignItems: "center" as const,
      justifyContent: "center" as const,
      gap: 6,
    };
  };

  const labelStyle = () => {
    const t = theme();
    const variant = props.variant ?? "primary";
    const size = props.size ?? "md";
    let color = t.primaryText;
    if (variant === "secondary" || variant === "outline" || variant === "ghost") {
      color = hovered() || props.active ? t.accent : t.textPrimary;
    } else if (variant === "danger" || variant === "success") {
      color = contrastingText(style().backgroundColor);
    }
    return {
      color,
      fontSize: size === "sm" ? 11 : size === "lg" ? 15 : 13,
      lineHeight: lineHeight(),
      fontWeight: "medium" as const,
    };
  };

  return (
    <Pressable
      style={style()}
      disabled={props.disabled}
      onPress={() => {
        if (!props.disabled) props.onPress?.();
      }}
      onHoverChange={setHovered}
      accessibilityRole="button"
      accessibilityLabel={props.accessibilityLabel ?? props.children ?? props.tooltip}
      tooltip={props.tooltip}
    >
      {props.icon && props.iconPosition !== "end" ? (
        <Icon name={props.icon} size={labelStyle().fontSize} color={labelStyle().color} />
      ) : null}
      {props.children ? <Text style={labelStyle()}>{props.children}</Text> : null}
      {props.icon && props.iconPosition === "end" ? (
        <Icon name={props.icon} size={labelStyle().fontSize} color={labelStyle().color} />
      ) : null}
    </Pressable>
  );
}
/**
 * A semantic row whose columns stack below the breakpoint. Children are equal
 * peers by default; pass `grow` to preserve intentional proportions or use 0
 * for an intrinsic trailing child. Every child still gets bounded shrinkage.
 */
export interface ResponsiveRowProps {
  readonly children: SolidChild;
  readonly breakpoint?: number;
  readonly gap?: number;
  readonly grow?: readonly number[];
  readonly style?: Style;
}

export function ResponsiveRow(props: ResponsiveRowProps) {
  const { windowSizeStore } = useGallery();
  const windowSize = useWindowSize(windowSizeStore);
  const breakpoint = props.breakpoint ?? 960;
  const stacked = createMemo(() => windowSize().width < breakpoint);
  const children = () => {
    const value = props.children;
    if (!Array.isArray(value)) return value;
    return value.map((child, index) => (
      <View
        style={{
          width: !stacked() && (props.grow?.[index] ?? 1) > 0 ? 0 : undefined,
          minWidth: 0,
          flexGrow: stacked() ? 0 : (props.grow?.[index] ?? 1),
          flexShrink: stacked() ? 0 : 1,
        }}
      >
        {child}
      </View>
    ));
  };

  return (
    <View
      style={{
        ...(props.style ?? {}),
        flexDirection: stacked() ? "column" : "row",
        minWidth: 0,
        flexShrink: 0,
        gap: props.gap ?? 12,
      }}
    >
      {children()}
    </View>
  );
}

/**
 * A dense intrinsic row with an owned horizontal scroll surface. Use this for
 * toolbars and samples whose controls must remain intact rather than stack.
 * The policy fields are applied after caller styling so overflow ownership
 * cannot be accidentally removed by omission or an overriding style object.
 */
export interface DenseRowProps {
  readonly children: SolidChild;
  readonly gap?: number;
  readonly style?: Style;
}

export function DenseRow(props: DenseRowProps) {
  return (
    <View
      style={{
        ...(props.style ?? {}),
        flexDirection: "row",
        justifyContent: "flex-start",
        minWidth: 0,
        flexShrink: 0,
        overflow: "scroll",
        gap: props.gap ?? 12,
      }}
    >
      {props.children}
    </View>
  );
}

export interface BadgeProps {
  readonly label: string;
  readonly variant?: "default" | "accent" | "success" | "warning" | "danger" | "neutral";
  readonly size?: "sm" | "md";
}

export function Badge(props: BadgeProps) {
  const { theme } = useGallery();
  const variant = () => props.variant ?? "default";
  const size = () => props.size ?? "md";

  const colors = () => {
    const t = theme();
    switch (variant()) {
      case "accent":
        return { bg: t.accentMuted, color: t.accent, border: t.accent };
      case "success":
        return { bg: t.successMuted, color: t.success, border: t.success };
      case "warning":
        return { bg: t.warningMuted, color: t.warning, border: t.warning };
      case "danger":
        return { bg: t.dangerMuted, color: t.danger, border: t.danger };
      case "neutral":
        return { bg: t.bgHover, color: t.textMuted, border: t.borderMuted };
      default:
        return { bg: t.bgActive, color: t.textSecondary, border: t.border };
    }
  };

  return (
    <View
      style={{
        padding: size() === "sm" ? 3 : 5,
        backgroundColor: colors().bg,
        borderRadius: size() === "sm" ? 4 : 6,
        borderWidth: 1,
        borderColor: colors().border,
        alignSelf: "flex-start",
        flexDirection: "row",
        alignItems: "center",
      }}
    >
      <Text
        style={{
          color: colors().color,
          fontSize: size() === "sm" ? 10 : 11,
          fontWeight: "semibold",
        }}
      >
        {props.label}
      </Text>
    </View>
  );
}

export interface CardProps {
  readonly title?: string;
  readonly description?: string;
  readonly badge?: string;
  readonly actions?: SolidChild;
  readonly children: SolidChild;
  readonly style?: Record<string, unknown>;
}

export function Card(props: CardProps) {
  const { theme, windowSizeStore } = useGallery();
  const windowSize = useWindowSize(windowSizeStore);
  const compact = createMemo(() => windowSize().width < 960);
  const hasHeader = () => Boolean(props.title || props.description || props.actions || props.badge);

  return (
    <View
      style={{
        minWidth: 0,
        flexShrink: 0,
        backgroundColor: theme().bgCard,
        borderWidth: 1,
        borderColor: theme().border,
        borderRadius: 10,
        padding: 18,
        gap: 16,
        ...(props.style ?? {}),
      }}
    >
      {hasHeader() ? (
        <View
          style={{
            flexDirection: compact() ? "column" : "row",
            justifyContent: "space-between",
            alignItems: "flex-start",
            minWidth: 0,
            marginBottom: 2,
            gap: 8,
          }}
        >
          <View style={{ width: compact() ? undefined : 0, gap: 4, flexGrow: 1, minWidth: 0, flexShrink: 1 }}>
            {props.title ? (
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8, minWidth: 0 }}>
                <Text
                  style={{
                    color: theme().textPrimary,
                    fontSize: 15,
                    fontWeight: "semibold",
                    minWidth: 0,
                    flexShrink: 1,
                    overflow: "hidden",
                    textOverflow: "ellipsis",
                  }}
                >
                  {props.title}
                </Text>
                {props.badge ? <Badge label={props.badge} variant="accent" size="sm" /> : null}
              </View>
            ) : null}
            {props.description ? (
              <Text style={{ color: theme().textSecondary, fontSize: 13, minWidth: 0 }}>{props.description}</Text>
            ) : null}
          </View>
          {props.actions ? <View style={{ flexDirection: "row", gap: 8, flexShrink: 0 }}>{props.actions}</View> : null}
        </View>
      ) : null}
      {hasHeader() ? <Divider margin={0} /> : null}
      {props.children}
    </View>
  );
}

export interface SectionHeaderProps {
  readonly title: string;
  readonly description?: string;
  readonly tag?: string;
  readonly actions?: SolidChild;
}

export function SectionHeader(props: SectionHeaderProps) {
  const { theme, windowSizeStore } = useGallery();
  const windowSize = useWindowSize(windowSizeStore);
  const compact = createMemo(() => windowSize().width < 960);

  return (
    <View
      style={{
        flexDirection: compact() ? "column" : "row",
        minWidth: 0,
        flexShrink: 0,
        justifyContent: "space-between",
        alignItems: compact() ? "stretch" : "flex-end",
        marginBottom: 4,
        gap: 12,
      }}
    >
      <View style={{ width: compact() ? undefined : 0, gap: 6, flexGrow: 1, minWidth: 0, flexShrink: 1 }}>
        {props.tag ? (
          <View style={{ flexDirection: "row", alignItems: "center", gap: 6 }}>
            <Badge label={props.tag} variant="neutral" size="sm" />
          </View>
        ) : null}
        <Text style={{ color: theme().textPrimary, fontSize: 24, fontWeight: "semibold", minWidth: 0 }}>
          {props.title}
        </Text>
        {props.description ? (
          <Text style={{ color: theme().textSecondary, fontSize: 14, minWidth: 0 }}>{props.description}</Text>
        ) : null}
      </View>
      {props.actions ? <View style={{ flexDirection: "row", gap: 8, flexShrink: 0 }}>{props.actions}</View> : null}
    </View>
  );
}

export interface InputProps {
  readonly label?: string;
  readonly placeholder?: string;
  readonly value: string;
  readonly onInput?: (val: string) => void;
  readonly onChange?: (val: string) => void;
  readonly multiline?: boolean;
  readonly password?: boolean;
  readonly disabled?: boolean;
  readonly helperText?: string;
  readonly error?: string;
  readonly width?: number;
}

export function Input(props: InputProps) {
  const { theme } = useGallery();

  return (
    <View style={{ gap: 6, width: props.width, minWidth: 0, flexShrink: 1 }}>
      {props.label ? (
        <Text style={{ color: theme().textPrimary, fontSize: 12, fontWeight: "medium" }}>{props.label}</Text>
      ) : null}
      <TextInput
        style={{
          backgroundColor: theme().bgInput,
          borderWidth: 1,
          borderColor: props.error ? theme().danger : theme().border,
          borderRadius: 6,
          padding: 8,
          color: theme().textPrimary,
          minWidth: 0,
          flexShrink: 1,
          fontSize: 13,
          minHeight: props.multiline ? 72 : undefined,
        }}
        value={props.value}
        placeholder={props.placeholder}
        multiline={props.multiline}
        disabled={props.disabled}
        onChangeText={(text) => {
          props.onInput?.(text);
          props.onChange?.(text);
        }}
      />
      {props.error ? (
        <Text style={{ color: theme().danger, fontSize: 11 }}>{props.error}</Text>
      ) : props.helperText ? (
        <Text style={{ color: theme().textMuted, fontSize: 11 }}>{props.helperText}</Text>
      ) : null}
    </View>
  );
}

export interface SwitchProps {
  readonly checked: boolean;
  readonly onChange: (checked: boolean) => void;
  readonly label?: string;
  readonly disabled?: boolean;
}

export function Switch(props: SwitchProps) {
  const { theme } = useGallery();

  return (
    <Pressable
      style={{ flexDirection: "row", alignItems: "center", gap: 10 }}
      disabled={props.disabled}
      onPress={() => {
        if (!props.disabled) props.onChange(!props.checked);
      }}
    >
      <View
        style={{
          width: 38,
          height: 22,
          borderRadius: 11,
          backgroundColor: props.checked ? theme().accent : theme().bgHover,
          borderWidth: 1,
          borderColor: props.checked ? theme().accent : theme().border,
          padding: 2,
          justifyContent: "center",
          opacity: props.disabled ? 0.5 : 1,
        }}
      >
        <View
          style={{
            width: 16,
            height: 16,
            borderRadius: 8,
            backgroundColor: "#FFFFFF",
            marginLeft: props.checked ? 16 : 0,
          }}
        />
      </View>
      {props.label ? <Text style={{ color: theme().textPrimary, fontSize: 13 }}>{props.label}</Text> : null}
    </Pressable>
  );
}

export interface CheckboxProps {
  readonly checked: boolean;
  readonly onChange: (checked: boolean) => void;
  readonly label?: string;
  readonly disabled?: boolean;
}

export function Checkbox(props: CheckboxProps) {
  const { theme } = useGallery();

  return (
    <Pressable
      style={{ flexDirection: "row", alignItems: "center", gap: 8 }}
      disabled={props.disabled}
      onPress={() => {
        if (!props.disabled) props.onChange(!props.checked);
      }}
    >
      <View
        style={{
          width: 18,
          height: 18,
          borderRadius: 4,
          borderWidth: 1,
          borderColor: props.checked ? theme().accent : theme().border,
          backgroundColor: props.checked ? theme().accent : "#00000000",
          alignItems: "center",
          justifyContent: "center",
          opacity: props.disabled ? 0.5 : 1,
        }}
      >
        {props.checked ? <Icon name="lucide:check" size={11} color="#FFFFFF" /> : null}
      </View>
      {props.label ? <Text style={{ color: theme().textPrimary, fontSize: 13 }}>{props.label}</Text> : null}
    </Pressable>
  );
}

export interface DividerProps {
  readonly orientation?: "horizontal" | "vertical";
  readonly margin?: number;
}

export function Divider(props: DividerProps) {
  const { theme } = useGallery();
  const orientation = () => props.orientation ?? "horizontal";
  const m = () => props.margin ?? 12;

  return (
    <View
      style={
        orientation() === "vertical"
          ? {
              width: 1,
              backgroundColor: theme().borderMuted,
              marginLeft: m(),
              marginRight: m(),
              alignSelf: "stretch",
            }
          : {
              height: 1,
              backgroundColor: theme().borderMuted,
              marginTop: m(),
              marginBottom: m(),
              alignSelf: "stretch",
            }
      }
    />
  );
}

export interface CodeSnippetProps {
  readonly code: string;
  readonly language?: string;
}

export function CodeSnippet(props: CodeSnippetProps) {
  const { theme, showStatus, root } = useGallery();
  const [copied, setCopied] = createSignal(false);
  let copyTimer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;
  onCleanup(() => {
    disposed = true;
    clearTimeout(copyTimer);
  });

  const handleCopy = async () => {
    const r = root();
    if (!r) {
      showStatus("Clipboard unavailable: native surface is disconnected.", "warning");
      return;
    }
    try {
      await r.setClipboardText(props.code);
      if (disposed) return;
      setCopied(true);
      showStatus("Code copied to clipboard", "success");
      clearTimeout(copyTimer);
      copyTimer = setTimeout(() => setCopied(false), 2000);
    } catch (error) {
      showStatus(`Unable to copy code: ${String(error)}`, "warning");
    }
  };

  return (
    <View
      style={{
        minWidth: 0,
        flexShrink: 0,
        overflow: "scroll",
        backgroundColor: theme().codeBg,
        borderWidth: 1,
        borderColor: theme().border,
        borderRadius: 8,
        padding: 14,
        gap: 8,
      }}
    >
      <View style={{ flexDirection: "row", justifyContent: "space-between", alignItems: "center", minWidth: 0 }}>
        <Badge label={props.language ?? "TypeScript"} variant="neutral" size="sm" />
        <Pressable
          style={{
            padding: 4,
            backgroundColor: copied() ? theme().successMuted : theme().bgCard,
            borderRadius: 4,
            borderWidth: 1,
            borderColor: copied() ? theme().success : theme().border,
          }}
          onPress={handleCopy}
        >
          <Text
            style={{
              color: copied() ? theme().success : theme().textSecondary,
              fontSize: 11,
              fontWeight: "medium",
            }}
          >
            {copied() ? "Copied" : "Copy"}
          </Text>
        </Pressable>
      </View>
      <Text
        selectable
        style={{
          color: theme().codeText,
          fontSize: 12,
          fontFamily: "monospace",
          lineHeight: 18,
        }}
      >
        {props.code}
      </Text>
    </View>
  );
}

export interface PropItem {
  readonly name: string;
  readonly type: string;
  readonly default?: string;
  readonly description: string;
}

export interface PropTableProps {
  readonly props: readonly PropItem[];
}

export function PropTable(props: PropTableProps) {
  const { theme } = useGallery();

  return (
    <View
      style={{
        minWidth: 0,
        flexShrink: 0,
        backgroundColor: theme().bgCard,
        borderWidth: 1,
        borderColor: theme().border,
        borderRadius: 8,
        overflow: "scroll",
      }}
    >
      <View
        style={{ flexDirection: "row", backgroundColor: theme().bgHover, padding: 8, minWidth: 480, flexShrink: 0 }}
      >
        <Text style={{ width: 140, flexShrink: 0, color: theme().textMuted, fontSize: 11, fontWeight: "bold" }}>
          PROP
        </Text>
        <Text style={{ width: 160, flexShrink: 0, color: theme().textMuted, fontSize: 11, fontWeight: "bold" }}>
          TYPE
        </Text>
        <Text style={{ width: 100, flexShrink: 0, color: theme().textMuted, fontSize: 11, fontWeight: "bold" }}>
          DEFAULT
        </Text>
        <Text style={{ flexGrow: 1, color: theme().textMuted, fontSize: 11, fontWeight: "bold" }}>DESCRIPTION</Text>
      </View>
      {props.props.map((item) => (
        <>
          <Divider margin={0} />
          <View style={{ flexDirection: "row", padding: 10, alignItems: "center", minWidth: 480, flexShrink: 0 }}>
            <Text
              style={{
                width: 140,
                flexShrink: 0,
                color: theme().accent,
                fontSize: 12,
                fontFamily: "monospace",
                fontWeight: "semibold",
              }}
            >
              {item.name}
            </Text>
            <Text style={{ width: 160, flexShrink: 0, color: theme().warning, fontSize: 11, fontFamily: "monospace" }}>
              {item.type}
            </Text>
            <Text
              style={{ width: 100, flexShrink: 0, color: theme().textMuted, fontSize: 11, fontFamily: "monospace" }}
            >
              {item.default ?? "-"}
            </Text>
            <Text style={{ flexGrow: 1, color: theme().textSecondary, fontSize: 12 }}>{item.description}</Text>
          </View>
        </>
      ))}
    </View>
  );
}

export interface TabItem {
  readonly id: string;
  readonly label: string;
  readonly count?: number;
}

export interface TabsProps {
  readonly tabs: readonly TabItem[];
  readonly activeTab: string;
  readonly onChange: (id: string) => void;
}

export function Tabs(props: TabsProps) {
  const { theme } = useGallery();

  return (
    <View style={{ flexDirection: "row", gap: 6, padding: 4, minWidth: 0, flexShrink: 0, overflow: "scroll" }}>
      {props.tabs.map((tab) => {
        const isActive = () => props.activeTab === tab.id;
        return (
          <Pressable
            style={{
              padding: 8,
              flexShrink: 0,
              borderRadius: 6,
              backgroundColor: isActive() ? theme().bgActive : "#00000000",
              flexDirection: "row",
              alignItems: "center",
              gap: 6,
            }}
            onPress={() => props.onChange(tab.id)}
          >
            <Text
              style={{
                color: isActive() ? theme().accent : theme().textSecondary,
                fontSize: 13,
                fontWeight: isActive() ? "semibold" : "medium",
              }}
            >
              {tab.label}
            </Text>
            {tab.count !== undefined ? (
              <Badge label={String(tab.count)} variant={isActive() ? "accent" : "neutral"} size="sm" />
            ) : null}
          </Pressable>
        );
      })}
    </View>
  );
}
