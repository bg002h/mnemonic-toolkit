# SPEC — ms1-repair-demote-to-candidate (F4 Cycle 2 / Cycle F)

**An ms1 seed card is a single-string bearer secret whose BCH SUBSTITUTION-correction spends the checksum's
error-detection budget, so a >4-error "repair" can alias to a DIFFERENT valid seed undetectably. This cycle
DEMOTES every ms1 substitution-correction to an exit-4 VERIFY-ME Candidate — never a silent exit-5 "recovered" —
and, inside `verify-bundle` where the user's TYPED seed is the ground truth, compares the corrected card to that
ground truth (`expected.ms1[i]`) via the existing check architecture: match ⇒ the ms1 checks pass; mismatch ⇒ a
loud failed check row ⇒ exit 4.**

- **Author:** Opus 4.8 (single-author per CLAUDE.md phase-2). **Consult:** Fable architect (Option B). **R0 review:** Fable (per user directive 2026-07-09 "fable for review, opus for fold"). Design decisions user-ratified 2026-07-09.
- **Source SHAs (recon + R0-verified):** toolkit `4c554295` (v0.80.0); ms-codec/ms-cli `mnemonic-secret master@c2fd4eb`; mk-codec `mnemonic-key main@1c9fbf7`.
- **Finding:** constellation-eval **F4**, ms1 leg (`FOLLOWUPS.md` `bch-repair-miscorrection-set-level-reverify`, Status(ms1)). Recon `cycle-prep-recon-f4-cycle2-ms1-demote.md`.
- **Target:** `mnemonic-toolkit` MINOR (`v0.81.0`) + `ms-cli` MINOR (`0.13.2`→`0.14.0`). **ms-codec / mk-codec / md-codec / md-cli / mk-cli NO-BUMP.** No GUI/`schema_mirror` (no clap surface change).
- **Status:** ✅ **R0-GREEN (0C/0I) @ round 3 (Fable).** rev-3 folded R0-round-1 (1C/3I/3M) + round-2 (0C/1I/2M). Reviews `cycleF-spec-r0-round-{1,2,3}.md`. **CLEARED for the IMPLEMENTATION_PLAN + plan-R0.** Two advisory notes for the plan: (1) secret-hygiene at the C1 mismatch check row (echo-vs-redact the seed strings — §8.6); (2) `ms1_decode` `--json` wire-value can now be `pass`-after-auto-repair (consumers self-update).

## §0 — Scope

**IN:**
1. **The substitution-demotion (Option B):** the toolkit `repair_card` **Ms1 arm** (`src/repair.rs:1112-1150`)
   returns `SetVerify::Unverified{reason}` whenever a substitution-correction touched ≥1 position (currently
   hardcoded `Blessed` @:1148). ms1 has no grouping → the rule is **touched ⇒ Unverified**. This alone routes
   `mnemonic repair --ms1` to exit 4 and makes all 9 auto-repair sites NOT short-circuit an ms1 correction — the
   Cycle-E gates are already kind-agnostic (recon §2 / R0-verified: 9 sites, zero per-site edits for the
   never-silently-bless half).
2. **`ms repair` (ms-cli) demotion:** `crates/ms-cli/src/cmd/repair.rs` — demote the binary
   `if any_correction {5} else {0}` (@:123-124) so any correction → **exit 4** (Candidate) + a stderr advisory;
   clean → exit 0; uncorrectable → exit 2 (already wired via `?`→`TooManyErrors`→`FormatViolation`).
