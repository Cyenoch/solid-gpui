import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { mountWebsite } from "./application.native";
mountWebsite(() => new EmbeddedTransport());
