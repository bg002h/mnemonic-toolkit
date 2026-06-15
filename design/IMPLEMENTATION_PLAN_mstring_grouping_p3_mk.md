# mstring display grouping — P3 (mnemonic-key / `mk`) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** Bring the `mk` CLI onto the standardized mstring display-grouping contract: `--group-size`/`--separator` on `mk encode` (default **space/5 print-once** — `mk encode` was UNBROKEN today, so this is a corrective default-output change); separator-stripping intake on all six mk1-intake subcommands via `read_mk1_strings`; canonical conformance vectors copied + checksum-pinned; a MINOR release of `mk-cli`.

**Architecture (deviation from spec §8 — for plan-R0 to ratify, parallel to P2):** `mk-cli` is **bin-only** (`[[bin]] name = "mk"`, no `lib.rs`). Host `render_grouped` + `strip_display_separators` + `is_display_separator` + `parse_separator` in a **NEW `crates/mk-cli/src/format.rs`** (the spec §8 "mk-cli formatting module"); conformance test = a **bin-crate `#[cfg(test)] mod`** in `format.rs`. **`mk-codec` is UNTOUCHED** → no mk-codec bump/publish. Only `mk-cli` bumps `0.8.0 → 0.9.0` (MINOR).

**Source SHA (grep-verified at write time):** `mnemonic-key` `main` `21786dc` (mk-cli 0.8.0 bin-only, mk-codec 0.4.0; mk-cli pins mk-codec `version = "0.4.0"`). Toolkit canonical vectors at `feature/mstring-display-grouping` (`design/display-grouping-vectors.tsv`, P0).

**Branch:** create `feature/mstring-display-grouping` in mnemonic-key.

**Spec:** `design/SPEC_mstring_display_grouping.md` (R0 GREEN). Implements the P3 rows of §9 (mk encode emit; `read_mk1_strings` intake).

---

## KEY FINDINGS / DECISIONS (recon `21786dc`)

1. **`mk-codec` decode tolerates NO separators** (`grep` of `crates/mk-codec/src/` finds no whitespace/hyphen strip). `read_mk1_strings` (`cmd/mod.rs:84`) does `.trim()` (`:93`) — **edge-only**, so interior separators are NOT stripped today. ⇒ the net-new strip coverage is **ALL interior separators** (space/hyphen/comma). Intake tests can use any; use **comma** for consistency with P1/P2.
2. **`read_mk1_strings` covers all SIX intake subcommands** — verified callers: `verify.rs:48`, `repair.rs:59`, `decode.rs:25`, `derive.rs:42`, `inspect.rs:29`, `address.rs:74`. One edit to `read_mk1_strings` covers all six.
3. **NO `mlock` module in `mk-cli`** (`src/` has no `mlock.rs`) → no g6 concern. The only fmt concern is the **rustfmt-1.95.0-pinned fmt gate** (`ci.yml` job `fmt (pinned 1.95.0)`); run `cargo +1.95.0 fmt --all` (no mlock exemption needed).
4. **`mk encode --separator bogus` exit = 64** (clap parse error → `main.rs:68-72` catch-all 64).
5. **Per-commit-green ORDERING = INTAKE-first, then EMIT (like P1).** No `mk encode | mk decode` CLI-pipe test exists, but intake-first is cleanest: land `read_mk1_strings` strip BEFORE `mk encode` emits grouped, so any decode/verify of grouped input already works. Execute: 1 → 2 → **4 (intake) → 3 (emit)** → 5 → 6 → 7.
6. **Known breaking test:** `tests/round_trip.rs::from_md1_derivation` invokes `mk encode` (CLI) and feeds the stdout lines DIRECTLY to `mk_codec::decode` (NOT via `mk decode`), so the CLI intake strip does NOT help it — it needs **`--group-size 0`** on that `mk encode` invocation. (`encode_decode_round_trip` + `verify_content_mismatch_exits_4` use `mk_codec::encode` for the strings → unbroken → unaffected.)
7. **`mk-cli` IS a crates.io crate** (`documentation = "https://docs.rs/mk-cli"`); release = publish + tag `mk-cli-v0.9.0`.

---

## Call-site inventory (grep-verified `21786dc`)

**Emit (apply `render_grouped` + flags):** `cmd/encode.rs::run` emit loop (`:93-95`): `for s in &strings { println!("{s}"); }` → `println!("{}", render_grouped(s, gs, sep))`. `EncodeArgs` += `group_size`/`separator`. (`--json` branch `emit_json` stays unbroken.)

