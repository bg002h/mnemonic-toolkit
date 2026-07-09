# SPEC — ms1-repair-demote-to-candidate (F4 Cycle 2 / Cycle F)

**An ms1 seed card is a single-string bearer secret with NO second oracle, so a >4-error BCH "repair" can alias
to a DIFFERENT valid seed with zero way to detect it. This cycle DEMOTES every ms1 correction to an
unverifiable Candidate — never a silent exit-5 "recovered" — and adds a verify-bundle AUTO-ORACLE that, when the
bundle's own mk1 xpub card is present and clean, derives the xpub from the corrected seed and compares it to the
mk1 to either promote the repair to a confident Bless or reject it as a wrong-fit.**

- **Author:** (this session) — single-author per CLAUDE.md phase-2. **Consult:** Fable architect (conservativeness = Option B + verify-bundle auto-oracle; exit-4 for the no-oracle standalone case) — user-ratified 2026-07-09.
- **Source SHAs (recon-verified):** toolkit `4c554295` (v0.80.0); ms-codec/ms-cli `mnemonic-secret master@c2fd4eb`; mk-codec `mnemonic-key main@1c9fbf7`.
- **Finding:** constellation-eval **F4**, ms1 leg (`design/FOLLOWUPS.md` `bch-repair-miscorrection-set-level-reverify`, Status(ms1)). Recon `cycle-prep-recon-f4-cycle2-ms1-demote.md`.
- **Target:** `mnemonic-toolkit` MINOR (`v0.81.0`) + `ms-cli` MINOR (`0.13.2`→`0.14.0`). **ms-codec / mk-codec / md-codec / md-cli / mk-cli NO-BUMP** (existing public API reused). No GUI/`schema_mirror` (no clap-surface change).
- **Status:** DRAFT — pending opus-architect R0 loop to 0C/0I BEFORE any implementation (CLAUDE.md).

## §0 — Scope

**IN:**
1. **The demotion (Option B):** the toolkit `repair_card` **Ms1 arm** (`src/repair.rs:1112-1150`) returns
   `SetVerify::Unverified{reason}` whenever the correction touched ≥1 position (currently hardcoded `Blessed`
   @:1148). ms1 has no grouping/reassembly concept → the rule is simply **touched ⇒ Unverified** (no
   Reject-vs-Candidate split). This alone routes `mnemonic repair --ms1` to exit 4 and makes all 9 auto-repair
   sites NOT short-circuit an ms1 correction — the Cycle-E gates are already kind-agnostic (recon §2).
2. **`ms repair` (ms-cli) demotion:** `crates/ms-cli/src/cmd/repair.rs` — demote the binary
   `if any_correction {5} else {0}` (@:123-124) so any correction → **exit 4** (Candidate) + a stderr advisory;
   clean → exit 0; uncorrectable → exit 2 (already wired via `?`→`TooManyErrors`→`FormatViolation`).
3. **verify-bundle AUTO-ORACLE (the novel piece):** at the 2 ms1 auto-repair sites in `verify_bundle.rs`
   (single-sig `:2079`, multisig per-cosigner `:2503`), when the corresponding mk1 xpub card is present AND
   decodes CLEANLY (zero repair), derive the xpub from the corrected ms1 seed and compare to the mk1's xpub → 
   **match promotes to Bless (exit 5)** / **mismatch → Reject (exit 2, new typed error)**; when no clean mk1 is
   available, **degrade to Candidate** (no promotion, the pre-existing `ms1_decode`/entropy-match check surfaces,
   exit-4-equivalent in the verify flow). Uses `bitcoin::bip32` + `mk_card.origin_path` (recon §5).
4. **Exit-code contract resolution + manual lockstep** across all 4 `40-cli-reference/*.md` chapters — including
   FIXING the pre-existing (post-Cycle-E) staleness where "exit-5 REPAIR_APPLIED consistent across all four
   CLIs" is already false for the mk1-Candidate case, and rewriting `41-mnemonic.md:3056-3059` (which currently
   claims ms1 has "no analogous risk"). Plus a `verdict` field consideration in the repair `--json` envelope.

**OUT (deferred / YAGNI):**
1. The standalone **`repair --verify-against <mk1|xpub|fingerprint>`** flag (user-deferred to a follow-on;
   would add clap surface + oracle-ranking). This cycle's oracle is verify-bundle-INTERNAL only (no new flag).
2. Any ms-codec / mk-codec / md* change (existing public API sufficient — recon §3).
3. Retroactively changing `mk repair`'s shipped exit-5+advisory Candidate behavior (user: keep mk as-is).
4. Reworking the `error-rs-retroactive-alphabetical-sort` backlog (only the ONE new variant lands alphabetically).

