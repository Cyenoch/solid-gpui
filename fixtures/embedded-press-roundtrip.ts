import { EmbeddedTransport } from "../packages/solid-gpui/dist/embedded.js";
import { mountCounter } from "./press-counter";
mountCounter(new EmbeddedTransport());
