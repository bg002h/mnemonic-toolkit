# SPEC — bip388-double-star-shorthand-support

**Accept the BIP-388 `/**` combined-wildcard shorthand on descriptor intake by expanding `…/**` →
`…/<0;1>/*` before it reaches the parser, instead of hard-rejecting it — for BOTH the concrete-xpub form
(`xpub/**`) AND the canonical `@N`-template form (`@0/**`).**

- **Author:** (this session) — single-author design per CLAUDE.md phase-2 convention.
- **Source SHA (all citations grep-verified against this):** `0964462d` (origin/master; Cycle B v0.77.0 shipped at tag `44e55c4e`, +1 FOLLOWUP-flip).
- **FOLLOWUP slug:** `bip389-double-star-shorthand-support` — **RETITLED to `bip388-double-star-shorthand-support`** (the FOLLOWUP misattributes `/**` to BIP-389; §2). Old slug greppable as a forwarding note.
- **Recon:** `cycle-prep-recon-bip389-double-star-shorthand-support.md` + xpub-search spike (§3). **R0:** `design/agent-reports/cycleC-spec-r0-round-1.md`.
- **Target release:** `mnemonic-toolkit-v0.78.0` (MINOR). md/ms/mk codecs **NO-BUMP**. **No GUI/`schema_mirror` impact** (no clap flag/subcommand/dropdown change).
- **Status:** R0-GREEN at SPEC round 2; **rev-5** — §0 scope expanded (PLAN-R0-r1 I1/I2: accept `/**` on ALL literal-descriptor user surfaces) + §0-item-6/§6/§7.11 corrected (PLAN-R0-r2 I3: compare-cost is EQUIVALENCE-only — rejects all multipath; `/**`≡`/<0;1>/*` there means identical-reject; FOLLOWUP filed). Reviews: `cycleC-spec-r0-round-{1,2}.md` + `cycleC-plan-r0-round-{1,2}.md`. In the IMPLEMENTATION_PLAN R0 loop (round-3 scoped convergence).

---

## §0 — Scope

**Decision on R0-round-1 I1: scope the `@N/**` template form IN** (resolution A). `@0/**` is the *canonical*
BIP-388 spelling (BIP-388 defines `/**` in the `@N` placeholder-template context), and the toolkit already
accepts the sibling `@0/<0;1>/*`; rejecting `@0/**` while accepting `xpub/**` would be incoherent.

**IN** — expand a final-use-site `/**` (per §5 precision) to `/<0;1>/*` on every intake surface that rejects it:
1. **Concrete-xpub form** `[fp/path]xpub…/**` via the concrete pipeline (`concrete_keys_to_placeholders` →
   `parse_descriptor`): `import-wallet --format descriptor`, `import-wallet --format bsms` (round-2 plaintext
   line), `bundle --descriptor` (Concrete branch) / `bundle --import-json` replay, `verify-bundle --descriptor`
   (Concrete fork). (`concrete_keys_to_placeholders` also has ~10 callers — coldcard/electrum/specter/sparrow/
   descriptor/bsms — all chokepoint-covered; those formats supply concrete text and gain `/**` support for free.)
2. **`@N`-template form** `@0/**` via the **AtN direct-lex path** — `bundle --descriptor "wpkh(@0/**)"`
   (`bundle.rs:1389` lexes the raw `@N` string directly) and `verify-bundle --descriptor` AtN fork
   (`verify_bundle.rs:1375`). (R0-round-1 I1 — this path bypasses `concrete_keys_to_placeholders`.)
3. **`xpub-search account-of-descriptor --descriptor`** — a STRUCTURALLY SEPARATE parser
   (`descriptor_intake.rs::parse_literal_xpub` → `miniscript::Descriptor::from_str` directly, bypassing the
   lexer). Spike-confirmed (§3) it rejects `/**`.