3. **Auto-repair fall-through ADVISORY (R0 I2 / M-R2-1):** at the ms1 `Unverified` fall-through for the
   **standalone-inline sites ONLY** (`convert`/`inspect`/`xpub-search` — today a silent `Ok(())` with NO output),
   emit a one-line stderr advisory ("a candidate correction exists but a seed card cannot be self-verified — run
   `mnemonic repair --ms1 …` to inspect it") so the complete-but-withheld candidate is not silently invisible.
   **NOT at the 2 verify-bundle ms1 sites** — there the candidate is surfaced via the `ms1_decode`/
   `ms1_entropy_match` check rows (§0.4), so an advisory would contradict a passing MATCH. Cleanest impl: the
   verify-bundle C1 wiring obtains the corrected string via a direct `repair_card` call, NOT via the
   advisory-emitting `try_repair_and_short_circuit` helper. Kind-gated to ms1 (do NOT change shipped mk1
   partial-set fall-through behavior).
4. **verify-bundle ground-truth comparison (R0 C1 — replaces the derived-xpub oracle):** at the 2 ms1
   auto-repair sites (`verify_bundle.rs` single-sig `:2079`, multisig per-cosigner `:2503`), the user's TYPED
   seed is already synthesized into `expected.ms1[i]` (non-empty by construction wherever these sites fire — the
   target of the adjacent `ms1_entropy_match` check @:2056/:2480). So: feed the auto-corrected ms1 string into
   the EXISTING check comparison against `expected.ms1[i]`. **Match ⇒ the `ms1_decode`/`ms1_entropy_match` checks
   PASS** (the repair recovered the right card, confirmed by the ground truth the user typed — note "recovered
   via auto-repair, confirmed against expected seed"). **Mismatch ⇒ `ms1_entropy_match` FAILS** (loud detail:
   "auto-repair candidate did NOT match the expected seed — this card is not a card for this seed") → the run
   emits the FULL check table + `result: mismatch` → **exit 4** (recon/R0 I3 — do NOT `?`-abort with a typed
   error, do NOT short-circuit). NO xpub derivation, NO mk1 dependency, NO mk1-decode reorder, NO passphrase, NO
   clean-mk1 precondition, NO multisig degrade, NO new `ToolkitError` variant.
5. **Exit-code contract resolution + manual lockstep** across all 4 `40-cli-reference/*.md` chapters — including
   FIXING the pre-existing (post-Cycle-E) staleness where "exit-5 REPAIR_APPLIED consistent across all four
   CLIs" is already false for the mk1-Candidate case, and rewriting `41-mnemonic.md:3056-3059` (currently claims
   ms1 has "no analogous risk"). Plus a `verdict` field in the repair `--json` envelope(s).

**OUT (deferred / YAGNI):**
1. The standalone **`repair --verify-against <mk1|xpub|fingerprint>`** flag — this is the RIGHT home for the
   seed→xpub *derivation* oracle (standalone repair has no `expected`, so a derived xpub vs a supplied companion
   card is genuinely the only available oracle there). Deferred to a follow-on; adds clap surface + oracle
   ranking. This cycle adds NO new flag.
2. Any ms-codec / mk-codec / md* change (existing public API sufficient — recon §3; R0-confirmed NO-BUMP).
3. Retroactively changing `mk repair`'s shipped exit-5+advisory Candidate behavior (user: keep mk as-is).

## §1 — Problem (recon + BIP-93 + R0)

ms1 encodes raw BIP-39 entropy as a SINGLE codex32 string — a bearer secret with no cross-chunk hash, no
fingerprint, no internal redundancy. Bounded-distance BCH SUBSTITUTION-correction guarantees ≤4; beyond that it
can alias to a DIFFERENT valid seed, and a miscorrection *presents as* a small correction (a k=1 apparent
correction can be a ≥8-true-error input — bound-able, never distinguishable). Auto-repair fires by default on
TTY intake → corroded ms1 → silent WRONG SEED. Highest-consequence F4 leg (mk1/md1 shipped v0.80.0 — they have
a reassembly oracle; ms1 does not). BIP-93: "implementations SHOULD NOT automatically proceed with a corrected
codex32 string without user confirmation." **Note (R0 I1): the *indel* recovery path is different** — it
enumerates candidates and re-checks the FULL checksum (does NOT spend the correction budget), so a UNIQUE
full-checksum indel candidate is a genuine self-verification (see §3).

## §2 — The substitution-demotion (Option B) — touched ⇒ Unverified

