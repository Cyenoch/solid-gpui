# Project-owned MIT licensing

Date: 2026-09-07.

The maintainer requested a more permissive project license. Project-owned code
now uses MIT, replacing Apache-2.0 in the root license, Rust workspace metadata,
the two public npm packages, and the independently authored ztracing stub.
The copyright name follows the repository's recorded author, JGBingZi.

[MIT](https://opensource.org/license/mit) allows use, modification, distribution,
sublicensing, and sale subject to retaining its notice. Apache-2.0 was also a
permissive license; it has an explicit patent grant and redistribution conditions
that are not reproduced in MIT. See the
[Apache-2.0 text](https://www.apache.org/licenses/LICENSE-2.0).

This change applies to project-owned code. Dependencies, vendored upstream code,
adapted JavaScript notices, fonts, icons, and reference checkouts retain their
existing licenses. Previously distributed versions retain their original terms.
The dependency inventory is still an inventory rather than a complete bundle of
third-party license texts.

## Packaging and verification

- Each project-owned Rust crate now has a local MIT `LICENSE` for Cargo archives.
  Before this change, `cargo package -p solid-gpui-macros --list --allow-dirty --locked`
  contained no license text; it now includes `LICENSE`.
- Both npm tarballs were generated and inspected: their manifests declare MIT,
  and their packaged `LICENSE` exactly matches the root text.
- Cargo metadata reports MIT for all six workspace packages.
- The notices generator reads project SPDX identifiers from manifests instead of
  hardcoding Apache-2.0. The regenerated third-party package records are unchanged.
  See [generation log](mit-notices.log).
- Packed core/router consumer smoke passed, including runtime execution and type
  checks. See [log](mit-package-smoke.log).
- All three release-preparation tests passed, including rollback on failure.
  See [log](mit-release-tests.log).
- `git diff --check` passed.

No package or release was published during this change.
