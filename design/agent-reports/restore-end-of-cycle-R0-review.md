# mnemonic restore — End-of-Cycle R0 Review

**Verdict: GREEN (0 Critical / 0 Important).** Cleared to tag `mnemonic-toolkit-v0.43.0`.

Whole-cycle gate over `master..HEAD` (18 commits, base `6566941` v0.42.0): P1 core + P2 formats/json + P3 docs/release + all folds. Reviewer (opus, full shell) ran the complete gate end-to-end; also the re-dispatch confirming the Phase-3 C1 fold.

## Critical / Important
None.

## Minor (none blocking)
- **m1 — README `#subcommands` grouped-narrative intro count "Twenty-one" is stale (`README.md:44`), PRE-EXISTING.** Master already had status="twenty-two"/intro="twenty-one" (1-off); this cycle correctly bumped the *status line* twenty-two→twenty-three (restore is new) but didn't touch the intro, so the gap widened to 2. NOT introduced here, NOT gated by `readme_version_current` (marker-only), and ambiguous (the intro is a category summary, not a 1:1 subcommand list) — resolving it correctly needs judgment about what it counts. **Left as-is** (fixing pre-existing ambiguous drift in a ship commit risks a wrong count); candidate for a future README-coherence pass (precedent: v0.36.3 docs-refresh).
- **m2** ms1 `--language` conflict message says "slot @0.ms1=" for a slot-less `--from ms1=` input (shared `slot_ms1` vocabulary) — clear+actionable; noted-not-folded by per-phase R0.
- **m3** text `template_label` ("bip84 (native segwit P2WPKH)") vs json `human_name` ("bip84") — intentional human-vs-machine; consistent per template.
- **m4** manual `--json`/`--format` examples truncate the xpub (`xpub6CatW…`) — readability; prefix matches.

## Verification ledger (every command RUN)
- **WATCH-ONLY-OUT (security):** `restore.rs` child derivation `Secp256k1::verification_only()` (:300); `account_xpriv` computed by the shared helper but DROPPED (:357, never emitted). Cross-mode leak scan (real passphrase + ms1 seed × 6 sinks = stdout+stderr × text/json/bitcoin-core) → **ZERO** passphrase/ms1/xprv/tprv/yprv/zprv/entropy-hex/"abandon" leaks.
- **Behaviors (exit codes RUN):** phrase no-pp bip84 → `73c5da0a` + `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2` + `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` (0); TREZOR-pp → `b4e3f5ed` (0); ms1-entr/entropy/seedqr → `73c5da0a` (path-independent); ms1 Japanese `mnem` → `0ed2c5a4`, `--language english` conflict → exit 2; all-4 default → bip44/49/84/86; multisig `--template wsh-sortedmulti` → BadInput exit 1; non-seed `--from xpub=` → exit 1; expect-fingerprint match→0 / mismatch→4-no-descriptor / +allow-mismatch→0+banner; expect-xpub without `--template`→ModeViolation 2; UNVERIFIED banner; `--format` all-4→ModeViolation 2; `--format specter`→ExportWalletMissingFields exit 2 (export-wallet parity); `--format descriptor` payload→stdout/block→stderr; `--json` shape+redaction; `--output` file; stdin-mutex→1; `--count 3`→3 addrs; testnet→tpub/tb1q/84'/1'.
- **RestoreMismatch:** alpha slot, exit_code→4, kind→`"RestoreMismatch"`, message→`restore: …`, no `details()` arm, build clean.
- **Manual examples (RUN VERBATIM):** `41-mnemonic.md` EX1-6 + `35-recovery-paths.md` recipe all match documented output; **C1 fold confirmed** — both seed literals now 12 words (`wc -w`=12), print `73c5da0a`/`#hpg6d6w2`/`bc1qcr8te4…`; grep for a 13-word seed = 0.
- **`make audit`:** 4 CLIs + FIXTURES_DIR → **EXIT=0** (20 transcripts, lychee 0 errors, markdownlint/cspell/flag-coverage/glossary/index clean).
- **Version (v0.43.0):** Cargo.toml/Cargo.lock/both markers/both Status lines (v0.43.x + twenty-three)/install.sh self-pin/CHANGELOG all at 0.43.0; `readme_version_current` PASS. FOLLOWUP `restore-multisig-cosigner-scope` filed (open, tier v0.5, 3 bridge options). gui-schema 28→29; runtime gui-schema 29 incl restore; lint_argv_secret_flags restore routes PASS.
- **Regression + scope:** `--no-fail-fast` FAILED count **0** (cli_restore 39 tests); clippy `--all-targets -D warnings` exit 0; NO `mnemonic-gui` in diff (only the toolkit's `cli_gui_schema.rs` test bump — paired GUI v0.24.0 is post-tag); file set = expected (code+tests+docs+version+design), no stray.

**Bottom line:** GREEN 0C/0I. Cleared to ship v0.43.0 + the paired GUI v0.24.0 mini-cycle.
