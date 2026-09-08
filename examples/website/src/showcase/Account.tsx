import { View } from "@solid-gpui/core";
import * as N from "@solid-gpui/core/components";
import { createSignal } from "@solid-gpui/core/runtime";
import { Copy, colors } from "../ui";
import { t } from "../i18n";
const card = {
  padding: 24,
  gap: 20,
  borderWidth: 1,
  borderColor: colors.line,
  borderRadius: 12,
  backgroundColor: colors.panel,
  minWidth: 0,
  flexShrink: 0,
} as const;
export function createAccountState() {
  const [name, setName] = createSignal("Alex Morgan");
  const [email, setEmail] = createSignal("alex@example.com");
  const [updates, setUpdates] = createSignal(true);
  const [saved, setSaved] = createSignal(false);
  return { name, setName, email, setEmail, updates, setUpdates, saved, setSaved };
}
export function Account(props: { state: ReturnType<typeof createAccountState> }) {
  const { name, setName, email, setEmail, updates, setUpdates, saved, setSaved } = props.state;
  return (
    <View style={card}>
      <Copy size={20}>Make it yours.</Copy>
      <Copy color={colors.muted}>A few details for a more personal workspace.</Copy>
      <Copy size={13}>Display name</Copy>
      <N.Input
        value={name()}
        onChange={(event) => {
          setName(event.value);
          setSaved(false);
        }}
      />
      <Copy size={13}>Email address</Copy>
      <N.Input
        value={email()}
        onChange={(event) => {
          setEmail(event.value);
          setSaved(false);
        }}
      />
      <N.Separator />
      <N.Switch
        label={t("Product updates")}
        checked={updates()}
        onChange={(value) => {
          setUpdates(value);
          setSaved(false);
        }}
      />
      <N.Button
        label={t("Save preferences")}
        variant="primary"
        disabled={!name().trim() || !email().includes("@")}
        onPress={() => setSaved(true)}
      />
      {() =>
        saved() ? (
          <N.Alert title={t("Preferences saved")} message={t("Your changes are saved for this session.")} />
        ) : (
          <Copy size={13} color={colors.muted}>
            Changes stay in this demo session.
          </Copy>
        )
      }
    </View>
  );
}
