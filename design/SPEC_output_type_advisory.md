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
| `convert` | **P**(cond)/**W** | `convert.rs:1099` gate; `secret_taxonomy.rs:76-85` | `any(secret target)? P : W`; never inert [I1] |
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

**Resolving principles** (state in SPEC): (a) public key-derived material (xpub/fingerprint/address/descriptor) = watch-only, not inert; (b) re-presenting the user's own public input, or a pass/fail/cost verdict, = inert; (c) a command that consumes a secret as INPUT but emits only public/derived metadata = inert; (d) class is per-exit-branch for branchy commands; (e) multi-artifact = max over artifacts on stdout.

## 4. Architecture

### 4.1 Toolkit helper (subsume D9 by complement)
In `crates/mnemonic-toolkit/src/secret_advisory.rs`:
```rust
pub enum OutputClass { PrivateKeyMaterial, WatchOnly, Template }
/// Emit the one-line stderr class advisory. Inert outputs do NOT call this.
pub fn emit_output_class_advisory<W: Write + ?Sized>(class: OutputClass, stderr: &mut W);
```
The `PrivateKeyMaterial` arm IS the (re-worded) D9 line. The legacy `secret_on_stdout_warning(CardKind, _)` + `secret_on_stdout_warning_unconditional` are **replaced**: the 3 inlined literals (`derive_child.rs:308`, `bundle.rs:931`, `convert.rs:1102`), the `_unconditional` callers (silent_payment, nostr), and the slip39 variant (`slip39.rs:684`) all route through `emit_output_class_advisory` (+ addenda where applicable). Keep `secret_in_argv_warning` and `warn_if_world_readable` unchanged (different advisories).

### 4.2 ms-cli helper (duplicate, byte-parity) [folds I4]
ms-cli does NOT depend on mnemonic-toolkit as a lib (independent crates). Duplicate `OutputClass` + the emit helper into `mnemonic-secret/crates/ms-cli/src/advisory.rs` (alongside the existing `secret_in_argv_warning`). ms-cli `run(args)` is by-value and emits via `eprintln!`/`std::io::stderr()` — the helper takes `&mut impl Write` and callers pass `std::io::stderr().lock()` (or the helper wraps `eprintln!`). Replace `ms repair`'s inline literal (`cmd/repair.rs:107-108`). A **cross-repo byte-parity test** pins ms-cli's three lines identical to the toolkit's wording (precedent: `advisory.rs:1` "ported from mnemonic-toolkit"; `repair.rs:28` "Byte-matches …").

### 4.3 Wiring rules
- **Fixed-class commands** call `emit_output_class_advisory(<constant>, stderr)` at the point where the artifact is known to have been written to stdout.
- **Multi-artifact commands** compute `worst_class_on_stdout` (bundle: secret-bearing predicate; repair/inspect: max over CardKind of cards on stdout; convert: any-secret-target predicate; import-wallet: entropy-on-stdout predicate) → one line.
- **TTY gate removed** [C3] on final-word/seed-xor×2/slip39×2 — the advisory now fires unconditionally (the redirected case is the dangerous one).
- **File-output suppression** [I5]: emit iff a wallet artifact is actually written to **stdout** this invocation. Exclusive-file paths (`seedqr --json-out`, `electrum-decrypt --json-out`) → no line. Side-effect-file paths (`final-word`/`slip39`/`seed-xor --json-out`, where stdout still carries the artifact) → still emit. `--json` mode → advisory still to **stderr** (JSON stays clean on stdout) [I6].
- **Inert commands** call nothing; commands with no `stderr` param (`compare-cost`, `gui-schema`) are NOT given one.

## 5. D9 re-word + mechanical sweep [folds D9 call, M5]
Re-wording `secret material on stdout` → `private key material (can spend)` touches, in the SAME PR:
- ~15 transcript/doc mirrors: `docs/manual/transcripts/{22-first-bundle,24-recover,24-recover-mk1,41-inheritance}.out`, `docs/technical-manual/transcripts/mnemonic-bundle-bip84-abandon.out`, the foreign-formats export-wallet transcripts (gain watch-only lines), manual prose `docs/manual/src/40-cli-reference/41-mnemonic.md` (multiple), `43-ms.md:255`, quickstart mirrors.
- ~9 toolkit + ~3 ms test files asserting `.contains("warning: secret material on stdout")` — re-pin to the new wording (and the conditional ones now assert the class predicate).
Re-grep the exact file/line set at implementation time (citations decay). The manual-prose-execution gate (`verify-examples.sh` + `make audit`) is the forcing function: it stays RED until transcripts are re-captured — budget the re-capture as a required phase step, not an afterthought.

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
- **Addenda preserved:** seed-xor/slip39/final-word addendum bytes pinned.
- **Consolidation guard:** no orphaned `secret material on stdout` literal remains (grep-based test or the byte-parity test).
- **ms-cli byte-parity:** ms-cli's 3 lines == toolkit's wording.

## 8. Phases (for the plan-doc)
- **P0** — toolkit: `OutputClass` lattice + `emit_output_class_advisory` + `worst_class_on_stdout` helpers in `secret_advisory.rs`; unit tests (lattice max, exact bytes). Replace the legacy helpers' internals (keep call sites compiling).
- **P1** — toolkit fixed-class wiring (derive-child/silent-payment/electrum-decrypt/seedqr/addresses/export-wallet) + TTY-gate drop (final-word/seed-xor/slip39) + addenda. Per-command cells.
- **P2** — toolkit multi-artifact + conditional (bundle/convert/repair/inspect/import-wallet/nostr) via worst_class_on_stdout. Both-ways cells.
- **P3** — toolkit inert audit (verify-bundle/decode-address/verify-message/compare-cost/gui-schema/xpub-search×4) — assert no line; file-output suppression cells.
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
