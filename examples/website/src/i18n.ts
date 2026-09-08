import { createSignal } from "@solid-gpui/core/runtime";
import { chinese } from "./locale.zh-CN";
import { saveLanguage } from "./site";
export const [locale, setLocale] = createSignal<"en" | "zh-CN">("en");
export function toggleLocale() {
  const next = locale() === "en" ? "zh-CN" : "en";
  setLocale(next);
  saveLanguage(next);
}
export function t(text: string): string {
  return locale() === "zh-CN" ? (chinese[text.trim().replace(/\s+/g, " ")] ?? text) : text;
}
export function translateChild(value: any): any {
  return typeof value === "string" ? t(value) : Array.isArray(value) ? value.map(translateChild) : value;
}
