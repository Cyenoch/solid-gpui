# Native layout and internationalization

Status: ready-for-agent

## Acceptance

Expose useful supported native semantic/layout contracts and add focused mixed-direction/Unicode acceptance; avoid fake browser APIs.

## Comments

Implementation and verification evidence will be recorded here.

Implemented primitive TextInput extended-grapheme navigation/deletion with undo coverage, and generated application-wide reduced-motion commands plus Gallery control. Full bidi geometry, native editor graphemes, automatic OS motion preference, and grid are not implemented. See ../layout-workload-seams.md for exact remaining seams.


Added grapheme-safe Rope navigation/deletion for generated editors, with a passing cross-chunk Unicode regression. Added canonical native grid tracks/spans, bounded validation, reactive style equality and a passing native geometry test. Full bidi geometry and automatic OS motion following remain open.

Implemented system/reduced/full motion state and app-owned event-driven NSWorkspace/UISettings/Portal subscriptions. The macOS build and shared reducer/native-command tests pass; platform API compilation and actual notification qualification are tracked separately. The newly introduced bool API was replaced, without compatibility wrappers.
