import { StdioTransport } from "../packages/solid-gpui/dist/stdio.js";
import { mountCounter } from "./press-counter";
mountCounter(new StdioTransport());
