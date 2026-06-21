# IMPLEMENTATION PLAN — cycle-11b: toolkit secret-hygiene / robustness cluster (L21 · L24 · L25)

**DESIGN ONLY — no code in this document.** This plan-doc feeds its own mandatory opus-architect **R0
review loop to 0 Critical / 0 Important** (CLAUDE.md hard gate) BEFORE any implementation begins. The
implementer does not write a line of code until this plan is R0-GREEN.

**Source spec (R0-GREEN round 2):** `design/BRAINSTORM_cycle11b_toolkit_hygiene.md`
**Spec R0 reviews:** `design/agent-reports/cycle11b-spec-r0-round1-review.md` (C1 + I1 folded),
`design/agent-reports/cycle11b-spec-r0-round2-review.md` (GREEN, 0C/0I).

---

## 0. Source SHA + branch-off-origin/master gotcha (READ FIRST)

| Item | Repo | SHA the spec cites | **CURRENT `origin/master` (live HEAD)** | Toolkit version |
|---|---|---|---|---|
| all three | `mnemonic-toolkit` | `4e8ad7923d03aea5569d4d73f22b6e99371037d8` (v0.65.0) | **`bea7a607`** (still v0.65.0) | `0.65.0` |

**`origin/master` has advanced past the spec's `4e8ad792`** to `bea7a607` via design-only commits
(`bea7a607` "design(cycles 10/11a/11b): folded specs + plan-doc + R0 review trail"; earlier `9b3a6a3a`
"design(cycles 10/11a/11b/12): cycle-prep recons + R0 reviews + cycle-10 GREEN spec", `4c2386b1`
"design(report): tick M12 + L20 FIXED"). **Verified: all six cited source files are byte-identical between
`4e8ad792` and live HEAD** (`git diff --quiet 4e8ad792 origin/master -- <file>` returns 0 for `convert.rs`,
`verify_bundle.rs`, `bundle.rs`, `pipeline.rs`, `error.rs`, `slot_input.rs`). The version is `0.65.0` at
both SHAs. **Every line number in §1 is valid against current `origin/master`.** (Plan R0 M1: the implementer
branches off `origin/master` live HEAD — `git worktree add ... origin/master` resolves to whatever HEAD is
at impl time; re-grep if any source-bearing commit has landed since.)

