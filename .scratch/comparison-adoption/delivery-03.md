# Ticket 03: native contract and build identity

Branch: `adopt/03-contract-identity`. Baseline integration ancestry: `e879b662` confirmed with `git merge-base --is-ancestor e879b66 HEAD` before implementation. No remote publication or integration-branch mutation is authorized or performed here.

## Interface

- `ModuleDefinition::new(name, "1.0.0", components, commands)` requires an explicit behavioral semantic version. `#[native_module(name = "namespace", version = "1.0.0")]` generates this registration. Missing/malformed versions fail explicitly.
- `digest()` and generated `catalogDigest`/`moduleDigest` identify the canonical exported DTO declarations, props, events, commands, slots, child/composition metadata, controlled metadata and module/component semantic versions. Source text is excluded; DTO documentation is retained in TypeScript but excluded from contract identity.
- `build_digest()` locks canonical contract digest, SDK version, normalized SDK source/dependency provenance and selected implementation sources. SDK provenance covers SDK/macros/iconify source, manifests, Cargo.lock, vendored Rust/manifests and the producer. Paths/lengths are framed, sources normalize CRLF to LF. SDK checkout completeness is required explicitly; no weaker alternate provenance path.
- `with_implementation(source)` replaces `with_contract(source)` without a historical wrapper. Repeated calls add source provenance. Annotated modules automatically include their complete selected source file (including comments/helpers) plus package version and macro-selected tokens. Application implementation dependencies outside that file must be included explicitly; this portable source-build lock is not a whole executable, toolchain, signing, or complete application dependency hash.
- `with_semantic_version("major.minor.patch")` changes observable behavioral contract independently of type changes; components may use it as well as modules. Versions are exact identity, not a permissive compatibility range.
- `contract()`, `build_identity()`, generated `nativeIdentity`/numbered exports expose inspectable identity metadata. Generated metadata uses digest summaries rather than repeating the entire catalog in every export.
- Strict props/invocation format is `53 47 4e 02` + 32-byte build digest + strict JSON DTO, carried in existing Bebop byte fields. `encode_native_request(build_digest, &dto)` and JS `encodeNativeRequest` own construction. Budget includes all 36 overhead bytes. Missing envelopes and stale builds fail before DTO admission/publication/command execution; results/events remain JSON.
- Native component build errors use `ExtensionError::BuildMismatch`; canonical contract lookup retains `ContractMismatch`. Worker, foreground and retained-view command admission all check build identity. `validate_build` exposes a read-only build check for artifact qualification; website extracted-bundle verification now uses it.
- TestHost `nativeProps` reads the format-2 payload through `decodeNativeRequest`; semantic testing does not certify host build identity. Native call `args` remain the full observed bytes. The Vite type-resolution fixture is now explicitly application-owned (`src/native-fixture.ts`) instead of mislabeled generated Rust output.

## Mechanical integration for tickets 06/07/08 and other native additions

1. Change every new native-module annotation to include `version = "1.0.0"` (choose a deliberate behavioral version).
2. Change direct construction from `ModuleDefinition::new(name, components, commands)` to `ModuleDefinition::new(name, "1.0.0", components, commands)`.
3. Rename new `with_contract(include_str!(...))` calls to `with_implementation(include_str!(...))`; include helper/service implementation sources explicitly when they are outside an annotated module's source file. No deprecated alias exists.
4. Direct Rust props/command fixtures must use `encode_native_request(module.build_digest(), &dto)`, including instance-method arguments. Test events and native results continue using `encode_json`.
5. Application-owned TS descriptor fixtures require `buildDigest` and `semanticVersion`; production descriptors come only from the actual host exporter.
6. Keep composition through `include`/`with_component`; both refresh canonical and build identity and preserve included behavioral versions/source provenance. No manual digest or entry bookkeeping.
7. Regenerate combined SDK, website and desktop-app bindings from the final selected sources once all native additions are merged. Existing generated files in this branch were produced by their actual hosts for focused qualification; do not resolve generated-file merge conflicts manually.

No ticket04 canonical wire/schema delta is required. Envelope format belongs to the generated NativeModule adapter seam, inside existing bounded bytes. Stock-host/package pairing in ticket02 must check both canonical and build identity; a source-only candidate differs from the old published build even though the package version remains `0.5.2`. This task does not advertise a new release or downloaded prebuilt artifact.

## Validation and generation checklist

Focused checks performed before integration merge:

- TDD first slice failed on missing build/identity APIs, then passed after implementation.
- `cargo test -p solid-gpui --no-default-features --test native_bridge_contracts --test native_executor`: 12 contract tests + 2 executor tests pass. Covers source comment edits, LF/CRLF, wrong-build and missing-envelope commands, wrong-build component props, DTO signature change, slots/events and module/component behavioral version changes, retained DTO documentation, strict JSON and executor ownership.
- `cargo test -p solid-gpui-macros`: 3 macro tests + 1 compiled authoring test pass.
- `cargo test -p solid-gpui --features gpui-component --lib native_call_tests`: 2 native foreground/worker call tests pass.
- `cargo test -p solid-gpui --features gpui-component --lib native::children_tests`: retained typed children/slot lifecycle case passes.
- Native JS tests: 8 pass. TestHost semantic suite: 7 pass with browser + source conditions. Website content/runtime/example suites: 7 pass; combined native/website run reports 15 passing.
- Core and Vite test-app typechecks pass. The touched fixture enables `.ts` import extensions consistent with its source-consuming Vite imports.
- SDK + website bindings regenerated with `bun scripts/native-codegen.ts`; desktop bindings with `--manifest Cargo.toml --package desktop-app-host --out examples/desktop-app/src/native.ts`. Native generator `--check` passed before the final integration merge.
- `git diff --check` passes. Rust/TS touched sources formatted with the pinned tools.

All Cargo checks use `CARGO_BUILD_JOBS=2` and the requested shared `CARGO_TARGET_DIR`. A shared-target stale macro artifact was observed and resolved by rebuilding the changed macro producer; no source fallback was introduced.

Documentation synchronized: CONTEXT, ADR0016, Rust integration, protocol native-adapter envelope, distribution, CI, troubleshooting, explicit Chinese guide copies, and website README statements. Website loads those authoritative guide files directly. Maintained prose describes our own interfaces and behavior; comparative learning prose is removed from the touched domain entry. Historical research is left to parent relocation.

Qualification limits: no full workspace tests, full website/WASM build, physical GPU/IME acceptance, extracted release packaging or cross-platform release runs in this owner task. The extracted-bundle code path now checks build provenance but its full production rehearsal remains an integration check. Linux/Windows builds were not run. SDK version is still the already published `0.5.2`; source-build identity differentiates this local candidate.

Before final integrated delivery: re-run combined generation + `--check`, focused typechecks/website content tests, required protocol checks, final full suite and package qualification in the parent session. Regenerate only after all source changes and format passes, since source provenance intentionally tracks those changes.
