# SPEC — Output-class stderr advisory, Phase 2: mk-cli + md-cli sibling sweep (+ folded Tier-0 md-codec pin bump)

**Cycle:** Cycle B Phase 2. Companion to the shipped Phase-1 spec `design/SPEC_output_type_advisory.md` (mnemonic + ms, v0.38.2 / ms-cli v0.5.1).
**FOLLOWUP:** `output-type-stderr-advisory-sibling-sweep-mk-md` (toolkit `design/FOLLOWUPS.md:3394`; mirrors in `mnemonic-key`, `descriptor-mnemonic`).
**Status of this spec:** **R0 GREEN** — reviewer-loop R0 RED (2C/3I/4M) → fold → R1 **GREEN (0C/0I)**. Reviews persisted at `design/agent-reports/output-type-advisory-phase2-spec-R{0,1}-review.md`. Cleared to advance to a plan-doc (which gets its own R0 gate).

> **Reading note:** §2 describes the **toolkit's** `secret_advisory.rs` as the *output reference* (it derives `Ord` and lives in a lib where all variants are constructed); §4 mandates the **ms-cli** `advisory.rs` as the *implementation template* for the bin-only mk/md CLIs (no `Ord`, `#[allow(dead_code)]`). Implement from §4, not §2.

## Source SHAs (citations below are from live reads at these checkouts; the plan-doc re-greps at impl time per project convention)
- mnemonic-toolkit — `master` `64943f2` (v0.38.2)
- mnemonic-key (mk) — `main` `e5620ce` (mk-codec 0.4.0 / mk-cli 0.6.0)
- descriptor-mnemonic (md) — `main` `c599292` (md-codec 0.35.0 / md-cli 0.6.1)

---

## 1. Goal

Extend the Phase-1 output-class stderr advisory to **mk-cli** and **md-cli**, completing the constellation-wide invariant **"no advisory line ⟺ inert stdout."** Behavior is **stderr-only**; no new flags, subcommands, or wire-shape changes anywhere. Fold in the **Tier-0 md-codec 0.34→0.35 pin bump** (a separate toolkit-side correctness fix) in the same toolkit release.

### Why now / why these two
mk/md outputs are non-secret (mk = watch-only xpubs by construction; md = keyless descriptor templates). Phase 1 deliberately shipped the **secret-bearing** surfaces (mnemonic, ms) first because mk/md's interim silence is benign (no fund-loss path). Phase 2 closes the consistency gap so the advisory signal is uniform across the whole constellation — which is precisely what preserves the loss-avoidance value of the scary `private key material (can spend)` line on mnemonic/ms: a user trained that *every* output command speaks its class will trust and act on that line. Bounded item: must close before the next constellation `install.sh` sibling-pin bump that re-pins mk/md.

---

## 2. Byte-parity source of truth (Phase 1, already shipped)

`mnemonic-toolkit/crates/mnemonic-toolkit/src/secret_advisory.rs`:
- `enum OutputClass { Template, WatchOnly, PrivateKeyMaterial }` (`:80`) — declaration order = ascending sensitivity; `#[derive(Ord)]` → `.max()` returns most-sensitive. **inert = `Option::None`** (the absence of a class, not a variant).
- `worst_class_on_stdout(&[OutputClass]) -> Option<OutputClass>` (`:83`) — `None` ⟺ all-inert ⟺ no line.
- `emit_output_class_advisory(class, &mut stderr)` (`:97`) — the single emitter. **Exact lines (note em-dash `—` U+2014):**
  - `PrivateKeyMaterial` → `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')`
  - `WatchOnly` → `note: stdout is watch-only — public keys only, cannot spend`
  - `Template` → `note: stdout is a keyless descriptor template (no keys)`

**Phase-1 precedent for repair/inspect (the analogy mk/md must follow):** the toolkit classes its own non-secret-equivalent outputs — `emit_repair_report` fires `card_kind_class(outcome.kind)` (`repair.rs:1334`, `Mk1→WatchOnly` / `Md1→Template`), `inspect` uses `worst_class_on_stdout(&kinds)` (`cmd/inspect.rs:151`), `bundle` fires WatchOnly/PrivateKeyMaterial (`cmd/bundle.rs:932`), `convert` fires `worst_class_on_stdout` with `None` for side-input-only outputs (`cmd/convert.rs:1101`). Full coverage of non-inert stdout is the established pattern.

