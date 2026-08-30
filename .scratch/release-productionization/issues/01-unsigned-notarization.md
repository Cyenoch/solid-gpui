# Candidate artifact is unsigned and not notarized

Status: needs-triage
Type: task

The host candidate archive and the manually uploaded workflow artifact are
unsigned and have no notarization or publisher-authentication step. The archive
contains per-file SHA-256 checksums only; those checksums verify extracted-file
consistency and do not authenticate the publisher.

Evidence: `README.md` Host release candidate section and
`.github/workflows/host-release-candidate.yml`.

## Comments

The current workflow intentionally stops at an unsigned candidate artifact and
does not publish a GitHub Release or package.
