# Reqwest client dependency patch

Source: crates.io `gpui-pre-reqwest-client` 0.3.3. Original Apache-2.0 license
and upstream metadata are retained.

Response readers now enter the client Tokio runtime for each poll. GPUI polls
image responses from its own executor; reqwest creates per-read timeout timers
during body polling, which otherwise panics with `there is no reactor running`.
The guard lives only for one poll and never crosses an await.

Regression: `cargo test -p solid-gpui --lib image_http_client_performs_a_real_local_request`.
It runs the actual configured HTTP client outside Tokio against a local server.
