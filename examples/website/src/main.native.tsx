import { StdioTransport } from "@solid-gpui/core/stdio";
import { mountWebsite } from "./application.native";
mountWebsite(() => new StdioTransport(), import.meta.hot ? import.meta.url : undefined);