---

## 3. Coverage rule & per-subcommand class mapping (architect-resolved)

**Rule:** *Every mk/md subcommand that writes a card string, decoded card content (xpub / template / addresses / binary payload), or derived output to stdout emits `emit_output_class_advisory` with the appropriate class. Subcommands whose stdout is a pass/fail diagnostic (`verify`), machine corpus data (`vectors`), or CLI infrastructure (`gui-schema`) emit nothing.*

mk **never** emits PrivateKeyMaterial; md **never** emits PrivateKeyMaterial — by construction.

### mk-cli (`mnemonic-key/crates/mk-cli/src/`)
| Subcommand | stdout (file:line) | Class | Note |
|---|---|---|---|
| `encode` | mk1 string(s) (`cmd/encode.rs:59–80`) | **WatchOnly** | encodes an xpub; `--json` envelope doesn't change class |
| `decode` | xpub + origin fields (`cmd/decode.rs:42–50`) | **WatchOnly** | xpub in plaintext |
| `inspect` | decode superset + xpub_fp + BCH variants (`cmd/inspect.rs:57–74`) | **WatchOnly** | xpub at `:58` |
| `repair` | corrected mk1 string(s) (`cmd/repair.rs:169`) | **WatchOnly** | emits a card; mirror toolkit `emit_repair_report`; success path only |
| `derive` | child xpub + origin (`cmd/derive.rs:78–97`) | **WatchOnly** | xpub at `:80` |
| `address` | addresses (`cmd/address.rs:234–263`) | **WatchOnly** | |
| `verify` | `OK: …` or `{"ok":true,…,"policy_id_stubs":[…]}` (`cmd/verify.rs:127–133`) | *inert* | pass/fail; stubs are 4-byte hashes, not keys (footgun F1) |
| `vectors` | SHA-pinned corpus JSON **on stdout** (`cmd/vectors.rs:44–46`) | *inert* | machine test-corpus data → inert **by rule** (R0 M2 — NOT "stdout empty"; contrast md `vectors` which writes files) |
| `gui-schema` | CLI-surface JSON (`cmd/gui_schema.rs:41–44`) | *inert* | infrastructure |

### md-cli (`descriptor-mnemonic/crates/md-cli/src/`)
| Subcommand | stdout (file:line) | Class | Note |
|---|---|---|---|
| `encode` | md1 string(s) (`cmd/encode.rs:59–80`) | **Template** | md1 is a keyless template; `--key @i=XPUB` binds xpubs *inside* the card but the artifact class is still Template (footgun F4) |
| `decode` | `@N` template (`cmd/decode.rs:28`) | **Template** | clearest Template case |
| `inspect` | template + structural metadata (`cmd/inspect.rs:46–58`) | **Template** | no keys on stdout |
| `bytecode` | raw codec bytes hex + bit/byte counts (`cmd/bytecode.rs:33–41`) | **Template** | the hex IS the template's binary form (footgun F2 — weakest call, kept Template per the invariant) |
| `repair` | corrected md1 (`cmd/repair.rs:147–151`, `:195–202`) | **Template** | success path only; error path `Ok(2)` → empty stdout → inert (footgun, correct) |
| `compile` | keyless template string (`cmd/compile.rs:25`) | **Template** | feature `cli-compiler` (default-OFF) — emit lives **inside** the gate (footgun F3) |
| `address` | addresses, lines or JSON (`cmd/address.rs:46–65`) | **WatchOnly** | derived from concrete xpubs; only md WatchOnly case |
| `verify` | `OK` only (`cmd/verify.rs:44`) | *inert* | no `--json`; one-word diagnostic |
| `vectors` | writes files to disk; **stdout empty** (`cmd/vectors.rs:47–71`) | *inert* | writes `--out` dir, no stdout (footgun) |
| `gui-schema` | CLI-surface JSON | *inert* | feature `json`; infrastructure |

> Line numbers are snapshots at the header SHAs; the plan-doc re-greps each before code (CLAUDE.md). Known off-by-1/2 from R0 M4 (no class verdict changes): mk `derive` child_xpub at `:79`/`:91` (not `:80`=fingerprint); mk `verify` JSON shape `:118–126` (the `:127–133` cite is the text path); md `repair` emit-text tail `:197–199`. The byte-parity source-of-truth (`secret_advisory.rs:80/:83/:97`) and all four Phase-1 precedent sites are EXACT.

