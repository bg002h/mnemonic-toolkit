# R0 REVIEW — cycle-11a GUI hygiene (M9 · L12 · L13) — Round 1

**Spec:** `design/BRAINSTORM_cycle11a_gui_hygiene.md`
**GUI repo:** `mnemonic-gui` `master = 0bbe3e1` (v0.45.0, pins toolkit v0.60.0).

## VERDICT: **NOT GREEN** — 0 Critical / 1 Important / 2 Minor

The spec is decision-complete, structurally accurate, and the three fixes are sound. The single blocking issue is a fabricated factual claim in the L12 justification — the "plausible-but-wrong external-fact" class the project's recon discipline exists to catch. No code-behavior consequence, but it must be corrected before GREEN.

## IMPORTANT (1) — blocking

**I1. The L12 "benign over-acceptance" justification rests on a fabricated toolkit refusal that does not exist.**

- **Location:** spec §3.2 line 115 and §7 D7 line 215.
- **Claim:** the double-origin form `wpkh([fp]@0[fp]/<0;1>/*)` "is toolkit-rejected regardless of the pin (`'@0 carries BOTH a prefix … and a suffix … ambiguous — supply exactly one'`, `parse_descriptor.rs:113-118`, exit 2)."
- **Reality (empirically + source-verified):**
  1. No such "ambiguous / supply exactly one" double-origin refusal exists anywhere in the toolkit. Whole-crate grep finds the only "supply exactly one" string at `compare_cost.rs:79` (a `--miniscript` vs `--descriptor` check — unrelated). `parse_descriptor.rs` has no double-origin ambiguity guard.
  2. The cited lines `parse_descriptor.rs:113-118` are the **multipath-alt u32 parsing** (`@{i} multipath alt ... is not u32`), not origin-position handling.
  3. The toolkit v0.60.0 binary classifies the double-origin form as `canonical` (exit 0); the `bundle` lexer **accepts** it (silently taking the suffix annotation), advancing to fingerprint-matching. It does NOT reject as ambiguous.
- **Why it blocks:** D7 ("don't tighten to exactly-one-of-prefix/suffix") is *justified by* this false premise. The behavioral conclusion survives (post-fix the pin applies `--account 0`, compatible with the toolkit's acceptance, so the over-acceptance IS benign — in fact more cleanly than argued). But a design doc the plan-doc inherits cannot ship with a fabricated source citation + non-existent error string; it propagates false protocol facts into plan and tests.
- **Required change:** Rewrite §3.2 line 115 and D7 to state the *true* toolkit behavior: the toolkit **accepts** the double-origin form (`parse_descriptor.rs:70` regex matches only the suffix bracket; prefix `[fp]` consumed as a key annotation per the `tr([fp/path]@N)` form at `:289`). The over-acceptance is benign because the GUI classifying it Canonical -> pinning `--account 0` is *compatible* with a descriptor the toolkit accepts. Remove the fabricated `parse_descriptor.rs:113-118` citation and the invented "ambiguous — supply exactly one" quote. D7's decision (don't tighten) stays; only the rationale must become factual. Re-grep any toolkit citation lifted into the plan-doc against current source.

## MINOR (2) — non-blocking, fold opportunistically

- **M-min-1.** The `canonicity_drift.rs:132` fixture-count comment ("11 Canonical + 4 NonCanonical + 3 ParseFails = 18") is not in the L12 edit list. Adding the suffix-form Canonical fixture makes it 12/19/16-classify. The fixture-add step should note updating this comment.
- **M-min-2.** Seedqr-introduction commit SHA mismatch (immaterial): spec §3.3 line 140 / D11 cite `19c1a16d`; independent verification found `5f0b7b45` ("v0.31.6 — SeedQR --from unification"). Both agree on the version (v0.31.6 < pinned v0.60.0), so the no-pin-bump conclusion is unaffected. Correct the SHA or drop it.

## Confirmations (SOUND — no change needed)

- **M9 fully sound.** Recursive `zeroize_keys` over `key`/`keys`/`children`; `hex` correctly excluded (public digest — `tree_model.rs:693-694`); `String: Zeroize` live (zeroize 1.8.2; precedent `secrets.rs:300`); wiring via `state.tree.as_mut()` -> `tree.root.zeroize_keys()`, no borrow conflict. Unconditional exit-zeroize of public xpubs correct for a teardown sweep.
- **L12 regex fix correct.** Verified via python `re` AND the live v0.60.0 binary: suffix form now matches (Canonical), prefix/no-origin/multipath unchanged; proposed sub-pattern byte-identical to the existing prefix bracket; consumes only the `--account` pin path (`conditional.rs:238-245`).
- **L13 + schema_mirror: "NO schema_mirror trigger" CONFIRMED** three ways: (a) `schema_mirror.rs:52-54` compares flag-NAMES only; `schema_mirror_secret_drift.rs:94-102` compares per-(sub,flag) secret-bits only; neither compares value enums. (b) toolkit `--from` uses a custom `parse_from_input` value_parser -> `gui-schema` emits `kind:"text"`, `choices:null`. (c) `--to` keeps the 13-value list. The CONVERT_FROM/TO_NODES split is a GUI-internal list, ungated. Seedqr `--from`-yes/`--to`-no asymmetry is real/intentional; index-1 ordering matches `NodeType::as_str`. No-pin-bump correct.
- **SemVer/process correct.** MINOR 0.46.0; PR + 5-target CI before tag (`FOLLOWUPS.md:716`); version sites `Cargo.toml:3` + `README.md:42`; toolkit pin `README.md:50` / `pinned-upstream.toml:22` correctly UNCHANGED; never `cargo fmt`.

**Path to GREEN:** Fold I1 (rewrite the L12 double-origin justification to factual toolkit behavior, drop the fabricated citation + invented error quote). M-min-1/2 optional. Persist this review, fold, re-dispatch the architect (a fold can introduce drift). The plan-doc then runs its own R0 loop.
