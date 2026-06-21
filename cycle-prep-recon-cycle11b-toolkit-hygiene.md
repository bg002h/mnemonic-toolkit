# Cycle-prep STRICT-GATE recon — cycle-11b: toolkit secret-hygiene / robustness cluster (L21, L24, L25)

**Recon ONLY — no implementation.** Verifies the bug-hunt's L21/L24/L25 citations against current
`origin/master`, renders STILL-REPRODUCES verdicts, recommends fix-sites + safety framing, and flags
cross-cutting (SemVer, clap/schema/manual lockstep, shared-file sequencing vs cycle-10/13).

- **Repo:** `mnemonic-toolkit` (this repo), default branch `master`.
- **`origin/master` SHA recorded:** **`4e8ad7923d03aea5569d4d73f22b6e99371037d8`** (toolkit **0.65.0**, "design(cycle7): M8 build-descriptor fix trail + bughunt M8/L23 ticks").
- **Source of findings:** `design/agent-reports/constellation-bughunt-2026-06-20.md` — L21 (§794-804), L24 (§905-916), L25 (§918-926).
- **Program plan context:** `design/PLAN_constellation_bughunt_fix_program.md` rows L21 (§210), L24 (§213), L25 (§214); workstreams §431 (WS-CONVERT-BIP38), §426 (WS-IMPORT-CLASSIFY), S-VERIFY §292; error.rs ordering §551-556.
- **Note:** all three are toolkit-only; no registry publish; toolkit MINOR/PATCH only.

---

## L21 — `convert (phrase|entropy)→bip38` silently encrypts with an EMPTY BIP-38 passphrase

**Class:** D-secret-leak (SECRET footgun). **Workstream:** WS-CONVERT-BIP38.

### Citation re-check — **DRIFTED line numbers, ACCURATE bug**
Bug-hunt cited `cmd/convert.rs:1366` (empty fallback), `:932` (guard), `:1502,1522` (asymmetric direct edges).
Current source (file is now 2001 lines):
- **Guard:** `cmd/convert.rs:931-933` — `bip38_edge && effective_passphrase.is_none() && effective_bip38_passphrase.is_none()` → `refusal_bip38_no_passphrase()`. The `&&` means the guard is **satisfied by `--passphrase` alone** (BIP-39 channel), even when `--bip38-passphrase` (the BIP-38 Scrypt channel) is unset. ✅ ACCURATE (was `:932`).
- **Empty fallback (composite arm):** `cmd/convert.rs:1376` — `let scrypt_pp = bip38_passphrase.unwrap_or("");` inside the `(Phrase|Entropy, Bip38)` arm of `compute_outputs`. The drifted `:1366` → now `:1376`. ✅ ACCURATE.
- **Asymmetric direct edges:** `cmd/convert.rs:1523` (`(wif→bip38)`) and `:1543` (`(bip38→wif)`) — both `let scrypt_pp = bip38_passphrase.unwrap_or(pbkdf2_passphrase);` i.e. they **fall back to `--passphrase`**. The drifted `:1502,1522` → now `:1523,1543`. ✅ ACCURATE — the asymmetry is real: direct edges fall back to `--passphrase`; the composite arm falls back to `""`.

The `unwrap_or("")` vs `unwrap_or(pbkdf2_passphrase)` divergence is now **explicitly documented as a v0.8 BREAKING design choice** in the source (doc-comment `:253-261`; arm comment `:1351-1357`: "on this composite arm `--passphrase` feeds BIP-39 PBKDF2 only; … If the latter is unset, BIP-38 encrypt uses `""` (no fallback to --passphrase)"). So the asymmetry is intentional-by-design, but the **footgun is still live**: a user who runs `convert --from phrase --to bip38 --path … --passphrase X` gets a BIP-38 card encrypted with the **empty** passphrase — and there is **no warning**, because the `bip38_edge` branch of the warning logic (`edge_uses_passphrase = … || bip38_edge`, `:961`) *suppresses* the "ignored passphrase" warning precisely on BIP-38 edges. Silent.

