/** The startup shell must remain usable without the application or WASM runtime. */
export function isUnsupportedBrowser(error: unknown): boolean {
  return String(error).includes("No browser graphics backend could be initialized.");
}

export function showStartupError(error: unknown, unsupported = isUnsupportedBrowser(error)) {
  const status = document.getElementById("status") ?? document.body.appendChild(document.createElement("div"));
  status.id = "status";
  status.lang = "en";
  const template = document.getElementById("startup-error") as HTMLTemplateElement;
  status.replaceChildren(template.content.cloneNode(true));
  status.setAttribute("role", "alert");
  status.querySelector("h1")!.textContent = unsupported ? "Browser not supported" : "Something interrupted startup";
  status.querySelector("[data-description]")!.textContent = unsupported
    ? "This experience needs WebAssembly and working WebGPU or WebGL2 graphics. They aren’t available in this browser right now."
    : "Solid GPUI couldn’t finish loading. Try again, or explore the documentation while we’re away.";
  status.querySelector("[data-guidance]")!.textContent = unsupported
    ? "Update your browser and enable graphics acceleration, or open this page in another browser."
    : "Check your connection and reload the page. If the problem continues, the technical details below may help.";
  status.querySelector("[data-details]")!.textContent = String(error);
  status.querySelector("button")!.addEventListener("click", () => location.reload());
  console.error(error);
}
