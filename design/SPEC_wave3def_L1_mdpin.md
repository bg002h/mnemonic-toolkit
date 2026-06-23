# SPEC — LANE1-mdpin: bump the deferred md-cli sibling pin v0.7.1 → v0.9.2 (3 sites) + reconcile the cross-tool-differential harness

**Lane:** LANE1-mdpin
**Repo:** `/scratch/code/shibboleth/mnemonic-toolkit` (only — no edit to `descriptor-mnemonic`)
**SemVer:** **NO-BUMP** (CI-pin + test-harness edit only; no toolkit library/CLI surface change — same pattern as the Wave-3 ms/mk/gui de-stale legs)
**Ship mechanism:** ONE ATOMIC commit to toolkit `master`, direct-FF (no PR). Stage paths explicitly (no `git add -A`).
**Classification:** funds-adjacent (touches the cross-tool md1 differential oracle test). Route through the standard R0 gate.
**Source SHA basis:** all citations re-grepped against the working tree at session start (HEAD = toolkit v0.71.0); md-cli v0.9.2 tag confirmed present locally in `descriptor-mnemonic` (`git tag --list` → `descriptor-mnemonic-md-cli-v0.9.2` exists).

---

## Why this is an M cascade, not a clean S pin-bump

The naive "just `sed` the 3 pins" fails: **md-codec 0.38.0** (commit `f9c1e57` "cycle-4 H6: encode-side 80-data-symbol cap in `wrap_payload`", shipped with md-cli **v0.8.1**, retained at v0.9.2 under md-codec 0.39.0) makes `md encode` **REFUSE single-string payloads >80 data symbols** with a non-zero exit:

> `payload is N data symbols; the codex32 regular code caps single strings at 80 (use chunked encoding / --force-chunked)` (exit 1)

The differential's `md_cli_ids()` calls `md encode` **without** `--force-chunked` and only reads `.phrase` / `.chunks` from JSON. At v0.9.2 every corpus entry's single-string encode exits non-zero → helper returns `None` → verdict `ToolError(MdCli)` → the gate goes **RED**.

**This was empirically reproduced by the spec author** (built md v0.9.2 `--features cli-compiler` from the tag; ran the unmodified test with `MD_BIN` = the v0.9.2 binary): every one of the 17 corpus entries reported `EXPECTED Match but got ToolError(MdCli)` / `md-cli =None`, `test result: FAILED`.

**The walkers still AGREE** — the breakage is purely the harness not passing `--force-chunked`. With `--force-chunked` added (spec author applied it temporarily, ran, then reverted — ships no code), all 17 entries reported `Match OK` with **byte-identical** policy_id / template_id between toolkit and md-cli (e.g. `wpkh` `1c0170fe82855f60eeca91a9899b0abe` / `45775d4d6561625de6efadaad70a1e9b`; `wsh-pk` `58d1803363f5599914a9f4ba0afa97d7` / `9208f59035e4912d4fca8182a897fafb`), `test result: ok. 1 passed`. So the differential SIGNAL (walker equivalence) is fully preserved; the fix is test-harness-only.

The chunked path is the encoding md-codec itself recommends; the 80-symbol single-string refusal is a funds-safety **hardening**, correctly accommodated by switching the harness to `--force-chunked`.

---

## Change 1 — bump install.sh canonical md pin (the source of truth for sibling-pin-check)

**File:** `scripts/install.sh`
**Line (verified):** `35`

**Current behavior** — `component_info()`'s `md)` arm pins the canonical md tag at v0.7.1:
```
            echo "md-cli|https://github.com/bg002h/descriptor-mnemonic|descriptor-mnemonic-md-cli-v0.7.1|yes|cli-compiler"
```

**Exact edit** — replace the tag substring `descriptor-mnemonic-md-cli-v0.7.1` → `descriptor-mnemonic-md-cli-v0.9.2` on line 35. Leave the rest of the pipe-delimited record (`md-cli|<url>|…|yes|cli-compiler`) untouched.

This is the canonical table `sibling-pin-check` parses; it MUST lead (the gate requires every workflow `--tag` to equal this value).

---

## Change 2 — bump manual.yml md-cli install pin

