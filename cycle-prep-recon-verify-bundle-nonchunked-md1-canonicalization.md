# cycle-prep recon — 2026-07-12 — `toolkit-inspect-nonchunked-md1-intake-gap` (verify-bundle leg)

**Origin/master SHA at recon time:** `de140a08`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** stale `cycle-prep-recon-*.md` scratch + `design/` SPEC/agent-report audit trail from prior cycles (no blockers); this recon file.

Slug verified: `toolkit-inspect-nonchunked-md1-intake-gap` — **verify-bundle leg only** (the inspect leg shipped v0.89.0). Citations are **ACCURATE**, but the residual note **under-scopes** the fix: it is two coupled facets, not a lone comparison change.

---

## Per-slug verification

### `toolkit-inspect-nonchunked-md1-intake-gap` (verify-bundle residual)
- **WHAT (from FOLLOWUPS.md, `⚠️ RESIDUAL` note, lines 38):** `verify-bundle` intake was not broadened for a plain NON-chunked single-string md1. The single-sig template path compares RAW strings (`verify_bundle.rs:696` `expected.md1 == args.md1`) against chunk-form toolkit-synthesized cards, so a non-chunked supplied string can never string-equal the chunk-form expected — "broadening intake alone can't help." Fix (if pursued): compare decoded descriptors / content-ids, own funds-reviewed R0 cycle.

- **Citations:**
  - `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:696` — `let md1_match = expected.md1 == args.md1;` — **ACCURATE**. Confirmed verbatim. `expected.md1` = `synthesize_unified(..., Md1Form::Template)` chunk-form (`:687-695`); `args.md1` = supplied `Vec<String>`. Raw `Vec<String>` equality.
  - "verify-bundle SUPPLIED-card decode sites use `reassemble`/`reassemble_with_opts` (chunk-form)" — **ACCURATE**. All supplied-md1 decode sites are chunk-form: the **classification gate** `:388` `if let Ok(d) = md_codec::chunk::reassemble(&md1_refs)`, plus `:2510`, `:3113`, `:3261`, `:3471` (`reassemble[_with_opts]`). No `decode_md1_string` anywhere in verify_bundle.rs.
  - "the single-sig template path" is the affected path — **ACCURATE but UNDER-SCOPED** (see finding below): it is one of two facets, and it is gated *upstream* by the classify decode.
  - Inspect-leg precedent (RESOLVED note, `inspect.rs`) — **ACCURATE, DRIFTED cite**: the `:207` cite already decayed; the live length-dispatch is `inspect.rs:256-260` (`chunks: [single] => decode_md1_string_with_opts(single, DecodeOpts::partial())`, `_ => reassemble_with_opts`).
  - md-codec API for canonicalization — **ACCURATE / present in the pinned vendor**: `vendor/md-codec/src/decode.rs:178,187` `decode_md1_string[_with_opts]`; `identity.rs:186` `compute_wallet_policy_id`; `lib.rs:60` re-exports `compute_wallet_descriptor_template_id` + `compute_wallet_policy_id`.

- **STRUCTURAL FINDING (recon-surfaced, not in the FOLLOWUP): the fix is TWO coupled facets.**
  1. **Intake never classifies a non-chunked card.** The template-bundle dispatch gate `verify_bundle.rs:386-407` decodes the supplied md1 with `reassemble` (`:388`, chunk-form only). A non-chunked single-string md1 fails `reassemble` → `if let Ok(d)` is false → the card **falls through past BOTH `verify_singlesig_template` (`:394`) and `verify_multisig_template` (`:405`)** into the general descriptor/policy dispatch (`:410+`), where it errors/mismatches. So a non-chunked card never even reaches the `:696` compare. **Facet 1 = length-dispatch this classify gate** (mirror `inspect.rs:256-260`).
  2. **The single-sig compare is byte-identity.** Even after facet 1 routes a non-chunked card into `verify_singlesig_template`, `:696` `expected.md1 == args.md1` (chunk-form expected vs non-chunked supplied `Vec<String>`) still fails. **Facet 2 = content-id comparison.**

- **READY IN-REPO PRECEDENT (recon-surfaced): the multisig keyless path already does facet 2.** `verify_bundle.rs:937-941` — `let md1_match = completed_template_id.as_bytes() == supplied_template_id.as_bytes();` via `md_codec::compute_wallet_descriptor_template_id(&outcome.completed)` and `compute_wallet_descriptor_template_id(d)`. The single-sig template path (`:696`) is the **lone raw-`Vec<String>` holdout**; the multisig path is a direct model for the single-sig fix (template-id bytes-compare, keyless-appropriate).

