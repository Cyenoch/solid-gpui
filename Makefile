.PHONY: ci rust-format rust-check bun-install bun-format bun-typecheck bun-test bun-build bun-pack-smoke protocol-golden-generate embedded-bun host-release-bundle host-release-check host-candidate-smoke host-embedded-candidate-smoke release-prep

ci: rust-format rust-check bun-ci

rust-format:
	cargo fmt --all -- --check

rust-check:
	cargo check --workspace --locked
	cargo clippy --workspace --all-targets --locked -- -D warnings
	cargo test --workspace --locked

bun-install:
	cd packages/react-gpui && bun install --frozen-lockfile
	cd packages/react-gpui-dev && bun install --frozen-lockfile

bun-format: bun-install
	cd packages/react-gpui && bun run format
	cd packages/react-gpui-dev && bun run format

bun-typecheck: bun-install
	cd packages/react-gpui && bun run typecheck
	cd packages/react-gpui-dev && bun run typecheck

bun-test: bun-install
	cd packages/react-gpui && bun run test
	cd packages/react-gpui-dev && bun run test

bun-build: bun-install
	cd packages/react-gpui && bun run build
	cd packages/react-gpui-dev && bun run build

bun-pack-smoke: bun-build
	bash scripts/package-pack-smoke.sh

protocol-golden-generate:
	mkdir -p fixtures/protocol
	cargo run -p react-gpui --example protocol_golden --locked -- fixtures/protocol/rust_to_ts.hex
	bun scripts/protocol-golden.ts fixtures/protocol

bun-ci: bun-format bun-typecheck bun-test bun-pack-smoke

embedded-bun:
	cargo check -p react-gpui-host --features embedded-bun --locked
	cargo test -p react-gpui-bun --features embedded-bun --locked

host-release-bundle:
	bash scripts/host-release.sh bundle

host-release-check:
	bash scripts/host-release.sh check

host-candidate-smoke:
	bash scripts/host-candidate-smoke.sh

host-embedded-candidate-smoke:
	bash scripts/host-embedded-candidate-smoke.sh

release-prep:
	bash scripts/release-prep.sh "$(VERSION)"