**File:** `.github/workflows/manual.yml`
**Line (verified):** `86`

**Current behavior** — the "Install md-cli" step pins v0.7.1:
```
        run: cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.7.1 md-cli --features cli-compiler
```

**Exact edit** — replace `descriptor-mnemonic-md-cli-v0.7.1` → `descriptor-mnemonic-md-cli-v0.9.2` on line 86. The preceding comment block (lines 81-85, which references `scripts/install.sh:35`) needs **no change** — the install.sh:35 line reference is still correct.

---

## Change 3 — bump cross-tool-differential.yml md-cli install pin + rewrite the stale skew comment

**File:** `.github/workflows/cross-tool-differential.yml`

### 3a — the pin (verified line `50`)

**Current behavior:**
```
        run: cargo install --git https://github.com/bg002h/descriptor-mnemonic --tag descriptor-mnemonic-md-cli-v0.7.1 md-cli --features cli-compiler
```

**Exact edit** — replace `descriptor-mnemonic-md-cli-v0.7.1` → `descriptor-mnemonic-md-cli-v0.9.2` on line 50.

### 3b — the comment (verified lines `42-49`)

**Current behavior** — lines 43-49 carry a now-stale rationale describing the v0.6.2→v0.7.1 skew as "wire-neutral … md-cli's source is byte-identical across the tags (only the md-codec pin moves to =0.37.0…)". That skew note is obsolete once we move to v0.9.2.