- **Action for brainstorm spec:** Scope the cycle as **two facets on the single-sig template path** (multisig path already content-id-based, but shares facet 1's intake gap — decide whether to length-dispatch the shared `:388` gate once, covering both): (a) length-dispatch the classify decode at `:388` so a non-chunked template md1 routes into `verify_singlesig_template`/`verify_multisig_template` (reuse the shipped `inspect.rs:256-260` `decode_md1_string_with_opts(single, DecodeOpts::partial())` vs `reassemble_with_opts` pattern — cite the v0.89.0 audit trail + `inspect-nonchunked-intake-*` reports); (b) replace `:696` raw compare with a content-id compare mirroring `:937-941` (`compute_wallet_descriptor_template_id` bytes, keyless template). **Funds-review focus (the reason this is its own R0 cycle):** switching single-sig from byte-identity to content-id RELAXES the match. The spec must prove the chosen id (template-id vs `compute_wallet_policy_id`) is the *faithful* equality for the verify-bundle contract and that its documented invariances (placeholder/explicit-vs-elided origin — `identity.rs:75`) never mask a funds-relevant difference. Guiding bound: never bless a wrong wallet; fail-closed reject stays the floor. Cite source SHA `de140a08`; re-grep the `:696`/`:388`/`:941` cites at write time (they will drift).

---

## Cross-cutting observations
1. **Citations all ACCURATE** (`:696`, chunk-form decode sites) — no DRIFTED-by-N or STRUCTURALLY-WRONG among the FOLLOWUP's own cites. The only drift is the *cross-referenced* inspect-leg `:207`→`:256-260` (already noted decayed in the RESOLVED entry).
2. **Under-scoping, not mis-citing.** The residual note frames the fix as "comparison-canonicalization" only and asserts "broadening intake alone can't help." That's true *for the single-sig compare in isolation*, but it obscures that a non-chunked card fails **upstream at the classify gate `:388`** and never reaches `:696` — so intake length-dispatch is *also* required (facet 1). The brainstorm must not implement facet 2 alone.
3. **Ready precedent lowers risk.** The multisig keyless path's `compute_wallet_descriptor_template_id` bytes-compare (`:937-941`) is an already-shipped, already-reviewed model for the single-sig content-id compare — the funds review can lean on it (same id, same keyless-template semantics).
4. **No claim-counting ambiguity.** Single slug, single leg.
5. **No incidental cross-pin staleness surfaced** in this path (the md-codec vendor pin `0.42.0` carries all needed APIs).

---

## Recommended brainstorm-session scope
- **One cycle, toolkit-only.** Slug: `toolkit-inspect-nonchunked-md1-intake-gap` verify-bundle leg. Two coupled facets on `verify_bundle.rs`: (1) length-dispatch the supplied-md1 classify decode at `:388` (and any sibling supplied-decode the spec elects to broaden), (2) content-id compare at `:696` mirroring `:937-941`.
- **Rough sizing:** ~30–80 LOC production (a length-dispatch helper reused at the classify gate + swapping one equality for a `compute_wallet_descriptor_template_id` bytes-compare) + a test matrix (non-chunked template md1 → verifies; non-chunked dead/mismatched card → mismatch/partial with correct exit; chunk-form byte-identical regression; funds-negative: a genuinely-different wallet must still FAIL). Comparable to the v0.89.0 inspect leg.
- **SemVer:** additive behavior (a previously-failing input form now verifies), **no clap flag / subcommand / dropdown change**. Strictly → **PATCH**; note the v0.89.0 inspect leg shipped as a **MINOR** (0.88→0.89) for the sibling additive intake, so the release convention may prefer MINOR for parity — defer to the spec/release call (not gating).
- **Locksteps:** **none triggered** — no clap flag-NAME change → GUI `schema_mirror` NOT fired; no `docs/manual/src/40-cli-reference/` flag surface change. Watch the `--json` wire-shape: if a non-chunked mismatch newly surfaces `result:"partial"`/exit 4 on verify-bundle, that is the **already-resolved** `verify-bundle-json-partial-result` class (GUI exit-4 badge, no wire-parse) — confirm no *new* verdict state is introduced. Optional docs touch: a prose note in the verify-bundle manual page that a non-chunked md1 is accepted (not a flag-reference edit).
- **Dependencies / ordering:** none — self-contained; inspect leg already shipped. Reuse the v0.89.0 pattern + audit trail (`design/{BRAINSTORM,SPEC,IMPLEMENTATION_PLAN}_inspect_nonchunked_intake.md`, `design/agent-reports/inspect-nonchunked-intake-*`).
- **Mandatory next gate:** brainstorm → spec + plan-doc → **opus architect R0 to 0C/0I BEFORE any code** (funds-adjacent verify path; the byte-identity→content-id relaxation is exactly a Critical-class question). Fold → persist verbatim to `design/agent-reports/` → re-dispatch until GREEN.