4. **BSMS `--json` canonicalize** (PLAN-R0 I1) — `import-wallet --format bsms` also feeds the raw BSMS
   descriptor line to `recanonicalize_descriptor` (`wallet_import/roundtrip.rs:231`, `from_str@241`) for the
   `--json` roundtrip/canonical envelope; on `/**` this soft-fails (bogus "canonicalize: parse failed" in the
   envelope, NOT a hard error — main parse already succeeds via `parse_descriptor:875`). Expand there too so
   the canonical field is clean. (Audit the sibling `canonicalize_*` — coldcard/electrum/sparrow/specter — for
   the same class at plan time.)
5. **`export-wallet --descriptor`** (PLAN-R0 I2) — a literal concrete-descriptor user surface that `from_str`s
   the user string directly (`export_wallet.rs:517`) and today HARD-rejects `/**`. It genuinely ACCEPTS the
   explicit `/<0;1>/*` (verified), so expanding `/**` there delivers real acceptance — closing the
   import-accepts/export-rejects asymmetry. One-line expander before `:517`. (Only the CONCRETE form reaches
   `:517`; the AtN `@0/**` stays rejected by the `is_at_n_form` gate `:508` by design.)
6. **`compare-cost --descriptor`** (PLAN-R0 I3 — EQUIVALENCE-only, not acceptance) — `cost/strip.rs:21`
   `from_str`s the user string, but compare-cost has a **PRE-EXISTING multipath limitation**: it rejects ALL
   multipath `/<0;1>/*` with `"multipath key cannot be a DerivedDescriptorKey"` (`translate_descriptor` calls
   `derive_at_index` with NO `into_single_descriptors` split; verified live). So expanding `/**`→`/<0;1>/*`
   there does NOT yield acceptance — it makes `/**` reject **IDENTICALLY** to the explicit `/<0;1>/*` (the
   current cryptic `/**` "invalid child number format" becomes the same multipath error), upholding the
   "`/**` ≡ `/<0;1>/*` on every literal surface" invariant (§6) AND future-proofing: when compare-cost gains
   multipath support, the pre-expanded `/**` works automatically. The pre-existing gap is filed as FOLLOWUP
   `compare-cost-multipath-descriptor-unsupported` (OUT of this cycle).
7. **`gui-schema --classify-descriptor`** — user input, but AUTOMATICALLY chokepoint-covered by the
   `parse_descriptor:875` expander (it calls `parse_descriptor`); no separate touch, add a one-line accept test.
8. **Fold in the BIP-388/BIP-389 misattribution correction** (§2 / §8).

**Scope principle (post-PLAN-R0):** accept a final-use-site `/**` on EVERY surface where a USER supplies literal
descriptor text; leave every surface that parses a TOOLKIT-generated/canonical form (which never contains `/**`)
untouched. The plan's P0 Task 1 enumerates + classifies the complete set.