**Exact edit** — replace the comment lines `42-49` (keeping line 42's `--features cli-compiler matches scripts/install.sh:35's pin.` sentence) with a rewrite that documents the new pin rationale + the 80-symbol cap. Suggested replacement body (preserve the leading `#` + 8-space YAML indentation, and keep the `--features cli-compiler matches scripts/install.sh:35's pin.` clause):

```yaml
        # `--features cli-compiler` matches scripts/install.sh:35's pin. PIN
        # AT md-cli v0.9.2: this is wire-neutral for the differential SIGNAL —
        # the toolkit and md-cli walkers emit byte-identical
        # wallet_policy_id / wallet_descriptor_template_id across the corpus
        # (proven at v0.9.2: all entries Match). The behavioral delta v0.9.2
        # carries is md-codec 0.38.0's encode-side 80-data-symbol cap on
        # SINGLE-STRING `md encode` (shipped with md-cli v0.8.1; commit
        # f9c1e57): payloads >80 data symbols are REFUSED unless chunked. The
        # harness therefore calls `md encode --force-chunked` (see
        # cli_cross_tool_differential.rs::md_cli_ids), which is what keeps this
        # gate GREEN at v0.9.2; without it every entry would ToolError.
```

(Exact prose is at implementer discretion — the load-bearing facts to capture: v0.9.2 pin, walker-equivalence preserved, the md-codec 0.38.0 / md-cli v0.8.1 80-symbol single-string cap, and the `--force-chunked` requirement. Drop the v0.6.2→v0.7.1 / md-codec =0.37.0 / #25-no-trigger narrative entirely — it is stale.)

> Note: editing this comment (and the test in Change 4) re-fires the differential gate on push via its paths-filter (`.github/workflows/cross-tool-differential.yml` + `crates/mnemonic-toolkit/tests/cli_cross_tool_differential.rs` are both in the `paths:` list at lines 21-23 / 26-28) — which is exactly the verification we want.

---

## Change 4 — MANDATORY same-commit test fix: add `--force-chunked` to `md_cli_ids()`

**File:** `crates/mnemonic-toolkit/tests/cli_cross_tool_differential.rs`
**Function:** `md_cli_ids()` (verified spans ~`191-220`)

**Current behavior** — the helper builds the `md encode` argv and pushes `--path`, the path, then `--json` (verified lines `201-203`):
```rust
    args.push("--path".to_string());
    args.push(entry.md_path.to_string());
    args.push("--json".to_string());
```
It already consumes BOTH outputs: `.phrase` single-string (lines 211-212) OR `.chunks` array (lines 213-217). Adding `--force-chunked` makes md emit `.chunks` for every entry — the existing `.chunks` arm handles it unchanged; the `.phrase` arm simply stops being taken.

**Exact edit** — insert `--force-chunked` into the argv **before** the `--json` push (keep `.chunks`/`.phrase` consumption exactly as-is):
```rust
    args.push("--path".to_string());
    args.push(entry.md_path.to_string());
    args.push("--force-chunked".to_string());
    args.push("--json".to_string());
```

**Do NOT** alter the `.phrase`/`.chunks` branch logic (lines 210-219), the `Entry`/`MdKey` structs, the corpus, the `FP` const, or the `inspect_ids` path. Do NOT touch `toolkit_ids()`. This is the minimal, sufficient edit (empirically proven GREEN).

**TDD note for the implementer:** this is a test-harness fix whose "test" IS the differential run itself. The TDD loop is: (1) bump the 3 pins, point `MD_BIN` at a built v0.9.2 md, run the `#[ignore]` differential → observe RED (`ToolError(MdCli)` ×17); (2) add `--force-chunked`; (3) re-run → GREEN (17/17 `Match OK`). The spec author already executed this loop end-to-end; it reproduces deterministically.

---

## What is explicitly NOT changed (verified)

- **No `42-md.md` edit.** Manual flag-coverage is forward-only (`md <sub> --help` flags must appear in the chapter). The v0.7.1↔v0.9.2 md help surface is byte-identical (subcommand list + per-subcommand flag set unchanged; the v0.7.1→v0.9.2 source diff only touched `after_long_help` text + an advisory stderr branch + parser internals — zero clap surface change). `--force-chunked` — the only flag the test newly passes — is ALREADY documented at `docs/manual/src/40-cli-reference/42-md.md:35`. Confirmed: flag-coverage stays GREEN with no doc edit.
- **No README pin.** The only 3 md `cargo install --tag` lines repo-wide are the 3 above (`grep -rn 'descriptor-mnemonic-md-cli-v0.7.1' scripts/ .github/ docs/` returns exactly install.sh:35, manual.yml:86, cross-tool-differential.yml:50). The manual prose chapters mention md-cli only in comments, not as install pins.
- **No toolkit version bump** (NO-BUMP) — no `Cargo.toml`, no READMEs, no `install.sh` self-pin, no fuzz lock. `install-pin-check.yml` is self-pin-only scope (toolkit version vs tag) and is NOT triggered by an md pin change.
- **No `descriptor-mnemonic` edit** — md-cli v0.9.2 is already published/tagged.
- **g6 mlock anchor** is toolkit-side and unaffected by an md pin. `bitcoind-differential.yml` does NOT consume md-cli.

---

## Verification (the spec MANDATES all three before commit; ABSOLUTE MD_BIN throughout)

> **LOCAL GOTCHA (mandatory):** the fish profile aliases `md` → `mkdir -p`. The differential test and the manual lint read `MD_BIN` from env — pass an **ABSOLUTE** path to the built v0.9.2 binary. Never rely on a PATH lookup of `md` (it silently mis-resolves to `mkdir`).

Build the v0.9.2 md binary first:
```
# in a detached worktree of descriptor-mnemonic at the tag:
git worktree add --detach <wt> descriptor-mnemonic-md-cli-v0.9.2
cd <wt> && cargo build -p md-cli --features cli-compiler --bin md
# → <wt>/target/debug/md ; confirm `<wt>/target/debug/md --version` == "md 0.9.2"
```

1. **sibling-pin-check (atomicity).** After all 3 edits:
   `grep -rn 'descriptor-mnemonic-md-cli-v' scripts/install.sh .github/workflows/` must show **only** `…v0.9.2` at exactly install.sh:35, manual.yml:86, cross-tool-differential.yml:50 — **zero** `v0.7.1` remaining. (Touching only 2 of 3 → the gate goes RED.)

2. **cross-tool-differential reports MATCH at v0.9.2.** Build the toolkit debug binary (`cargo build --bin mnemonic`), then:
   ```
   MNEMONIC_BIN=<abs target/debug/mnemonic> MD_BIN=<abs v0.9.2 md> \
     cargo test -p mnemonic-toolkit --test cli_cross_tool_differential -- --ignored --nocapture
   ```
   PASS criterion: all 17 entries `Match OK`, `test result: ok. 1 passed`. **Spec-author-proven GREEN** (and proven RED without the `--force-chunked` edit — both directions reproduce deterministically).

3. **manual flag-coverage unchanged.**
   ```
   make -C docs/manual lint \
     MNEMONIC_BIN=<abs mnemonic> MD_BIN=<abs v0.9.2 md> MS_BIN=<abs ms> MK_BIN=<abs mk>
   ```
   PASS criterion: step `4/6 flag-coverage` passes for md with no `42-md.md` change (every md v0.9.2 flag already documented). Verified GREEN.

---

## FOLLOWUP flips (same commit)

**File:** `design/FOLLOWUPS.md`
**Entry:** `install-sh-sibling-pins-stale-vs-flag-bearing-clis` (verified at lines `120-124`; the `Status:` line is `124`).

**Edit** — update the `Status:` line (124) to record the md-cli leg as resolved. The current text reads:
> **The md-cli leg is deliberately HELD at `descriptor-mnemonic-md-cli-v0.7.1`** (all 3 md sites: install.sh:35, manual.yml:86, cross-tool-differential.yml:50) because bumping it re-fires the frozen `cross-tool-differential` walker-divergence baseline (needs a `#[ignore]`-gated differential re-run against md-cli v0.9.x, unprovable in this lane).

Replace that sentence with a resolution note, e.g.:
> **The md-cli leg is now RESOLVED (2026-06-23, NO-BUMP):** bumped `descriptor-mnemonic-md-cli-v0.7.1 → v0.9.2` at all 3 md sites (install.sh:35, manual.yml:86, cross-tool-differential.yml:50) in one atomic commit. The deferred concern (the bump turns `cross-tool-differential` RED) was REAL and is reconciled in the same commit: md-codec 0.38.0's 80-data-symbol single-string-encode cap (shipped md-cli v0.8.1) required adding `--force-chunked` to `cli_cross_tool_differential.rs::md_cli_ids()`. The `#[ignore]`-gated differential re-ran GREEN at v0.9.2 (all 17 corpus entries Match; walker ids byte-identical). Manual flag-coverage stays GREEN (md help surface unchanged v0.7.1↔v0.9.2). With ms/mk/gui already de-staled (Wave-3) and md-cli now bumped, **all 4 sibling legs of this slug are RESOLVED.**

If the whole-slug status now satisfies "RESOLVED" per the project's convention (all 4 legs done), additionally flip the entry's status marker / Tier per the resolved-entry style used elsewhere in the file (e.g. the `✓ RESOLVED (… date …)` heading form). Keep the historical "Why deferred" context intact for the audit trail.

---

## Commit (single, atomic)

Staged paths (explicit):
```
git add scripts/install.sh \
        .github/workflows/manual.yml \
        .github/workflows/cross-tool-differential.yml \
        crates/mnemonic-toolkit/tests/cli_cross_tool_differential.rs \
        design/FOLLOWUPS.md
```
Suggested message:
```
ci(pins): bump md-cli sibling pin v0.7.1 → v0.9.2 (3 sites) + force-chunked differential reconcile

Atomic NO-BUMP. md-codec 0.38.0 (md-cli v0.8.1) caps single-string `md encode`
at 80 data symbols, so the cross-tool differential's md_cli_ids() now passes
--force-chunked. Walkers proven byte-identical at v0.9.2 (17/17 Match);
manual flag-coverage unchanged (md help surface identical v0.7.1↔v0.9.2).
Resolves install-sh-sibling-pins-stale-vs-flag-bearing-clis (md leg).
```

---

## Risk summary

LOW and bounded, given Change 4 ships in the SAME commit. Walker semantics proven byte-identical across v0.7.1, v0.9.2, and toolkit (uniform corpus mechanism → all 17 follow the proven `wpkh`/`wsh-pk` controls). The only behavioral delta v0.9.2 introduces is the funds-safety 80-symbol single-string refusal — a hardening, correctly accommodated by `--force-chunked`. **If shipped pin-only WITHOUT Change 4 → `cross-tool-differential` goes RED** (caught immediately on push by its paths-filter, which includes both the test and the workflow). No funds-path code changes; manual flag-coverage carries no regression (surface identical).