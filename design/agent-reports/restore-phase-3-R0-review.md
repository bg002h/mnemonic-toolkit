# mnemonic restore — Phase 3 R0 Review (docs + release-prep v0.43.0)

**Verdict (round 0): RED (1C / 0I)** → fold C1 (controller, verified) → **GREEN**.

Commits `a8b309c` (manual section + recovery recipe), `bfee27f` (v0.43.0 bump). Release plumbing (versions, FOLLOWUP, scope, gates) all GREEN round-0; the one blocker was a copy-paste-broken seed literal — exactly the architect-must-run-prose failure class.

## Critical (1) — FOLDED + verified

**C1 — Worked-example seed literal was 13 words (invalid BIP-39); copy-pasting failed `BIP-39 word count 13 invalid` (exit 1), in BOTH new docs.** The prose said "`abandon` × 11 + `about`" correctly, but the `seed="…"` literal had 8 `abandon` on line 1 + 4 on line 2 + `about` = 12 abandon + about. `41-mnemonic.md:799` and `35-recovery-paths.md:42`. All documented *outputs* were correct (generated with the right seed); only the input literal was wrong (not transcript-gated → `make audit` was green despite it).
**Fold (controller):** removed one `abandon` from line 1 of both files (8→7 → 11 total + about = 12 words). **Verified at runtime:** both literals now 12 words; the verbatim example (bash line-continuation join) → `master fingerprint: 73c5da0a (passphrase: none)` + `wpkh([73c5da0a/84'/0'/0']xpub6CatW…/<0;1>/*)#hpg6d6w2` + `first recv: bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`, exit 0 — matches the documented output exactly. `make audit` re-run → EXIT=0.

## Important
None.

## Minor
- **m1** `--json`/`--format` examples truncate the xpub (`xpub6CatW…`) for line-width — reasonable readability; prefix matches real output. Non-blocking.

## Verification ledger (reviewer RAN round-0; controller RAN the fold-verify)
- **Flag coverage:** all 16 `--flag`s from `restore --help` present in `41-mnemonic.md` (`missing_count=0`). ✓
- **Worked examples (RUN, corrected seed):** EX1 bip84 → `73c5da0a`/`#hpg6d6w2`/`bc1qcr8te4kr…` exit 0; all-4 → bip44/49/84/86; hard-gate `--expect-fingerprint deadbeef` → `✗ MISMATCH` exit 4 no descriptor; TREZOR passphrase-stdin+@env → `b4e3f5ed` exit 0; `--format descriptor` payload→stdout block→stderr; `--json` byte-matches; UNVERIFIED banner; `--format` no `--template` → exit 2; non-seed `xpub=` → exit 1; argv-leak advisory. All match the manual. ✓
- **`make audit`:** all 4 CLIs + FIXTURES_DIR → **EXIT=0** (20 transcripts pass, lychee 0 errors, flag-coverage/cspell/markdownlint clean), re-confirmed post-fold. ✓
- **anchor-dangler-baseline addition LEGITIMATE:** `mnemonic-restore` is the same cross-chapter class as the existing `mnemonic-bundle`/`-export-wallet`/`-import-wallet` (`41-mnemonic.md#…` unresolvable in single-file HTML); `id="mnemonic-restore"` present; `id="multisig-wallet-md1-is-lost-or-unreadable"` present + NOT dangling — masks nothing. ✓
- **Version bump complete (v0.43.0):** `Cargo.toml:3`, `Cargo.lock`, both README markers, both README `Status:` lines (`v0.43.x` + count `twenty-two`→`twenty-three`; verified 23 actual subcommands incl restore), `scripts/install.sh` self-pin, `CHANGELOG.md` v0.43.0 (dated 2026-06-04, MINOR, restore + "Multisig deferred" + FOLLOWUP cite). `readme_version_current` PASS. ✓
- **FOLLOWUP `restore-multisig-cosigner-scope`:** coherent; cites `template_from_descriptor` mod.rs:262, `extract_multisig_threshold` bundle.rs:1015 (private), `bundle_run_unified_descriptor` bundle.rs:1138, `build_descriptor_string` pipeline.rs:18, `to_miniscript_descriptor` to_miniscript.rs:53 + `MissingPubkey` :72; coheres with SPEC §11 (a/b/c); target v0.44.0. ✓
- **Gate + scope:** `--no-fail-fast` FAILED count **0** (30 restore tests); clippy clean; `git diff 3bca407..HEAD` NO `mnemonic-gui`, NO `src/*.rs` behavior (only Cargo.toml version line + docs + Cargo.lock + README/CHANGELOG/install.sh/FOLLOWUPS + baseline/list). ✓
- **Watch-only docs:** examples use the public test seed via off-argv channels; `xprv`/`tprv`/`wif` mentions are negative assertions; recovery recipe uses placeholders. ✓

**Bottom line:** C1 fixed + runtime-verified; everything else converged round-0. Phase 3 GREEN. The end-of-cycle R0 (over `master..HEAD`) re-runs the manual examples + `make audit` + full gate — the re-dispatch that confirms this fold.