## §1 — Problem (recon + BIP-93)

ms1 encodes raw BIP-39 entropy as a SINGLE codex32 string — a bearer secret with **no cross-chunk hash, no
fingerprint, no internal redundancy**. The BCH regular code guarantees correction of ≤4 substitutions; beyond
that, bounded-distance decoding can alias to a DIFFERENT valid codeword = a different 80-bit seed, with **zero
downstream signal it is wrong**. And a miscorrection *presents as* a small correction (a k=1 apparent
correction can be a ≥8-true-error input — bound-able, never distinguishable — Fable consult). Toolkit
auto-repair fires by default on TTY `convert`/`inspect`/`verify-bundle`/`xpub-search` intake. So a corroded ms1
→ silent WRONG SEED → funds to/from the wrong wallet. This is the highest-consequence, hardest-to-detect leg of
F4 (mk1/md1 shipped in v0.80.0 — they HAVE a reassembly oracle; ms1 does not). BIP-93 normative: "implementations
SHOULD NOT automatically proceed with a corrected codex32 string without user confirmation."

## §2 — The demotion (Option B) — touched ⇒ Unverified

- **Ms1 arm** (`repair.rs:1112-1150`, the `repair_via_ms_codec` result @:1148): return
  `SetVerify::Unverified{ reason }` iff `!repairs.is_empty()`, else `Blessed` (a genuinely clean decode with
  zero corrections still passes exit 0). `reason` = an ms1-specific text distinct from mk1's ("a corrected seed
  card cannot be self-verified — confirm the derived address/xpub against a known-good copy before use; BIP-93
  recommends confirming a corrected codex32 string"). No new type — `SetVerify::Unverified{reason:String}`
  (`:459-465`) already exists.
- **Consequence, for free (recon §2, kind-agnostic gates — verify in R0, do NOT assume):**
  - `mnemonic repair --ms1 <corrupted>`: `cmd/repair.rs::run` @:164-167 sets `candidate_seen` on any
    `Unverified` → `indel_exit_code(ambiguous_seen||candidate_seen, …)` @:244 → **exit 4** + advisory. Zero
    `cmd/repair.rs` change.
  - All 9 `try_repair_and_short_circuit` sites (`repair.rs:1675-1707`): the gate falls through
    (`!matches!(set_verify, Blessed)`) → an ms1 correction is NOT auto-applied/short-circuited. Zero per-site
    change for the "never silently bless" half.
- **Clean/uncorrectable unchanged:** clean decode (0 corrections) → Blessed → exit 0 / auto-pass;
  uncorrectable → `ms_codec` `TooManyErrors` → existing typed error → exit 2.

## §3 — verify-bundle auto-oracle

**Principle:** ms1 can borrow an oracle from the bundle's own mk1 xpub card. Only in `verify_bundle`, only when
a CLEAN mk1 is in hand.

