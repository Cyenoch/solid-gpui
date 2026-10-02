# Standalone application and stock host delivery

Status: ready-for-agent
Blocked by: none

Implement spec outcomes 1 (release protocol checks) and 2. Provide an exact-version scaffold that works outside the SDK workspace, optional verified prebuilt stock-host acquisition/selection, and generic application packaging/extraction checks. Follow existing artifact authority and native export identity. Cover stock TS app and custom Rust app without manual SDK knowledge duplication, no install hooks or source fallback, no moving main/latest inputs. Keep Bun/QuickJS runtime choice explicit. Add version-paired release packaging workflow/source artifact support as necessary; do not publish. Own Vite CLI/project/artifact/tooling and packaging changes; compiler subpath handled by ticket 10. Synchronize getting-started/distribution related guides/translations/examples. Derive host release assertions from canonical protocol metadata, not v5 literals. Run clean-consumer real native checks where available and record unqualified targets honestly.
