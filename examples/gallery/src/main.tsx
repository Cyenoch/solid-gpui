import { mountGallery } from "./application";
import { StdioTransport } from "@solid-gpui/core/stdio";
mountGallery(() => new StdioTransport());
