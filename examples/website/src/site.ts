import { createSignal } from "@solid-gpui/core/runtime";
export type SiteState = { route: string; width: number; height: number; language: "en" | "zh-CN" };
export interface SiteServices {
  copyText(text: string): Promise<void>;
  openUrl(url: string): Promise<void>;
  saveLanguage(language: "en" | "zh-CN"): void;
}
export const [docsQuery, setDocsQuery] = createSignal("");
export const headerHeight = () => (size().width < 500 ? 105 : 65);
export const [size, setSize] = createSignal({ width: 1280, height: 800 });
export const [sourceDocument, setSourceDocument] = createSignal<string>();
export const [siteError, setSiteError] = createSignal("");
let services: SiteServices;
export function configureSite(value: SiteServices) {
  services = value;
}
export function copyText(text: string) {
  return services.copyText(text);
}
export function openUrl(url: string) {
  void services.openUrl(url).catch((error) => setSiteError(String(error)));
}
export function saveLanguage(language: "en" | "zh-CN") {
  services.saveLanguage(language);
}
