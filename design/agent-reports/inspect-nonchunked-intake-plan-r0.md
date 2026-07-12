# Plan R0 — inspect non-chunked md1 intake — Opus, adversarial

**Persisted per CLAUDE.md.** VERDICT: **GREEN (0 Critical / 0 Important).** Plan sound to implement. Reviewer transplanted the exact dispatch snippet into a scratch crate and confirmed it COMPILES; no prompt-injection.

## Snippet correctness — VERIFIED
`decode_card(kind: CardKind, chunks: &[&str])` (`inspect.rs:205`). `match chunks { [single] => decode_md1_string_with_opts(single, ..), _ => reassemble_with_opts(chunks, ..) }` compiles: `[single]` binds `&&str` under match-ergonomics, deref-coerces to the `&str` param; the `_` arm feeds `&[&str]` unchanged (identical to today). `md_codec::{decode_md1_string_with_opts, reassemble_with_opts, DecodeOpts}` re-exported (`lib.rs:49,52`).

## Confirmations
- All 4 SPEC-R0 Minors + 5 answers folded. M1 lock constructible (`encode_md1_string`/`tree` public `lib.rs:55,41`; bad-root-tag BCH-clean single → `MdCodec` not `FutureFormat` → auto-fire 0 edits → terminal Err, NOT exit 5). M2 reuse pre-validated (`dead_chunks()` `cli_inspect_partial.rs:76-80` already calls `decode_md1_string_with_opts(single, partial())` on `DEAD_SINGLES`; `CANONICAL_SINGLE:46` is the valid single). BOUNDARY mutation-sensitive. Scope structurally enforced (`decode_card` inspect-private; verify_bundle uses its own `:696` string-compare). Release ritual complete; pre-tag `cargo +1.95.0 fmt --all -- --check` matches CI `rust.yml:63-79` exactly. SemVer MINOR correct.

## Minor findings (fold into impl)
- **M-a** citation precision: exit-3 at `error.rs:587` (not :588); `WireVersionMismatch→FutureFormat` arm head `error.rs:1054` (body 1055-1057). codex32 `:182-186` + SHA correct.
- **M-b** rewrite now-FALSE in-source comments: `inspect.rs:239-241` ("Intake is CHUNK-FORM only … unsupported version 2 gap") + test-module doc `cli_inspect_partial.rs:15-19`. Documentation integrity.
- **M-c** implement all 9 SPEC §6 tests (plan condensed to 6): add §6.1.2 (positional-form intake), §6.2.6 (doctored chunk-set-id → `ChunkSetIdMismatch`, INV-3 lock), §6.3.7 (bad-BCH → codex32 reject, INV-2 lock). Guarded by construction + existing suite, but retain explicitly.
- **M-d** cross-binary parity: use `md inspect` (SPEC §6.4 #9), matching the v0.75.0 parity gate, not `md decode`.

## VERDICT: GREEN (0C/0I) — sound to implement; fold M-a..M-d into the impl.
