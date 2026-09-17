// Reached only through `import()` from fixtures/embedded-static-graph.tsx, so the
// packaged module graph has to carry it as a module of its own: no static import
// anywhere else in the graph mentions it.
//
// The application can only report this value if the import resolved inside the
// executable, because the string exists nowhere else.

export const dynamicMarker = "dynamic-module-ok";
