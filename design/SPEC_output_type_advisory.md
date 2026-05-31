# SPEC — output-type stderr advisory (Phase 1: mnemonic + ms)

**Status:** brainstorm-approved (architect NEEDS-REVISION folded); pending formal SPEC R0.
**Cycle:** B. **SemVer:** PATCH — mnemonic-toolkit (tag-only) + ms-cli (crates.io re-publish).
**Source SHA at write:** mnemonic-toolkit `18cfdce`; mnemonic-secret ms-cli v0.5.0 lineage. Citations grep-verified at write time.
**Brainstorm-stage architect review:** `design/agent-reports/output-type-advisory-brainstorm-R0-review.md` (3C/6I/5M, folded).

---

## 1. Problem / Goal

When a CLI writes a wallet artifact to **stdout**, the user has no in-band signal of its security nature — whether it can *spend* (private key material), only *watch* (xpubs/addresses), or is a keyless *template*. Today only secret outputs get a stderr advisory (the "D9" `secret material on stdout` line), and even that is inconsistent (inlined literals, TTY-gated on some commands, silent on mk1/md1). A user piping `mnemonic convert … | ms decode` cannot tell, from the absence of a warning, whether the output is safe.

**Goal:** every output-producing command in `mnemonic` and `ms` emits **exactly one** stderr line classifying the worst-case security nature of what it wrote to stdout, so that **"no advisory line ⟺ inert output"** holds across both CLIs. Serves the user's standing no-new-key-hazards / no-signing boundary by making spend-capability visible everywhere. (Phase 2 — `mk` + `md` — is filed as a FOLLOWUP; §10.)

## 2. The classification

### 2.1 Lattice [folds C1]
```
PrivateKeyMaterial ≻ WatchOnly ≻ Template ≻ inert
```
A command's class is the **worst (max) class over all artifacts actually written to stdout this invocation** (`worst_class_on_stdout`). Most commands are fixed-class; the **multi-artifact commands** — `bundle`, `repair`, `inspect`, `import-wallet`, `convert` — compute the max.

### 2.2 The emitted lines (exact bytes — pinned [folds M5])
Em-dash is U+2014. `'> file.txt'` matches the live D9 literal (`secret_advisory.rs:62`).
- **PrivateKeyMaterial** (`warning:` — it is a hazard):
  `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')`
- **WatchOnly** (`note:` — informational):
  `note: stdout is watch-only — public keys only, cannot spend`
- **Template** (`note:`):
  `note: stdout is a keyless descriptor template (no keys)`
- **inert:** no line.

### 2.3 Addenda [folds C3, M2]
Commands with command-specific safety information KEEP it as an addendum line printed AFTER the unified line (never replacing it):
- `seed-xor split`/`combine`: the existing "ALL N shares required to reconstruct … no authentication tag …" clause.
- `slip39 split`/`combine`: the existing group/member-threshold clause + slip39-combine's "verify the recovered wallet's derived address before trusting".
- `final-word`: the existing "candidate list is secret material — pairing the partial phrase with any candidate yields a valid seed phrase" clause (more precise than the generic line).
Pin each addendum's exact bytes in the SPEC's test cells.

## 3. Per-command class table (grep-verified at `18cfdce`)

