# ztracing stub
This crate replaces Zed's `ztracing` package for the React GPUI host graph.
It removes GPL-licensed tracing packages from distributed binaries.
See `.scratch/license-audit/` for the blocker evidence and landing record.
The `instrument` attribute reproduces upstream's documented no-op semantics:
it discards its arguments and returns the annotated item unchanged.
See `.scratch/license-audit/spike-option3-stub.md` § “Upstream no-op semantics” for the citation.
The boundary is host-graph only; runtime tracing APIs require an extension.
See `.scratch/license-audit/spike-option3-stub.md`'s out-of-graph runtime table before extending this stub.
This replacement is independently authored and Apache-2.0 licensed.
Delete this patch and stub to reversibly restore the upstream dependency.