### STILL-REPRODUCES verdict — **YES, SILENT**
Flag combo that triggers: `convert --from phrase|entropy --to bip38 --path <p> --passphrase <X>` (and NO `--bip38-passphrase` / `--bip38-passphrase-stdin`). The `--passphrase` satisfies the line-931 guard (passes), is consumed by BIP-39 PBKDF2 only, the BIP-38 Scrypt layer gets `bip38_passphrase = None → unwrap_or("") = ""`, and the "ignored" warning is suppressed for BIP-38 edges → an **empty-passphrase BIP-38 ciphertext, silently**. A v0.7 user migrating from dual-purpose `--passphrase` produces effectively-unprotected ciphertext. **Reproduces, silent — confirmed.**

### Fix-site + SECRET-safety framing (recommendation)
- **Fix-site:** `cmd/convert.rs` — the composite-arm fallback at **`:1376`** (the actual empty encrypt), with the decision enforced at the **`:931-933` guard region** (the natural funds-safety gate, where `refusal_bip38_no_passphrase` already lives).
- **Recommended choice — REFUSE (funds-safe default), no new flag.** When the edge is a composite `(Phrase|Entropy → Bip38)` AND `effective_bip38_passphrase.is_none()`, return a clean `ConvertRefusal` (e.g. *"composite (phrase|entropy)→bip38 requires `--bip38-passphrase` (or `--bip38-passphrase-stdin`); `--passphrase` feeds only BIP-39 PBKDF2 on this edge and would leave the BIP-38 layer empty-encrypted"*). This is the smaller, fail-closed change; it matches the project's funds-safety discipline (an empty-passphrase BIP-38 card a user *thinks* is protected is a silent-secret-leak class bug) and reuses the existing **`ToolkitError::ConvertRefusal(String)`** variant (`error.rs`) — **no new error variant**, **no new clap flag**, so **no GUI schema / manual lockstep** beyond the manual prose note for the new refusal text. The bug-hunt's own fix line offers "refuse … (funds-safe default), or emit a loud warning"; refuse is the recommended of the two.
- **Reject the `--allow-empty-passphrase` opt-in:** an empty-passphrase BIP-38 card is never a sane production artifact for an engravable steel card; an opt-in flag would add clap/schema/manual lockstep surface (see Cross-cutting) for a near-zero-value escape hatch. If the user genuinely wants it, the explicit path is `--bip38-passphrase ""` (the user must type the empty string deliberately) — which the refusal message can point at. **Decision: REFUSE, do not add a flag.**
- **SemVer:** because the fix ADDS a refusal (rejects a previously-accepted invocation), this is **FORMAL** per the program's rule-clause-1; per `PLAN §210` L21 is listed toolkit **PATCH**. A new refusal of a previously-accepted CLI invocation is behaviourally a bug-fix, not a new feature — **PATCH is defensible** (no new surface), but if the cycle prefers to be conservative about "changes accepted-input contract," **MINOR** is also reasonable. **Recommend PATCH** (no flag, no variant, pure funds-safety hardening), consistent with the PLAN.

---

## L24 — `verify-bundle` descriptor-mode `--slot @N.path` OOB-indexes `new_paths[idx]` → panic for `idx ≥ n`

**Class:** E-panic-dos. **Workstream:** S-VERIFY (same `bundle.rs`/`verify_bundle.rs` zone as H1/H12).

### Citation re-check — **DRIFTED line numbers (H12 fold inserted lines), ACCURATE bug**
Bug-hunt cited `cmd/verify_bundle.rs:1374-1393` (`new_paths` built with exactly `n`), `:1425` (write, no guard), `:1456-1462` (range-checked loop runs after). The **H12 fix already landed on this branch** (`verify_bundle.rs:1372` carries an `// H12 — taproot-aware default-origin script-type` comment), shifting the region down:
- `new_paths` built with exactly `n` entries: **`:1384-1404`** — every constructor arm yields `(0..n).map(...)` or maps over `Divergent(v)` (length n). ✅ ACCURATE (was `:1374-1393`).
- **OOB write, no `idx<n` guard:** **`:1435`** — `new_paths[*idx as usize] = crate::cmd::bundle::derivation_path_to_origin(&user_path);` inside the `for (idx, slot_path) in &by_index_path` override loop (`:1422-1436`). No bound check precedes it. ✅ ACCURATE (was `:1425`).
- **Range-checked loop runs AFTER:** **`:1466`** `for idx in 0..(n as u8)`. ✅ ACCURATE (was `:1456`).
- **Reference gate that the mirror omits:** `cmd/bundle.rs:1373-1388` — an unconditional `slots.iter().map(|s| s.index+1).max() != n` → `ToolkitError::DescriptorParse("descriptor has n=… placeholders but --slot vec covers … slots")`, fired BEFORE bundle's own override loop. `verify_bundle.rs` has **no equivalent gate** between its `let n = …` (`:1349`) and the `:1435` write — confirmed by scanning `:1349-1435` (only an unrelated `.max()` at `:453` in a different early fn). ✅ Guard-asymmetry confirmed.
- `slot_input.rs:validate_slot_set` (`:249-275`): checks index **contiguity** (`0..=max_idx`, no gaps) and per-slot subkey validity, but does **NOT** range-check against `n` (n unknown at that layer — exactly as the report states). It is called on the descriptor path at `verify_bundle.rs:1351`. ✅ ACCURATE.