**OUT (YAGNI):**
1. `import-wallet --format bitcoin-core` — Core NEVER emits `/**` (only split `/0/*`+`/1/*`, verified vs
   bitcoin/bitcoin PR #22838 in the v0.77.0 cycle). Moot.
2. `import-wallet --format specter`/`--format bip388`/Sparrow native-JSON — these ALREADY expand `@N/**` via
   `expand_bip388_policy` (`wallet_import/pipeline.rs:282-302`). Untouched.
3. Rewriting already-tagged v0.76.0/v0.77.0 CHANGELOG history (correct terminology going forward).
4. Any codec change (`UseSitePath` already represents `/<0;1>/*`; this accepts a new SPELLING).
5. `/***`, `/**'`, or any non-exact-`/**` form — only the precise BIP-388 `/**` (§5 precision).

## §1 — Problem & current behavior

Since v0.76.0 (Cycle A) the lexer fail-closed-rejects any fixed/non-multipath residue after a placeholder. A
`/**` (concrete or `@N`) hits this: `lex_placeholders`'s `wild` regex group `/\*(?:'|h)?`
(`parse_descriptor.rs:97-99`) consumes exactly `/*`, leaving a stray `*` that the residue check
(`parse_descriptor.rs:203-213`) refuses (`:206-211`):
> `…a fixed single step like /0/* (or the /** shorthand) is un-representable (found residue near <residue>)`

Separately `xpub-search`'s literal funnel rejects `/**` via miniscript (§3). Both are the correct floor for an
*un-representable* step — but `/**` IS representable (exactly `/<0;1>/*`), so the correct behavior is to
ACCEPT it by expansion, matching the shipped BIP-388-JSON path. `/**` is emitted by common wallets in bare
form (Sparrow descriptor export, Nunchuk, Ledger, Core wallet-policy tooling), so this is a real interop gap.

## §2 — Primary-source basis (verified) + the misattribution correction

**Verified via WebFetch of bip-0389.mediawiki + descriptors.md + BIP-388 (recon §primary-source):**
- `/**` is defined by **BIP-388 ("Wallet Policies")**: *"The `/**` in the placeholder template represents
  commonly used paths for receive/change addresses, and is equivalent to `/<0;1>/*`."*
- **BIP-389 ("Multipath Descriptors")** defines ONLY the explicit `/<0;1>/*`; neither it nor `descriptors.md`
  mentions `/**`.
- The equivalence `/**` ≡ `/<0;1>/*` is CORRECT — only the BIP NUMBER in the FOLLOWUP + shipped source/docs is
  wrong.

**Misattribution + stale-behavior correction sites (R0-round-1 I2 — expanded to the full grep set; all at `0964462d`):**
- `src/parse_descriptor.rs:189` — comment "…or the BIP-389 `/**` shorthand, whose wild group eats only /* and
  leaves a stray *". Both wrong-BIP AND (post-fix) describes a residue path a pre-expanded `/**` no longer
  reaches. Rewrite.
- `src/parse_descriptor.rs:206-211` — the **reject MESSAGE** names `(or the /** shorthand) is un-representable`.
  Post-fix `/**` IS representable → **drop the `(or the /** shorthand)` clause** (message keeps `/0/*` as the
  un-representable exemplar). User-facing (§8).
- `tests/cli_import_wallet_descriptor.rs:159` (+ `:191`) — "BIP-389 combined shorthand `/**`". Correct → BIP-388.
- `src/wallet_import/sparrow.rs:42` — "@N/** cosigner placeholders (BIP-389 multipath shorthand)". Correct → BIP-388.
- `docs/manual/src/40-cli-reference/41-mnemonic.md:145` (+ `:157`) — "BIP-389 combined-wildcard shorthand `/**`".
  Correct → BIP-388. **Do NOT touch `:141`** ("the BIP-389 **multipath** form `/<a;b>/*`" — CORRECT).
- **LEAVE (correct multipath refs):** `41-mnemonic.md:141`, `sparrow.rs:372`, `45-foreign-formats.md:272/307/1038`,
  tech-manual glossary/bibliography, `cli_import_wallet_bitcoin_core.rs`, and the tagged CHANGELOG / `cycleA-*`
  audit records.

## §3 — The xpub-search spike (RESOLVED)

`mnemonic xpub-search account-of-descriptor --phrase <seed> --descriptor "wpkh([b8688df1/84h/0h/0h]xpub…/**)"`
→ **`error: --descriptor parse: at derivation index '**': invalid child number format`** (miniscript's error,
NOT the lexer reject); control `/<0;1>/*` parses fine → search. Confirms `parse_literal_xpub` bypasses the
lexer and rejects `/**` → needs its own touch.

## §4 — Current-source anchor points (grep-verified @ `0964462d`)

| Symbol / site | Location | Role |
|---|---|---|
| `lex_placeholders` | `src/parse_descriptor.rs:60` | reject floor; regex `:97-99` (`wild` `/\*(?:'\|h)?`); residue reject `:203-213` (msg `:206-211` names `/**`; comment `:189`) |
| `substitute_nums_sentinel` (pre-pass precedent) | `src/parse_descriptor.rs:373`, invoked `:875`; `from_str(&substituted)` `:897` | existing string pre-pass at the right position — the model for `expand_literal_double_star` |
| `classify_descriptor_form` / `DescriptorForm{AtN,Concrete}` | `src/wallet_import/pipeline.rs:175-196` | `@\d`-only → AtN; key_regex → Concrete |
| AtN direct-lex (bundle) | `src/cmd/bundle.rs:1389` (`lex_placeholders(&descriptor_str)`) | `@N/**` path — bypasses concrete pipeline (I1) |
| AtN direct-lex (verify-bundle) | `src/cmd/verify_bundle.rs:1375` | `@N/**` path (I1) |
| `concrete_keys_to_placeholders` | `src/wallet_import/pipeline.rs:330-400` (push_str `:391`) | concrete funnel → `parse_descriptor`; ~10 callers |
| `expand_bip388_policy` (prior art) | `src/wallet_import/pipeline.rs:282-302` | ALREADY expands JSON `@N/**` → `/<0;1>/*` |
| `parse_literal_xpub` (xpub-search) | `src/cmd/xpub_search/descriptor_intake.rs:291-298` | direct `miniscript::from_str` `:297`; rejects `/**` (§3) |
| `parse_bip388_json` (xpub-search) | `src/cmd/xpub_search/descriptor_intake.rs:189-199` | delegates to `expand_bip388_policy` — works |
| reject-tests (REPURPOSE) | `parse_descriptor.rs:1731-1738`; `tests/cli_import_wallet_descriptor.rs:191-217` | flip expect-reject → expect-accept |

## §5 — Design (R0-round-1 I1 mechanism reframe + M2 precision)

**Mechanism: a single shared string pre-expander** `expand_literal_double_star(desc: &str) -> Cow<str>`,
applied to the RAW descriptor string at every entry point BEFORE its parser. A `lex_placeholders`-internal
extension is INSUFFICIENT (R0-round-1 I1): `lex_placeholders` returns occurrences, not a string, and
`parse_descriptor` separately feeds the raw string to `MsDescriptor::from_str(&substituted)` (`:897`) which
would still reject `/**`; and `parse_literal_xpub` never touches the lexer at all. So the expander operates on
the STRING, before any parser.

**Call sites (the PLAN must grep-verify this is the COMPLETE minimal set — every consumer of user descriptor
text that reaches `lex_placeholders` OR `MsDescriptor::from_str`):**
1. Top of `parse_descriptor::parse_descriptor` (alongside `substitute_nums_sentinel`, `:875`, before both the
   `lex_placeholders(input)` and `from_str(&substituted)`) — covers the **Concrete** pipeline (all ~10
   `concrete_keys_to_placeholders` callers + `import-json` replay).
2. `bundle.rs:1389` and `verify_bundle.rs:1375` — the **AtN direct-lex** sites that bypass `parse_descriptor`.
   (Or expand `descriptor_str` earlier in each command, before the AtN/Concrete split — PLAN to pick the
   minimal chokepoint; if an earlier per-command chokepoint covers both branches, prefer it over 2 sites.)
3. `parse_literal_xpub` (`descriptor_intake.rs:297`) — before `miniscript::from_str`, for **xpub-search**.

**Coverage (post-fix):** concrete ✅ · AtN template ✅ · xpub-search ✅ · JSON `@N/**` ✅ (unchanged, via
`expand_bip388_policy`).

**Precision (R0-round-1 M2 — funds-adjacent):** rewrite `/**` → `/<0;1>/*` ONLY when the `/**` is a final
use-site step, i.e. immediately followed by the residue-terminator set `)`, `,`, `}`, whitespace (per
`parse_descriptor.rs:204`), `#`, or end-of-string. This EXCLUDES `/***` (next char `*`) and `/**'` (next char
`'`) — those are not `/**` and keep their existing reject. Anchor on the key-expression/terminator boundary,
NEVER a naive global `str::replace("/**", …)`. **"Final use-site step" is per-key/terminator-bounded, NOT
"the last `/**` in the string":** `wsh(sortedmulti(2,K0/**,K1/**))` has TWO `/**`, BOTH must expand (the §7
multisig oracle backstops this). Model on the existing `substitute_nums_sentinel` string pre-pass.

**Idempotence:** `expand_bip388_policy` (JSON) already emits `/<0;1>/*` (no `/**` survives to the new expander →
no-op); a `/**`-free body is unchanged. No double-expansion.

## §6 — Semantics after expansion (funds property)

`…/**` → `…/<0;1>/*` derives receive at chain 0, change at chain 1 (BIP-388-defined). **The invariant:
an expanded `/**` MUST produce output BYTE-IDENTICAL to the explicit `/<0;1>/*` spelling on EVERY surface —
both the successful outputs (descriptor / md1 cards / `--json` / derived addresses, the funds anchor) AND the
error/exit behavior** (e.g. at `compare-cost`, where BOTH forms reject identically via the pre-existing
multipath limitation — §0 item 6). `/**` is a pure synonym for `/<0;1>/*`, never observably different. This
is the §7.3/§7.11 oracle.

## §7 — Test / oracle matrix (TDD-first)

1. **REPURPOSE `lex_rejects_double_star_shorthand` (`parse_descriptor.rs:1731`)** → assert a `/**` body now
   lexes/accepts (expanded), yielding the same occurrences as `/<0;1>/*`.
2. **REPURPOSE `descriptor_double_star_shorthand_rejected_with_multipath_remedy` (`cli_import_wallet_descriptor.rs:191`)**
   → `import-wallet --format descriptor` with `xpub/**` now SUCCEEDS (exit 0), same bundle as `/<0;1>/*`.
3. **ADD equivalence oracle (funds anchor, R0-round-1 M4 — name the spelling):** for `bundle --descriptor`,
   `import-wallet --format descriptor`, `verify-bundle --descriptor`, `xpub-search account-of-descriptor` — a
   **concrete-xpub** `wpkh([fp]xpub/**)` / `wsh(sortedmulti(2,…xpub/**,…xpub/**))` / `tr([fp]xpub/**)` input
   produces BYTE-IDENTICAL output (descriptor / md1 cards / `--json` / derived addresses) to the same input
   written `/<0;1>/*`. The `/<0;1>/*` reference path is pre-existing (not derived from the expander) → non-
   tautological.
4. **ADD AtN-form oracle (R0-round-1 I1/M4):** `bundle --descriptor "wsh(sortedmulti(2,@0/**,@1/**))"` (and
   the `verify-bundle` AtN fork) SUCCEEDS and equals the `@N/<0;1>/*` spelling. (This is the surface the SPEC
   originally missed — it MUST have an explicit cell.)
5. **ADD `xpub-search account-of-descriptor --descriptor "…xpub/**"` parses** (spike's failing case flips to
   the `/<0;1>/*` control behavior).
6. **ADD `import-wallet --format bsms` round-2 descriptor with `/**`** accepts + equals `/<0;1>/*`.
7. **ADD precision guards:** `/***` and `/**'` still reject; a descriptor with no `/**` is a no-op (all
   existing descriptor tests stay green); the multisig two-`/**` case (test #3) proves all keys expand.
8. **ADD regression: specter/bip388-JSON `@N/**` still works** (the `expand_bip388_policy` path untouched).
9. Misattribution: corrected doc-comments/test-names compile; the reworded reject message is asserted by a
   test on a genuinely un-representable step (`/0/*` still rejects, message no longer mentions `/**`).
10. **ADD floor-not-weakened composite (R0-round-2 N1):** `/0/**` → (expanded) `/0/<0;1>/*` STILL rejects —
    the leading fixed `/0` step remains un-representable (Cycle A floor). Proves the `/**` expander does not
    weaken the fixed-step floor for a fixed-step+shorthand combo.
11. **ADD newly-scoped literal-descriptor surfaces (PLAN-R0 I1/I2):**
    - **`export-wallet --descriptor "wpkh([fp]xpub/**)"`** now accepts + emits the same export as the
      `/<0;1>/*` spelling (closes the import-accepts/export-rejects asymmetry).
    - **`compare-cost --descriptor "…xpub/**"`** rejects with the EXACT SAME error + exit as the explicit
      `/<0;1>/*` (both `"multipath key cannot be a DerivedDescriptorKey"` — the pre-existing multipath
      limitation, PLAN-R0 I3 / FOLLOWUP `compare-cost-multipath-descriptor-unsupported`). Equivalence, NOT
      acceptance. (Anti-tautology: assert the `/**` stderr/exit == the `/<0;1>/*` stderr/exit AND != the raw
      `"invalid child number format"` the unexpanded `/**` gave pre-fix.)
    - **`import-wallet --format bsms --json`** with `/**`: the `roundtrip`/canonical envelope field is CLEAN
      (equals the `/<0;1>/*` canonical form), NOT a bogus "canonicalize: parse failed".
    - **`gui-schema --classify-descriptor "…/**"`** classifies (chokepoint-covered) — a one-line accept cell.

Full `cargo test -p mnemonic-toolkit` MUST be green per-phase.

## §8 — Lockstep (docs + message)

- **Reject message (`parse_descriptor.rs:206-211`) + comment (`:189`):** reword per §2 (drop the `/**`
  clause; `/**` is now accepted). This is behavior-lockstep, not just prose.
- **Manual prose (`docs/manual/`):** `41-mnemonic.md` §"Non-representable use-site steps" (`:137-168`,
  authoritative block — now state `/**` is ACCEPTED/expanded, not rejected; incl. `:164` "The `/**` shorthand
  rewrites to the same explicit `/<0;1>/*` form" — semantic rewrite, R0-round-1 M3) + `:218,659,711-713,1204,
  1253-1255,1466,3510` + `45-foreign-formats.md:127-133`. Correct BIP-389→BIP-388 wherever `/**` is named
  (NOT the correct `/<a;b>/*`-is-BIP-389 lines). Re-grep at impl.
- **`CHANGELOG.md`:** new `[0.78.0]` entry (BIP-388); retire the v0.76.0 `/**`-hard-fails "interim" bullet
  narrative (mirror how v0.77.0 retired the Core-split bullet).
- **`verify-examples`:** none affected (`.examples-build/` grep of `/**` = no hits; reject-tests are unit/CLI).
  Re-verify at impl.
- **GUI/`schema_mirror`:** NONE (no clap surface change).

## §9 — Release ritual / SemVer

MINOR → `v0.78.0` (accepting a previously-rejected input form = additive; same shape as v0.77.0). Version
sites (v0.77.0 ritual + Cycle B learnings): `crates/mnemonic-toolkit/Cargo.toml` + workspace `Cargo.lock` +
`fuzz/Cargo.lock` + BOTH READMEs (`<!-- toolkit-version -->`) + **`scripts/install.sh:32` SELF-pin**
(R0-round-1 M1 — path is `scripts/install.sh`, NOT repo-root; and the sibling md/ms/mk pins just below
(`:35`) are a FROZEN baseline — ONLY line 32 bumps, touching the md-cli pin breaks `sibling-pin-check`) +
`.examples-build/gen.sh:44` version-check + embedded gen.sh strings + regen `Examples.md`
(`EXAMPLES_BIN_DIR=…`) + CHANGELOG (tag-gated). Codecs NO-BUMP. Direct-FF + tag.

## §10 — Risks / R0 focus

1. **Match precision (§5/M2)** — rewrite ONLY a terminator-bounded final `/**`, never a stray `**`; all keys
   in a multisig expand. Backstopped by §7.3/§7.4 equivalence oracles + §7.7 precision guards.
2. **Complete call-site set (§5)** — the PLAN must grep-verify every `lex_placeholders` / `MsDescriptor::
   from_str` / `concrete_keys_to_placeholders` consumer of user descriptor text is covered (the "4th path" —
   AtN direct-lex — is now explicitly in §0/§5; the post-impl review greps for a 5th).
3. **Idempotence** with `expand_bip388_policy` (no double-expansion; JSON path untouched).
4. **Equivalence is the funds property** — expanded `/**` ≡ explicit `/<0;1>/*` in every output (§6/§7.3-4).
5. **Reject-message correctness** — the reworded message must still correctly reject genuinely un-representable
   steps (`/0/*`) and no longer mention `/**` (§7.9).

---

*R0 gate: converge to 0C/0I via the opus-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation, per CLAUDE.md.*
