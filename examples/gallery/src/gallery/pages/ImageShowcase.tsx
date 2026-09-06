import { Image, Text, View, type ImageObjectFit } from "@solid-gpui/core";
import { createSignal } from "@solid-gpui/core/runtime";
import type { SolidChild } from "../types";
import { useGallery } from "../context";
import { Button, Card, DenseRow, Divider, PropTable, SectionHeader } from "../components/ui";

export function ImageShowcase(): SolidChild {
  const { theme, showStatus } = useGallery();

  const [objectFit, setObjectFit] = createSignal<ImageObjectFit>("cover");
  const [borderRadius, setBorderRadius] = createSignal(8);
  const [imageWidth, setImageWidth] = createSignal(280);
  const [imageHeight, setImageHeight] = createSignal(180);

  const sampleUrl = "https://images.unsplash.com/photo-1579546929518-9e396f3cc809?w=600&auto=format&fit=crop&q=80";
  const fallbackUrl = "https://images.unsplash.com/photo-1557683316-973673baf926?w=600&auto=format&fit=crop&q=80";

  return (
    <View style={{ gap: 24, minWidth: 0, flexShrink: 0 }}>
      <SectionHeader
        title="Image Component"
        description="Image renders native raster assets and remote image URLs with customizable object-fit scaling (contain, cover, fill, scaleDown, none) and automatic fallback sources."
        tag="Host Component"
      />

      {/* Interactive ObjectFit Playground */}
      <Card
        title="Image ObjectFit & Scaling Playground"
        description="Adjust scaling modes and dimensions to see how the image adapts to its viewport."
      >
        <View style={{ gap: 16 }}>
          {/* Controls bar */}
          <View
            style={{
              backgroundColor: theme().bgActive,
              padding: 12,
              borderRadius: 8,
              gap: 12,
            }}
          >
            {/* ObjectFit buttons */}
            <DenseRow gap={10} style={{ alignItems: "center" }}>
              <Text style={{ color: theme().textSecondary, fontSize: 13, fontWeight: "medium" }}>Object Fit:</Text>
              {(["contain", "cover", "fill", "scaleDown", "none"] as const).map((fit) => (
                <Button
                  variant={objectFit() === fit ? "primary" : "ghost"}
                  size="sm"
                  onPress={() => {
                    setObjectFit(fit);
                    showStatus(`Image objectFit set to: ${fit}`);
                  }}
                >
                  {fit}
                </Button>
              ))}
            </DenseRow>

            {/* Dimension controls */}
            <DenseRow gap={16} style={{ alignItems: "center" }}>
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Width: {imageWidth()}px</Text>
                <Button size="sm" variant="secondary" onPress={() => setImageWidth((w) => Math.max(120, w - 40))}>
                  -
                </Button>
                <Button size="sm" variant="secondary" onPress={() => setImageWidth((w) => Math.min(480, w + 40))}>
                  +
                </Button>
              </View>
              <Divider orientation="vertical" margin={4} />
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Height: {imageHeight()}px</Text>
                <Button size="sm" variant="secondary" onPress={() => setImageHeight((h) => Math.max(100, h - 20))}>
                  -
                </Button>
                <Button size="sm" variant="secondary" onPress={() => setImageHeight((h) => Math.min(300, h + 20))}>
                  +
                </Button>
              </View>
              <Divider orientation="vertical" margin={4} />
              <View style={{ flexDirection: "row", alignItems: "center", gap: 8 }}>
                <Text style={{ color: theme().textSecondary, fontSize: 13 }}>Radius: {borderRadius()}px</Text>
                <Button size="sm" variant="secondary" onPress={() => setBorderRadius((r) => Math.max(0, r - 4))}>
                  -
                </Button>
                <Button size="sm" variant="secondary" onPress={() => setBorderRadius((r) => Math.min(32, r + 4))}>
                  +
                </Button>
              </View>
            </DenseRow>
          </View>

          {/* Image Preview Frame */}
          <View
            style={{
              backgroundColor: theme().bgMuted,
              borderRadius: 8,
              borderWidth: 1,
              borderColor: theme().border,
              minHeight: 240,
              minWidth: 0,
              flexShrink: 0,
              overflow: "scroll",
            }}
          >
            <View
              style={{
                alignSelf: "stretch",
                minWidth: imageWidth() + 48,
                padding: 24,
                alignItems: "center",
                justifyContent: "center",
              }}
            >
              <Image
                source={sampleUrl}
                fallbackSource={fallbackUrl}
                objectFit={objectFit()}
                style={{
                  width: imageWidth(),
                  height: imageHeight(),
                  flexShrink: 0,
                  borderRadius: borderRadius(),
                  borderWidth: 1,
                  borderColor: theme().accent,
                  backgroundColor: "#1E293B",
                }}
              />
            </View>
          </View>
        </View>
      </Card>

      {/* Props Reference */}
      <Card title="Image Props Reference" description="All supported properties on the native Image component.">
        <PropTable
          props={[
            {
              name: "source",
              type: "string",
              default: "required",
              description: "Image URI (file://, http://, https://, data:)",
            },
            {
              name: "fallbackSource",
              type: "string",
              default: "undefined",
              description: "Fallback URI to display if primary source fails to load",
            },
            {
              name: "objectFit",
              type: "'contain'|'cover'|'fill'|'scaleDown'|'none'",
              default: "'cover'",
              description: "Image scaling & fitting behavior within container bounds",
            },
            {
              name: "style.width",
              type: "number",
              default: "undefined",
              description: "Width of image container in pixels",
            },
            {
              name: "style.height",
              type: "number",
              default: "undefined",
              description: "Height of image container in pixels",
            },
            {
              name: "style.borderRadius",
              type: "number",
              default: "0",
              description: "Corner radius for clipping the image",
            },
            { name: "style.borderWidth", type: "number", default: "0", description: "Border stroke width" },
            { name: "style.borderColor", type: "string", default: "undefined", description: "Border stroke color" },
          ]}
        />
      </Card>
    </View>
  );
}