**Intake (separator strip):** `cmd/mod.rs::read_mk1_strings` — `:93` `let s = line.trim();` → `strip_display_separators(line)` (then the non-empty guard); `:101` `out.push(a.clone());` → `out.push(crate::format::strip_display_separators(a));`. Covers all 6 subcommands.

**Pre-existing tests that BREAK:** `tests/round_trip.rs::from_md1_derivation` (mk encode → `mk_codec::decode`) → add `--group-size 0`. **Suite-sweep (Task 5) MUST re-confirm:** `cli_slip132.rs`, `gui_schema.rs` (new flags appear in `mk encode` — confirm assertion-based, not exhaustive golden), `cli_address.rs`, `cli_repair.rs`, `cli_derive.rs`, `cli_output_class.rs`, `version_help_exit_codes.rs` — grep for any exact `mk1…` stdout literal, `mk encode` stdout consumed-then-asserted, or `.len() ==` on encode output.

---

## Task 1: `mk-cli` `format.rs` — pure fns + `parse_separator`

**Files:** Create `crates/mk-cli/src/format.rs`; add `mod format;` to `crates/mk-cli/src/main.rs`.

- [ ] **Step 1:** Create `format.rs` with `is_display_separator`, `render_grouped`, `strip_display_separators`, `parse_separator` (byte-identical bodies to P1/P2) + a `#[cfg(test)] mod tests` with the standard unit tests (`render_grouped_separators_and_unbroken`, `strip_display_separators_ws_hyphen_comma`, `parse_separator_keyword_and_literal`).
- [ ] **Step 2:** Add `mod format;` to `main.rs` (after `mod error;`).
- [ ] **Step 3:** `cargo test -p mk-cli --bin mk format::` → PASS.
- [ ] **Step 4: Commit.**

## Task 2: Conformance vectors (copy + checksum + bin-crate test + CI)

**Files:** Create `design/display-grouping-vectors.tsv` (+ `.sha256`); add `#[cfg(test)] mod conformance` to `format.rs`; modify `.github/workflows/ci.yml`.

- [ ] **Step 1:** `cp` toolkit canonical TSV to `design/`; generate `.sha256`; `diff` byte-identical; `sha256sum -c`.
- [ ] **Step 2:** Add the conformance `#[cfg(test)] mod` (same driver as P2, reading `concat!(env!("CARGO_MANIFEST_DIR"), "/../../design/display-grouping-vectors.tsv")` — `CARGO_MANIFEST_DIR` = `crates/mk-cli`).
- [ ] **Step 3:** `cargo test -p mk-cli --bin mk conformance_vectors_pass` → PASS.
- [ ] **Step 4: CI checksum gate.** Add a step to a job with a plain checkout (the `fmt (pinned 1.95.0)` job, or the build job) BEFORE its cargo step:
```yaml
      - name: conformance-vector checksum pin
        run: cd design && sha256sum -c display-grouping-vectors.tsv.sha256
```
- [ ] **Step 5: Commit** TSV, `.sha256`, `format.rs`, `ci.yml`.

## Task 4: INTAKE — `read_mk1_strings` separator strip (all 6 subcommands)

**Files:** `cmd/mod.rs`; a new intake test.