**Derivation + compare (prior art: `emit_full_path_parent_fingerprint_check`, `verify_bundle.rs:3239-3429`):**
decode the corrected ms1 → `Payload::Entr|Mnem` (wire-language-wins) → `bip39::Mnemonic` → `.to_seed(passphrase)`
→ `Xpriv::new_master` → `master.derive_priv(&secp, &mk_card.origin_path)` → compare the FULL resulting `Xpub`
(depth+chaincode+pubkey) to `mk_card.xpub`. `mk_card.origin_path` (`mk_codec::KeyCard`, already public) is the
exact path — NO md1, NO account guess. Full-xpub compare is the primary oracle (fingerprint is an optional
cheaper pre-check; may be omitted in privacy mode → don't rely on it).

**Verdict wiring at the 2 sites:**
- **Precondition (oracle engages) — ALL of:** the corresponding mk1 is supplied AND `mk_codec::decode`s
  **cleanly with zero repair** (a mk1 that itself needed repair is only a Candidate → do NOT chain two unverified
  corrections; recon §5 hard-part 2). Else → **no oracle** → degrade to Candidate.
- **Oracle result:** derived xpub == mk1.xpub → **Bless** (the ms1 repair is confidently correct → the ms1
  auto-repair may short-circuit/pass as recovered, exit-5-equivalent in verify flow). Mismatch → **Reject**: a
  NEW `ToolkitError` variant (alphabetically placed, e.g. `Ms1OracleXpubMismatch`) → exit 2, message naming the
  mismatch (do NOT leak key bytes).
- **Single-sig site (`:2079`):** mk1 currently decodes AFTER the ms1 block (`:2107`). Restructure: decode mk1
  up-front (cheap; harmless if done speculatively) so the oracle has it, WITHOUT changing the emitted `checks`
  Vec push ORDER (schema-relevant — ms1 rows before mk1 rows must stay).
- **Multisig site (`:2503`):** `card_for_cosigner` is already decoded before the ms1 loop → the cosigner's mk1
  `KeyCard` is in scope; no reorder. `MappingFailure::NotSupplied`/partial-cosigner → no oracle → Candidate
  (explicit branch, no unwrap/panic — recon §5 hard-part 3).
- **`--no-auto-repair`:** the oracle path respects it identically (skip entirely — recon §5 hard-part 5).
- **Passphrase (recon §5 hard-part 1):** thread `args.passphrase` into the derivation exactly as the prior-art
  fn does. Document the limitation: the oracle proves match/mismatch against the SUPPLIED passphrase only; a
  right-seed/wrong-passphrase derives a different plausible xpub → reads as mismatch (a false Reject is
  fail-safe here — it refuses rather than blessing a possibly-wrong seed; but the message should hint "verify
  the passphrase too").

## §4 — Exit-code contract (RESOLVED — user 2026-07-09)

| Surface | clean (0 corr) | correction, no oracle | correction, oracle MATCH | correction, oracle MISMATCH | uncorrectable |
|---|---|---|---|---|---|
| `ms repair` (standalone) | exit 0 | **exit 4 + advisory** (Candidate) | n/a (no oracle standalone) | n/a | exit 2 |
| `mnemonic repair --ms1` | exit 0 | **exit 4 + advisory** (Candidate) | n/a | n/a | exit 2 |
| toolkit auto-repair (convert/inspect/xpub) | pass | no short-circuit → Candidate surfaces | n/a (no bundle mk1) | n/a | error |
| **verify-bundle** ms1 auto-repair | pass | Candidate (no clean mk1) | **Bless / exit-5-equiv** | **Reject / exit 2** | error |
| `mk repair` (unchanged, Cycle E) | exit 0 | exit 5 + advisory | — | — | exit 2 |

**Principled distinction (codify in the manual):** exit-5 "REPAIR_APPLIED" = **confidently recovered** (has an
oracle: mk1/md1 reassembly, or the verify-bundle xpub oracle); **exit-4 VERIFY-ME** = a correction that CANNOT
be self-verified and needs external confirmation. `mk repair`'s single-plate exit-5+advisory is defensible (mk1
CAN be reassembled later to self-verify); ms1 standalone has NO path → exit 4. This makes the current blanket
"exit-5 consistent across all four CLIs" sentence (all 4 chapters) obsolete → rewrite it (§6).

## §5 — Test / oracle matrix (TDD-first)
1. **(FUNDS ANCHOR) `mnemonic repair --ms1 <corrupted>` → exit 4 + advisory**, NOT exit 5; the corrected seed is
   presented as a candidate, not "recovered". A clean ms1 → exit 0.
2. **`ms repair <corrupted>` → exit 4 + advisory**; clean → exit 0; uncorrectable (>guaranteed, `TooManyErrors`)
   → exit 2.
3. **auto-repair (convert/inspect/xpub) on a corrected ms1 does NOT short-circuit** (no silent apply) — drive
   the default-TTY path (`MNEMONIC_FORCE_TTY`).
4. **(ORACLE — match) verify-bundle** with a clean mk1 + a lightly-corrupted ms1 whose corrected seed derives to
   the mk1's xpub → the ms1 repair is Blessed (verify proceeds/exit-5-equiv), NOT left Candidate.
5. **(ORACLE — mismatch, FUNDS) verify-bundle** with a clean mk1 + an ms1 corrupted such that the correction
   yields a seed deriving to a DIFFERENT xpub → **Reject** (`Ms1OracleXpubMismatch`, exit 2), NOT blessed.
   (Construct via a pinned seed whose correction genuinely mis-derives — or, more simply, pair a clean mk1 for
   wallet A with a corrupted ms1 for wallet B.)
6. **(ORACLE — no oracle) verify-bundle** where mk1 ITSELF needed repair (Candidate) → oracle does NOT engage →
   ms1 stays Candidate (no false Bless from chaining two unverified corrections).
7. **(ORACLE — multisig partial) verify-bundle multisig** with a cosigner whose mk1 is NotSupplied → that
   cosigner's ms1 correction degrades to Candidate (no panic).
8. **`--no-auto-repair`** suppresses the oracle path identically.
9. **checks-Vec push order unchanged** in the single-sig path despite the mk1-decode reorder (schema-stable).
10. Determinism; full `cargo test -p` green in both touched repos.

## §6 — Manual lockstep (all 4 chapters + JSON)
- Rewrite the blanket "exit-5 `REPAIR_APPLIED` is consistent across all four CLIs" sentence (`41-mnemonic.md`
  auto-fire tables @:750/:818-820, `42-md.md:334`, `43-ms.md:360`, `44-mk-cli.md:239`) → the §4 principled
  model (exit-5 = confidently recovered; exit-4 = candidate needing external verification), covering BOTH the
  already-shipped mk1-Candidate exit-4 case AND the new ms1 semantics.
- Rewrite `41-mnemonic.md:3056-3059` ("ms1 … no partial-set concept applies" / "no analogous risk") → ms1 has a
  WORSE, fully-undetectable variant of the miscorrection risk, closed for the bundle case by the new xpub
  oracle, left as exit-4 Candidate for the standalone case.
- `43-ms.md` `ms repair` chapter: document exit 0/4/2 + the advisory + the no-self-verification caveat.
- verify-bundle chapter: document the ms1 xpub auto-oracle (Bless on match, Reject on mismatch, Candidate when
  no clean mk1).
- `--json` repair envelope: add a `verdict` field (`"blessed"|"candidate"|"reject"`) so a consumer keying on
  stdout can distinguish (not gated by schema_mirror — wire-shape; note consumers self-update). Confirm no
  verify-examples golden churns beyond intended (regen the ms1 repair transcripts if their stderr/exit changes).

## §7 — Cross-repo coordination + release
- **Changes:** toolkit (`repair.rs` Ms1 arm + `verify_bundle.rs` oracle + new `ToolkitError` variant + manual) +
  ms-cli (`cmd/repair.rs` exit-4 demotion + advisory). **ms-codec/mk-codec/md* NO source change.**
- **SemVer:** toolkit MINOR (`v0.81.0`); ms-cli MINOR (`0.13.2`→`0.14.0`). Codecs NO-BUMP.
- **Release order:** ms-cli first (bump + tag `ms-cli-v0.14.0` + **manual `cargo publish -p ms-cli`**, user-gated
  post-tag — ms-codec stays 0.7.0, already published), then toolkit v0.81.0 advancing the ms-cli sibling pin.
- **ms-cli sibling-pin advance — 4 sites (recon §7 gotcha, NOT single-sourced):** `scripts/install.sh:38`,
  `.github/workflows/{quickstart.yml:87, technical-manual.yml:117, manual.yml:90}` (`ms-cli-v0.13.2`→`v0.14.0`).
  `manual-gui.yml:165` (ms-cli-v0.13.0) is the GUI's own cadence — OUT of scope, do NOT touch. Toolkit self-pin
  (`install.sh:32` mnemonic-toolkit-v0.80.0→v0.81.0). g6 mlock job reads install.sh dynamically — no extra edit.
- **Toolkit version sites:** Cargo.toml + workspace/fuzz Cargo.lock + both READMEs + install.sh self-pin +
  `.examples-build` corpus + CHANGELOG `[0.81.0]` + FOLLOWUPS ms1-leg → RESOLVED. NO re-vendor (no dep-pin move;
  ms-codec pin `"0.7"` already satisfied). Verify sibling-pin-check + install-pin-check + examples green.
- **GUI:** no companion (no clap surface). GUI's own toolkit pin independently stale (v0.70.0) — unaffected.

## §8 — Risks / R0 focus
1. **The "kind-agnostic gates" claim (recon §2) is load-bearing — R0 must VERIFY** the Ms1-arm flip actually
   routes exit-4 + no-short-circuit at all 9 sites without per-site edits (don't take on faith).
2. **The oracle must NEVER promote to Bless on a chained/weak comparison** — mk1-itself-repaired, multisig
   partial, privacy-mode-no-fingerprint → all must degrade to Candidate, never Bless. §5.6/§5.7 pin it.
3. **Mismatch is a hard Reject (exit 2), not a soft Candidate** — a corrected seed that derives to the WRONG
   xpub is provably wrong; refusing is fail-safe. But a wrong PASSPHRASE also reads as mismatch → the message
   must not imply "the seed is definitely wrong" (§3 passphrase note).
4. **checks-Vec push-order / schema stability** in the single-sig reorder (§5.9).
5. **Exit-code parity claim** — the manual rewrite must be correct for ALL four CLIs incl. the already-shipped
   mk1 asymmetry, not just bolt on ms1.
6. **No key-material leakage** in the new Reject message or `--json` verdict; secret-hygiene bar
   ([[feedback_secret_hygiene_first_class_bar]]) — the corrected seed / derived xpriv are secret-adjacent.
7. **ms-codec truly NO-BUMP** — confirm no signature/behavior change sneaks in.

---
*R0 gate: converge to 0C/0I via the opus-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation, per CLAUDE.md.*
