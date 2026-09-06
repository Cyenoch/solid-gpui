import { Icon, Text, View } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import {
  Button as NativeButton,
  Checkbox as NativeCheckbox,
  Switch as NativeSwitch,
  Progress,
  Input as NativeInput,
} from "@solid-gpui/core/components";
import { Link } from "@solid-gpui/router";
import { Badge, Card, CodeSnippet, ResponsiveRow, SectionHeader } from "../components/ui";
import { useGallery } from "../context";
import { CATEGORIES, PAGES, pagePath } from "../types";
import { createClient, BuildBadge, type WorkspaceReport } from "../generated/native";

const EXAMPLE = `import { createClient } from "./generated/native";

// root is the connected Solid GPUI native surface.
// The generated client invokes the registered Rust module.
const report = await createClient(root).analyzeWorkspace({
  name: name(),
  readiness: ready(),
  builds: builds(),
});

// These values are computed in Rust, not duplicated in TypeScript.
setReport(report);`;

export function OverviewPage() {
  const { theme, root } = useGallery();
  const [name, setName] = createSignal("Untitled workspace");
  const [ready, setReady] = createSignal(true);
  const [preview, setPreview] = createSignal(true);
  const [builds, setBuilds] = createSignal(0);
  const [report, setReport] = createSignal<WorkspaceReport | null>(null);
  const [analyzing, setAnalyzing] = createSignal(false);
  const [analysisError, setAnalysisError] = createSignal<string | null>(null);

  const analyzeInRust = async () => {
    if (analyzing()) return;
    setAnalysisError(null);
    setReport(null);
    const surface = root();
    if (!surface) {
      setAnalysisError("Native surface is disconnected.");
      return;
    }
    setAnalyzing(true);
    try {
      setReport(
        await createClient(surface).analyzeWorkspace({
          name: name(),
          readiness: ready(),
          builds: builds(),
        }),
      );
    } catch (error) {
      setAnalysisError(error instanceof Error ? error.message : String(error));
    } finally {
      setAnalyzing(false);
    }
  };

  // These sections flow vertically; no flexible main-axis allocation is needed.
  // Nested flex columns multiply intrinsic layout work on every native scroll frame.
  // Use block flow and section margins while leaving text heights unconstrained.
  return (
    <View style={{ minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        tag="WORKBENCH"
        title="Make something native."
        description="Explore the primitives, tune the details, and try the real controls."
      />
      <Card
        style={{ marginTop: 24 }}
        title="Workspace playground"
        description="Generated gpui-component controls. Solid signals, native input, live results."
        badge="Interactive"
      >
        <ResponsiveRow breakpoint={1080} gap={24} grow={[1, 1]}>
          <View style={{ gap: 18, minWidth: 0, flexShrink: 0 }}>
            <View style={{ gap: 8 }}>
              <Text style={{ color: theme().textSecondary, fontSize: 12 }}>Workspace name</Text>
              <NativeInput
                value={name()}
                onChange={(event) => setName(event.value)}
                placeholder="Name your workspace"
              />
            </View>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0, flexShrink: 0 }}>
              <NativeCheckbox accessibilityLabel="Ready to build" checked={ready()} onChange={setReady} size="small" />
              <Text style={{ color: theme().textPrimary, fontSize: 13, minWidth: 0, flexShrink: 1 }}>
                Ready to build
              </Text>
            </View>
            <View style={{ flexDirection: "row", alignItems: "center", gap: 10, minWidth: 0, flexShrink: 0 }}>
              <NativeSwitch
                accessibilityLabel="Show live preview"
                checked={preview()}
                onChange={setPreview}
                size="small"
              />
              <Text style={{ color: theme().textPrimary, fontSize: 13, minWidth: 0, flexShrink: 1 }}>
                Show live preview
              </Text>
            </View>
            <ResponsiveRow breakpoint={720} gap={8} grow={[0, 0]}>
              <NativeButton
                label="Build workspace"
                variant="primary"
                size="small"
                disabled={!ready()}
                onPress={() => setBuilds((n) => n + 1)}
              />
              <NativeButton
                label="Reset"
                variant="default"
                size="small"
                onPress={() => {
                  setBuilds(0);
                  setName("Untitled workspace");
                  setReady(true);
                  setPreview(true);
                }}
              />
            </ResponsiveRow>
            <Text style={{ color: theme().textMuted, fontSize: 11, minWidth: 0 }}>
              Uncheck readiness to test disabled state. Every build updates the preview.
            </Text>
          </View>
          <View
            style={{
              minWidth: 0,
              flexShrink: 0,
              padding: 20,
              gap: 16,
              backgroundColor: theme().bgApp,
              borderWidth: 1,
              borderColor: theme().borderMuted,
              borderRadius: 8,
            }}
          >
            <View
              style={{
                flexDirection: "row",
                alignItems: "center",
                justifyContent: "space-between",
                gap: 8,
                minWidth: 0,
              }}
            >
              <Text style={{ color: theme().textMuted, fontSize: 10, fontWeight: "semibold" }}>LIVE OUTPUT</Text>
              <Badge label={preview() ? "Connected" : "Hidden"} variant={preview() ? "success" : "neutral"} size="sm" />
            </View>
            {preview() ? (
              <>
                <Icon name="lucide:folder-open" size={26} color={theme().accent} />
                <Text style={{ color: theme().textPrimary, fontSize: 19, fontWeight: "semibold", minWidth: 0 }}>
                  {name() || "Untitled workspace"}
                </Text>
                <Text style={{ color: theme().textSecondary, fontSize: 12 }}>
                  {builds() === 0
                    ? "Your workspace is ready for its first build."
                    : `${builds()} successful ${builds() === 1 ? "build" : "builds"}. State updated on the native surface.`}
                </Text>
                <BuildBadge builds={builds()} />
                <Progress value={Math.min(builds() * 20, 100)} style={{ height: 8, minWidth: 0 }} />
                <Text style={{ color: theme().textMuted, fontSize: 11 }}>
                  {Math.min(builds(), 5)} / 5 build milestones
                </Text>
              </>
            ) : (
              <Text style={{ color: theme().textSecondary, fontSize: 13 }}>
                Preview hidden. Your workspace state is retained.
              </Text>
            )}
          </View>
        </ResponsiveRow>
      </Card>
      <Card
        style={{ marginTop: 24 }}
        title="Run workspace analysis in Rust"
        description="Send the playground values through a generated native function client. Clear the name to inspect a real domain error."
        badge="Rust API"
      >
        <View style={{ alignItems: "flex-start", gap: 14, minWidth: 0, flexShrink: 0 }}>
          <NativeButton
            label={analyzing() ? "Analyzing…" : "Analyze in Rust"}
            variant="primary"
            size="small"
            disabled={analyzing()}
            onPress={analyzeInRust}
          />
          {analysisError() ? (
            <Text style={{ color: theme().danger, fontSize: 13, minWidth: 0 }}>{analysisError()}</Text>
          ) : null}
          {report() ? (
            <View style={{ gap: 10, minWidth: 0, flexShrink: 0, alignSelf: "stretch" }}>
              <Text style={{ color: theme().textMuted, fontSize: 11 }}>LAST RUST RESPONSE</Text>
              <Text style={{ color: theme().textPrimary, fontSize: 18, fontWeight: "semibold", minWidth: 0 }}>
                {report()!.normalizedName}
              </Text>
              <Text style={{ color: theme().textSecondary, fontSize: 12, minWidth: 0 }}>Slug: {report()!.slug}</Text>
              <Badge label={report()!.status} variant="accent" />
              <Progress value={report()!.progress.percent} style={{ height: 8, minWidth: 0 }} />
              <Text style={{ color: theme().textSecondary, fontSize: 12 }}>
                {report()!.progress.completed} / {report()!.progress.target} milestones · {report()!.progress.percent}%
              </Text>
              <Text style={{ color: theme().textSecondary, fontSize: 12 }}>
                Next milestone: {report()!.nextMilestone === null ? "None" : report()!.nextMilestone}
              </Text>
              {report()!.recommendations.map((recommendation) => (
                <Text style={{ color: theme().textPrimary, fontSize: 12, minWidth: 0 }}>{recommendation}</Text>
              ))}
            </View>
          ) : null}
        </View>
      </Card>
      <View style={{ marginTop: 24, minWidth: 0, flexShrink: 0 }}>
        <SectionHeader
          title="Explore the library"
          description="18 working showcases, from layout fundamentals to complete mini apps."
        />
        {CATEGORIES.filter((category) => category.id !== "getting-started").map((category) => (
          <Card title={category.title} description={category.description} style={{ marginTop: 14 }}>
            <View style={{ minWidth: 0, flexShrink: 0 }}>
              {PAGES.filter((page) => page.category === category.id).map((page, index) => (
                <Link
                  to={pagePath(page.id)}
                  style={{
                    marginTop: index === 0 ? 0 : 4,
                    padding: 10,
                    minWidth: 0,
                    flexShrink: 0,
                    cursor: "pointer",
                    borderRadius: 6,
                  }}
                >
                  <View style={{ flexDirection: "row", alignItems: "center", gap: 12, minWidth: 0 }}>
                    <Icon name={page.icon} size={16} color={theme().accent} />
                    <View style={{ gap: 3, flexGrow: 1, flexShrink: 1, minWidth: 0 }}>
                      <Text style={{ color: theme().textPrimary, fontSize: 13, fontWeight: "medium", minWidth: 0 }}>
                        {page.title}
                      </Text>
                      <Text style={{ color: theme().textMuted, fontSize: 12, minWidth: 0 }}>{page.description}</Text>
                    </View>
                    <Icon name="lucide:chevron-right" size={13} color={theme().textMuted} />
                  </View>
                </Link>
              ))}
            </View>
          </Card>
        ))}
      </View>
      <Card
        style={{ marginTop: 24 }}
        title="TypeScript calls, Rust computes"
        description="The generated function client used by the analysis action above."
      >
        <CodeSnippet language="TSX" code={EXAMPLE} />
      </Card>
      <Text style={{ marginTop: 24, color: theme().textMuted, fontSize: 11, minWidth: 0 }}>
        End of workbench · Choose a showcase in the navigation to keep exploring.
      </Text>
    </View>
  );
}
