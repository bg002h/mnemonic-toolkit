# Post-impl whole-diff R0 — inspect non-chunked md1 intake (v0.89.0, commit 364b337b) — Opus, adversarial

**Persisted per CLAUDE.md.** VERDICT: **GREEN (0 Critical / 0 Important).** Sound to tag v0.89.0. 2 documentation-only Minors (M1 folded; M2 cosmetic). Reviewer ran 39 tool-uses incl. a 20-position corruption fuzz + all-uppercase/mixed-case cases + real md-cli 0.13.0 parity; no prompt-injection.

## Empirical (reviewer's own runs)
- `cargo test -p mnemonic-toolkit` (MD_BIN set): all binaries ok, process exit 0 (the reviewer's run reported the counts per-binary; my own full run = 3749 passed / 0 failed).
- `cli_inspect_partial` (MD_BIN=md 0.13.0): 13 passed / 0 failed / 1 ignored; the new parity cell genuinely ran.
- clippy `--all-targets`: 0 warnings. Pinned-1.95.0 fmt: only `mlock.rs` diffs (g6-exempt); the 2 changed files clean.
- Adversarial: valid single → exit 0 + template; 2 dead singles → exit 4 + `origin: «unspecified»` + VERIFY-ME; 20 single-symbol corruptions → 0 leaks (never exit 0/5, never the wpkh template); corrupted under FORCE_TTY → exit 1 (v0.86.0 demote, not 5); all-uppercase valid → exit 0 (BIP-173 case-insensitive, correct); mixed-case → reject; 4-chunk keyed card → unchanged. `md decode` byte-identical templates.

## Funds invariants verified in the COMPILED md-codec 0.42.0 (registry, not local checkout)
- INV-1 (BCH before flag read): `unwrap_string` BCH-verifies (codex32.rs:182) BEFORE the chunked-flag `(b>>3)&1` is read (decode.rs:191); corruption `?`-short-circuits → flag never read → cannot re-route (empirically proven by the 20-position fuzz).
- INV-2 (content-id oracle unconditional for chunked; none for single by construction): flag=1 → `reassemble_with_opts(&[s], opts)` identical to multi-chunk; a single non-chunked payload's integrity = codex32 BCH + `decode_payload` validators.
- INV-3 (`EmptyOriginOverride` fatal-in-partial; `partial()` relaxes ONLY `MissingExplicitOrigin`): confirmed via `DecodeOpts` docstring decode.rs:25-34.
- INV-4 (no new acceptances): only a genuinely-valid non-chunked md1 is broadened; runs the full `decode_payload_with_opts` gauntlet.
- Dispatch: `[single]` binds `&&str`, deref-coerces to `&str`; `_` arm is `reassemble_with_opts` verbatim. verify-bundle untouched (`decode_card` called only from inspect.rs:121; verify_bundle uses strict `reassemble` at :388 + raw `expected.md1 == args.md1` at :696). Chunked path byte-identical. Tests mutation-sensitive (reverting the arm fails 5 of 6 new cells).

## Minor (documentation-only)
- **M1 (FOLDED):** the parity-cell comment claimed `md inspect` "exits 0 on a dead card, incompatible with exit-4" — FALSE for md-cli 0.13.0 (`md inspect` on a dead card exits 4, honoring the contract; a pipe masked `head`'s exit in the original check). The cell's conclusion (use `md decode`) still holds; the rationale was stale. Comment corrected: `md decode` chosen for consistency with the chunked parity cell; `md inspect` would also serve.
- **M2 (cosmetic, left):** the new parity cell soft-skips when MD_BIN is unset (early-return) vs the sibling's `#[ignore]` gate. Both acceptable.

## VERDICT: GREEN (0C/0I) — tag v0.89.0.
