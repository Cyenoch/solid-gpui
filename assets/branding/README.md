# Solid GPUI brand assets

`solid-gpui.png` is the approved, original 720 × 720 project icon. Preserve its
blue gradient, letterform, proportions, and dark background. Use this artwork
for Solid GPUI branding in documentation, websites, and application packages.

Generate the checked-in sizes and platform formats from the original:

```sh
uv run scripts/generate-brand-assets.py
```

Run from the repository root. The script pins Pillow through its inline dependency
metadata; normal website and application builds use the generated files directly.
Browser shell images are written to `examples/website/public/brand` and referenced
as `/brand/icon-*.png`; Vite applies its base path in development and Pages builds.
Other formats live beside the original in this directory.

| Asset                                          | Use                                                      |
| ---------------------------------------------- | -------------------------------------------------------- |
| `solid-gpui.png`                               | Canonical artwork and repository README                  |
| `public/brand/icon-32.png`                     | Browser favicon                                          |
| `icon-64.png`                                  | Shared Web and native navigation, embedded as a data URL |
| `public/brand/icon-180.png`                    | Browser startup screen and Apple touch icon              |
| `icon-128.png`, `icon-256.png`, `icon-512.png` | Linux desktop icon theme                                 |
| `solid-gpui.icns`                              | macOS application bundle                                 |
| `solid-gpui.ico`                               | Windows executable icon resource                         |

The original is copied without modification. Only derived formats are resized;
ICNS includes the platform's required sizes. Functional UI icons and custom-icon
fixtures remain separate from the project mark.
