# RECON — Minors + Docs tail (constellation-eval 2026-07-06)

**Scope:** M2 (SLIP-39 message), M3 (BIP-85 Portuguese), M4 (BSMS exit code), D1 (md1 BIP
framing) — the eval's Minor/Doc tier — plus FOLLOWUPS.md `gui-manual-repair-exit-code-lockstep`
(the "Docs item"). Read-only recon; nothing changed. Source: `design/agent-reports/constellation-eval-2026-07-06.md`.

Current versions at recon time: toolkit `0.84.0`, md-codec `0.41.0` / md-cli `0.12.0`,
ms-codec `0.7.0` / ms-cli `0.14.1`, mk-cli `0.12.1`.

The eval's own proposed "Cycle G — Minor advisories (M2/M3/M4) + D1" was **never executed under
that name**; the toolkit's actual v0.82.0 "Cycle G" was an unrelated batch (compare-cost-multipath
+ repair-engine-outcome-zeroization). No FOLLOWUPS.md entry tracks M2/M3/M4 by name or by their
distinctive error strings — confirmed via grep. D1 turned out to be resolved as an
incidental side-effect of the separate 2026-07-10 cross-repo BIP-alignment cycle.

---

## M2 — SLIP-39 `combine` error text falsely implies a wrong passphrase is detectable

**Verbatim finding:**
> M2 — SLIP-39 `combine` error text falsely implies a wrong passphrase is detectable.
> `toolkit:crates/mnemonic-toolkit/src/cmd/slip39.rs`. Per SLIP-0039 the digest is interpolated from
> *pre-decryption* shares — "there is no way to verify that the correct passphrase was used" — so a mistyped
> passphrase exits 0 and prints a *different* valid-looking master. A user who reads "no digest error =
> passphrase verified" could retire the real backup. **Fix:** correct the message (don't imply passphrase
> verification); keep the existing "verify a derived address" note.

**Repo/file:** toolkit, `crates/mnemonic-toolkit/src/cmd/slip39.rs`.

