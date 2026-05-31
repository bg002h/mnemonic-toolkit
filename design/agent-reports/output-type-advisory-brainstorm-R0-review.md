# Architect review (brainstorm-stage, pre-SPEC) — output-type-stderr-advisory Phase 1 (mnemonic + ms)

**Date:** 2026-05-31 · **Reviewer:** opus architect · **SHA:** mnemonic-toolkit `18cfdce`, mnemonic-secret ms-cli v0.5.0 lineage.
**Verdict: NEEDS-REVISION** (3C/6I/5M) — architecture sound; refinements fold into the SPEC. Supersedes the A1 §B sketch; corrects 3 of its assertions.

> Persisted verbatim per CLAUDE.md before the fold.

## Critical
- **C1 — multi-artifact commands emit MULTIPLE classes on one stdout; class is worst-case-on-stdout, not a constant.** `bundle` (`cmd/bundle.rs:909-917` md1(T)+mk1(W) always; `:895` ms1(P) gated on `:928 any_secret_bearing()`), `repair` (`cmd/repair.rs:215` ms1→P, mk1→W, md1→T — mk1/md1 currently silent no-op), `inspect` (`cmd/inspect.rs:155`), `import-wallet` (`:2111 entropy=` row). **Fix:** lattice **P ≻ W ≻ T ≻ inert**; each multi-artifact command computes `worst_class_on_stdout` → one line. bundle: `any_secret_bearing()? P : W` (md1+mk1 always → never below W, never pure-T). repair/inspect: CardKind→{Ms1:P, Mk1:W, Md1:T}, max over cards on stdout. "emit(class)" constant only for genuinely-fixed commands.
- **C2 — `passphrase-of-xpub` is INERT, not secret.** `cmd/xpub_search/passphrase_of_xpub.rs:340-361`/`:323-338` emit only match/path/searched report; the passphrase is INPUT (`:260-289`, Zeroizing), never on stdout. **Fix:** → inert. Principle: *consumes a secret as input but emits only public/derived metadata = inert.*
- **C3 — always-emit must DROP the TTY gate on 5 commands.** `final-word` (`cmd/final_word.rs:101 is_terminal()`), `seed-xor split` (`cmd/seed_xor.rs:241`), `seed-xor combine` (`:365`), `slip39 split` (`cmd/slip39.rs:544`), `slip39 combine` (`:681`) warn only on TTY. Always-emit requires firing in the redirected (`> file`) case — the dangerous one the harness exercises. **Fix:** SPEC states the gate is removed; the bespoke clauses (seed-xor "ALL N shares required / no auth tag", slip39 group/member text, slip39-combine "verify the recovered wallet's address") stay as ADDENDA after the unified line — do NOT drop them.

