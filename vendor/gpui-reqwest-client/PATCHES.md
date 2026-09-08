# Reqwest client dependency patch

Source: crates.io `gpui-pre-reqwest-client` 0.3.3. Original Apache-2.0 license
and upstream metadata are retained.

The adapter uses upstream `reqwest` 0.13.4 instead of the old `gpui-pre-reqwest`
fork, removing its unmaintained `rustls-pemfile` dependency (RUSTSEC-2025-0134).
Upstream Reqwest configures redirects per client, so the adapter reuses a bounded
set of connection pools keyed by redirect limit. Every pool shares the configured
proxy, user agent, TLS verifier, and read timeout. Requests without an explicit
policy keep the default limit of 10; `FollowAll` keeps its limit of 100.
Unsupported proxy schemes fail during construction instead of silently bypassing
the proxy. The unused conversion from a prebuilt Reqwest client was removed so
all pooled clients have reproducible configuration.

Response readers now enter the client Tokio runtime for each poll. GPUI polls
image responses from its own executor; reqwest creates per-read timeout timers
during body polling, which otherwise panics with `there is no reactor running`.
The guard lives only for one poll and never crosses an await.

Regression: `cargo test -p solid-gpui --lib image_http_client_performs_a_real_local_request`.
It runs the actual configured HTTP client outside Tokio against a local server.

`cargo test -p gpui-pre-reqwest-client --lib --locked` also checks mixed redirect
policies and configured headers against a local server, streamed uploads across
pending reads, response-body timeouts, and proxy configuration. Both macOS CI and
Linux host checks run these adapter tests explicitly because this patched
dependency is outside the root workspace.