### 3.1 mnemonic (24 subcommands)
| Subcommand | Class | Evidence | Note |
|---|---|---|---|
| `bundle` | **P**(cond)/**W** | `bundle.rs:909-917` md1(T)+mk1(W) always; `:895` ms1(P) gated `:928` | `any_secret_bearing()? P : W` |
| `verify-bundle` | inert | match/mismatch verdict | |
| `convert` | **P/W/inert**(cond) | `convert.rs:1099` gate; targets `:31-52`; `secret_taxonomy.rs:76-85` | max over targets {secret→P; xpub/address/descriptor/mk1→W; **path/fingerprint→inert**} [folds I3] |
| `addresses` | **W** | `addresses.rs:248`; doc `:1` "watch-only" | |
| `export-wallet` | **W** | `export_wallet.rs:246` "watch-only by SPEC §3" | incl. `--from-import-json` |
| `import-wallet` | **W**(cond **P**) | `import_wallet.rs:2089-2138`; `:2111 entropy=` | P iff `entropy=` (seed overlay) on stdout |
| `derive-child` | **P** | `derive_child.rs:304-309` BIP-85 | |
| `final-word` | **P** | `final_word.rs:84-86`; TTY-gated `:101` | drop TTY gate [C3]; addendum [M2] |
| `seed-xor split` | **P** | `seed_xor.rs:215`; TTY-gated `:241` | drop TTY gate; addendum |
| `seed-xor combine` | **P** | `seed_xor.rs:349`; TTY-gated `:365` | drop TTY gate; addendum |
| `slip39 split` | **P** | `slip39.rs:536-540`; TTY-gated `:544` | drop TTY gate; addendum |
| `slip39 combine` | **P** | `slip39.rs:677`; TTY-gated `:681` | drop TTY gate; addendum |
| `gui-schema` | inert | `gui_schema.rs:1295` (no stderr param) | |
| `repair` | **P/W/T**(cond) | `repair.rs:215` Ms1→P; Mk1→W, Md1→T silent today | CardKind→class, max on stdout [C1] |
| `inspect` | **P/W/T**(cond) | `inspect.rs:144-156` `any_ms1`→P; mk1/md1 silent | same CardKind map [C1] |
| `compare-cost` | inert | `compare_cost.rs:67` (no stderr param) | |
| `nostr` | **W**(npub)/**P**(nsec) | `nostr.rs:177-186` npub→W (silent today); `:248,:251` nsec→P | per-exit-branch [I3] |
| `silent-payment` | **P** | `silent_payment.rs:286` priv keys | |
| `decode-address` | inert | `decode_address.rs:1-3` "no key material"; echoes input | [I2] |
| `verify-message` | inert | `verify_message.rs:1-3` "PUBLIC … no secrets" | |
| `electrum-decrypt` | **P** | `electrum_decrypt.rs:147-149` (stdout branch only) | file-output suppresses [I5] |
| `seedqr encode` | **P** | `seedqr.rs:323` digits — **net-new** secret coverage | [M4] |
| `seedqr decode` | **P** | `seedqr.rs:295` phrase — **net-new** | [M4] |
| `xpub-search path-of-xpub` | inert | `path_of_xpub.rs:311-332` path/searched report | |
| `xpub-search account-of-descriptor` | inert | matched-cosigners report | |
| `xpub-search address-of-xpub` | inert | `address_of_xpub.rs:266-272` match report (echoed addr) | |
| `xpub-search passphrase-of-xpub` | inert | `passphrase_of_xpub.rs:340-361`; passphrase is INPUT `:260-289` | [C2] |

### 3.2 ms-cli (8 subcommands)
| Subcommand | Class | Evidence |
|---|---|---|
| `ms encode` | **P** | `encode.rs:146` ms1 (entropy) — **net-new** |
| `ms decode` | **P** | `decode.rs:101-102` entropy+phrase — **net-new** |
| `ms derive` | **W** | `derive.rs:239-242` fp+account xpub; `:1-5` "no private keys" [architect correction] |
| `ms inspect` | inert | `inspect.rs:110-125` OK/FAIL |
| `ms verify` | inert | `verify.rs:116-158` OK/FAIL |
| `ms vectors` | inert | fixed public test vectors |
| `ms repair` | **P** | `repair.rs:107-108` inline literal (already fires) |
| `ms gui-schema` | inert | schema JSON |

**Resolving principles** (state in SPEC): (a) public key **material** you'd use to watch (xpub/address/descriptor/mk1) = watch-only; a bare derivation **path** or 4-byte **fingerprint** carries no usable key material → **inert** [folds I3]; (b) re-presenting the user's own public input, or a pass/fail/cost verdict, = inert; (c) a command that consumes a secret as INPUT but emits only public/derived metadata = inert; (d) class is per-exit-branch for branchy commands; (e) multi-artifact = max over artifacts on stdout.

**Auto-repair short-circuit branch [folds C1].** `verify-bundle`, `xpub-search` (all 4 modes), `inspect`, and `convert` are inert on their NORMAL branch, but each can short-circuit into `repair::emit_repair_report` (`repair.rs:1333`) when given a correctable card with auto-repair — which writes the **repaired card to stdout** (exit 5) and must emit that card's class (Ms1→P, Mk1→W, Md1→T). So their §3 "inert" label applies to the normal branch ONLY; the auto-repair exit-5 branch emits a class line. Tests must cover both branches (§7, P3).

## 4. Architecture

### 4.1 Toolkit helper (subsume D9 by complement)
In `crates/mnemonic-toolkit/src/secret_advisory.rs`:
```rust
pub enum OutputClass { PrivateKeyMaterial, WatchOnly, Template }
/// Emit the one-line stderr class advisory. Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: Write + ?Sized>(class: OutputClass, stderr: &mut W);
```
The `PrivateKeyMaterial` arm IS the (re-worded) D9 line. The legacy `secret_on_stdout_warning(CardKind, _)` + `secret_on_stdout_warning_unconditional` are **replaced** — route **every** current caller through `emit_output_class_advisory`/`worst_class_on_stdout` (re-grep the full caller set at plan time; at `18cfdce` it is):
- the 3 inlined `PrivateKeyMaterial` literals: `derive_child.rs:308`, `bundle.rs:931`, `convert.rs:1102`;
- the `_unconditional` callers: `silent_payment.rs:286`, `nostr.rs:251`, **`electrum_decrypt.rs:149`** [folds I1];
- the slip39 **combine** variant `slip39.rs:684` (a FUSED line — replace the "reconstructed secret material" prefix with the unified P-line; KEEP "verify the recovered wallet's … address" as the §2.3 addendum) [M2];
- **the auto-repair short-circuit** [folds C1 — the hidden emit site]: `repair.rs:1333` (`emit_repair_report` → `secret_on_stdout_warning(outcome.kind, _)`, writes the repaired card to stdout) AND the repair-command emit `cmd/repair.rs:216`. `emit_repair_report` is reached from `verify_bundle.rs` (6 sites), `xpub_search/seed_intake.rs:182` (all 4 xpub-search modes), `inspect.rs:135`, `convert.rs:994` — so those commands are inert on their NORMAL branch but emit the **repaired card's class** (CardKind→{Ms1:P, Mk1:W, Md1:T}) on the exit-5 auto-repair branch (see §3 note).

Keep `secret_in_argv_warning` and `warn_if_world_readable` unchanged (different advisories).

### 4.2 ms-cli helper (duplicate, byte-parity) [folds I4]
ms-cli does NOT depend on mnemonic-toolkit as a lib (independent crates). Duplicate `OutputClass` + the emit helper into `mnemonic-secret/crates/ms-cli/src/advisory.rs` (alongside the existing `secret_in_argv_warning`). ms-cli `run(args)` is by-value and emits via `eprintln!`/`std::io::stderr()` — the helper takes `&mut impl Write` and callers pass `std::io::stderr().lock()` (or the helper wraps `eprintln!`). Replace `ms repair`'s inline literal (`cmd/repair.rs:107-108`). A **cross-repo byte-parity test** pins ms-cli's three lines identical to the toolkit's wording (precedent: `advisory.rs:1` "ported from mnemonic-toolkit"; `repair.rs:28` "Byte-matches …").

### 4.3 Wiring rules
- **Fixed-class commands** call `emit_output_class_advisory(<constant>, stderr)` at the point where the artifact is known to have been written to stdout.
- **Multi-artifact commands** compute `worst_class_on_stdout` (bundle: secret-bearing predicate; repair/inspect: max over CardKind of cards on stdout; convert: any-secret-target predicate; import-wallet: entropy-on-stdout predicate) → one line.
- **TTY gate removed** [C3] on final-word/seed-xor×2/slip39×2 — the advisory now fires unconditionally (the redirected case is the dangerous one).
- **File-output suppression** [I5]: emit iff a wallet artifact is actually written to **stdout** this invocation. Exclusive-file paths (`seedqr --json-out`, `electrum-decrypt --json-out`) → no line. Side-effect-file paths (`final-word`/`slip39`/`seed-xor --json-out`, where stdout still carries the artifact) → still emit. `--json` mode → advisory still to **stderr** (JSON stays clean on stdout) [I6].
- **Inert commands** call nothing; commands with no `stderr` param (`compare-cost`, `gui-schema`) are NOT given one.
- **Emit at run-level [M4].** Several stdout helpers take only `stdout` (`seedqr::emit_*`, `import_wallet::emit_summary`, ms-cli `cmd/*::emit_*` use `println!`). The class advisory is emitted by the **run-level caller** (which has `stderr` / `std::io::stderr().lock()` in scope) AFTER the artifact is written — not inside the stdout-only helper. The helper signature `emit_output_class_advisory<W: Write + ?Sized>(class, &mut W)` accepts both a threaded `&mut W` and `std::io::stderr().lock()`.

## 5. D9 re-word + mechanical sweep [folds D9 call, M5]
Re-wording `secret material on stdout` → `private key material (can spend)` touches, in the SAME PR (re-grep the exact set at plan time — citations decay):
- **FOUR doc trees, each with its own execution/lint gate** [folds I2] — all must be re-captured or their gate goes RED: `docs/manual` (`verify-examples.sh`; transcripts `{22-first-bundle,24-recover,24-recover-mk1,41-inheritance}.out` + foreign-formats export-wallet transcripts gaining watch-only lines; prose `src/40-cli-reference/41-mnemonic.md` (multiple), `43-ms.md:255`); `docs/technical-manual` (`mnemonic-bundle-bip84-abandon.out`); `docs/manual-gui` (its own `tests/verify-examples.sh`; `src/40-mnemonic/4b-slip39-combine.md`); `docs/quickstart` (its own `Makefile`; `src/20-singlesig/{23-bundle,26-recover}.md`, `src/30-multisig/32-bundle.md`).
- **12 toolkit + 3 ms-cli test files** [folds I2] asserting `secret material on stdout`: toolkit `cli_bundle_full`, `cli_bundle_slip0132_info`, `cli_indel`, `cli_silent_payment`, `cli_nostr`, `cli_electrum_decrypt`, `cli_inspect`, `cli_slip39_advisories`, `cli_bundle_multisig`, `cli_bundle_watch_only`, `cli_derive_child`, `cli_secret_in_argv_warning`; ms-cli `tests/cli_repair.rs`, `src/main.rs`, `README.md`. Re-pin to the new wording (conditional ones now assert the class predicate).
- The auto-repair short-circuit literal (`repair.rs:1333`) + `cmd/repair.rs:216` re-word too (per §4.1).

The manual-prose-execution gate (`make audit` over all 4 doc trees' `verify-examples.sh` + `docs/quickstart/Makefile`) is the forcing function: each stays RED until its transcripts are re-captured — budget the re-capture (rebuild all 4 binaries first) as a required Phase-5 step, not an afterthought.

## 6. SemVer / lockstep [folds M3]
- **PATCH** both (stderr-only, no new flag) — precedent `silent-default-with-stderr-notice` / v0.37.11.
- **GUI `schema_mirror`:** flag-NAME parity only; stderr text ungated → **NO GUI lockstep**. No `cli-subcommands.list` / `lint.sh` change.
- **Distribution:** mnemonic-toolkit tag-only (v0.38.2); ms-cli crates.io re-publish (v0.5.1). Coordinated: ship ms-cli first (or together); update the toolkit's ms-cli sibling pin only if the toolkit consumes the new ms-cli behavior in transcripts (it does not — independent advisories — so no pin bump required, but the manual mirrors both).
- **Manual mirror:** both CLIs' `40-cli-reference` chapters note the advisory; execution-gated transcripts re-captured.

## 7. Testing
- **Per-command class cell:** each of the 24+8 subcommands asserts it emits exactly its class line (or nothing if inert). Conditional commands tested BOTH ways: `convert --to xprv`(P) vs `--to xpub`(W); `bundle` seed(P) vs `--descriptor`-concrete watch-only(W); `repair` ms1(P) vs mk1(W); `nostr` nsec(P) vs npub(W); `import-wallet` full(P) vs watch-only(W).
- **TTY-gate removal:** the 5 commands emit the line when stdout is NOT a TTY (piped) — regression cell per command.
- **File-output suppression:** `seedqr --json-out` / `electrum-decrypt --json-out` emit NO stdout-class line; `slip39 --json-out` still emits (artifact on stdout).
- **`--json` stderr-parity:** for each newly-covered watch-only command, JSON clean on stdout + class line on stderr.
- **Addenda preserved [M3]:** pin the exact addendum bytes for `final-word` (`final_word.rs:104`), `seed-xor split` (`seed_xor.rs:244`) + `combine`, `slip39 split` (`slip39.rs:548`) + `combine` (`slip39.rs:684`'s "verify the recovered wallet's … address" clause). NOTE the re-word turns each fused single line into TWO lines (unified P-line + addendum) — every TTY-gate-removal transcript captures a one-line→two-line change; size the re-capture accordingly.
- **Auto-repair-branch cells [C1]:** for `verify-bundle` / `xpub-search` / `inspect` / `convert`, the normal branch emits NO line (inert), but a correctable-card-with-auto-repair input emits the repaired card's class (ms1→P) at exit 5. Test both branches.
- **convert-inert cell [I3]:** `convert --to path` (and `--to fingerprint`) alone emits NO line (inert); `--to xpub --to path` emits the watch-only line (max).
- **Consolidation guard:** no orphaned `secret material on stdout` literal remains anywhere (grep-based test) — covers `repair.rs:1333` + `cmd/repair.rs:216` + electrum-decrypt.
- **ms-cli byte-parity:** ms-cli's 3 lines == toolkit's wording.

## 8. Phases (for the plan-doc)
- **P0** — toolkit: `OutputClass` lattice + `emit_output_class_advisory` + `worst_class_on_stdout` helpers in `secret_advisory.rs`; unit tests (lattice max, exact bytes). Replace the legacy helpers' internals (keep call sites compiling).
- **P1** — toolkit fixed-class wiring (derive-child/silent-payment/electrum-decrypt/seedqr/addresses/export-wallet) + TTY-gate drop (final-word/seed-xor/slip39) + addenda. Per-command cells.
- **P2** — toolkit multi-artifact + conditional (bundle/convert/repair/inspect/import-wallet/nostr) via worst_class_on_stdout. Both-ways cells.
- **P3** — toolkit inert audit (verify-bundle/decode-address/verify-message/compare-cost/gui-schema/xpub-search×4) — assert no line on the NORMAL branch; re-route the auto-repair short-circuit (`repair.rs:1333`) through the class helper + assert the exit-5 branch DOES emit (ms1→P); file-output suppression + convert-inert (path/fingerprint) cells.
- **P4** — ms-cli: duplicate enum+helper in `advisory.rs`; wire encode/decode(P)/derive(W)/repair(P); byte-parity test.
- **P5** — D9 re-word sweep: transcripts re-captured + doc mirrors + test `.contains()` re-pins; `make audit` GREEN (both repos).
- **P6** — versions (toolkit v0.38.2, ms-cli v0.5.1); FOLLOWUP `output-type-stderr-advisory-sibling-sweep-mk-md` filed (mirrored); end-of-cycle R0; ship (toolkit tag + ms crates.io).

## 9. Risks
- **R1 — transcript-recapture sweep is wide.** Mitigated: `make audit` is the gate; re-capture (not author) per the coldcard/recipe-2 precedent; rebuild all 4 binaries first.
- **R2 — cross-repo string drift.** The ms-cli byte-parity test pins it; the D9 wording lives in two repos.
- **R3 — a command's class predicate is subtly wrong** (e.g. convert with a mixed target set). Mitigated by the both-ways cells + the worst_class_on_stdout lattice.
- **R4 — interim cross-CLI gap** (mk/md silent until Phase 2). Bounded by the mirrored FOLLOWUP (must close before the next install.sh sibling-pin bump); only benign (non-secret) outputs uncovered.

## 10. Filed follow-on
`output-type-stderr-advisory-sibling-sweep-mk-md` — Phase 2: mk-cli (watch-only on decode/derive/address/inspect; greenfield — mk-cli has no advisory module today) + md-cli (template on decode/encode, watch-only on `md address`). Completes the constellation-wide "no line ⟺ inert" invariant; `template` gets its real exercise in md. Mirrored across all repos' FOLLOWUPS.md with cross-citing Companion lines; bounded by the next sibling-pin bump.
