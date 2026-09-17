import { defineConfig } from "vite";
import { solidGpui, solidGpuiSource } from "@solid-gpui/vite";

export default defineConfig({
  root: import.meta.dirname,
  plugins: [solidGpuiSource({ root: import.meta.dirname, exclude: [] }), solidGpui({ target: "web" })],
});
