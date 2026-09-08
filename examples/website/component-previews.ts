import { componentExamples } from "./component-examples";

/** Window chrome needs its own window, not a nested documentation card. */
export const previewNotes: Record<string, string> = {
  AppMenuBar:
    "AppMenuBar reflects application-wide menus configured through root.setMenus. Use it in desktop window chrome.",
  TitleBar: "TitleBar controls a desktop window. Use this composition at the root of your desktop app.",
  WindowBorder: "WindowBorder controls a desktop window. Use this composition at the root of your desktop app.",
  NativeMenu: "NativeMenu opens an operating-system menu. Try this example in the desktop app.",
};

export const browserPreviewNames = new Set(
  componentExamples.flatMap((example) => example.names).filter((name) => !previewNotes[name]),
);