## Important
- **I1 — `convert` is never inert.** `cmd/convert.rs:1099` secret-gate; `secret_taxonomy.rs:76-85` secret set. Every target is secret or public-key-material → `any(secret)? P : W`. `--to fingerprint`/xpub/path/address/descriptor → W. Principle: *public key-derived material (xpub/fingerprint/address/descriptor) = watch-only, not inert.*
- **I2 — `decode-address`/`verify-message`/`verify-bundle`/`compare-cost`/`gui-schema` = inert.** `decode_address.rs:1-3` "no key material"; re-presents user input. `compare_cost.rs:67` + `gui_schema.rs:1295` have NO stderr param (structural proof of inert — don't add it). Principle: *re-presenting user's own public input, or a pass/fail/cost verdict, = inert.*
- **I3 — `nostr` is conditional per exit-branch.** `cmd/nostr.rs:177-186` npub→W (silent today); `:248,:251` nsec→P. `silent-payment` (`:286`) = P unconditional. Principle: *class is per-exit-path.*
- **I4 — ms-cli has NO `secret_on_stdout` helper** (only `advisory.rs:1-15` argv-leak). The `ms repair` line is an inline literal (`cmd/repair.rs:107-108`). ms-cli emits via `println!`/`eprintln!`, by-value `run(args)`. **Fix:** duplicate `OutputClass` + helpers into `ms-cli/src/advisory.rs` (helper takes `&mut impl Write` or uses `eprintln!`); route encode/decode→P, derive→W, repair→P (replace inline literal); cross-repo byte-parity test pinning the 3 lines against the toolkit wording.
- **I5 — file output must SUPPRESS the stdout-class line.** `electrum-decrypt` precedent: advisory only in the stdout branch (`cmd/electrum_decrypt.rs:147-149`); `--json-out` branch emits `warn_if_world_readable` instead. `seedqr --json-out` is EXCLUSIVE (`cmd/seedqr.rs:280-297` if-else → nothing on stdout → no line); `final-word`/`slip39`/`seed-xor --json-out` are SIDE-EFFECT (stdout still gets the artifact → still emit). **Fix:** trigger = *artifact actually written to stdout this invocation*; enumerate exclusive-file (no line) vs side-effect-file (line).
- **I6 — `--json` advisory→stderr** (pinned by `tests/cli_bundle_full.rs:70-89`). Require a `--json`-mode stderr-parity test for each newly-covered watch-only command (addresses/convert→xpub/nostr-npub/export-wallet) so a W-line can't regress into JSON stdout.

## Minor
- **M1** — export-wallet watch-only by construction (`cmd/export_wallet.rs:246`); threads stderr already; `--from-import-json` same.
- **M2** — final-word's candidate-list precise text (`:104`) > generic; keep as addendum (a candidate isn't itself spendable until paired).
- **M3** — `schema_mirror` flag-NAME parity only → NO GUI lockstep; no `cli-subcommands.list`/`lint.sh` change. State explicitly.
- **M4** — `seedqr encode` (`:323` digits) + `decode` (`:295` phrase) are net-new SECRET coverage (silent today) — call out so not missed.
- **M5** — pin exact bytes; live literal uses em-dash U+2014 and **`'> file.txt'`** (resolved — keep `.txt`).

## Per-command class table (grep-verified) — P≻W≻T≻inert
**mnemonic (24):** bundle P(cond)/W · verify-bundle inert · convert P(cond)/W · addresses W · export-wallet W · import-wallet W(cond P) · derive-child P · final-word P · seed-xor split P · seed-xor combine P · slip39 split P · slip39 combine P · gui-schema inert · repair P/W/T(cond) · inspect P/W/T(cond) · compare-cost inert · nostr W(npub)/P(nsec) · silent-payment P · decode-address inert · verify-message inert · electrum-decrypt P · seedqr encode P · seedqr decode P · xpub-search path-of-xpub inert · account-of-descriptor inert · address-of-xpub inert · passphrase-of-xpub inert.
**ms-cli (8):** encode P · decode P · derive W · inspect inert · verify inert · vectors inert · repair P · gui-schema inert.

## D9 text-change tension — CALL: RE-WORD (adopt the unified line)
Always-emit already re-captures every covered transcript, so re-wording the secret line costs ZERO additional files; split vocabulary (D9's old text for P + new notes for W/T) is a permanent 3-grammar inconsistency. Re-word to `warning: stdout carries private key material (can spend) — redirect or encrypt (e.g. '> file.txt' or '| age -e ...')`. Cost: ~15 transcript/doc mirrors + ~9 test `.contains()` assertions updated in-PR (manual-cli-surface-mirror lockstep). Pin exact bytes.

## SPEC folds
(1) OutputClass lattice + worst_class_on_stdout; name multi-artifact (bundle/repair/inspect/import-wallet/convert) vs fixed commands. (2) inert corrections (passphrase-of-xpub + 4 xpub-search + decode-address/verify-message/verify-bundle/compare-cost/gui-schema). (3) drop TTY gate on the 5 + addendum clauses. (4) file-output suppression ("artifact-on-stdout"). (5) ms-cli enum+helper duplication + byte-parity test. (6) re-word secret line, pin bytes, update tests+mirrors. (7) per-class --json stderr-parity cells; net-new seedqr secret coverage. (8) no GUI lockstep; PATCH both; toolkit tag + ms crates.io.