**BRANCH-OFF GOTCHA (the spec's I1 fold — MANDATORY).** The implementer MUST:

```
git worktree add ../wt-cycle11b origin/master    # branch off LIVE origin/master, NOT the local worktree
```

The **local working tree is on `feature/own-account-subset-search` off v0.60.0** (the own-account cycle,
unmerged). Building cycle-11b there would (a) reintroduce stale-0.60.0 version-site values and corrupt the
version-site bump, (b) ride v0.60.0-era source that predates v0.65.0's fixes. **Do NOT trust any
working-tree line numbers** — every citation in this plan was re-grepped against `git show origin/master:`.

**Toolkit version coordination (carry into impl).** cycle-11b takes **0.65.1** FIRST. cycle-10's md-codec
pin-bump ALSO targets the toolkit and renumbers to **0.65.2** AFTER this. The implementer branches off the
**live** `origin/master` at impl time and reconciles `Cargo.toml`/`Cargo.lock` mechanically if cycle-10 or
any other MINOR-bearing cycle has landed first (whoever lands second renumbers — see §6).

---

## 1. Citation table (re-grepped live against `origin/master = bea7a607`, source-identical to `4e8ad792`)

### L21 — `src/cmd/convert.rs`

| Live line | Token | Role |
|---|---|---|
| `:77` | `"seedqr" => Self::Seedqr,` | argv parse — `(Seedqr, Bip38)` is argv-reachable |
| `:508` | `fn refusal_bip38_no_passphrase() -> ToolkitError {` | existing `ConvertRefusal` helper to mirror |
| `:637` | `\| (Seedqr, Bip38)` | `classify_edge` whitelist — Seedqr→Bip38 is a permitted edge |
| `:765` | `fn run(...)` | the OUTER handler — where `effective_bip38_passphrase` (`Option<String>`) is live |
| `:850` | `let effective_bip38_passphrase: Option<String> = …` | the `Option<String>` binding (NOT in scope at the refusal site `:1350`) |
| `:932` | `if bip38_edge && effective_passphrase.is_none() && effective_bip38_passphrase.is_none() {` | existing guard inside `run()` (`&&` → `--passphrase` alone satisfies it) |
| `:933` | `return Err(refusal_bip38_no_passphrase());` | the guard's refusal |
| `:954` | `let edge_uses_passphrase = edge_uses_pbkdf2 \|\| bip38_edge;` | suppresses the "ignored passphrase" warning on BIP-38 edges → silent |
| `:980` | `let bip38_passphrase = effective_bip38_passphrase.as_deref();` | the `Option<&str>` passed into `compute_outputs` (this IS the in-scope param at the refusal site) |
| `:1217` | `fn compute_outputs(...)` | the function CONTAINING the refusal site `:1350` |
| `:1223` | `bip38_passphrase: Option<&str>,` | **the in-scope passphrase parameter at the refusal site — predicate tests THIS (`bip38_passphrase.is_none()`)** |
| `:1231` | `Seedqr \| Phrase \| Entropy => {` | **outer composite arm** (inside `compute_outputs`) — reaching the inner `Bip38 =>` proves `from ∈ {Seedqr,Phrase,Entropy}` |
| `:1350` | `Bip38 => {` | **composite `Bip38 =>` sub-arm head** — the POSITION-BASED refusal site |
| `:1376` | `let scrypt_pp = bip38_passphrase.unwrap_or("");` | **the silent empty-encrypt** the refusal must precede |
| `:1518` | `Bip38 => {` (direct, inside a different outer arm) | direct `(wif↔bip38)` — UNAFFECTED |
| `:1523` | `let scrypt_pp = bip38_passphrase.unwrap_or(pbkdf2_passphrase);` | direct edge falls back to `--passphrase` — left as-is |
| `:1537` / `:1543` | second direct `Bip38 =>` / `unwrap_or(pbkdf2_passphrase)` | other direct edge — UNAFFECTED |

**SCOPE NOTE (plan R0 I1 — load-bearing, won't-compile otherwise).** The refusal site (`:1350`) is **inside
`fn compute_outputs` (`:1217`)**, whose only passphrase parameter is **`bip38_passphrase: Option<&str>`**
(`:1223`). The `Option<String>` `effective_bip38_passphrase` is a binding in **`run()` (`:765`, set `:850`)**
and is **NOT in scope inside `compute_outputs`** (verified: zero occurrences of `effective_bip38_passphrase`
in `:1217-1600`). So the arm-head predicate MUST test the in-scope parameter **`bip38_passphrase.is_none()`**,
NOT `effective_bip38_passphrase.is_none()` (the latter is a compile error at `:1350`). These are semantically
identical: `run()` sets `let bip38_passphrase = effective_bip38_passphrase.as_deref();` (`:980`), and
`as_deref()` maps `None→None` and `Some("")→Some("")`, so `bip38_passphrase.is_none()` ⟺
`effective_bip38_passphrase.is_none()` — preserving the `is_none()`-not-`is_empty()` invariant and the
`--bip38-passphrase ""` (`Some("")`) GREEN path. (`effective_bip38_passphrase` is `Option<String>` cloned
from `args.bip38_passphrase`: `Some("")` for `--bip38-passphrase ""`, `None` when absent, always `Some(...)`
for `--bip38-passphrase-stdin`.)

### L24 — `src/cmd/verify_bundle.rs` (gate source = `src/cmd/bundle.rs`)

| Live line | File | Token | Role |
|---|---|---|---|
| `:1349` | `verify_bundle.rs` | `let n = descriptor_resolved.n as usize;` | `n` in scope |
| `:1351` | `verify_bundle.rs` | `crate::slot_input::validate_slot_set(&args.slot)?;` | contiguity-only check; **insert gate immediately AFTER this** |
| `:1371` | `verify_bundle.rs` | `if is_non_canonical {` | entry to override region; **insert gate BEFORE this** |
| `:1388`/`:1390`/`:1393` | `verify_bundle.rs` | `(0..n).map(...)` / `Divergent(v)` | `new_paths` built with exactly `n` entries |
| `:1417-1420` | `verify_bundle.rs` | `by_index_subkeys.entry(s.index)…insert(s.subkey)` | subkey map build (M2 gate-1) |
| `:1427-1429` | `verify_bundle.rs` | `if !subkeys.contains(Phrase) && !…(Seedqr) && !…(Ms1)` | subkey filter (M2 gate-2) |
| `:1431` | `verify_bundle.rs` | `continue;` | non-phrase-bearing slots skip BEFORE the OOB write |
| `:1435` | `verify_bundle.rs` | `new_paths[*idx as usize] = …derivation_path_to_origin(&user_path);` | **the unguarded OOB write → panic** |
| `:1466` | `verify_bundle.rs` | `for idx in 0..(n as u8) {` | the later range-checked loop (already safe) |
| `:1373-1388` | `bundle.rs` | `if slots.iter().map(\|s\| s.index as usize + 1).max().unwrap_or(0) != n { return Err(ToolkitError::DescriptorParse(format!("descriptor has n={n} placeholders but --slot vec covers {} slots", …))); }` | **the reference gate to mirror exactly** |

`validate_slot_set` (`slot_input.rs:249`) checks contiguity `0..=max_idx` (`:260`) only — NOT range-vs-`n`.
So a contiguous slot set whose max index exceeds `n` passes, reaches `:1435`, and panics. (Spec's §0 cited
the subkey gate as `:1417-1421`/`:1430-1436`; the live exact lines are build `:1417-1420`, filter
`:1427-1429`, `continue` `:1431`, write `:1435` — same structure, refined numbers; use these.)

### L25 — `src/wallet_import/pipeline.rs`

| Live line | Token | Role |
|---|---|---|
| `:53` | `pub(crate) fn has_any_key_token(s: &str) -> bool {` | the classifier to extend (position-aware x-only) |
| `:56` | `Regex::new(r"[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+\|\b0[23][0-9a-fA-F]{64}\b")` | current regex: xpub-family + `02/03` 66-hex — NOT bare 64-hex x-only |
| `:176` | `pub(crate) fn classify_descriptor_form(input: &str) -> Result<DescriptorForm, ToolkitError> {` | caller |
| `:185` | `(false, false) => {` | the dual-Err arm |
| `:187-191` | `"…concrete descriptors must carry a key origin, e.g. [<fp>/84h/0h/0h]xpub…"` | the CORRECT message for key-bearing-but-origin-less |
| `:196-203` | `"this descriptor has no keys to engrave … keyless script (hashlock/timelock only) … export-wallet"` | the WRONG message x-only currently hits |
| `:529` | `fn classify_keyless_routes_to_export_wallet() {` | regression guard — `sha256(64-hex)` stays keyless |
| `:557` | `fn has_any_key_token_distinguishes_keys_from_hashes() {` | **regression guard — 66-hex `02/03` compressed key stays keyed (M3) AND `sha256`/`ripemd160` literals stay keyless** |

### Error variants + version sites (verified `origin/master`)

| Site | Live value | Role |
|---|---|---|
| `error.rs:89` | `ConvertRefusal(String)` | L21 reuse; exit_code `2` at `error.rs:562`; alphabetically placed |
| `error.rs:123` | `DescriptorParse(String)` | L24+L25 reuse; exit_code `2` at `error.rs:569`; alphabetically placed |
| `crates/mnemonic-toolkit/Cargo.toml:3` | `version = "0.65.0"` | version-of-record |
| `README.md:13` | `<!-- toolkit-version: 0.65.0 -->` | root README |
| `crates/mnemonic-toolkit/README.md:9` | `<!-- toolkit-version: 0.65.0 -->` | crate README |
| `scripts/install.sh:32` | `…mnemonic-toolkit-v0.65.0\|no\|` | install self-pin |
| `fuzz/Cargo.lock:574-575` | `name = "mnemonic-toolkit"` / `version = "0.65.0"` | fuzz lock |
| `CHANGELOG.md:9` | `## mnemonic-toolkit [0.65.0] — 2026-06-21` | top entry (gate-enforced — see §6) |

**No new `ToolkitError` variant; no clap/`--json`/dropdown change.** → NO `schema_mirror`, NO `secret_drift`,
NO manual flag-table edit. The alphabetical-insertion discipline does not bite this cycle.

---

## 2. Execution model

- **Single implementer**, in a **fresh git worktree off `origin/master`** (`git worktree add ../wt-cycle11b
  origin/master`). NOT the local own-account worktree (§0).
- **TDD, RED-first** every phase: write the failing test(s), run them, confirm RED for the stated reason,
  then implement to GREEN.
- **NEVER `cargo fmt` the toolkit.** `mlock.rs` is permanently fmt-exempt (MEMORY `g6_fmt_exemption`); a
  `cargo fmt --all` would touch it and break the asymmetric-pin invariant. Hand-format new code to match
  surrounding style.
- **Toolkit tests live in the BIN target** → gate with **`cargo test -p mnemonic-toolkit`** (NOT `--lib`;
  `--lib` misses the bin-target unit + integration tests). The full-suite gate is mandatory per MEMORY
  `r0_review_run_full_package_suite` — a convert/import phase ripples into argv/schema/version lints outside
  any one phase's targets.
- **Per-phase gate (BOTH must pass before advancing):**
  1. `cargo test -p mnemonic-toolkit` — FULL package suite, GREEN.
  2. `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- **Per-phase architect review** persisted verbatim to
  `design/agent-reports/cycle11b-phase-N-<round>-review.md` BEFORE the fold-and-commit step (CLAUDE.md;
  MEMORY `r0_review_run_full_package_suite`). Reviewer-loop to 0C/0I per phase.
- **Stage paths explicitly** (no `git add -A`).
- After all phases: one **mandatory, non-deferrable whole-diff adversarial execution review** (§5).

---

## 3. Phased plan

Three independent, file-disjoint fixes (`convert.rs` ∥ `verify_bundle.rs` ∥ `pipeline.rs`). They MAY land
in one branch sequentially (recommended — single small PATCH). Ordering by blast radius: **P1 = L24** (panic
→ clean error, mechanical mirror, lowest risk), **P2 = L21** (funds-safety refusal + manual prose, highest
care), **P3 = L25** (cosmetic message + additive regex). Each phase is self-contained; no inter-phase
dependency.

### P1 — L24: `verify-bundle` descriptor-mode OOB panic → typed `DescriptorParse`

**RED tests first** (`crates/mnemonic-toolkit/tests/` — extend an existing `verify_bundle` descriptor test
file or add a focused one):

- **RED (over-n panic → typed reject).** A genuinely **non-canonical 2-key** descriptor +
  `--slot @0.phrase=<seed> --slot @1.phrase=<seed> --slot @2.phrase=<seed> --slot @2.path=m/84'/0'/0'`.
  Today: panics (OOB on `new_paths[2]`). After fix: exit **2** (`DescriptorParse`), stderr names the
  mismatch ("n=2 … covers 3 slots").
  - **CRITICAL (plan R0 I2): `@2` MUST carry BOTH `phrase` AND `path` — two `--slot @2.*` flags.** Each
    `SlotInput` holds exactly ONE `subkey` (`slot_input.rs:99`), so `--slot @2.path=…` ALONE yields
    `@2 = {Path}`, which (a) is rejected by `validate_slot_set` (`:1351`) FIRST — `is_legal_set`
    (`slot_input.rs:347-371`) has NO bare-`[Path]` arm → `SlotInputViolation{kind:"invalid-set"}`, exit 2 —
    so the test would pass for the WRONG reason (vacuous: never reaches the gate, never reproduces the
    panic), AND (b) even past validation would hit `continue` at `:1431` (no phrase-bearing subkey). Use the
    legal phrase-bearing set **`[Phrase, Path]`** for `@2` (legal-set arm `slot_input.rs:365`; the
    established multi-`--slot @N.*`-per-index pattern, cf. existing two-flag-per-slot tests). Then `@2`
    passes validation, lands in `by_index_subkeys` (`:1417-1420`), clears the `:1427-1429` filter, and
    reaches the unguarded `new_paths[2]` write at `:1435`.
  - **M2 fixture preconditions (PIN in a fixture comment so the override loop actually reaches `:1435`):**
    1. Descriptor MUST be genuinely **non-canonical** — assert `canonical_origin(<descriptor>).is_none()`
       so control enters the `is_non_canonical` block at `:1371`. A canonical descriptor skips the override
       region entirely → the test would pass vacuously, never exercising the gate. (`canonical_origin(...)
       .is_none()` is the exact assertion used by existing tests; a `wsh(...)` general-policy-wrapper 2-key
       descriptor is constructible non-canonical.)
    2. `@2` MUST carry a **subkey ∈ {Phrase, Seedqr, Ms1}** (here `@2.phrase=…` co-located with `@2.path=…`)
       so it (i) lands in `by_index_subkeys` (`:1417-1420`) and (ii) clears the `:1427-1429`
       `subkeys.contains(…)`-else-`continue` filter, so the loop reaches the `new_paths[*idx as usize] =`
       write at `:1435`. A path-only override on a non-phrase-bearing slot hits `continue` at `:1431` (or is
       rejected at `:1351` per the CRITICAL note above) and the RED would NOT reproduce.
- **REGRESSION (exact-coverage still verifies).** The canonical/non-canonical 2-key path-override flow with
  `@0`/`@1` overrides only (`max(idx+1) == n`) still verifies — the gate doesn't over-fire. If an existing
  verify-bundle descriptor-override green-path test exists, assert it stays GREEN.

**Implementation.** Insert, **immediately after** `validate_slot_set(&args.slot)?;` (`:1351`) and **before**
`if is_non_canonical {` (`:1371`), the EXACT `bundle.rs:1373-1388` gate (mirrored byte-for-byte so the two
descriptor-mode bindings stay symmetric and share identical error text):

```
if args.slot.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n {
    return Err(ToolkitError::DescriptorParse(format!(
        "descriptor has n={n} placeholders but --slot vec covers {} slots",
        args.slot.iter().map(|s| s.index as usize + 1).max().unwrap_or(0),
    )));
}
```

(Mirror `bundle.rs`'s field/binding names exactly — `bundle.rs` iterates `slots`; `verify_bundle.rs` iterates
`args.slot`. Verify the binding name at gate-write time against the live `verify_bundle.rs` scope.)
Reuses `DescriptorParse` (exit 2). Catches the over-`n` panic AND the under-`n` case (bundle.rs parity).

**Gate position within the window (plan R0 M2).** Anywhere in `:1351`→`:1371` is functionally correct (the
`canonicity_probe` parse at `:1361` does not consult slot indices, and the gate does not depend on the
parse). **Prefer placing it immediately after `validate_slot_set` (`:1351`) and BEFORE the `canonicity_probe`
parse (`:1361`)** — this matches `bundle.rs` ordering exactly (bundle.rs's gate precedes its canonicity
probe), so an over-`n` slot set is rejected before the parse and the two bindings stay symmetric.

**S-VERIFY annotation (D8 — REQUIRED).** Add a comment on the inserted gate noting it **duplicates
`bundle.rs:1373-1388` and should fold into the shared function when the S-VERIFY `bundle.rs ↔
verify_bundle.rs` descriptor-mode dedup lands** (cite the FOLLOWUP slug filed in §7). The S-VERIFY dedup
(`PLAN_constellation_bughunt_fix_program.md §292`) is **NOT scheduled this cycle** — land the standalone gate.

**Gate:** full `cargo test -p mnemonic-toolkit` + clippy. Persist
`design/agent-reports/cycle11b-phase-1-round1-review.md`; loop to 0C/0I.

### P2 — L21: composite `(seedqr|phrase|entropy)→bip38` REFUSE on unset `--bip38-passphrase`

**RED tests first** (`crates/mnemonic-toolkit/tests/cli_convert_bip38.rs` is the existing home — extend it):

- **RED-1 (phrase).** `convert --from phrase="<seed>" --to bip38 --path m/0'/0 --passphrase X`
  (NO `--bip38-passphrase`) → exit **2** (`ConvertRefusal`); stderr mentions `--bip38-passphrase` and the
  `""` escape hatch. Today: silently emits a `6P…` empty-passphrase ciphertext, exit 0 → RED.
- **RED-2 (entropy).** `convert --from entropy=<hex> --to bip38 --passphrase X` (NO `--bip38-passphrase`) →
  same refusal (exit 2). RED today.
- **RED-3 (SEEDQR — the C1 source).** `convert --from seedqr=<valid-seedqr-digits> --to bip38 --passphrase X`
  (NO `--bip38-passphrase`) → same refusal (exit 2). **This test MUST be present** — `(Seedqr, Bip38)` is an
  argv-permitted edge (`:637`/`:77`) that decodes to a phrase and hits the same `:1376` empty-encrypt; the
  position-based arm-head refusal covers it structurally. Without it, the seedqr footgun is uncovered and the
  suite would falsely certify L21 fixed (the exact C1 gap).
- **GREEN-1 (explicit empty STILL encrypts — phrase).** `… --passphrase X --bip38-passphrase ""` → exit 0,
  valid `6P…`; round-trip `convert --from bip38=<that> --to wif --bip38-passphrase ""` recovers the WIF.
  Pins `Some("")` ≠ `None`.
- **GREEN-1b (explicit empty STILL encrypts — SEEDQR).** `convert --from seedqr=<digits> --to bip38
  --passphrase X --bip38-passphrase ""` → exit 0, valid `6P…`. Pins the refusal does NOT over-fire on the
  seedqr source when the explicit-empty path is taken.
- **GREEN-2 (real passphrase STILL encrypts).** `… --bip38-passphrase secret` → exit 0, valid `6P…`,
  round-trips with `--bip38-passphrase secret`.
- **REGRESSION (direct edge untouched).** `convert --from wif=<wif> --to bip38 --passphrase X` → exit 0
  (direct edge falls back to `--passphrase` per `:1523`; refusal does NOT fire on direct edges).

**Implementation — POSITION-BASED refusal (D2/D3).** At the **head of the composite `Bip38 =>` sub-arm**
(`convert.rs:1350`) — which sits **inside** the outer `Seedqr | Phrase | Entropy =>` arm (`:1231`), so
reaching it already proves `from ∈ {Seedqr, Phrase, Entropy}` — insert, **before** the `:1376`
`unwrap_or("")`:

```
if bip38_passphrase.is_none() {
    return Err(<ConvertRefusal with the §2.1 message>);
}
```

- **Predicate tests `bip38_passphrase` — the in-scope `Option<&str>` parameter of `compute_outputs`
  (`:1223`), NOT `effective_bip38_passphrase`** (which is out-of-scope at `:1350` and would not compile — see
  the §1 SCOPE NOTE). `bip38_passphrase.is_none()` ⟺ `effective_bip38_passphrase.is_none()` because `run()`
  sets `bip38_passphrase = effective_bip38_passphrase.as_deref()` (`:980`).
- **NO `from`-set test is written** — the arm's *position* inside `:1231` is the membership proof. A written
  `{Phrase, Entropy}` list could silently drop `Seedqr` (the C1 gap); position-based forecloses it.
- Predicate is `.is_none()` — **NOT `.is_empty()`** — so `--bip38-passphrase ""` (`Some("")` → `Some("")`
  after `as_deref()`) STILL encrypts (GREEN-1/1b).
- **Refusal message** (the spec's §2.1 string; mention the missing flag, *why* `--passphrase` doesn't cover
  the BIP-38 layer, and the `--bip38-passphrase ""` escape hatch):
  > `composite (seedqr|phrase|entropy)→bip38 requires --bip38-passphrase (or --bip38-passphrase-stdin); on
  > this edge --passphrase feeds only BIP-39 PBKDF2 and the BIP-38 layer would be encrypted with the empty
  > passphrase. To deliberately use an empty BIP-38 passphrase, pass --bip38-passphrase "" explicitly.`
  Reuse the `refusal_bip38_no_passphrase()` helper pattern (`:508`) — add a sibling helper (e.g.
  `refusal_composite_bip38_no_bip38_passphrase()`) or inline the `ConvertRefusal(String)`.
- **Direct `(wif↔bip38)` edges LEFT AS-IS** (`:1518`/`:1537` — separate outer arms; `:1523`/`:1543` fall
  back to `--passphrase`, documented v0.8 behaviour, not the silent-empty footgun). Structurally untouched.

**Manual PROSE — SAME COMMIT (D14; prose-only, gate-free).** (The path is `50-comparing/`, NOT
`40-cli-reference/` for the edge table.)
- **`docs/manual/src/50-comparing/56-bip39-vs-bip38-pass.md`** — edge table at lines **49-54** (verified
  `origin/master`). Today: `(phrase, bip38) composite` (`:53`, "BIP-38 (independent; no fallback)"),
  `(entropy, bip38) composite` (`:54`, "BIP-38 (defaults to `""` if unset; BREAKING)"), and **NO
  `(seedqr, bip38)` row**. Required edits:
  1. **Amend** the `(phrase, bip38)` and `(entropy, bip38)` rows so an **unset** `--bip38-passphrase` reads
     **REFUSED** (the `(entropy, bip38)` row's stale "defaults to `''` if unset" → "REFUSED if unset;
     `--bip38-passphrase ""` for an explicit empty passphrase").
  2. **ADD a `(seedqr, bip38)` row** ("REFUSED if unset; `--bip38-passphrase ""` for explicit empty") — OR
     **generalize** the three composite rows into one `(phrase|entropy|seedqr, bip38) composite` row. Either
     way the **seedqr source MUST appear** (leaving it out is the prose mirror of the C1 gap).
- **`docs/manual/src/40-cli-reference/41-mnemonic.md`** — the `--bip38-passphrase` row (`:802`). Add a
  one-line note: "on a composite `(seedqr|phrase|entropy)→bip38` edge, `--bip38-passphrase` is **required**;
  an unset value is refused (it would otherwise encrypt with the empty passphrase). Pass
  `--bip38-passphrase ""` to deliberately use an empty BIP-38 passphrase." **NO flag-table row added/removed**
  — a clarification of the existing row. (Manual lint checks flag-NAME coverage only → prose change keeps it
  GREEN; this is discipline, not a gate.)

**Gate:** full `cargo test -p mnemonic-toolkit` + clippy. Persist
`design/agent-reports/cycle11b-phase-2-round1-review.md`; loop to 0C/0I.

### P3 — L25: position-aware x-only detection in `has_any_key_token`

**RED test first** (`pipeline.rs` unit-test mod, alongside `:557`):

- **RED (x-only routes to the RIGHT message).** `import-wallet --descriptor "tr(<64-hex-xonly>,
  pk(<64-hex-xonly2>))"` (no origin, no `@N`) → today the **"no keys to engrave / keyless script"** message
  (`:196-203`); after fix → the **"…must carry a key origin…"** message (`:187-191`). Assert exit **2** in
  BOTH (descriptor rejected either way — message-only change); assert the post-fix message contains "must
  carry a key origin" and does NOT contain "keyless script". (A `has_any_key_token` unit assertion on a
  `tr(<64hex>,...)` / `pk(<64hex>)` string is the cheapest RED; pair with one CLI-level cell if a convenient
  harness exists.)

**Implementation — position-aware, NOT a regex widen (D9, option (a) recommended).** Extend
`has_any_key_token` (or a helper it calls) to recognize a bare **64-hex** token **only when it sits in a
taproot KEY position** — i.e. directly after `tr(`, or as the argument of `pk(`/`pk_k(`/`pk_h(` — via
context-anchored matches (e.g. `tr(<64hex>`, `pk(<64hex>`), NOT a bare-token `\b[0-9a-fA-F]{64}\b` widen. A
bare widen would mis-flag a keyless `sha256(<64-hex>)` hashlock as keyed (a NEW regression).

- **M3 — the anchor MUST be ADDITIVE.** The current regex (`:56`) already matches `02/03`-prefixed **66-hex**
  compressed keys via `\b0[23][0-9a-fA-F]{64}\b`, and `has_any_key_token_distinguishes_keys_from_hashes`
  (`:557`) asserts those classify as keys (GREEN today). A new `pk(<64hex>` / `tr(<64hex>` anchor for x-only
  MUST still accept a `pk(02…{64})` / `pk(03…{64})` compressed key (66 hex after `pk(`) — do NOT naively
  anchor *exactly* 64 hex after `pk(` and flip the 66-hex assertion to RED.
- **Hash literals MUST stay keyless** — `sha256(`/`hash256(`/`ripemd160(`/`hash160(` 64-hex arguments MUST
  NOT be classified as keys (the `:529`/`:557` keyless assertions stay GREEN).
- **(b) structural parse** is the bug-hunt's more-robust alternative; deferred — option (a) is sufficient for
  a PATCH cosmetic fix and adds no parse dependency. If R0 prefers (b) and it is deferred, leave a residual
  FOLLOWUP for the structural upgrade (§7).

**REGRESSION (must stay GREEN — assert explicitly):**
- `classify_keyless_routes_to_export_wallet` (`:529`) — `sha256(<64-hex>)` hashlock routes to the keyless /
  export-wallet message.
- `has_any_key_token_distinguishes_keys_from_hashes` (`:557`) — **BOTH** the 66-hex compressed-key-is-keyed
  assertion (M3) **AND** the `sha256`/`ripemd160` hash-literal-is-keyless assertions.

**No behaviour change beyond the message** — both `(false,false)` arms still `Err`; the x-only case re-routes
from "keyless" to "must carry a key origin".

**Gate:** full `cargo test -p mnemonic-toolkit` + clippy. Persist
`design/agent-reports/cycle11b-phase-3-round1-review.md`; loop to 0C/0I.

---

## 4. SemVer / lockstep (no new surface)

- **L21 / L24 / L25 all PATCH** → toolkit **0.65.0 → 0.65.1** (`PLAN_constellation_bughunt_fix_program.md
  §210/§213/§214`). L21 is FORMAL-classed (adds a refusal of a previously-accepted invocation) but
  behaviourally a bug-fix with no new surface → PATCH is correct.
- **No new `ToolkitError` variant** — L21 reuses `ConvertRefusal` (`error.rs:89`), L24+L25 reuse
  `DescriptorParse` (`error.rs:123`). Both alphabetically placed already.
- **No clap / `--json` / dropdown change** → **NO `mnemonic-gui/src/schema/mnemonic.rs` schema_mirror
  update**, **NO `secret_drift`**, **NO manual flag-table edit**. (Precisely why the `--allow-empty-passphrase`
  opt-in for L21 was rejected.)
- **No registry publish** — the toolkit is not a registry crate; only the local tag `mnemonic-toolkit-v0.65.1`.
- **L21 manual prose** is the only manual leg (§3 P2 above), SAME COMMIT, gate-free.

---

## 5. Whole-diff adversarial review (MANDATORY, non-deferrable)

After all three phases are individually GREEN, run ONE independent adversarial execution review over the
**entire cycle-11b diff** (R0 reviewed plan correctness; this catches implementation-introduced regressions
TDD misses — CLAUDE.md per-phase policy step 4). Persist verbatim to
`design/agent-reports/cycle11b-whole-diff-review.md`. The review MUST run the **full
`cargo test -p mnemonic-toolkit` suite** (not targeted targets) + `cargo clippy --workspace --all-targets
-- -D warnings`. Specific adversarial checks:

- L21: the refusal is at the composite arm head (`:1350`), tests the in-scope `bip38_passphrase` param (NOT
  `effective_bip38_passphrase` — confirm it compiles), and does NOT leak into the direct `(wif↔bip38)` arms
  (`:1518`/`:1537`); `.is_none()` (not `.is_empty()`) — `--bip38-passphrase ""` still encrypts; the seedqr
  source is covered (RED-3 present and GREEN).
- L24: the gate is `!= n` (exact-coverage, both over- and under-`n`), positioned after `:1351` / before
  `:1371`; carries the S-VERIFY fold comment; the M2 RED fixture genuinely reaches `:1435`.
- L25: the anchor is additive — 66-hex compressed keys still keyed (`:557`), hash literals still keyless
  (`:529`/`:557`).

**If Agent-API dispatch fails mid-session, flag it explicitly and defer the formal review to API recovery**
— never silently substitute inline self-review (CLAUDE.md ultracode step 5).

---

## 6. Version-site sweep + CHANGELOG (MANDATORY lockstep — all five sites + CHANGELOG → 0.65.1)

**All five sites are at `0.65.0` on `origin/master`; NONE has drifted** (the spec's I1 fold corrected an
earlier false "drifted to 0.60.0" claim — that was a dirty-own-account-worktree artifact). Bumping only
`Cargo.toml` while leaving the other four at 0.65.0 would *introduce* the exact drift the earlier draft
wrongly believed existed. **Bump ALL of these to 0.65.1 in ONE atomic commit (no escape hatch):**

| # | Site | `origin/master` value | → |
|---|---|---|---|
| 1 | `crates/mnemonic-toolkit/Cargo.toml:3` | `version = "0.65.0"` | `0.65.1` |
| 2 | `README.md:13` | `<!-- toolkit-version: 0.65.0 -->` | `0.65.1` |
| 3 | `crates/mnemonic-toolkit/README.md:9` | `<!-- toolkit-version: 0.65.0 -->` | `0.65.1` |
| 4 | `scripts/install.sh:32` | `mnemonic-toolkit-v0.65.0` | `mnemonic-toolkit-v0.65.1` |
| 5 | `fuzz/Cargo.lock:575` | `version = "0.65.0"` | `0.65.1` (regenerate or hand-edit the `mnemonic-toolkit` entry) |
| 6 | `CHANGELOG.md` | top entry `[0.65.0]` | **NEW** `## mnemonic-toolkit [0.65.1] — <date>` section above it |

**CHANGELOG is GATE-ENFORCED** (unlike sites 2-5): `.github/workflows/changelog-check.yml` fires on every
`mnemonic-toolkit-v*` tag push and fails the build if `^## mnemonic-toolkit \[0.65.1\]` is absent. The new
section documents: L21 (composite-bip38 empty-passphrase refusal — funds-safety), L24 (verify-bundle
descriptor-mode OOB-panic → `DescriptorParse`), L25 (import-wallet x-only message re-route). Note: no codec
bump, no GUI schema-mirror, no manual flag-table — only manual PROSE.

**Sites 2-5 are NOT gate-enforced** (`project_toolkit_release_ritual_version_sites`) → the lockstep is
**discipline**; treat all five + CHANGELOG as one atomic bump. After the bump: **re-run `cargo test -p
mnemonic-toolkit` + clippy + `cargo build` in `fuzz/`** (to confirm `fuzz/Cargo.lock` is consistent) BEFORE
tagging (MEMORY `older_timelock_advisory` release-lesson: re-run the suite after the bump, before the tag).

**Renumber-on-collision (carry into impl).** If cycle-10's md-codec pin-bump (or any MINOR-bearing cycle)
has landed on `origin/master` first, cycle-11b renumbers (whoever lands second renumbers `Cargo.toml` +ALL
five sites + CHANGELOG). The implementer reconciles `Cargo.toml`/`Cargo.lock`/`fuzz/Cargo.lock` mechanically
at impl time against the **live** `origin/master`.

**Tag:** `mnemonic-toolkit-v0.65.1` (direct fast-forward + tag; no PR, no registry publish — toolkit is a
direct-FF repo per MEMORY lane model).

---

## 7. FOLLOWUP flips + bug-hunt report ticks

**FINDING (plan-author — flag for R0):** the spec's §5 names three FOLLOWUP slugs that **do NOT exist in
`design/FOLLOWUPS.md` on `origin/master`**. Re-grep confirms:
- `convert-composite-bip38-empty-passphrase-refusal` — **absent.** The nearest existing slug
  `bip38-distinct-passphrase-flag` (`FOLLOWUPS.md:1968`) is already `resolved` (v0.8). L21 is a NEW residual
  footgun on top of that resolved work → **FILE this slug NEW and CLOSE it in the L21 shipping commit.**
- `verify-bundle-bundle-rs-descriptor-mode-dedup` — **absent.** The S-VERIFY thesis lives in
  `PLAN_constellation_bughunt_fix_program.md §292` and the closest existing dedup slugs
  (`synthesize-descriptor-deduplicate-with-unified` `:2577`, `restore-emit-dispatch-3way-dedup` `:419`) are
  both `resolved` and cover *different* call-sites. → **FILE this slug NEW as OPEN**, carrying the L24
  gate-fold note (the L24 gate comment cites it). It stays OPEN — cycle-11b lands only the standalone gate.
- `import-classify-xonly-position-aware` — **absent.** `bundle-keyless-descriptor-honest-refusal` (`:81`) is
  `resolved` (the L25-class honest-message landed; current pipeline.rs reflects it). The x-only gap is a NEW
  residual → **FILE this slug NEW and CLOSE it in the L25 shipping commit.** If R0 picks structural-parse (b)
  and defers it, leave a residual OPEN slug for the structural upgrade.

| Slug | Action |
|---|---|
| `convert-composite-bip38-empty-passphrase-refusal` (toolkit) | **FILE NEW + flip to `resolved 0.65.1`** in the L21 commit. |
| `verify-bundle-bundle-rs-descriptor-mode-dedup` (toolkit) | **FILE NEW as OPEN** — carries the L24 gate; folds into the S-VERIFY shared fn when that dedup lands. The L24 gate comment cites this slug. |
| `import-classify-xonly-position-aware` (toolkit) | **FILE NEW + flip to `resolved 0.65.1`** in the L25 commit (residual OPEN slug if R0 defers structural-parse). |

(Per MEMORY `followup_status_discipline`: verify "open"/"absent" status at decision time — done above —
and flip/file in the shipping commit.)

**Bug-hunt report ticks** — `design/agent-reports/constellation-bughunt-2026-06-20.md`. On `origin/master`
the `### - [ ]` lines are: **L21 `:823`, L24 `:939`, L25 `:952`**. **RE-GREP these at SHIP time** — they
drift every merge (`git show origin/master:design/agent-reports/constellation-bughunt-2026-06-20.md | grep
-n '### - \[ \] L2[145]'`). Tick `[ ]` → `[x]` for all three in the shipping commit (or the design-tick
commit), each annotated `FIXED — toolkit v0.65.1`.

---

## 8. Shared-file sequencing (carry into impl)

All three files sit in active multi-cycle zones (`PLAN_constellation_bughunt_fix_program.md §Cross-cutting`):
- `cmd/convert.rs` (L21) ↔ **S-NET** (`§616/§687`) — L21's edit is localized (`:1350` arm head); mechanical
  rebase if cycle-13 carries convert.rs hunks.
- `cmd/verify_bundle.rs` (L24) ↔ **S-VERIFY** dedup (`§292`) — the one collision-sensitive item; land the
  standalone gate now, fold into the shared fn when S-VERIFY dedups (S-VERIFY not scheduled now).
- `wallet_import/pipeline.rs` (L25) ↔ **S-NET / H15** (`§426/§619`) — rebase onto any cycle-13 pipeline.rs
  hunks.

cycle-10's md-codec pin-bump is orthogonal (Cargo metadata only; meets only at `Cargo.toml`/`Cargo.lock`,
mechanical). If cycle-13 (S-NET/S-VERIFY) lands first, cycle-11b rebases onto / serializes after it rather
than running concurrently in the same file zones.

---

## 9. Mandatory R0 gate (CLAUDE.md hard gate)

**NO code before THIS PLAN is R0-GREEN (0C/0I).** This plan-doc MUST pass the opus-architect R0 review loop
to 0 Critical / 0 Important BEFORE implementation begins — and the loop continues after every fold
(re-dispatch the architect; folds can introduce drift). Persist each plan R0 review verbatim to
`design/agent-reports/cycle11b-plan-r0-<round>-review.md` BEFORE applying folds. R0 reviews run the **full
`cargo test -p mnemonic-toolkit` suite**, not targeted targets. Only after this plan is R0-GREEN does the
implementer branch off `origin/master`, write the RED tests, and execute P1→P2→P3 with per-phase R0, then
the mandatory whole-diff review (§5). **Proceeding past ANY gate (start coding, advance a phase, tag, ship)
with an open Critical or Important finding is prohibited.**
