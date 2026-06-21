# R0 REVIEW — cycle-11b toolkit-hygiene (L21 · L24 · L25) — Round 1

**Spec:** `design/BRAINSTORM_cycle11b_toolkit_hygiene.md`
**Toolkit:** `origin/master = 4e8ad792` (v0.65.0).
**Verdict: NOT GREEN — 1 Critical, 1 Important**

All three findings reproduce on origin/master. The §0 *code* citations are accurate against origin/master (note: the local worktree is checked out at the unrelated v0.60.0 `feature/own-account-subset-search` branch, so a naive working-tree grep shows different line numbers — the spec correctly grepped origin/master for code). The fix families are sound. Two issues block GREEN.

## CRITICAL

### C1 — L21 refusal predicate D2 omits `from == Seedqr`; the footgun survives for the seedqr source

The composite `Bip38 =>` arm is reached for `from in {Seedqr, Phrase, Entropy}`, not `{Phrase, Entropy}`. The enclosing outer match is `Seedqr | Phrase | Entropy =>` at `convert.rs:1231` (origin/master). `(Seedqr, Bip38)` is an explicitly-permitted, argv-reachable edge (`classify_edge` whitelist `:637`; `"seedqr" => Self::Seedqr` `:77`). SeedQR decodes to a BIP-39 phrase and travels the identical composite path into `:1376`'s `bip38_passphrase.unwrap_or("")` — the exact silent-empty encrypt L21 targets.

The spec's **D2 predicate** (`from in {Phrase, Entropy}`, §0/§2.1/§6-D2/§3.1-RED) and the §3.1 RED test both **exclude Seedqr**. Compounded by the §2.1/D3 blessing to relocate the guard to `:932`: enforcing D2 literally there leaves `convert --from seedqr=<...> --to bip38 --passphrase X` still silently empty-encrypting — the footgun half-closed, and the surviving half is the one the test suite never exercises.

**Required (all of):**
1. **Fix the predicate** in D2/§2.1/§0/§6: `from in {Seedqr, Phrase, Entropy}` (the full composite source set matching the outer-match arm at `:1231`). Equivalently, if enforcing at the `:1350` arm head, state the refusal is **position-based** (no `from` test — the arm IS the composite set) and drop the `from`-set language so a future reader can't reintroduce the gap.
2. **Add a Seedqr RED test** to §3.1: `convert --from seedqr=<valid> --to bip38 --passphrase X` (no `--bip38-passphrase`) -> exit 2 with the refusal; and a Seedqr GREEN: `--bip38-passphrase ""` still encrypts.
3. **Manual prose (§4.4):** `56-bip39-vs-bip38-pass.md` edge table (`:52-54`) has no `(seedqr,bip38)` row. Add one ("REFUSED if unset") or generalize the composite rows to `(phrase|entropy|seedqr, bip38)`.

This is funds-safety (the L21 priority axis): a half-closed silent-weak-encryption footgun the proposed test suite would certify as "fixed" is worse than the current fully-open state — it manufactures false confidence.

## IMPORTANT

### I1 — §4.5 / D15 version-site "drift" claim is false against origin/master; release ritual mis-specified

§4.5/D15 assert the README ×2 / `install.sh` / `fuzz/Cargo.lock` "have **already silently drifted** … pinned at **0.60.0**." **False against origin/master @ 4e8ad792** (verified): Cargo.toml, README.md `<!-- toolkit-version: -->`, crates/mnemonic-toolkit/README.md, scripts/install.sh (`mnemonic-toolkit-v0.65.0`), fuzz/Cargo.lock are **ALL at 0.65.0** — none drifted. The `0.60.0` values are from the **dirty local worktree** (own-account branch off v0.60.0). The author grepped the working tree for version sites while correctly grepping origin/master for code.

**Why Important not Critical:** the correct ritual lands at the same end state (0.65.1 everywhere). But the spec's framing ("these four already drifted; refreshing is optional cleanup; R0 may skip them to keep minimal") would, against the **real** origin/master, leave four sites at 0.65.0 while Cargo.toml goes 0.65.1 — *introducing* the exact drift the spec wrongly believes exists.

**Required:**
1. Correct §4.5/D15: on origin/master all five sites are at 0.65.0; none drifted. The ritual MUST bump all five (READMEs ×2, install.sh, fuzz/Cargo.lock, Cargo.toml) + CHANGELOG to 0.65.1 in lockstep — **mandatory, not optional**.
2. Remove the "may skip already-drifted sites" escape hatch.
3. (Process) The cycle MUST branch off `origin/master` @ 4e8ad792, NOT the local own-account worktree. `git worktree add` off origin/master.

## MINOR (non-blocking)

- **M1:** Several §0 inline prose line refs mix worktree and master numbering. Load-bearing master citations (`:932`, `:1350`, `:1376`, `:1351`/`:1371`/`:1435`, `:1378`/`:1381`, pipeline `:56`/`:185`/`:529`/`:557`) all correct. Standardize on origin/master numbering.
- **M2 (L24 trigger precision):** the §3.2 RED test must ensure the descriptor is genuinely `is_non_canonical` (`canonical_origin(...).is_none()`) AND `@2` carries `Phrase`/`Seedqr`/`Ms1` (the `:1417-1419` subkey gate) so the override loop reaches `:1435`. Pin in the fixture comment.
- **M3 (L25 anchor set):** option (a)'s `pk(` anchor must keep the existing `0[23]…{64}` 66-hex compressed-key assertions GREEN (`has_any_key_token_distinguishes_keys_from_hashes` `:557`), not just the hash ones.

## CORRECT and verified (no action)

- **L21 `is_none()` vs `is_empty()`:** confirmed. `effective_bip38_passphrase = args.bip38_passphrase.clone()` (`Option<String>`, `:853`) -> `Some("")` for `--bip38-passphrase ""`, `None` absent; `--bip38-passphrase-stdin` always `Some(...)`. `is_none()` preserves the explicit-empty path. **D4/GREEN-1 sound.**
- **L21 reuses `ConvertRefusal`** (`error.rs:89`, exit 2 `:532`) — no new variant, no new flag. No schema_mirror/secret_drift/manual-flag-table edit.
- **L21 direct `(wif<->bip38)` edges genuinely separate** (`:1518`/`:1523`, `:1537`/`:1543`) — correctly left as-is.
- **L24:** OOB write at `:1435` genuinely unguarded + CLI-reachable; `validate_slot_set` (`slot_input.rs:249`) checks only contiguity + subkey-set, NOT range-vs-n. The `bundle.rs:1373-1388` gate transcribes exactly; insert after `:1351`/before `:1371`; `DescriptorParse` (exit 2) correct. **Fully correct** (modulo M2).
- **L25:** display-only — both `(false,false)` sub-arms (`:185-200`) return Err; no funds path, no `--json` shape. Hash-literal-stays-keyless invariant pinned by `:557`/`:529`. **Correct** (modulo M3).
- **Gate impact:** no new variant; no clap/`--json`/dropdown change -> no schema_mirror/secret_drift; L21 manual PROSE (`56-bip39-vs-bip38-pass.md` + `41-mnemonic.md:802`) is the only manual leg, same-commit; lint (flag-NAME only) won't flag prose.

## Verdict

**NOT GREEN.** Block on **C1** (Seedqr predicate gap — funds-safety) and **I1** (false version-site-drift claim). Fold M1–M3 opportunistically. After folding C1 + I1, persist this review, re-dispatch R0 (the loop continues until 0C/0I).
