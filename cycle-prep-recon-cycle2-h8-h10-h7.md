# cycle-prep P0 STRICT-GATE recon — cycle-2 (H8 / H10 / H7)

**Program:** constellation bug-fix program (next funds-loss batch — all toolkit, no codec-publish dependency)
**Recon date:** 2026-06-20 · **Recon only** — no source edited, no brainstorm/impl started.
**Source docs:** `design/agent-reports/constellation-bughunt-2026-06-20.md` (H8/H10/H7 entries) +
`design/PLAN_constellation_bughunt_fix_program.md` (workstream map).

## Live SHAs / branch / sync

| repo | default branch | live SHA (this recon) | sync |
|---|---|---|---|
| `mnemonic-toolkit` | `origin/master` | **`f9467cc581ba89b5ae25cc0fd0eea80d1b5053c5`** | `git fetch -q origin` ✓ |
| `descriptor-mnemonic` (md) | `origin/main` | **`58cc9ec25b3d35120c8e785d3c2ce7f48322529b`** | `git fetch -q origin` ✓ |

> **Source has MOVED since the cycle-2 citations were filed.** toolkit `origin/master` is now `f9467cc5`
> = release 0.61.0 (cycle-1 CRITICALs H12/H1/H13 merged). All citations below were re-grepped against
> `git show origin/master:<path>` / `git grep origin/master` — NOT the working tree (it is on another
> instance's WIP branch `feature/bundle-md1-template-multisig`).

**Cycle-1 commits on `origin/master` (relevant to drift):**
- `080ac03e` H13 — REJECT hardened multipath at toolkit descriptor lex (`parse_descriptor.rs`)
- `1e1e3f3d` H13 C1 — reject malformed double-marker multipath at toolkit lex (`parse_descriptor.rs`)
- `c4b46624` H12 — descriptor-mode taproot multisig defaults BIP-48 origin to `3'` (`bundle.rs`)
- `4ed3bbe9` H1 — verify-bundle `md1_xpub_match` widens to structural policy compare (`verify_bundle.rs`)
- merges `36255edf` (H13) / `12344847` (H12+H1); release `f9467cc5`

These edited exactly H7's files (`parse_descriptor.rs`, `bundle.rs`, `verify_bundle.rs`) — so **H7's cited
line numbers drifted** (file grew). They did NOT touch `synthesize.rs` (H8) or `wallet_export/*` +
`export_wallet.rs` (H10), so **H8 and H10 citations are line-stable**.

---

## H8 — `--md1-form=template` drops the BIP-39 wordlist language → non-English seed re-emits as English → wrong master seed

**class:** B-policy-collapse (→ wrong master seed) · **HIGH** · "highest-impact funds-loss this hunt"
**workstream (plan):** S-TEMPLATE · **source SHA verified against:** `f9467cc5`
**file:** `crates/mnemonic-toolkit/src/synthesize.rs`

| # | cited | live | tag | evidence |
|---|---|---|---|---|
| 1 | `:1265` template ms1 emit hardcodes English | `:1265` | **ACCURATE** | `let emit_lang = c.language.unwrap_or(bip39::Language::English);` (inside `synthesize_template_descriptor`'s ms1-emit loop) |
| 2 | `:486-488` call site drops `run_language` | `:487` | **ACCURATE** | `return synthesize_template_descriptor(descriptor, cosigners, privacy_preserving);` — `run_language` is in scope (param at `:471`) but NOT forwarded |
| 3 | `:1158-1162` fn sig lacks language param | `:1158-1162` | **ACCURATE** | `fn synthesize_template_descriptor(descriptor, cosigners, privacy_preserving) -> Result<Bundle, ToolkitError>` — 3 params, no language |
| 4 | `:547` keyed path correctly uses `run_language` | `:547` | **ACCURATE** | `let emit_lang = c.language.unwrap_or(run_language);` (the asymmetry the template path regressed) |

**REPRODUCES: YES.** Keyed path (`:547`) falls back to `run_language`; template path (`:1265`) hardcodes
`English` and its fn (`:1158`) never receives `run_language` because the call site (`:487`) drops it. A
non-English seed under `--md1-form=template` re-emits its ms1 as English → faithful restore reconstructs a
different phrase → different master seed → funds engraved under the template are unrecoverable.

> **Bite-scope nuance (for the spec, not a downgrade):** the defect bites via the `c.language == None`
> fallback. If a slot already carries `c.language = Some(non-English)` (e.g. an import-json mnem source) the
> template path emits correctly. The hole is the descriptor-`@N` phrase/entropy path where slot language is
> `None` and run-level `--language` is the sole carrier — exactly what the `:547` comment calls out.

**Protocol fact (BIP-39) — CONFIRMED.** Seed = PBKDF2-HMAC-SHA512 over the NFKD-normalized **mnemonic
phrase string** (salt `"mnemonic"`+passphrase). The seed is a function of the *phrase text*, NOT the raw
entropy — so the same entropy rendered through a different-language wordlist yields a different word string
→ a different 512-bit seed → different keys/addresses. Wordlist language is seed-load-bearing.

**Action for spec:** thread `run_language` into `synthesize_template_descriptor` (add the param; forward it
at the `:487` call site) and replace the `:1265` hardcoded English fallback with
`c.language.unwrap_or(run_language)` — mirroring the keyed path's `:547`. Add a non-English template-form
ms1 round-trip test (master-fp divergence assertion). **Mirrors the keyed path → the structural anchor of
S-TEMPLATE.** Also closes L9 (the parallel template path's missing refusals) if grouped.

**Cycle-1 interaction:** NONE. No cycle-1 commit touched `synthesize.rs`. All 4 lines live-stable.

---

## H10 — Unsorted `multi(...)` silently exported as BIP-67 `sortedmulti` to Coldcard/Jade/Electrum → wrong addresses

**class:** A-wrong-address (B→A; differential-oracle-PROVEN) · **HIGH**
**workstream (plan):** WS-EXPORT-MULTISIG · **source SHA verified against:** `f9467cc5`

| # | cited | live | tag | evidence |
|---|---|---|---|---|
| 1 | `wallet_export/coldcard.rs:258-369` | `258-370` | **ACCURATE** | `emit_coldcard_multisig_text` — lex-sorts xpubs only for `*SortedMulti` (`:341-346`); for `wsh-multi` writes slot-index order, but the emitted file (`Name:/Policy:/Derivation:/Format:/<XFP>: xpub`) has NO sorted/unsorted field. NO refusal/warning |
| 2 | `wallet_export/jade.rs:43-46` | `43-46` | **ACCURATE** | all four (sorted+unsorted) delegate byte-identically to `emit_coldcard_multisig_text`; same defect inherited |
| 3 | `wallet_export/electrum.rs:131-191` | `131-191` | **ACCURATE** | `emit_electrum_multisig_json` writes xpubs in slot-index order into `x1/x2/…`; Electrum's loader sorts (BIP-67) anyway; no sorted/unsorted field, no refusal |
| 4 | `cmd/export_wallet.rs:122-128` | `120-128` | **ACCURATE** | `ColdcardMultisig` arm accepts `WshMulti \| WshSortedMulti \| ShWshMulti \| ShWshSortedMulti \| TrMultiA \| TrSortedMultiA` → `ColdcardEmitter::emit` with no sorted-vs-unsorted refusal |

**Faithful-format contrast (verified):** `descriptor.rs`, `bitcoin_core.rs`, `sparrow.rs`, `bip388.rs`
carry the literal descriptor token (`multi(` vs `sortedmulti(`) verbatim → preserve the distinction. The
toolkit DOES model it (`WshMulti` ≠ `WshSortedMulti` as distinct `CliTemplate` variants); it is lost only
at the three field-less vendor emitters.

**REPRODUCES: YES.** `export-wallet --template wsh-multi` (or `sh-wsh-multi`) with `--format
coldcard`/`jade`/`electrum` is accepted with no refusal/warning and emits a field-less file the target
reconstructs as BIP-67 sortedmulti → different witnessScript/address whenever keys aren't already
lexicographically ordered.

**Protocol fact (BIP-67) — CONFIRMED.** BIP-67 = deterministic lexicographic sort of compressed pubkeys in
P2(W)SH multisig, so `sortedmulti`'s script/address DIFFERS from literal-order `multi(...)` whenever keys
aren't pre-sorted. `multi` encodes a consensus-significant literal order; Coldcard/Electrum multisig file
formats are sortedmulti-only (no field to express unsorted) → silent reinterpretation.

**Action for spec:** refuse unsorted `wsh-multi`/`sh-wsh-multi` for electrum/coldcard/jade with a
point-to-faithful-format error (descriptor / bitcoin-core / sparrow / bip388), **OR** gate behind an
explicit new flag `--allow-sortedmulti-coercion` with a loud warning. **The flag path triggers lockstep
(below); the pure-refusal path does not.**

> **`--allow-sortedmulti-coercion` flag CONFIRMED ABSENT** on `f9467cc5` (zero matches across
> `crates/mnemonic-toolkit/src/`, the manual, and `mnemonic-gui/src/schema/` — the `sortedmulti` hits in
> the GUI schema are template-type enum *values*, not this flag).
> **LOCKSTEP (only if the fix ADDS the flag):** new clap flag on `export-wallet` ⇒ update
> `mnemonic-gui/src/schema/mnemonic.rs` (`schema_mirror` flag-NAME gate) AND
> `docs/manual/src/40-cli-reference/41-mnemonic.md` in the SAME PR (paired sibling PR if cross-repo
> authoring isn't feasible). A **pure-refusal** fix changes only `--help`/error text → manual-only touch,
> no GUI schema gate (flag-NAME set unchanged). **Recommend the refusal arm** to avoid the GUI lockstep
> unless a deliberate escape hatch is wanted.

**Cycle-1 interaction:** NONE. No cycle-1 commit touched `wallet_export/*` or `export_wallet.rs`.

---

## H7 — Prefix-form `[fp/path]@N` key-origin annotation silently ignored → origin path dropped + per-@N fingerprint guard bypassed

**class:** B-policy-collapse · **HIGH** · **HIGHEST-DRIFT finding** (cycle-1 grew the cited files)
**workstream (plan):** S-VERIFY (shared lexer) · **source SHA verified against:** `f9467cc5`

| # | cited | live | tag | evidence |
|---|---|---|---|---|
| 1 | `parse_descriptor.rs:60-140` lex_placeholders, regex `:69-71` | fn `:60`; regex `:82-84` | **DRIFTED +~13** (defect structurally intact) | live regex (`:84`) `@(\d+)(?:\[([0-9a-fA-F]{8})((?:/\d+(?:'|h)?)*)\])?(?:/<([^>]*)>)?(/\*(?:'|h)?)?` — the `[...]` origin block is anchored ONLY after `@(\d+)` (SUFFIX form). Prefix `[fp/path]@N` → groups 2/3 = `None`, bracket dropped |
| 2 | `parse_descriptor.rs:319` substitute_synthetic strip | fn `:353`; strip-regex `:369-371` | **DRIFTED +~34** | strip-regex strips only the suffix bracket; a leading `[fp/path]` LEAKS into the descriptor handed to `Descriptor::from_str` → md1/mk1 |
| 3 | `bundle.rs:1369-1370,1569-1626` consumption + bypassed fp guard | lex call `:1369`; `fingerprint_annos[idx]` `:1581-1582`; guard `:1617-1622` | **ACCURATE/barely-drifted** | guard `if let Some(anno) = anno_fp { if anno != master_fp { Err } }` keyed on the lexer's `fingerprint_anno` → prefix form = `None` ⇒ guard never entered ⇒ master-fp cross-check BYPASSED |
| 4 | `cmd/verify_bundle.rs` shared lexer | import `:1283`; `lex_placeholders` `:1342`; `resolve_placeholders` `:1345` | **ACCURATE** (no line cited) | verify-bundle calls the SAME lexer ⇒ same prefix-form mis-parse; its phrase arm has NO compensating per-@N fp check (that guard is bundle.rs-only), so it inherits the dropped origin path identically |

**REPRODUCES: YES (both halves, on both bundle and verify-bundle paths).** (a) `lex_placeholders` (live
`:84`) matches the origin block only in suffix position — empirically, prefix `[fp/path]@0` yields
`fingerprint_anno=None, origin_path_anno=None` and the bracket is dropped (origin path lost; slot xpub built
at master/default path). (b) The per-@N fp cross-check (live `bundle.rs:1617-1622`) reads
`fingerprint_annos[idx] = None` for the prefix form → the funds-safety fingerprint guard is bypassed.
`substitute_synthetic` (live `:353`) strips only the suffix bracket → prefix text leaks to md1/mk1.

**Cycle-1 interaction (the headline):** cycle-1's H13 (`080ac03e` + `1e1e3f3d`) **rewrote the SAME
`lex_placeholders` regex — but ONLY the group-4 multipath segment** (`(?:/<([0-9;]+)>)?` →
`(?:/<([^>]*)>)?`, to capture+reject hardened/malformed alts). The key-origin `[fp/path]` capture
(groups 2/3) is **byte-for-byte UNCHANGED** (traces to the original suffix-only design commit `7d54882d`).
So **cycle-1 did NOT fix, worsen, or otherwise change H7** — only the line numbers drifted.
**Merge-adjacency to flag for the spec:** a cycle-2 H7 fix edits the SAME `lex_placeholders` / regex literal
that cycle-1's H13 just landed in, so the patch must **preserve H13's hardened-multipath reject** (don't
regress the group-4 capture/validation while adding group-2/3 prefix alternation). Not a conflict if cycle-2
branches off `f9467cc5`; it is a within-file ordering note.

**Acuteness fact (found during verification):** the toolkit's OWN help-text and tests USE the canonical
prefix form — `bundle.rs:2300` help: *"Override per-placeholder with `[fp/path]@N`"*; tests/doc-comments at
`parse_descriptor.rs:329,2989,2992,3054`. So the toolkit advertises the prefix syntax to users while its
lexer silently ignores it. This elevates H7's real-world reachability beyond "obscure manual footnote."

**Protocol fact (BIP-380) — CONFIRMED.** BIP-380 Key Expressions → Optional Origin Information defines the
origin as a **PREFIX** `[fingerprint/derivation/path]KEY` — the bracketed origin appears BEFORE the key. So
`[fp/path]@N` (origin-before-placeholder) is the **BIP-380-canonical** position and the toolkit's
`@N[fp/path]` suffix is non-canonical. A user/tool following the standard naturally writes the prefix form —
exactly the form silently mis-parsed.

**Action for spec:** accept BOTH annotation positions in `lex_placeholders` (capture a leading
`[<8hex>(/path)?]` before `@N`, populate `fingerprint_anno`/`origin_path_anno` identically; mirror the strip
in `substitute_synthetic`), **OR** reject the prefix form with an explicit typed `DescriptorParse` error and
fix the help-text/manual. Add prefix-vs-suffix fp-mismatch + identical-md1/mk1 round-trip tests on BOTH
bundle and verify-bundle. **Preserve cycle-1's H13 hardened-multipath reject in the same function.**

---

## Cross-cutting observations

1. **All three REPRODUCE on `f9467cc5`.** Zero refuted, zero downgraded. H8 + H10 citations line-stable; H7
   citations DRIFTED (cycle-1 grew the files) but the defect is structurally intact at the new lines.
2. **Cycle-1 interaction is confined to H7** (and is benign: line-drift + a within-file merge-adjacency note,
   no behavior change). H8 and H10 are fully independent of cycle-1.
3. **No codec-publish dependency for any of the three** — all toolkit-only (matches the plan's cycle-2 framing).
   Contrast with cycle-1's H13 (md-cli tag→toolkit pin lockstep).
4. **SemVer:** each is a toolkit **MINOR** (behavior change / new refusal / possibly a new flag). If batched,
   one MINOR covers all three.
5. **Lockstep surface = H10 only, and ONLY if the fix adds `--allow-sortedmulti-coercion`.** H8 (no CLI
   surface change) and H7 (lexer-internal + at most a `--help`/manual text touch) carry no GUI schema_mirror
   obligation. Default the H10 fix to **pure refusal** to keep the GUI out of the loop; reserve the flag (and
   its GUI+manual paired-PR) for a deliberate escape-hatch decision in the brainstorm.
6. **Workstream zones are disjoint** — H8 = `synthesize.rs`; H10 = `wallet_export/{coldcard,jade,electrum}.rs`
   + `export_wallet.rs`; H7 = `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs`. No file overlap among
   the three ⇒ they CAN run as three concurrent single-subagent workstreams.

## Recommended cycle-2 brainstorm scope

- **All three in ONE cycle, three concurrent workstreams** (disjoint files, no shared state):
  - **WS-S-TEMPLATE (H8)** — `synthesize.rs`: thread `run_language`; ~10-20 LOC + 1 non-English round-trip test.
    Consider folding **L9** (parallel template-path refusals) here since it's the same path.
  - **WS-EXPORT-MULTISIG (H10)** — `wallet_export/{coldcard,jade,electrum}.rs` + `export_wallet.rs`: refuse
    unsorted `wsh-multi`/`sh-wsh-multi` for the three field-less formats; ~30-60 LOC + refusal tests +
    differential-oracle assertion. **Decide refusal-vs-flag at brainstorm** (refusal = no GUI lockstep).
    Consider folding **H11** (Coldcard/Jade divergent-path `m/0'/0'` collapse, plan's WS-EXPORT-MULTISIG mate)
    if a second cycle-2 export item is wanted — same files.
  - **WS-S-VERIFY-LEX (H7)** — `parse_descriptor.rs` + `bundle.rs` + `verify_bundle.rs`: accept-both-positions
    or reject-prefix; ~40-80 LOC + prefix/suffix round-trip + fp-mismatch tests on both paths. **Must preserve
    cycle-1 H13's hardened-multipath reject** (same `lex_placeholders` regex literal). Touches the post-cycle-1
    code most — sequence/branch off `f9467cc5`.
- **Rough total LOC:** ~80-160 impl + tests across the three.
- **SemVer:** toolkit **MINOR** (single bump if batched).
- **Lockstep flag:** **only H10, and only if `--allow-sortedmulti-coercion` is added** → GUI schema_mirror +
  manual lockstep (paired-PR). Recommend pure-refusal default to avoid it.
- **Split recommendation:** keep as **one cycle** — disjoint zones, uniform toolkit-MINOR, no codec-publish
  chain. H7 is the only one entangled with just-merged cycle-1 code, but only by line-adjacency, not behavior.
  Mandatory per the repo conventions: a single brainstorm SPEC + R0 GREEN (0C/0I) gate before any code, then a
  single subagent per workstream (TDD), then a mandatory post-impl adversarial whole-diff review.

---

_Recon only. No brainstorm or implementation started. Verified against toolkit `f9467cc5` / md `58cc9ec`._
