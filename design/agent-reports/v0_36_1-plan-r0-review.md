# v0.36.1 — Plan-doc R0 architect review (opus) — MANDATORY pre-implementation gate

**Date:** 2026-05-23
**Cycle:** v0.36.1 `silent-payment` `--passphrase`/`--passphrase-stdin` + `--change-address` (m=0)
**Branch:** `v0.36.1-sp-passphrase-change`
**Reviewer:** opus (feature-dev:code-reviewer), R0 (agentId a56618079accdb7fc)
**Target:** `design/IMPLEMENTATION_PLAN_v0_36_1_sp_passphrase_change.md`

---

## Critical
None.

## Important
None blocking.

## Minor (recommended folds; none gates implementation)
- **M1 — hoist the dual-stdin guard.** `--secret-stdin` reads ALL of stdin at `silent_payment.rs:151-154` (`read_to_string`) DURING secret resolution. The plan placed the `passphrase_stdin && secret_stdin` guard "after the secret is resolved" → safe-by-accident (guard fires before the passphrase read, but after the secret read drained stdin). FOLD: hoist the guard to the TOP of `run` (alongside the `--label 0` refusal @:138), BEFORE any stdin read — matches derive_child.rs:129-134 + convert.rs:799-813.
- **M2 — passphrase-stdin must use `read_stdin_passphrase`, NOT `.trim()`.** BIP-39 passphrase is byte-exact PBKDF2 salt; leading/trailing whitespace is SIGNIFICANT. `--secret-stdin` uses `.trim()` (:154) — copying that two lines up would silently corrupt whitespace-bearing passphrases. FOLD: read `--passphrase-stdin` via `convert::read_stdin_passphrase` (`pub(crate)` @convert.rs:719; doc :714-718 "preserves … leading/trailing spaces … and tabs"), as convert.rs:814-818 + derive_child.rs:149-163 do. (Real latent correctness gap in the spec text.)
- **M3 — JSON never-publish marker (open-question b).** A bare `change_address: Option<String>` is the receiving-address footgun in machine-readable form (a GUI/automation consumer could render it as a receive target — many wallets DO publish change addresses for watch-only). FOLD: emit an explicit marker — nested `change_address: { address, warning }` OR a sibling `change_address_warning` string present iff `change_address`. Add a test asserting the marker. (Human path already guarded; this hardens the JSON contract the plan itself left open.)
- **M4 — test oracle is sufficient as-is (no change).** "differs + sp1q prefix" is complete: encode/label crypto already byte-exact vs official vectors; the only NEW code is passphrase→salt threading. Keep the xprv+passphrase == xprv-no-passphrase equality assertion as the warn-and-ignore regression pin.
- **M5 — tighten edit-set prose.** Exactly ONE caller of `resolve_master_xpriv` in `run` (`:163`); the "both call sites" refers to the two `derive_master_seed` calls INSIDE the fn (:86,:112). Edit-set: { fn sig @:81; two derive_master_seed args @:86,:112; one caller @:163; xprv-branch warn @:92-94; the `to_master` closure @:83 must capture `passphrase` }.

## Verification summary (confirmed correct)
- **A:** `resolve_master_xpriv` @silent_payment.rs:81 `(secret:&str, network:CliNetwork)->Result<Xpriv,_>`; `derive_master_seed(&mnemonic,"")` @:86 (to_master closure) + :112 (phrase); `derive_slot.rs:32` `(mnemonic:&Mnemonic, passphrase:&str)->Zeroizing<[u8;64]>` — `""`→`&passphrase` swap, no helper sig change. xprv branch @:92-94 passphrase-independent. ONLY caller @:163 (grep-confirmed). to_master closure must capture passphrase.
- **B:** secret-stdin reads @:151-154; runtime guard is the project pattern for stdin-contention (clap conflicts_with can't express "both want stdin"); `conflicts_with="passphrase"` on --passphrase-stdin matches derive_child.rs:97. Keep runtime guard (hoist per M1); do NOT add clap conflicts_with vs --secret-stdin.
- **C:** `flag_is_secret` already matches --passphrase(:52)+--passphrase-stdin(:53) — no secrets.rs change; gui-schema emits secret:true once wired. --change-address NOT secret (public). `secret_in_argv_warning` @secret_advisory.rs:34 + `mlock::pin_pages_for` @mlock.rs:90 — existing --secret resolution uses both (:147,:160); pattern matches.
- **D:** `labeled_spend_key<C:Verification>(secp:&Secp256k1<C>, b_scan:&SecretKey, b_spend_pub:PublicKey, m:u32)->Result<PublicKey,_>` @:45 (b_spend_pub by value/Copy; returns Result → `?` required, plan has it). `bip0352_label_hash` @:33 no m≥1 guard (ser_32(0)). `encode_sp_address(hrp:Hrp, &PublicKey, &PublicKey)->String` @:63. PRIMARY (BIP-352): m=0 IS the reserved change label; receiver sends own change there; emitting clearly-tagged + additive is safe (own derived address, not secret); footgun design sound (--label 0 stays refused as the publish-path guard).
- **F:** 3 new flag NAMES ⇒ GUI schema_mirror fires (add to SILENT_PAYMENT_FLAGS). gui-schema subcommand COUNT unchanged (cli_gui_schema.rs counts subcommands not flags; silent-payment already @ entry; plan correctly does NOT touch the 25-count). cli-subcommands.list already has the line. SemVer PATCH correct. GUI bump = PATCH per v0.34.6→GUI v0.19.2 precedent (resolve Phase 3 Step 2 ambiguity to PATCH).
- **H:** `secret_src` ArgGroup (:17-22) = [secret, secret_file, secret_stdin]; new fields do NOT auto-join it; no collision. Ordering passphrase-first correct. --change-address + --label N + --passphrase compose with no special-casing (same b_scan/b_spend; m=0 only via --change-address; --label 0 stays refused).

VERDICT: GREEN (0C/0I)

---

## Fold disposition (controller)
GREEN gate satisfied — implementation may begin. Folding the recommended Minors into the plan text first (per reviewer): M1 (hoist guard to top of run), M2 (read_stdin_passphrase for --passphrase-stdin — prevents whitespace corruption), M3 (JSON `change_address_warning` sibling marker + test), M5 (tighten edit-set prose). M4 = no change (oracle sufficient). GUI bump resolved to PATCH. Doc-only folds, no R1 re-dispatch (no Critical/Important).