### CLI-reachability verdict — **YES, CLI-REACHABLE (operator-misuse, not attacker)**
A contiguous slot set whose max index exceeds the descriptor's placeholder count passes `validate_slot_set`
(contiguity OK), reaches the override loop, and panics on the OOB write. Concretely:
`verify-bundle --descriptor "<non-canonical 2-key descriptor>" --slot @0.phrase=… --slot @1.phrase=… --slot @2.path=m/…`
— indices 0,1,2 are contiguous (passes), the descriptor is non-canonical (enters the `is_non_canonical` block at `:1371`), `@2` is a path-override on a phrase-bearing-adjacent slot, `new_paths` has length n=2 → `new_paths[2]` **panics (index out of bounds)**. Gating: non-canonical descriptor + a phrase/seedqr/ms1-bearing slot at the OOB index (the loop's `subkeys.contains(Phrase|Seedqr|Ms1)` filter at `:1430-1436`). **Operator-misuse, not attacker-controlled input** — no funds/secret impact, just an ugly panic instead of a clean error (denial-of-self). Real, reachable, but low-severity.

### Fix-site + framing
- **Fix-site:** `cmd/verify_bundle.rs` — add the `bundle.rs:1373-1388`-style `max(idx+1) != n` gate (clean `ToolkitError::DescriptorParse`) **before** the path-override loop at `:1422` (ideally right after `let n = …` at `:1349`, or just before the `is_non_canonical` block at `:1371`). Reuses **existing** `DescriptorParse(String)` variant — **no new error variant**.
- **Structural (the program's S-VERIFY thesis):** the deeper fix is to **deduplicate the `bundle.rs ↔ verify_bundle.rs` descriptor-mode binding into one shared function** so this exact guard-drift (and the H1 class) cannot recur (`PLAN §292`, §1053 of the report). For a narrow cycle-11b, the minimal one-gate fix is sufficient and self-contained; the dedup is a larger S-VERIFY structural item that may be out of cycle-11b scope.
- **SemVer:** pure robustness (panic → clean error), no surface change → **PATCH** (matches `PLAN §213`).

---

## L25 — `import-wallet` keyed/keyless classifier blind to raw x-only taproot keys → wrong "keyless" error message

**Class:** other (cosmetic err-msg). **Workstream:** WS-IMPORT-CLASSIFY.

### Citation re-check — **DRIFTED line numbers, ACCURATE bug**
Bug-hunt cited `wallet_import/pipeline.rs:53-60` (`has_any_key_token` regex), `:160-180` (`classify_descriptor_form` `(false,false)` arm). Current source:
- `has_any_key_token` regex: **`:53-61`**, literal `[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+|\b0[23][0-9a-fA-F]{64}\b` (`:56`). Matches xpub-family + `02/03`-prefixed 66-hex compressed pubkeys, but **NOT bare 32-byte (64-hex) x-only** keys. The doc-comment (`:49-52`) *explicitly* says it "Deliberately does NOT match bare 64-hex, which is ambiguous (an x-only taproot pubkey vs a `sha256()`/`hash256()` hash literal)." ✅ ACCURATE (was `:53-60`).
- `classify_descriptor_form` `(false,false)` arm: **`:176-209`** (the arm body at `:185-208`). When neither `@N` nor `[fp/path]xpub` match and `has_any_key_token` is false, it returns the **"this descriptor has no keys to engrave … keyless script (hashlock/timelock only) … Emit it as a watch-only descriptor file"** error (`:198-205`). ✅ ACCURATE (was `:160-180`).

### STILL-REPRODUCES verdict — **YES, COSMETIC (wrong/misleading error message only)**
A `tr(<xonly>, pk(<xonly2>))` with no `[fp/path]` origin and no `@N`: `key_regex` requires `[fp/path]xpub` → no match (`has_concrete=false`); `at_n_probe` no match (`has_at_n=false`); falls into `(false,false)`; `has_any_key_token` sees no xpub and no 02/03 → `false` → routes to the **"keyless script (hashlock/timelock only)"** error. But the descriptor **does** carry keys (x-only) — so the message is **misleading** (tells the user it's keyless when it has taproot keys). **Reproduces.** Benign: BOTH `(false,false)` arms return `Err` (the descriptor is rejected either way for lacking origins), so this affects ONLY the error text — no funds/secret/routing impact.

### Fix-site + framing
- **Fix-site:** `wallet_import/pipeline.rs` — either (a) the bug-hunt's structural recommendation: replace regex key-sniffing with structural descriptor parsing for taproot x-only positions, or (b) the minimal text fix: detect a bare-64-hex x-only token in taproot key positions and route to the correct "key present but no origin" message (`:189-196`). Reuses **existing** `DescriptorParse(String)` variant — **no new variant**.
- **Caveat (ambiguity):** the 64-hex token is genuinely ambiguous (x-only pubkey vs `sha256()`/`hash256()` hash literal) per the doc-comment — a robust fix must consult **position in the parsed tree** (taproot key-position vs hash-fragment argument), which is why the bug-hunt prefers structural parsing over a regex widen. A naive regex widen to accept any `\b[0-9a-fA-F]{64}\b` would mis-flag keyless sha256-hashlock descriptors as keyed → a NEW wrong message. **Recommend the position-aware/structural fix, NOT a bare regex widen.**
- **SemVer:** cosmetic error-message correction, no surface change → **PATCH** (matches `PLAN §214`, TRIVIAL/PATCH).

---

## Cross-cutting

### SemVer roll-up
- L21 → **PATCH** recommended (REFUSE, no flag, reuses `ConvertRefusal`); FORMAL per the rule (adds a refusal). MINOR only if the cycle wants to flag the accepted-input-contract change — not recommended.
- L24 → **PATCH** (robustness; panic→clean `DescriptorParse`).
- L25 → **PATCH** (cosmetic message).
- **Cycle-11b composite version: toolkit PATCH → 0.65.1** (assuming no concurrent MINOR-bearing cycle lands first; if a MINOR cycle precedes it on the integration branch, renumber accordingly).

### clap-flag / `--json` / GUI schema_mirror / manual lockstep
- **With the recommended fixes, NO new clap flag, NO new subcommand, NO dropdown-value change, NO `--json` wire-shape change** → **no `mnemonic-gui/src/schema/mnemonic.rs` schema_mirror update required**, and **no `docs/manual/src/40-cli-reference/41-mnemonic.md` flag-table change**. Only **manual prose** SHOULD note the new L21 refusal (the manual already documents the BIP-38 vs BIP-39 passphrase split — see `docs/manual/src/50-comparing/56-bip39-vs-bip38-pass.md`, `…/40-cli-reference/41-mnemonic.md`); a one-line "composite phrase→bip38 now requires `--bip38-passphrase`" is in-keeping but not gate-enforced.
- **IF the cycle instead chooses the `--allow-empty-passphrase` opt-in for L21 (NOT recommended):** that is a NEW clap flag on `convert` → it MUST be mirrored into `mnemonic-gui/src/schema/mnemonic.rs` (the convert subcommand flag list at `:1084-1204+` already enumerates `--passphrase`, `--bip38-passphrase`, `--bip38-passphrase-stdin`, `--passphrase-stdin`) in lockstep (paired-PR rule), AND added to the manual flag table — a materially larger lockstep footprint. This is the second reason to prefer REFUSE.

### error.rs alphabetical-ordering convention
- **No new `ToolkitError` variant is needed for any of the three** — L21 reuses `ConvertRefusal(String)`, L24 and L25 reuse `DescriptorParse(String)` (variants already present and already alphabetically placed in `error.rs`). So the alphabetical-insertion discipline (CLAUDE.md; `PLAN §551`) does **not** bite cycle-11b. (Section 6.3 of the PLAN anticipated WS-CONVERT-BIP38 "(empty-passphrase refusal)" *might* add a variant — but the existing `ConvertRefusal` is the natural fit, so it does not.)

### Shared-toolkit-file collision / sequencing vs cycle-10 (md-codec pin-bump) and cycle-13
This lane **touches three toolkit files**; the collision map:
- **`cmd/convert.rs` (L21):** shared with **S-NET** (M14 `convert.rs:1100-1113`, L11 `convert.rs:1480-1491`) per `PLAN §616, §687` — WS-CONVERT-BIP38 is explicitly told to **serialize after S-NET** (shared `convert.rs`). If cycle-13 carries any S-NET/convert.rs hunks, **cycle-11b's L21 and cycle-13 must serialize on the convert.rs zone — not two concurrent branches.** L21's edit is localized (`:931-933` guard + `:1376` arm), so a rebase is mechanical, but it IS the same file.
- **`cmd/verify_bundle.rs` (L24):** the **S-VERIFY zone** — shared with **H1 + H12** (already-landed-here, but the structural dedup is open) per `PLAN §584, §292, §588`. The H12 fold already shifted L24's line numbers (see above). If cycle-13 carries the S-VERIFY `bundle.rs↔verify_bundle.rs` dedup, **L24's one-line gate should land WITH that dedup (or rebase onto it)** — the dedup would absorb L24's gate into the single shared function. **Flag: do NOT land L24 as a standalone gate if cycle-13's S-VERIFY dedup is imminent** — it would be immediately refactored away; coordinate so L24's fix is the gate inside the shared function.
- **`wallet_import/pipeline.rs` (L25):** shared with **S-NET / H15** per `PLAN §426, §619, §688` — WS-IMPORT-CLASSIFY is told to **rebase onto S-NET** (shared `pipeline.rs`). Same-file serialization with any cycle-13 S-NET pipeline.rs hunks.
- **cycle-10 (md-codec pin-bump, WS-MD-BCH):** that lane edits `md-codec/src/{codex32,encode,bch_decode}.rs` then does a **toolkit pin-bump (PATCH)** that touches `Cargo.toml`/`Cargo.lock` only (and possibly README pin sites + `fuzz/Cargo.lock`). **No source-file overlap with L21/L24/L25** — the only collision is **`Cargo.lock`** (per-file cheat-sheet `PLAN §570`: accept-theirs + `cargo check --workspace`, never `cargo update -w`) and the **toolkit version line in `Cargo.toml`** (whoever lands second renumbers). **Sequencing note: cycle-10's pin-bump and cycle-11b can run on separate branches; they only meet at `Cargo.toml`/`Cargo.lock` resolution, which is mechanical.**

**Net sequencing recommendation:** cycle-11b's three fixes are individually small and self-contained, but **all three files are in active multi-cycle zones** (convert.rs↔S-NET, verify_bundle.rs↔S-VERIFY, pipeline.rs↔S-NET). If cycle-13 carries S-NET/S-VERIFY work, **cycle-11b should rebase onto (or serialize after) cycle-13's S-NET + S-VERIFY landings** rather than run concurrently in the same file zones; the one genuinely-collision-sensitive item is **L24 vs the S-VERIFY dedup** (don't ship a throwaway standalone gate). cycle-10's pin-bump is orthogonal (Cargo metadata only).

---

## Summary verdicts table

| Finding | Citation | STILL-REPRODUCES | Fix-site (current line) | New variant? | New flag? | SemVer |
|---|---|---|---|---|---|---|
| **L21** (SECRET) | DRIFTED→ACCURATE | **YES, SILENT** | `convert.rs:1376` (+ guard `:931-933`) | no (`ConvertRefusal`) | **no — REFUSE recommended** | PATCH (FORMAL) |
| **L24** (PANIC) | DRIFTED (H12 fold)→ACCURATE | **YES, CLI-reachable** (operator-misuse) | `verify_bundle.rs` gate before `:1422` (mirror `bundle.rs:1373-1388`) | no (`DescriptorParse`) | no | PATCH |
| **L25** (cosmetic) | DRIFTED→ACCURATE | **YES, cosmetic** (wrong msg only) | `pipeline.rs:185-208` / `has_any_key_token:53-61` (position-aware, not regex-widen) | no (`DescriptorParse`) | no | PATCH |