**Still a real gap?** YES, confirmed live at `crates/mnemonic-toolkit/src/cmd/slip39.rs:757`:
```rust
DigestVerificationFailed => "slip39 combine: reconstructed master digest mismatch — wrong --passphrase OR a share was substituted".to_string(),
```
SLIP-0039's digest check runs on the Feistel-encrypted master secret (EMS) *before* passphrase-based
decryption (confirmed against SLIP-0039 spec structure), so `DigestVerificationFailed` can **never**
actually be caused by a wrong passphrase — only by a substituted/corrupted/mismatched-secret share.
Listing "wrong --passphrase" as a cause is factually false and, worse, implies the *absence* of this
error verifies the passphrase, which it does not. The pre-existing mitigating note ("verify the
recovered wallet's expected derived address before trusting", line 683) is unconditional on every
successful combine — unaffected by this fix, exactly as the eval's fix direction assumes.

**Locus:**
- `crates/mnemonic-toolkit/src/cmd/slip39.rs:757` (the message itself, inside `map_slip39_error`).
- `crates/mnemonic-toolkit/tests/cli_slip39_refusals.rs:336` — asserts the exact current string; must
  update in lockstep.
- `docs/manual/src/40-cli-reference/41-mnemonic.md:2194` — mirrors the identical string in the
  combine-errors table; mirror invariant requires updating in the same PR.

**Proposed fix + effort:** rewrite the message to state the digest check only catches share
substitution/corruption, and explicitly does NOT verify the passphrase (e.g. "…digest mismatch — a
share was substituted, corrupted, or belongs to a different secret (this check runs before passphrase
decryption and does NOT verify --passphrase)"). ~4 files touched, ~10-15 LOC total (1 source line +
1 test assertion + 1 manual table row + doc-comment note). No GUI hit (`grep` of `mnemonic-gui/src`
and `docs/manual-gui` for the string returns nothing — no GUI lockstep). **Repo:** toolkit only.
**SemVer:** PATCH (message/doc correctness; zero behavior/exit-code change).

**Decision needed?** **NO — clear-cut.** Purely a factual-correctness edit to an error string; no
product/scoping question. Exact wording is a copyediting choice, not a decision that needs the user.

**Lockstep:** `docs/manual/src/40-cli-reference/41-mnemonic.md:2194` (same-PR). No GUI/schema impact.

---

## M3 — BIP-85 refuses Portuguese with a factually false message

**Verbatim finding:**
> M3 — BIP-85 refuses Portuguese with a factually false message.
> `toolkit:crates/mnemonic-toolkit/src/cmd/derive_child.rs`. Current BIP-85 assigns Portuguese language code
> `9'`; the toolkit hard-refuses "portuguese is not assigned a BIP-85 path code", blocking recovery of a
> spec-conformant child wallet. Fail-closed (no wrong output). **Fix:** add the `9'` arm.

**Repo/file:** toolkit, `crates/mnemonic-toolkit/src/cmd/derive_child.rs`.

**Still a real gap?** YES, confirmed live at `crates/mnemonic-toolkit/src/cmd/derive_child.rs:342-358`
(`resolve_bip85_language`): the match block maps English(0)…Czech(8) then hard-errors on
`CliLanguage::Portuguese` with `"--language portuguese is not assigned a BIP-85 path code"`.
**Independently re-verified against the authoritative BIP-0085 text** (fetched
`raw.githubusercontent.com/bitcoin/bips/master/bip-0085.mediawiki`, "Application: BIP39" language
table): Portuguese **is** assigned code `9'`, immediately following Czech(8') — the toolkit's own
refusal-message rationale ("no BIP-85 code assigned") is simply wrong. Corroborating evidence already
in-tree: `crates/mnemonic-toolkit/src/language.rs` independently maps `CliLanguage::Portuguese => 9`
in three other places (`cli_language_to_wire_code`, `wire_code_to_bip39`, `bip39_to_wire_code` — the
ms1 wire-language byte table), so `9` for Portuguese is already an established, tested convention
elsewhere in this same codebase — this refusal looks like an isolated oversight, not a deliberate
scoping choice.

BIP-85's own "Test Vectors" section (verified via the same fetch) provides **no official vector for
any non-English language** (only English at 12/18/24 words) — so there is no independent oracle to
pin against beyond the existing weak-oracle pattern the codebase already uses for other
no-official-vector languages (see `bip39_japanese_diverges_from_english`,
`tests/cli_derive_child.rs:492-533`: derive English vs. the target language at the same
master+index, assert outputs differ, assert non-ASCII/wordlist-membership). The same pattern applies
cleanly to Portuguese and is no weaker than the coverage Korean/Spanish/Chinese/French/Italian/Czech
already get.

**Locus:**
- `crates/mnemonic-toolkit/src/cmd/derive_child.rs:353-357` — delete the `Portuguese => Err(...)` arm;
  add `CliLanguage::Portuguese => (9, bip39::Language::Portuguese),`.
- `crates/mnemonic-toolkit/src/cmd/derive_child.rs:325-341` — doc-comment table needs a `| 9 | Portuguese |` row (currently stops at 8/Czech and states "Portuguese ... is refused").
- `crates/mnemonic-toolkit/tests/cli_derive_child.rs:535-560` — `bip39_portuguese_refused_no_bip85_code` must flip from a refusal assertion to a positive divergence/non-refusal test (mirror `bip39_japanese_diverges_from_english`).

**Proposed fix + effort:** mechanical one-arm addition + doc-comment row + test flip; ~20-40 LOC
total including the new positive test. **Repo:** toolkit only. **SemVer:** the project's own
convention bumps MINOR for user-visible behavioral changes even when the change is a bugfix
correcting a false refusal (precedent: v0.75.0→v0.76.0 for the C1 funds-fix, v0.77.0→v0.78.0 for the
`/**` shorthand fix) — recommend **MINOR**, not PATCH, since `--language portuguese --application
bip39` flips from "always exit 1" to "produces new output."

**Decision needed?** **NO — clear-cut.** The spec fact is unambiguous and independently re-verified
against the authoritative BIP text (not just the eval's claim); the fix mirrors an existing,
already-tested pattern (`language.rs`'s Portuguese=9 mapping, and the Japanese-divergence test
shape). No scoping ambiguity — Portuguese is a BIP-39 wordlist the toolkit already fully supports
everywhere else (`--language portuguese` works for every other command).

**Lockstep:** none found. `docs/manual/src/40-cli-reference/41-mnemonic.md` lists `portuguese` as a
valid `--language` value already (lines 1030/1675/1761) with no special BIP-85 carve-out to remove;
no GUI dropdown restriction found (BIP-85 language choices aren't separately enumerated in the GUI
schema beyond the shared `--language` enum, which already includes `portuguese`).

---

## M4 — BSMS Round-1 signature-verify failure exits 0 by default

**Verbatim finding:**
> M4 — BSMS Round-1 signature-verify failure exits 0 by default (stderr NOTICE + `verified=false` only);
> `--bsms-verify-strict` exits 2. `toolkit:crates/mnemonic-toolkit/src/cmd/import_wallet.rs:418`. Inverts the
> conventional `verify && proceed` exit-code contract for automation that gates on `$?`. Refuter's context
> (valid, keeps this minor): the failure is loudly surfaced to a human, a fail-closed mode exists, the
> standalone `--bsms-round1` path binds no descriptor, and the lenient default is documented-intended
> (FOLLOWUP `bsms-verify-signatures`). **Fix (hardening):** make lenient mode exit non-zero when any record
> status is `Failed`, or make strict the default.

**Repo/file:** toolkit, `crates/mnemonic-toolkit/src/cmd/import_wallet.rs`.

**Still a real gap?** YES, confirmed live — citation is exact and unchanged:
`crates/mnemonic-toolkit/src/cmd/import_wallet.rs:418` is still `return Ok(0);` inside the standalone
Round-1-only mode (`args.blob.is_none()`), reached regardless of whether any
`Round1Verification.status` is `Failed { reason }`. The lenient-mode per-record loop
(`verify_bsms_round1_files`, `:2514-2639`) only ever returns `Err` when `strict == true`; in lenient
mode a signature-verify failure is pushed as `Round1VerificationStatus::Failed` with a stderr NOTICE
(`:2621-2626`) and the function still returns `Ok(...)` — so the caller's unconditional `Ok(0)` at
line 418 fires whether every record verified or every record failed.

**Locus:** `crates/mnemonic-toolkit/src/cmd/import_wallet.rs:404-419` (the standalone-mode early
return) — the fix needs an `any(status == Failed)` check over `round1_verifications` before choosing
the return code (Option A), or a change to `bsms_verify_strict`'s clap default (Option B, elsewhere in
the same file's arg definitions ~line 240).

**Proposed fix + effort:** small either way (~10-20 LOC + a CHANGELOG note + manual sync), but the
**two fix directions are materially different in blast radius**:
- **Option A (lenient mode: exit non-zero iff any record `Failed`)** — additive-only for callers who
  already treat "any nonzero = don't proceed"; but it silently changes the meaning of `--bsms-round1`
  standalone-mode exit codes for anyone currently relying on "always 0 unless a hard parse/IO error."
- **Option B (flip `--bsms-verify-strict` to default-on)** — makes the CURRENT default (lenient,
  proceed-anyway) opt-in via a new flag name/negation, which is a bigger behavior flip for any
  existing automation that invokes `import-wallet --bsms-round1 ...` today expecting exit 0 on a
  mismatched-but-tolerated signature.

**Repo:** toolkit only. **SemVer:** MINOR either way, but this is exactly the kind of default-behavior
change that needs a loud CHANGELOG "may break `$?`-gated scripts" callout regardless of direction.

**Decision needed?** **YES — this is the one item that genuinely needs a product decision**, and
the eval says so explicitly ("Fix (hardening): ... **or** ..."). Concretely, ask:
1. Should the *default* (`--bsms-verify-strict` unset) start failing (nonzero exit) when any Round-1
   record's signature doesn't verify — while stopping short of the currently-strict behavior (typed
   `BsmsSignatureMismatch`, exit 2, that aborts before any output)? I.e. keep printing the per-record
   report but return e.g. exit 4 (mirrors the "VERIFY-ME candidate" idiom used elsewhere in this
   codebase — see the ms1/mk1 repair exit-4 precedent below) instead of exit 0?
2. Or should `--bsms-verify-strict` simply become the default, with a new opt-out flag for the
   current lenient behavior?
3. Does this interact with the FOLLOWUP `bsms-verify-signatures` closure narrative (`design/FOLLOWUPS.md:3089`),
   which explicitly frames "lenient default (stderr NOTICE) + `--bsms-verify-strict` mode" as the
   **intended, shipped** v0.27.0 design — i.e. is this eval finding actually requesting a reversal of
   a deliberate earlier design decision?

**Lockstep:** `docs/manual/src/40-cli-reference/41-mnemonic.md:1350-1351` documents current exit-0-on-success
prose but is silent on the lenient-failure exit code (a related, smaller doc gap — worth fixing in the
same PR regardless of which option is chosen). No GUI/schema impact under either option (no new flag
name, no flag removed — `schema_mirror` only gates flag names).

---

## D1 — md1 multi-chunk wire framing: BIP text vs. implementation

**Verbatim finding:**
> D1 — md1 multi-chunk wire framing: the vendored BIP text contradicts the shipped implementation.
> `md:bip/bip-mnemonic-descriptor.mediawiki` (lines 246/306/773) vs `md:crates/md-codec/src/chunk.rs`.
> The BIP mandates bit-boundary fragments ("no byte-aligned framing"; "Decoders MUST accept any valid
> division"); the implementation splits at **byte** boundaries and drops `floor((symbol_bits-37)/8)`
> per-chunk pad bits. A spec-conforming decoder therefore rejects (ChunkSetIdMismatch) every real ≥2-chunk
> md1 card — demonstrated on the production-default 3-chunk `bip84` bundle (spec-rule CSI `0x453d1` vs wire
> `0x434df`). The encoder/decoder round-trip byte-identically, so this is **fail-closed, not silent
> wrong-address** — the refuter is right that it's a documentation/conformance defect, not a funds-theft bug.
> But it undercuts the project's *recovery-independence* goal (the BIP is the stated artifact for
> reconstructing a card without the reference binary), and the frozen corpus has **no ≥2-chunk vector** to
> cross the spec/implementation boundary. **Fix:** align the BIP text to the byte rule (document the
> end-pad + `floor((bits-37)/8)`) *or* align code to the bit rule with a wire-version bump; either way add a
> ≥2-chunk conformance vector.

**Repo/file:** md (descriptor-mnemonic), `bip/bip-mnemonic-descriptor.mediawiki` vs `crates/md-codec/src/chunk.rs`.

**Still a real gap? NO — RESOLVED**, as an incidental side-effect of the separate, already-shipped
2026-07-10 cross-repo BIP-alignment cycle (md-codec 0.41.0 + md-cli 0.12.0, commit `ff4ace23` "bip(md1):
Phase-1 BIP-prose alignment to shipped md-codec"). That cycle's own SPEC
(`descriptor-mnemonic/design/SPEC_md1_bip_alignment_and_code_honesty.md:73`) explicitly names this
finding: **"B-C1 (=D1) §Chunking rewrite: fragments = whole bytes of the byte-padded assembled
payload; header/fragment boundary at bit 37 (contiguous, no slack); reassembly concatenates fragment
BYTES. Delete 'encoder-chosen bit boundaries', 'Decoders MUST accept any valid division', and the
3-bit-slack framing (lines 201/246/306/773/786)."** The commit message for `ff4ace23` states:
"Aligns bip-mnemonic-descriptor.mediawiki to the shipped code + resolves the four load-bearing
contradictions (the D1 class)." Direction chosen: **align the BIP text to the code** (byte-aligned
framing), not the reverse — confirmed live in the current 1247-line BIP file, e.g. `§"Payload"` line
309 ("split into whole-byte fragments... See §Chunking for the byte-aligned framing") and `§"Chunking"`
line 829 ("each chunk carries a whole-byte fragment... division is therefore fixed deterministically
by the near-equal whole-byte split — there is no encoder-chosen bit-boundary freedom... a decoder
recovers each slice as ⌊(symbol-aligned-bit-count − 37) / 8⌋ whole bytes"). The old line numbers cited
in the eval (246/306/773) no longer correspond to the same content — the whole section was rewritten
under the new SPEC.

The **≥2-chunk conformance-vector gap** (test item #4 in the eval's §2 program, tied to D1) is also
closed: `crates/md-codec/tests/wire_golden.rs:250-349` adds a frozen `wsh_sortedmulti_2chunk` golden
whose header comment explicitly states **"this closes the D1 chunk-split-boundary mutation gap"**
(`wire_golden.rs:348-349`), backed by a corpus fixture (`tests/vectors/wsh_sortedmulti_2chunk.*`).

**Locus / fix / effort / decision:** N/A — closed. No further action.

**Lockstep:** none pending.

---

## Docs item — `gui-manual-repair-exit-code-lockstep` (FOLLOWUPS.md task #11)

**Verbatim FOLLOWUPS.md entry** (`design/FOLLOWUPS.md:71-73`):
> ### `gui-manual-repair-exit-code-lockstep` — the GUI-manual book still carries the pre-Cycle-F "exit-5 REPAIR_APPLIED uniform across all four CLIs" claim
>
> - **Surfaced:** 2026-07-09, Cycle F (`ms1-repair-demote-to-candidate`) P2 (docs) — the end-user manual `docs/manual/src/40-cli-reference/*` was brought into lockstep with the ms1-repair demotion (ms1 substitution-correction → exit-4 VERIFY-ME candidate, not exit-5), but the SEPARATE GUI-manual book `docs/manual-gui/src/{40-mnemonic/4i-repair.md, 60-ms/6a-repair.md, 70-mk/79-repair.md, 50-md/5A-repair.md}` carries the identical now-stale "exit-5 REPAIR_APPLIED consistent across all four CLIs" sentence + related exit-5 auto-fire prose. **Out of Cycle F's scope** (SPEC §6 enumerated only the 4 `docs/manual` chapters). **Fix:** a docs-only lockstep pass over the GUI-manual repair chapters, mirroring the P2 rewrite (the "verified now / verifiable-by-reassembly later" model; ms1 = exit-4 candidate; the toolkit `RepairJson` superset note). **Severity:** LOW (docs-only; the GUI-manual is on its own cadence, currently pinned to an older toolkit anyway). **Tier:** manual-gui.

**Is the GUI-manual repair exit-code documentation currently out of sync with real behavior? YES —
confirmed, and worse/more nuanced than the FOLLOWUP entry describes.**

Ground-truth exit codes as of current source (not uniform, contra the manual-gui claim):

| Surface | Substitution-correction outcome | Exit | Citation |
|---|---|---|---|
| `ms repair` (ms-cli 0.14.1) | ANY correction (no set-level self-verify possible) | **4** always | `mnemonic-secret/crates/ms-cli/src/cmd/repair.rs:153` `Ok(if any_correction { 4 } else { 0 })` |
| `mnemonic repair --ms1` (toolkit) | same demotion | **4** | mirrors ms-cli per Cycle F (`bch-repair-miscorrection-set-level-reverify`, FOLLOWUPS.md:34) |
| `mk repair` (mk-cli 0.12.1) | ANY correction (Bless or Unverified-candidate alike) | **5** always | `mnemonic-key/crates/mk-cli/src/cmd/repair.rs:165` `Ok(if any_correction { 5 } else { 0 })` |
| `mnemonic repair --mk1` (toolkit) | Bless (full-set decode-Ok) | **5** | FOLLOWUPS.md:32 |
| `mnemonic repair --mk1` (toolkit) | Unverified candidate (incomplete/single-plate) | **4** | FOLLOWUPS.md:32 — **diverges from `mk repair`'s own exit 5 for the identical semantic case** |
| `md repair` / `mnemonic repair --md1` | ANY correction (chunk-set-hash-protected bless; no candidate state exists for md1) | **5** | `descriptor-mnemonic/crates/md-cli/src/cmd/repair.rs:166` `Ok(if any_correction { 5 } else { 0 })` |

So the manual-gui's blanket "exit-5 uniform/consistent across all four CLIs" claim is false on at
least two independent axes: (a) ms1 substitution-correction is exit 4, not 5, on **both** `ms repair`
and `mnemonic repair --ms1`; and (b) even within mk1, the toolkit's own `mnemonic repair --mk1` and
the standalone `mk repair` binary disagree (4 vs. 5) for the identical "unverified candidate" outcome
— a pre-existing cross-CLI asymmetry the eval's F4 fix introduced and that no manual chapter
currently documents.

**Verbatim stale text, confirmed still present:**
- `docs/manual-gui/src/60-ms/6a-repair.md:47` — `` | `5` | `REPAIR_APPLIED` — at least one substitution corrected; stdout = repair report + corrected string | `` and `:51-53` — "The exit-5 `REPAIR_APPLIED` code is uniform across all four CLIs (`mnemonic` / `mk` / `ms` / `md`), so wrapper scripts can branch on `exit == 5`."
- `docs/manual-gui/src/50-md/5A-repair.md:63` — `` | `5` | at least one chunk corrected (`REPAIR_APPLIED`); stdout = repair report + corrected chunks. | `` and `:67-68` — "The exit-5 `REPAIR_APPLIED` code is consistent across all four CLIs, so wrapper scripts can use a uniform `exit == 5` signal."
- `docs/manual-gui/src/70-mk/79-repair.md:55` — `` | `5` | at least one string corrected (`REPAIR_APPLIED`); stdout = report + corrected strings | `` and `:59-61` — "The exit-`5` `REPAIR_APPLIED` code is uniform across all four CLIs (`mnemonic`/`mk`/`ms`/`md`) so wrapper scripts can branch on a single `exit == 5` signal."
- **Correction to the FOLLOWUP entry's own file list:** `docs/manual-gui/src/40-mnemonic/4i-repair.md` does **NOT** carry the stale uniformity sentence (grepped: no "uniform"/"consistent" hits) — it already documents `--max-subst`-driven exit 4 ("candidate required ≥1 substitution — verify each", line 97) generically per code. Only 3 of the 4 files listed in the FOLLOWUP body are actually stale; `4i-repair.md` needs no change (or only a light pass to make sure its generic exit-4 row also covers the ms1-always/mk1-sometimes distinction, at author's discretion).

**Fix + effort:** docs-only rewrite of the exit-code table + prose paragraph in 3 files
(`6a-repair.md`, `5A-repair.md`, `79-repair.md`), each ~10-20 LOC (1 table row + 1 prose paragraph +
a corrected worked-example if any example currently asserts `exit 5` for an ms1 substitution repair
— `6a-repair.md`'s "Worked example — one-character repair" section (starts line 55) should be checked
for an asserted exit code and updated if it demonstrates a substitution repair). Mirrors the already-
shipped P2 rewrite model in `docs/manual/src/40-cli-reference/*` (the "verified now /
verifiable-by-reassembly later" framing) — that prose can likely be adapted near-verbatim.
**Repo:** manual-gui (a subtree/book inside mnemonic-toolkit per the file paths shown). **SemVer:**
NO-BUMP, doc-only (per project convention — `manual-prose-execution-gate` precedent cited elsewhere
in FOLLOWUPS.md).

**Decision needed?** **NO — clear-cut**, EXCEPT for one small judgment call: whether to also
document the newly-confirmed `mk repair`-vs-`mnemonic repair --mk1` exit 4/5 asymmetry (not
mentioned in the original FOLLOWUP text, discovered during this recon) while doing this pass, or file
it as its own separate FOLLOWUP/doc-nit and keep this fix scoped to "remove the false uniformity
claim + fix ms1's row." Recommend folding it in (same table, same paragraph, marginal extra effort)
rather than a second pass, but flagging for the executing agent to decide at implementation time.

**Lockstep:** self-contained within `docs/manual-gui/`; no code, no GUI schema, no `docs/manual`
(non-gui) changes needed (that book is already correct per Cycle F P2).

---

## Recommended sequencing

### Clear-cut — "just do it" (no user decision required)

All four are toolkit/manual-gui-only, independently shippable, and can run as separate small
R0-gated PATCH/MINOR cycles (or one batched cycle, since none touch overlapping code):

- **Toolkit, `mnemonic-toolkit` repo (2 independent small cycles, could batch):**
  - M2 — SLIP-39 combine error-message correction (PATCH). Files: `cmd/slip39.rs`,
    `tests/cli_slip39_refusals.rs`, `docs/manual/src/40-cli-reference/41-mnemonic.md`.
  - M3 — BIP-85 Portuguese `9'` support (MINOR). Files: `cmd/derive_child.rs`,
    `tests/cli_derive_child.rs`.
- **manual-gui book (1 small doc-only cycle, NO-BUMP):**
  - Docs item — `gui-manual-repair-exit-code-lockstep`: fix the false exit-5-uniform claim in
    `6a-repair.md` / `5A-repair.md` / `79-repair.md` (NOT `4i-repair.md` — already correct); consider
    folding in the newly-found `mk repair`-vs-toolkit exit 4/5 asymmetry.
- **No action needed:**
  - D1 — already resolved by the 2026-07-10 BIP-alignment cycle; confirm-and-close only (flip
    tracking status if a FOLLOWUP entry is later filed for it; none currently exists to close).

### Needs a user decision first

- **M4 — BSMS Round-1 lenient-mode exit code.** Two structurally different fixes on the table
  (per-record-failure-aware nonzero exit vs. flipping `--bsms-verify-strict` to default-on), each a
  different default-behavior change with different blast radius for `$?`-gated automation, and the
  lenient default is on record as a **deliberate** earlier design choice (FOLLOWUP
  `bsms-verify-signatures`, resolved v0.27.0). Needs the user to pick a direction (and possibly the
  exact non-zero exit code — e.g. reusing the exit-4 "VERIFY-ME candidate" idiom already established
  for ms1/mk1 repair, for consistency) before any implementation/R0 cycle starts.
