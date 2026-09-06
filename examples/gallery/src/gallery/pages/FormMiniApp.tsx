import { Text, TextInput, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { useGallery } from "../context";
import {
  Button,
  Card,
  Checkbox,
  CodeSnippet,
  DenseRow,
  Divider,
  Input,
  ResponsiveRow,
  SectionHeader,
  Switch,
} from "../components/ui";

export function FormMiniApp(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [username, setUsername] = createSignal("alex_dev");
  const [email, setEmail] = createSignal("alex@example.com");
  const [role, setRole] = createSignal<"developer" | "designer" | "manager">("developer");
  const [newsletter, setNewsletter] = createSignal(true);
  const [twoFactor, setTwoFactor] = createSignal(false);
  const [bio, setBio] = createSignal("Full-stack engineer passionate about native desktop UIs.");
  const [submittedJson, setSubmittedJson] = createSignal<string | null>(null);

  // Validation logic
  const emailError = () => {
    const val = email();
    if (!val) return "Email is required";
    if (!val.includes("@") || !val.includes(".")) return "Invalid email format";
    return undefined;
  };

  const usernameError = () => {
    const val = username();
    if (!val) return "Username is required";
    if (val.length < 3) return "Username must be at least 3 characters";
    return undefined;
  };

  const handleSubmit = () => {
    if (emailError() || usernameError()) {
      showStatus("Please fix form errors before submitting");
      return;
    }

    const payload = {
      username: username(),
      email: email(),
      role: role(),
      newsletter: newsletter(),
      twoFactorEnabled: twoFactor(),
      bio: bio(),
      submittedAt: new Date().toISOString(),
    };

    setSubmittedJson(JSON.stringify(payload, null, 2));
    showStatus("Form submitted");
  };

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Form Builder & Validation"
        description="Form validation, checkboxes, switches, radio groups, multi-line text areas, and JSON output."
        tag="Mini App"
      />

      <ResponsiveRow breakpoint={1200} gap={20} grow={[3, 2]}>
        {/* Form Left Column */}
        <View style={{ flexGrow: 3, minWidth: 0, flexShrink: 1 }}>
          <Card
            title="User Profile & Account Settings"
            description="Fill out the fields below to trigger real-time reactive validation."
          >
            <View style={{ gap: 16 }}>
              {/* Username & Email */}
              <ResponsiveRow breakpoint={1600} gap={14}>
                <View style={{ flexGrow: 1, minWidth: 0, flexShrink: 1 }}>
                  <Input
                    label="Username"
                    placeholder="Enter username..."
                    value={username()}
                    onChange={(v) => setUsername(v)}
                    error={usernameError()}
                  />
                </View>
                <View style={{ flexGrow: 1, minWidth: 0, flexShrink: 1 }}>
                  <Input
                    label="Email Address"
                    placeholder="name@domain.com"
                    value={email()}
                    onChange={(v) => setEmail(v)}
                    error={emailError()}
                  />
                </View>
              </ResponsiveRow>

              {/* Role Selection (Radio Group) */}
              <View style={{ gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>Primary Role:</Text>
                <DenseRow gap={10}>
                  {(["developer", "designer", "manager"] as const).map((r) => (
                    <Button variant={role() === r ? "primary" : "secondary"} size="sm" onPress={() => setRole(r)}>
                      {r.charAt(0).toUpperCase() + r.slice(1)}
                    </Button>
                  ))}
                </DenseRow>
              </View>

              <Divider margin={4} />

              {/* Toggles: Switch & Checkbox */}
              <ResponsiveRow
                breakpoint={1800}
                gap={12}
                style={{ justifyContent: "space-between", alignItems: "flex-start" }}
              >
                <Switch label="Two-Factor Authentication" checked={twoFactor()} onChange={(val) => setTwoFactor(val)} />
                <Checkbox
                  label="Subscribe to Product Newsletter"
                  checked={newsletter()}
                  onChange={(val) => setNewsletter(val)}
                />
              </ResponsiveRow>

              <Divider margin={4} />

              {/* Bio TextArea */}
              <View style={{ gap: 6 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>
                  User Bio / Notes
                </Text>
                <TextInput
                  multiline
                  value={bio()}
                  onChangeText={(v) => setBio(v)}
                  style={{
                    backgroundColor: theme().bgInput,
                    borderWidth: 1,
                    borderColor: theme().border,
                    borderRadius: 6,
                    padding: 10,
                    color: theme().textPrimary,
                    fontSize: 13,
                    lineHeight: 18,
                    minHeight: 70,
                    minWidth: 0,
                    flexShrink: 1,
                  }}
                />
              </View>

              {/* Submit Button Row */}
              <View style={{ flexDirection: "row", justifyContent: "flex-end", gap: 10, marginTop: 8 }}>
                <Button variant="primary" onPress={handleSubmit}>
                  Save & Submit Profile
                </Button>
              </View>
            </View>
          </Card>
        </View>

        {/* Form Right Column: Live JSON Payload Preview */}
        <View style={{ flexGrow: 2, minWidth: 0, flexShrink: 1 }}>
          <Card title="Submitted Output" description="Real-time serialized JSON representation of the form data.">
            <CodeSnippet
              code={
                submittedJson() ??
                `{
  "status": "Ready to submit",
  "hint": "Click 'Save & Submit Profile' to view payload."
}`
              }
            />
          </Card>
        </View>
      </ResponsiveRow>
    </View>
  );
}