- [ ] **Step 1: Failing test.** Add `tests/decode_grouped.rs` (or extend an existing test): build an unbroken mk1 via `mk_codec::encode` (the `round_trip.rs` V1 fixture), comma-group it (`comma5`), and assert `mk decode <comma-grouped>` succeeds with the same output as the unbroken form (or that it decodes — e.g. stdout contains the expected stub/fingerprint). Comma is the net-new separator.
- [ ] **Step 2:** Run → FAIL (comma rejected by mk-codec; `read_mk1_strings` doesn't strip interior).
- [ ] **Step 3: Implement** in `cmd/mod.rs::read_mk1_strings`: `:93` `let s = line.trim();` → `let s = crate::format::strip_display_separators(line);` (keep the `if !s.is_empty()` guard; note `s` is now `String` not `&str` → `out.push(s)` directly); `:101` `out.push(a.clone());` → `out.push(crate::format::strip_display_separators(a));`.
- [ ] **Step 4:** Run → PASS.
- [ ] **Step 5: Commit.**

## Task 3: EMIT — `mk encode --group-size`/`--separator`

**Files:** `cmd/encode.rs`; `tests/round_trip.rs` (fix `from_md1_derivation`); a new encode-flags test.

- [ ] **Step 1: Failing CLI tests.** Add `tests/encode_grouping_flags.rs`: `encode_default_groups_space_5` (first stdout line has `' '` at idx 5; space-stripped starts with `mk1`), `encode_unbroken_group_size_0`, `encode_separator_hyphen`, `encode_rejects_bad_separator` (exit 64). Use the `round_trip.rs` V1 fixture args (`--xpub V1_XPUB --origin-fingerprint aabbccdd --origin-path "m/48'/0'/0'/2'" --policy-id-stub 11223344`).
- [ ] **Step 2:** Run → FAIL.
- [ ] **Step 3: Implement.** `cmd/encode.rs`: `EncodeArgs` += `#[arg(long, default_value_t = 5)] pub group_size: u16,` + `#[arg(long, default_value = "space", value_parser = crate::format::parse_separator)] pub separator: char,`. Emit loop → `println!("{}", crate::format::render_grouped(s, args.group_size as usize, args.separator));`. (`--json` unchanged.) **Fix the breaking test:** `tests/round_trip.rs::from_md1_derivation` — add `"--group-size", "0"` to its `mk encode` args (so the stdout lines fed to `mk_codec::decode` stay unbroken).
- [ ] **Step 4:** Run → PASS (new tests + round_trip).
- [ ] **Step 5: Confirm gui-schema test.** `cargo test -p mk-cli --test gui_schema` → PASS (new flags auto-appear; if it pins an exact `encode` flag set, update it).
- [ ] **Step 6: Commit.**

## Task 5: Full suite + fmt (1.95.0) + clippy

- [ ] **Step 1:** `cargo test --workspace` → ALL green. Fix any pre-existing exact-output test (suite-sweep list above).
- [ ] **Step 2:** `cargo +1.95.0 fmt --all` then `cargo +1.95.0 fmt --all --check` → clean. (mk-cli has NO mlock.rs → full fmt is fine; no g6 exemption.)
- [ ] **Step 3:** `cargo clippy --workspace --all-targets -- -D warnings` → clean.
- [ ] **Step 4: Commit** any fixups.

## Task 6: Sibling FOLLOWUP companion

- [ ] File `display-grouping-render-strip-v1` in mnemonic-key's `FOLLOWUPS.md` (`git ls-files | grep -i followup`) with a `Companion:` line cross-citing the toolkit/md/ms entries. Commit.

## Task 7: Version bump + RELEASE (autonomous — authorized)

**Files:** `crates/mk-cli/Cargo.toml` (0.8.0 → 0.9.0); CHANGELOG if present; `Cargo.lock`.

- [ ] **Step 1:** Bump `mk-cli` `0.8.0 → 0.9.0` (MINOR). **mk-codec UNCHANGED (0.4.0).** Update CHANGELOG (`git ls-files | grep -i changelog`). `cargo build` to refresh lock.
- [ ] **Step 2:** `cargo test --workspace && cargo +1.95.0 fmt --all --check && cargo clippy --workspace --all-targets -- -D warnings` → green.
- [ ] **Step 3: Commit** the bump.
- [ ] **Step 4: RELEASE.** ff-merge `feature/mstring-display-grouping` → `main`, push. `cargo publish -p mk-cli --dry-run` then `cargo publish -p mk-cli`. Tag `mk-cli-v0.9.0`; push the tag. Verify CI green on main. (mk-codec NOT published — unchanged.)

---

## Self-Review (write-time)

**Spec coverage:** §3 algorithm → Task 1 + Task 2 vectors. §5 separator parser → Task 1. §6 print-once + json invariant → Task 3 (emit loop only; json untouched). §8 fn placement → Task 1 (deviation: mk-cli-local, ratify). §9.1 emit (`mk encode`, was unbroken → corrective grouping) → Task 3. §9.2 intake (`read_mk1_strings`, all 6 subcommands) → Task 4. SemVer MINOR (mk-cli only) → Task 7.

**Open items for plan-R0:** (1) ratify the §8 deviation (mk-cli-local fns; bin-crate conformance). (2) confirm INTAKE-first/EMIT-second ordering keeps every commit green. (3) confirm the breaking-test enumeration is COMPLETE — sweep `tests/` for any exact `mk encode` stdout pin beyond `from_md1_derivation` (P1/P2 lesson: the architect found several the plan's grep missed). (4) confirm `gui_schema.rs` is assertion-based. (5) confirm `--separator bogus` → exit 64.
