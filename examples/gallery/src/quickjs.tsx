import { EmbeddedTransport } from "@solid-gpui/core/embedded";
import { mountGallery } from "./application";

mountGallery(() => new EmbeddedTransport());
