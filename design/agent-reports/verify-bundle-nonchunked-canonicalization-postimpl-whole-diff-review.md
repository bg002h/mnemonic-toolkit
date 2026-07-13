# Post-implementation whole-diff review — verify-bundle non-chunked md1 canonicalization (pre-tag v0.90.0)

**Reviewer:** Fable architect (`model:"fable"`), 2026-07-12. Read-only adversarial review of `git diff de140a08..HEAD` (4 commits: Facet 1 `2fd7a1b9`, Facet 2 `5a5fb59e`, fmt `efe8294b`, release `b5350b43`). **Usage:** 39 tool-uses, ~681s, 126702 tokens.

## Verdict: RED — 0 Critical / 1 Important / 2 Minor
The two code facets are **correct and funds-safe**; every funds-invariant (INV-1..6, INV-KEYED) checked out against source. The Important is a documentation-integrity defect in the release commit (not code).

## Important
### I-1 — FOLLOWUPS.md pre-claimed this review's verdict + cited a nonexistent artifact
`design/FOLLOWUPS.md:38` (committed in `b5350b43`) said "Full R0 pipeline GREEN (design + SPEC ×2 + plan ×3 + **post-impl whole-diff, all 0C/0I**; suite 3760/0)" and listed `…-postimpl-whole-diff-review.md` as an artifact — while THIS review was still running and that file existed neither tracked nor untracked. The same commit's CHANGELOG correctly said "the mandatory post-impl whole-diff R0 gates the tag" — the two docs disagreed; the "single source of truth" file asserted a 0C/0I gate result for a review that had not occurred (and which is in fact RED). **Fix:** persist this review verbatim to the cited path; correct the FOLLOWUPS wording to the actual outcome (code 0C/0I; post-impl found 1 Important docs + 2 Minor, folded) — no code change.
**RESOLUTION (folded, commit after b5350b43):** review persisted here; FOLLOWUPS.md:38 reworded to "post-impl whole-diff review: code 0C/0I; 1 Important (this pre-claim, corrected) + 2 Minor folded." No pre-claim remains.

## Minor
### M-1 — dead-card fail-closed lock could go vacuous again under a clap-surface change
`crates/mnemonic-toolkit/tests/cli_verify_bundle_md1_template.rs` `verify_bundle_nonchunked_dead_card_falls_through_strict` asserted only `.failure()` + `!stdout.contains("OK")`. A future arg-surface change re-introducing a pre-gate (clap exit 64) failure would keep it green without exercising strict classify (the exact vacuous-lock class the implementer already fixed once). Suggest pinning `.code(2)` and/or the `--template is required` stderr fragment.
**RESOLUTION (folded):** hardened to `.code(2)` + assert stderr contains "--template".

### M-2 — "Suite 3760/0; clippy clean" unverified by this (read-only) review
The human gate must run `cargo test -p mnemonic-toolkit` on HEAD before tagging.
**RESOLUTION:** already satisfied — main loop independently reran the full suite (`CARGO_EXIT=0`, 3760 passed / 0 failed / 19 ignored, 0 failures/panics) + clippy exit 0 + pinned-1.95.0 fmt gate PASS, before the release commit.

## Hunt results (all CLEAN unless above)
1. **Facet 2 funds-safety CLEAN.** `d` is the SUPPLIED classify-decoded card (param, `&d` at :406); `d_expected` from `synthesize_unified(..., Md1Form::Template)` — not swapped/vacuous. `compute_md1_encoding_id` = SHA-256(`encode_payload`)[0..16]; `encode_payload` serializes header + path_decl (verbatim) + use-site + tree + full TLV (incl. UNKNOWN entries, `tlv.rs:197`) → sensitive to every funds-relevant byte. Only relaxations vs byte-compare are form-only (chunk/HRP-case/checksum → same descriptor). Version blur impossible (`Header::read` rejects version≠4); placeholder-permutation blur impossible (strict decode rejects non-canonical order, `validate.rs:27-35`).
2. **Facet 1 strictness CLEAN.** `decode_md1_string` strict; dead/pathless → `MissingExplicitOrigin` → fall-through → `--template is required` ModeViolation exit 2. Chunked-of-1 routes to `reassemble_with_opts(&[s], default)` ≡ `reassemble` (byte-identical). BCH verified before flag read (corruption can't re-route). No new exit-0 path except the intended valid non-chunked template.
3. **Error handling CLEAN.** `?` on both id-computes → `MdCodec` → nonzero exit, never a false pass; a genuine mismatch never errors (equality differs → `md1_match=false` → exit 4).
4. **Multisig path UNTOUCHED.** WDT-id compare unchanged (:958-962); no-`--from` floor :896-913; general/keyed path untouched; non-chunked keyed structurally impossible (INV-KEYED).
5. **Test integrity CLEAN (+M-1).** `…noncanonical_encoding_mismatch` asserts BOTH `.code(4)` AND `md1_template_match.passed==false` (probative). The two `--mk1` deviations are necessary (clap `required_unless_present_any`, :183) and STRENGTHENING (plan drafts failed clap at exit 64 pre-gate). The SPEC-#6 keyed-multichunk substitution is planr0-round-1-authorized.
6. **Release correctness CLEAN except I-1.** All version sites 0.90.0; `.examples-build` diff version-strings-only + gen.sh↔Examples.md mutually consistent; FOLLOWUPS verify-bundle leg RESOLVED + dead-card residual filed `open`.
7. **Scope CLEAN.** Source hunks only verify_bundle.rs classify gate + single-sig compare; no unauthorized edits.

## Proof of work
`git diff de140a08..HEAD` (full hunks); verify_bundle.rs :150-220/:360-490/:560-853/:880-1010/:2895-2924; vendor/md-codec decode.rs :1-260, chunk.rs :290-419, identity.rs :1-111, encode.rs :50-170, header.rs, validate.rs :17-62, tlv.rs :212-292, codex32.rs; SPEC (full), IMPLEMENTATION_PLAN :120-200/:486-495, FOLLOWUPS diff, git ls-files/log (missing artifact + v0.89.0 precedent ordering). Suite NOT executed (read-only) — see M-2 resolution.
