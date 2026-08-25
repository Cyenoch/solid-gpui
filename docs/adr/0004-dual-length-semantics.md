# ADR-0004: Keep resource and text-input length limits in their native units

- **Status:** Accepted
- **Date:** 2026-08-25

## Context

The protocol carries values with two different safety contracts. Resource-like
strings such as image paths and URLs become bytes in a MessagePack frame and
must be bounded by the bytes consumed by the wire and host filesystem. A
TextInput `maxLength`, however, governs JavaScript text edits and native
selection positions, which are expressed in UTF-16 code units. Treating these
limits as one generic character count creates different behavior at the
TypeScript/Rust boundary and around non-BMP characters.

## Decision

- Resource limits are measured in UTF-8 bytes at the validation boundary.
  This applies to host paths, URLs, and other explicitly documented resource
  strings.
- TextInput `maxLength` is measured in UTF-16 code units. JavaScript clamps
  controlled values and native state applies the same unit so selection and
  marked-range offsets continue to refer to JavaScript string positions.
- The two units are named explicitly in public documentation and validation
  errors; neither is converted into a shared abstract character count.

## Alternatives rejected

- **Use Unicode scalar values or user-perceived characters everywhere:**
  rejected because neither is the unit used by the JavaScript selection API,
  and neither bounds encoded resource bytes.
- **Use UTF-8 bytes for TextInput `maxLength`:** rejected because a single
  non-BMP character could consume four bytes while occupying two UTF-16 code
  units, causing native and JavaScript selection/edit positions to diverge.
- **Use UTF-16 units for resources:** rejected because a string can satisfy a
  code-unit limit while exceeding the intended wire or host byte budget.

## Consequences

- Validation code must choose the unit based on the domain field instead of
  reusing one generic length helper.
- Emoji and other non-BMP text have intentionally different behavior in
  TextInput limits versus resource limits; tests must cover both boundaries.
- A future field needs an explicit unit decision before it reuses either
  limit, which makes accidental cross-domain coupling visible in review.
