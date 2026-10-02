import { StdioTransport } from "@solid-gpui/core/stdio";
import { mountReferenceStudio } from "./reference-studio.application";

const transport = new StdioTransport();
const app = mountReferenceStudio(transport);
await app.root.setTitle("Reference Studio — native acceptance");
await app.root.resize(1280, 720);
let closed = false;
const close = () => {
  if (closed) return;
  closed = true;
  app.dispose();
  transport.dispose();
};
process.once("SIGTERM", close);
process.once("SIGINT", close);
process.once("beforeExit", close);