- **Ms1 arm** (`repair.rs:1148`): return `SetVerify::Unverified{reason}` iff `!repairs.is_empty()`, else
  `Blessed`. `reason` = ms1-specific ("a corrected seed card cannot be self-verified — confirm the derived
  address/xpub against a known-good copy before use; BIP-93 recommends confirming a corrected codex32 string").
  No new type. **R0-verified:** `repairs` non-empty ⇔ ms-codec returned corrections ⇔ residue≠0 (+ ms-codec's
  defensive post-correction re-verify) → no false-Bless-with-touch; a clean decode (0 corrections) → `Blessed` →
  exit 0 unchanged.
- **Consequence, for free (R0-verified kind-agnostic gates):** `mnemonic repair --ms1` → `candidate_seen` @:164
  → `indel_exit_code(…||candidate_seen,…)` @:244 → **exit 4** + advisory (zero `cmd/repair.rs` change). All 9
  `try_repair_and_short_circuit` sites fall through (no silent apply). Clean → exit 0; uncorrectable →
  `TooManyErrors` → exit 2.

## §3 — verify-bundle ground-truth comparison + the indel carve-out

**Substitution path (§0.4):** the corrected ms1 is compared to `expected.ms1[i]` (byte-equal). This is strictly
stronger than any derived-xpub oracle (it pins the EXACT expected card — no aliasing window) and folds into the
existing `ms1_decode`/`ms1_entropy_match` check rows. **Match ⇒ checks pass** (repair confirmed by the typed
seed). **Mismatch ⇒ `ms1_entropy_match` fails, full table, exit 4.** No derivation, no short-circuit, no typed
error. `expected.ms1[i]` is present by construction wherever the site fires (a watch-only slot skips the site),
so there is no "no ground truth available" branch to handle — a decisive simplification over the rev-1 oracle.

**Indel path (R0 I1/I-R2-1 — carve-out, keep exit-5, justified; `mnemonic repair --max-indel` ONLY):** indel
recovery is reachable ONLY from `mnemonic repair --max-indel≥1` (`cmd/repair.rs:64,170`). `ms repair` (ms-cli
`RepairArgs={ms1,json}`) has NO indel flag, and NO auto-repair inline site (convert/inspect/xpub/verify-bundle)
has indel plumbing — they route through `repair_card` (substitution-only); a length-corrupted ms1 hits
`Err(_)→Ok(())` fall-through, never indel. So this exit-5 carve-out is scoped to `mnemonic repair --ms1` alone
(the §4 table marks the other three surfaces `n/a — no indel path`). `--max-indel≥1` routes length-mismatched
ms1 through the indel recovery (`cmd/repair.rs:169-224`), which enumerates candidates and RE-VALIDATES the FULL
BCH checksum on each (does NOT spend the correction budget), returning exit 5 only for a UNIQUE checksum-valid
candidate
(multi-hit → `Ambiguous` → exit 4). A unique full-checksum indel candidate is trustworthy to ~2⁻(checksum bits)
— cryptographically stronger than the 32-bit cross-chunk hash on which we ALREADY bless mk1/md1 — so it is a
genuine self-verification, NOT the spend-the-checksum aliasing the substitution demotion targets. **Decision:
keep ms1 indel-`Unique` at exit 5** with this documented justification (add a §4 indel row + a pinning test +
the manual explanation). *(Coordinator note: the conservative alternative — demote indel to exit-4 too, uniform
"touched⇒candidate" — is a one-line kind-branch in the indel exit-code path; flagged to the user for optional
override. Default = keep-5-justified.)*

## §4 — Exit-code contract (RESOLVED — user 2026-07-09; R0-corrected)

| Surface | clean | substitution correction | corrected==expected (verify only) | corrected≠expected (verify only) | unique full-checksum indel | uncorrectable |
|---|---|---|---|---|---|---|
| `ms repair` | exit 0 | **exit 4 + advisory** | n/a | n/a | n/a — no indel path | exit 2 |
| `mnemonic repair --ms1` | exit 0 | **exit 4 + advisory** | n/a | n/a | exit 5 (§3 indel; `--max-indel` only) | exit 2 |
| toolkit auto-repair (convert/inspect/xpub) | pass | no short-circuit + **stderr advisory** (I2) | n/a | n/a | n/a — no indel path | error |
| **verify-bundle** ms1 | pass | ms1 checks evaluated vs expected ↓ | **ms1 checks PASS** (verify proceeds) | **`ms1_entropy_match` FAILS → exit 4** (full table) | n/a — no indel path | error |
| `mk repair` (unchanged, Cycle E) | exit 0 | exit 5 + advisory | — | — | — | exit 2 |

**Principled distinction (codify in manual — R0 M2):** exit-5 "REPAIR_APPLIED" = **verified now** (mk1/md1
reassembly hash; unique full-checksum indel) **OR verifiable-by-reassembly later** (mk1 single-plate). **exit-4
VERIFY-ME** = a bounded-distance SUBSTITUTION correction that spent the checksum and has no self-oracle. Do NOT
phrase exit-5 as "an oracle verified it" (false for mk1 single-plate).

## §5 — Test / oracle matrix (TDD-first)
1. **(FUNDS ANCHOR) `mnemonic repair --ms1 <subst-corrupted>` → exit 4 + advisory**, NOT exit 5; corrected seed
   presented as candidate. Clean ms1 → exit 0.
2. **`ms repair <subst-corrupted>` → exit 4 + advisory**; clean → 0; uncorrectable (`TooManyErrors`) → 2.
3. **auto-repair (convert/inspect/xpub) on a corrected ms1 does NOT short-circuit** (no silent apply) AND emits
   the stderr advisory (I2) — drive default-TTY (`MNEMONIC_FORCE_TTY`).
4. **(C1 — MATCH) verify-bundle** with a subst-corrupted ms1 whose correction == `expected.ms1[i]` (the typed
   seed's own card) → `ms1_decode`+`ms1_entropy_match` PASS (verify proceeds), noted as auto-repair-recovered.
5. **(C1 — MISMATCH, FUNDS ANCHOR — the wrong-bundle attack) verify-bundle** `--slot @0.phrase="<seed E>" --ms1
   <corroded→corrects to wallet A's ms1> --mk1 <clean mk1 A> --md1 <md1 A>` → the corrected ms1 (wallet A) ≠
   `expected.ms1` (seed E) → **`ms1_entropy_match` FAILS → exit 4**, full check table emitted, NOT "recovered",
   NO exit-5, NO exit-2 abort. (Pins that C1's ground-truth compare closes the rev-1 false-Bless.)
6. **(I1 indel) `mnemonic repair --ms1 <single-indel, unique checksum-valid>` → exit 5** (kept); a multi-hit
   indel → `Ambiguous` → exit 4.
7. **(M3 mixed-kind) `mnemonic repair --ms1 <corrupted> --mk1 <clean>` → exit 4 dominates** (candidate OR-fold).
8. **`--no-auto-repair`** suppresses the auto path identically (no advisory, no compare).
9. **`--json` verdict** field present + correct per §6 (M1) on the reachable envelope(s).
10. Determinism; full `cargo test -p` green in both touched repos.

## §6 — Manual lockstep (all 4 chapters + JSON)
- Rewrite the blanket "exit-5 `REPAIR_APPLIED` consistent across all four CLIs" sentence (`41-mnemonic.md`
  @:750/:818-820, `42-md.md:334`, `43-ms.md:360`, `44-mk-cli.md:239`) → the §4 principled model, phrased per M2
  ("verified now, or verifiable-by-reassembly later" — NOT "an oracle verified it"), covering the shipped
  mk1-Candidate exit-4 case AND the new ms1 semantics. Correct the per-kind auto-fire tables (`41-mnemonic.md:
  818-820`) whose "Auto-fire (exit 5 + repair report)" rows are false for ms1 (I2 — now: no short-circuit +
  advisory).
- Rewrite `41-mnemonic.md:3056-3059` ("ms1 … no analogous risk") → ms1 has a WORSE, undetectable variant of the
  substitution-miscorrection risk, demoted to exit-4 Candidate standalone, confirmed against the typed seed
  inside verify-bundle; the indel path stays exit-5 (full-checksum self-verify).
- `43-ms.md` `ms repair` chapter: exit 0/4/2 + the advisory + the no-self-verification caveat.
- verify-bundle chapter: the ms1 corrected-vs-expected comparison (pass on match, `ms1_entropy_match` fail →
  exit 4 on mismatch).
- **`--json` `verdict` (M1/M-R2-2):** specify the envelope(s) — standalone `emit_repair_json` (`cmd/repair.rs:279`,
  `RepairJson`) has reachable `{blessed(clean 0-corr), candidate(subst)}` for ms1 (ms1 reject is not an envelope
  — it's a verify-bundle check-row). **Indel does NOT emit via `RepairJson`** — it uses the separate `IndelJson`
  envelope (`cmd/repair.rs:350-365`, `confident:bool`, no `verdict`), so do NOT attribute an indel value to
  `RepairJson.verdict`. Add `verdict: "blessed"|"candidate"` to `RepairJson`; note wire-shape (consumers
  self-update, not schema_mirror-gated). Regen any ms1 repair transcript whose stderr/exit changes.

## §7 — Cross-repo coordination + release
- **Changes:** toolkit (`repair.rs` Ms1 arm + fall-through advisory + `verify_bundle.rs` corrected-vs-expected
  wiring + `cmd/repair.rs` json verdict + manual) + ms-cli (`cmd/repair.rs` exit-4 demotion + advisory).
  **ms-codec/mk-codec/md* NO source change.** **No new `ToolkitError` variant** (R0 I3 — mismatch is a check row).
- **SemVer:** toolkit MINOR `v0.81.0`; ms-cli MINOR `0.13.2`→`0.14.0`. Codecs NO-BUMP.
- **Release order:** ms-cli first (bump + tag `ms-cli-v0.14.0` + **manual `cargo publish -p ms-cli`**, user-gated
  post-tag; ms-codec stays 0.7.0), then toolkit v0.81.0 advancing the ms-cli sibling pin.
- **ms-cli sibling-pin advance — 4 sites (recon §7):** `scripts/install.sh:38`, `.github/workflows/
  {quickstart.yml:87, technical-manual.yml:117, manual.yml:90}` (`ms-cli-v0.13.2`→`v0.14.0`). `manual-gui.yml:165`
  (GUI cadence) OUT of scope. Toolkit self-pin `install.sh:32` → v0.81.0. g6 mlock job reads install.sh
  dynamically — no extra edit.
- **Toolkit version sites:** Cargo.toml + workspace/fuzz Cargo.lock + both READMEs + install.sh self-pin +
  `.examples-build` corpus + CHANGELOG `[0.81.0]` + FOLLOWUPS ms1-leg → RESOLVED. NO re-vendor (ms-codec pin
  `"0.7"` already satisfied, no dep move). Verify sibling-pin-check + install-pin-check + examples green.
- **GUI:** no companion (no clap surface). GUI's own toolkit pin independently stale (v0.70.0) — unaffected.

## §8 — Risks / R0 focus
1. **C1 ground-truth compare** — confirm `expected.ms1[i]` is ALWAYS present (non-empty) wherever the 2 ms1
   sites fire (watch-only slots skip) so there is no un-handled no-ground-truth branch; the corrected string is
   compared to the EXPECTED (typed-seed) card, never to another supplied card. §5.5 pins the wrong-bundle attack.
2. **Mismatch is a failed check row → exit 4 (full table), NOT a `?`-abort/typed-error/short-circuit** (I3) —
   the `--json` envelope + remaining rows must still emit.
3. **Indel carve-out (I1)** — keep exit-5 ONLY for a UNIQUE full-checksum indel candidate; multi-hit →
   Ambiguous → exit 4; document the substitution-vs-indel trust distinction. (User may override to uniform
   demote.)
4. **Silent fall-through advisory (I2)** — kind-gated to ms1; does NOT alter shipped mk1 partial-set behavior;
   the auto-fire manual tables updated to match.
5. **Exit-code parity manual rewrite (M2)** — correct for ALL four CLIs incl. the shipped mk1 asymmetry; do not
   phrase exit-5 as "oracle-verified".
6. **No key-material leakage** in the advisory / check-row detail / `--json` verdict
   ([[feedback_secret_hygiene_first_class_bar]]).
7. **ms-codec truly NO-BUMP** — no signature/behavior change.

---
*R0 gate: converge to 0C/0I via the Fable-architect loop (persisted to `design/agent-reports/`) BEFORE any
implementation; Opus folds. Per CLAUDE.md + user directive 2026-07-09.*
