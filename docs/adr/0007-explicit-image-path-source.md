# ADR-0007: Keep Image sources as explicit host-resolved paths

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

An Image host node crosses from JavaScript into a GPUI process and must remain
bounded, inspectable, and independent from the renderer's runtime memory. The
host already owns image loading and GPUI's image cache. The protocol currently
needs only a source path and object-fit policy, while package examples and
release bundles must decide how to ship the referenced asset.

## Decision

Image uses a validated string path on the wire. The host converts it to a
`PathBuf` and lets GPUI load the resource; relative paths resolve from the host
process working directory, while production examples should derive an
absolute path and ship the asset explicitly. Image nodes do not carry child
content or inline image bytes. A missing or undecodable image renders as
silent blank space in this protocol version and does not send a payload or
JavaScript error event.

## Alternatives rejected

- **Embed bytes or data URLs in the Commit Batch:** rejected because it adds
  payload size/privacy pressure to an otherwise metadata-sized tree commit,
  duplicates GPUI's image cache inputs, and creates a second decoding path.
- **Use an implicit renderer-runtime asset resolver:** rejected because the
  host owns the window and image cache; a renderer cwd/module resolver would
  make ProcessAdapter and EmbeddedBunAdapter resolve different resources.
- **Add remote URL fetching to this host path:** rejected because networking,
  redirects, trust policy, and caching are a separate capability boundary, not
  a safe default for a native host resource.
- **Fail the whole surface on image load failure:** rejected because a missing
  decorative asset should not discard an otherwise valid retained tree; the
  current contract intentionally contains the failure as blank output.

## Consequences

- Examples and packaged applications must ship image files and account for the
  host working directory; ESM/Bun examples can use `new URL(..., import.meta.url)`
  to derive an absolute path.
- Image failures are not observable through JavaScript today, so applications
  that need fallback UI must validate assets before rendering or provide a
  non-image fallback alongside the node.
- The wire stays compact and deterministic, and GPUI remains the single image
  loading/cache owner.
