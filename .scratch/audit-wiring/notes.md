# Advisory audit wiring

- `make audit` runs `bun audit` in `packages/react-gpui`, `bun audit` in `packages/react-gpui-dev`, then `cargo deny check advisories` from the repository root.
- Both Bun audits passed with Bun 1.4.0 and reported `No vulnerabilities found` (12 and 59 packages respectively).
- cargo-deny 0.20.2 reported `advisories ok` using the root `deny.toml`.
- `deny.toml` strictly fails all advisories except the three documented, currently unmaintained crates: `RUSTSEC-2024-0436` (`paste`, `.scratch/supply-chain/issues/01-paste.md`), `RUSTSEC-2026-0206` (`rustybuzz`, `.scratch/supply-chain/issues/02-rustybuzz.md`), and `RUSTSEC-2026-0192` (`ttf-parser`, `.scratch/supply-chain/issues/03-ttf-parser.md`). Each ignore is intended to be dropped when its issue closes; drop the ttf-parser ignore when the skrifa migration issue closes.
- If `cargo-deny` is not installed, `make audit` prints `cargo-deny not installed; skipping advisory scan` and continues. CI installs cargo-deny 0.20.2 explicitly before `make ci`.
- `make ci` includes `audit` after the existing Bun gates via the prerequisite ladder (`ci: rust-format rust-check bun-ci audit`).
