/// <reference types="vite/client" />
import { mountGallery } from "@solid-gpui/gallery/application";
import { StdioTransport } from "@solid-gpui/core/stdio";
mountGallery(() => new StdioTransport(), import.meta.hot ? import.meta.url : undefined);