---

## 4. Helper design (greenfield in both CLIs) — mirror the shipped ms-cli precedent literally

Codecs/CLIs are **upstream** of the toolkit; neither mk-cli nor md-cli may depend on `mnemonic-toolkit`. Each repo gets its own copy. **The literal template is `mnemonic-secret/crates/ms-cli/src/advisory.rs` + `tests/cli_output_class.rs` (Phase-1's shipped port), NOT the toolkit's lib-internal `secret_advisory.rs`** — the toolkit version lives in a lib where all variants are constructed and derives `Ord`, which has different dead-code/lattice characteristics than a bin-only CLI needs (R0 C1/C2).

- New module **`output_advisory.rs`** in each CLI's `src/` (ms-cli named it `advisory.rs`; either name is fine), containing:
  - `#[allow(dead_code)] #[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum OutputClass { PrivateKeyMaterial, WatchOnly, Template }` — **no `PartialOrd`/`Ord`**, exactly as ms-cli (`advisory.rs:26-32`).
  - `pub fn emit_output_class_advisory<W: Write>(class, &mut stderr)` with the three literal lines written using the **`\u{2014}` escape** for the em-dash (ms-cli `advisory.rs:40-41`).
  - **Do NOT port `worst_class_on_stdout`** (ms-cli didn't) and **do NOT port `card_kind_class`** (it maps the toolkit's multi-kind `repair::CardKind`). Every emitting mk/md handler produces a single card-kind per invocation, so no max is needed — mk emits a literal `OutputClass::WatchOnly`; md emits `OutputClass::Template` (or `WatchOnly` for `address`).
- **`#[allow(dead_code)]` is REQUIRED on the enum** (R0 C1): bin-only crates (mk-cli/md-cli have only `main.rs`, no `lib.rs`) flag never-constructed variants under `cargo clippy --all-targets -- -D warnings` (both CIs enforce this), and `#[cfg(test)]` construction does NOT keep bin-target items live. mk constructs ONLY `WatchOnly` (both `Template` and `PrivateKeyMaterial` unconstructed); md constructs `Template`+`WatchOnly` (`PrivateKeyMaterial` unconstructed). The unused variants exist only for byte-parity completeness of the advisory text.
- **`missing_docs = "warn"` + `-D warnings`** (both workspaces, R0 M1): every `pub` item in the new module needs a `///` doc — copy ms-cli's doc comments.
- **Emit placement — at EVERY success return site (R0 I1), not just the final one.** Five+ md handlers `return Ok(0)` early from inside a `#[cfg(feature = "json")] if json { … }` block BEFORE the text-path return: `md encode` (`cmd/encode.rs:69`+`:95`), `md decode` (`:24`+`:30`), `md inspect` (`:42`+`:59`), `md bytecode` (`:30`+`:41`), `md address` (`:57`+text), and `md compile` (`:21`+`:25`, inside the `cli-compiler` gate). The emit must fire on BOTH the json and text branches of each (json is a **default** feature). `md repair` is the exception: json/text fall through to one `Ok(if any_correction {5} else {0})` (`cmd/repair.rs:153`) → single emit at `:152`, which correctly skips the `Ok(2)` error path (`:121`). All mk emitting handlers have a single trailing `Ok(0)` after the json/text dispatch → one emit point each. The plan-doc MUST enumerate the exact emit line(s) per handler. Match existing CLI I/O style (mk/md `eprintln!`-family / direct stderr).

---

## 5. Byte-parity contract & test plan — model on ms-cli's `cli_output_class.rs`

- **Per-repo byte-parity test** modeled on `mnemonic-secret/crates/ms-cli/tests/cli_output_class.rs::byte_parity_advisory_lines` (R0 C2): assert the emitted lines are byte-identical to **hard-coded literal constants** of the three toolkit lines (em-dash as `\u{2014}`), via substring `contains` on the actual CLI stderr. Do NOT assert a `.max()` lattice or `worst_class_on_stdout` — neither is ported. "Byte-identical" is scoped to the three literal strings only.
- **Per-subcommand integration cells** (each CLI's `tests/`): assert the advisory **presence + exact class line** on every emitting subcommand, and **absence** on `verify` / `vectors` / `gui-schema`. Explicit cells required (R0 I1) for BOTH output modes wherever a `--json` early-return exists: `md encode --json`, `md decode --json`, `md inspect --json`, `md bytecode --json`, `md address --json` (and `md compile --json` under `#[cfg(feature = "cli-compiler")]`), in addition to their text-mode cells. Plus edge cases: `md vectors` (no advisory; writes files, stdout empty), `md repair` error path (exit 2 → no advisory), `md encode --key` (still Template), mk `--json` mode (still emits), mk `repair` exit-5 path (correction applied → still WatchOnly).
- **No automated cross-repo gate** is possible (no shared dep). Parity rests on the identical literals + the FOLLOWUP's "byte-identical wording" discipline — the same model Phase 1 used (held empirically per the Phase-1 record).

---

## 6. Scope boundaries / lockstep

- **No clap surface change ⇒ no GUI `schema_mirror` change** (it gates flag-NAMES; advisory adds none) and **no manual flag-coverage change** (the `40-cli-reference` mirror is flag-driven).
- **CI-gated manual transcript re-capture (REQUIRED).** The `verify-examples` harness captures stderr (pair mode `2>&1`; triple mode `.err`). **Confirmed affected set = `docs/manual/transcripts/24-recover-md1.{cmd,out}` ONLY** (R0 M3 full sweep: `$MK_BIN` is never invoked in any transcript; `cross-format-recipes/` + `foreign-formats/` use only `$MNEMONIC_BIN`; `cli-help/*.txt` are `--help` captures with no advisory). `24-recover-md1` runs `$MD_BIN decode …` → will now carry the `template` note. It is **pair-mode** (`2>&1` merged, no `.err`): stdout (`println!`, block-buffered on a pipe) and the stderr advisory (unbuffered) may interleave non-deterministically, so the plan MUST re-capture AND re-run `verify-examples` to confirm idempotency before committing the new `.out`. The impl should still re-sweep `transcripts/` at code time to catch any newly-added transcript. Re-capture only when current output is CORRECT (Phase-1 lesson: a re-capture once masked a stale wire-version defect).
- **Optional prose:** a manual note explaining the advisory lines is nice-to-have, not gating; follow Phase-1's precedent (and the `manual-prose-command-execution-gate`).

---

## 7. Folded Tier-0 — md-codec 0.34→0.35 pin bump (toolkit-side correctness fix)

Independent of the advisory work; ships in the same toolkit tag.
- **Defect:** toolkit FOLLOWUP `md-codec-decode-with-correction-supports-non-chunked-md1` (`FOLLOWUPS.md:396`) is `Status: resolved v0.24.0 cycle` and asserts *"`mnemonic repair --md1` now accepts non-chunked single-string md1 inputs end-to-end — no toolkit code change required beyond the md-codec dep version bump"* — but the pin is still `md-codec = "0.34.0"` (`Cargo.toml:22`; `Cargo.lock:644` resolves 0.34.0). md-codec **0.35.0** (published; carries the non-chunked-form pre-pass) was never consumed. So `mnemonic repair --md1` on a non-chunked single-string md1 still hits the chunked-only path (`repair.rs:~1199`, `RepairError::UnparseableInput`). The descriptor-mnemonic primary correctly shows the codec fix shipped (`descriptor-mnemonic/design/FOLLOWUPS.md:76`, RESOLVED in md-codec-v0.35.0).
- **Pre-flight (R0 I3b):** the toolkit pin is a **caret** (`^0.34.0 = >=0.34.0,<0.35.0`), so `cargo update` will NOT pull 0.35.0 — the explicit edit to `"0.35"` is required. md-codec 0.35.0 must be published to crates.io for the bump to resolve (codecs are registry deps; only miniscript has a `[patch]`). **CONFIRMED on crates.io** (present in the local registry cache `~/.cargo/registry/cache/.../md-codec-0.35.0.crate`).
- **Fix:** bump pin to `md-codec = "0.35"`; `cargo build`/`cargo test` to re-resolve `Cargo.lock` (run cargo BEFORE staging the lockfile — stale-lock gotcha); add the end-to-end smoke test the FOLLOWUP promised (encode a small payload non-chunked via `md`, corrupt one symbol, `mnemonic repair --md1` recovers it — exercise the full path incl. the `parse_chunk` pre-gate, not just the codec call); **correct the Resolution prose to the actual bump SHA** (it is currently asserting a capability no shipped build has).
- Additive API ⇒ low risk (the 0.34→0.35 diff is one additive pre-pass in `chunk.rs`; chunked `reassemble` untouched); no GUI/manual flag change; no toolkit transcript invokes `mnemonic repair` → zero transcript drift from Tier-0.

---

## 8. Cross-repo phasing & versions

Three independent units; mk ⊥ md (no shared state). Each gets its own R0-gated TDD pass.
1. **mk-cli** PATCH (0.6.0 → **0.6.1**): `output_advisory.rs` + emit in 6 handlers (encode/decode/inspect/repair/derive/address) + tests → **git tag `mk-cli-v0.6.1`** (R0 I3: the artifact the toolkit pin gate consumes via `cargo install --git … --tag`; crates.io publish is a separate optional step).
2. **md-cli** PATCH (0.6.1 → **0.6.2**): `output_advisory.rs` + emit in 7 handlers (encode/decode/inspect/bytecode/repair/compile[feature-gated] = Template; address = WatchOnly) + tests → **git tag `descriptor-mnemonic-md-cli-v0.6.2`** (gate-relevant artifact; crates.io publish separate/optional).
3. **toolkit** PATCH: bump the **5 sibling-pin tag sites** (R0 I2 — `sibling-pin-check.yml` scans ALL workflows, not just manual.yml): `scripts/install.sh:35` (md→`descriptor-mnemonic-md-cli-v0.6.2`), `install.sh:41` (mk→`mk-cli-v0.6.1`), `.github/workflows/manual.yml:77` (mk), `manual.yml:84` (md), `.github/workflows/quickstart.yml:71` (mk) — use the exact tag-prefix forms. + **md-codec lib pin 0.34→0.35** (Tier-0) + Tier-0 smoke test + Tier-0 FOLLOWUP correction + CI-gated transcript re-capture (`24-recover-md1`) + companion FOLLOWUP closures in all repos → toolkit tag. (`install.sh:38` pins `ms-cli-v0.5.0` vs reality v0.5.1 — pre-existing, out of scope, NOT a gate trigger since the gate compares install.sh↔workflows, not against reality.)

**Gates (project-mandatory):** opus R0 on this SPEC (0C/0I) → opus R0 on the plan-doc (0C/0I) → per-phase reviewer-loop → end-of-cycle R0. No code before SPEC R0 GREEN.

---

## 9. Footguns (carry into the plan-doc)
- **F1** mk `verify --json` carries `policy_id_stubs` (4-byte hashes) — NOT keys → inert is correct; don't mis-class WatchOnly.
- **F2** md `bytecode` = Template (arguable; the hex is the template's binary form — kept Template to preserve the invariant).
- **F3** md `compile` is feature-gated (`cli-compiler`, default-off) → emit inside the gate; integration cell `cfg`-gated.
- **F4** md `encode --key @i=XPUB` still Template (xpubs bound inside the card; artifact class unchanged).
- **F5** mk/md `--json` mode still emits (format ≠ class).
- **F6** mk/md `repair` currently emit NO advisory — the gap Phase 2 closes; add after emit, success path only (md error path `Ok(2)` stays inert).
- **md `vectors`** writes to files, stdout empty → inert (don't add an advisory).
- **clippy `-D warnings`** CI gate (toolkit + siblings, `--all-targets`): in a **bin-only crate** the test exercising a variant does NOT keep it live — the `OutputClass` enum **requires `#[allow(dead_code)]`** (R0 C1). mk leaves both `Template` + `PrivateKeyMaterial` unconstructed; md leaves `PrivateKeyMaterial`. Watch for orphaned imports + `missing_docs` (M1).
- **Multi-return-site emit** (R0 I1): md's `--json` branches `return Ok(0)` early — the emit must fire on EVERY success return, not just the final one; `json` is a default feature. Enumerate per-handler in the plan.
- **5-site sibling-pin bump** (R0 I2): install.sh ×2 + manual.yml ×2 + quickstart.yml ×1; the gate scans all workflows. Git tag (not crates.io) is the gate-relevant deliverable (R0 I3).
- **Cargo.lock stale-lock**: run cargo before staging the lockfile in the toolkit Tier-0 bump.
