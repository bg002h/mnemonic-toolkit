# cycle-prep recon — 2026-05-23 — silent-payment-passphrase + silent-payment-change-address-m0

**Origin/master SHA at recon time:** `6100d85`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/`

Slug(s) verified: `silent-payment-passphrase`, `silent-payment-change-address-m0`. Both CLEAN — every citation ACCURATE against current source; crypto verified vs BIP-352/BIP-39. Both are additive flags on the SAME subcommand (`silent-payment`); recommend ONE combined PATCH cycle.

---

## Per-slug verification

### `silent-payment-passphrase`
- **WHAT:** v0.35.0 resolves seeds with an EMPTY BIP-39 passphrase; add `--passphrase`/`--passphrase-stdin` threaded into `derive_master_seed` so a passphrase-protected wallet's SP address can be derived. xprv input is passphrase-independent.
- **Citations:**
  - `cmd/silent_payment.rs::resolve_master_xpriv` — the `derive_master_seed(&mnemonic, "")` calls — **ACCURATE.** TWO call sites: `:86` (the `to_master` closure used by the ms1 + entropy-hex paths) and `:112` (the BIP-39 phrase path). Both pass `""`. These are exactly where a resolved passphrase threads.
  - `derive_master_seed` signature already takes a passphrase — **ACCURATE.** `derive_slot.rs:32`: `pub fn derive_master_seed(mnemonic: &Mnemonic, passphrase: &str) -> Zeroizing<[u8; 64]>`. Threading is a one-arg swap (`""` → the resolved passphrase); no signature change.
  - `--passphrase`/`--passphrase-stdin` "already secret-classed in `secrets.rs::flag_is_secret`" — **ACCURATE.** `secrets.rs:52-53` matches `"--passphrase" | "--passphrase-stdin"` (also in the doc list `:73-74`). So the flag NAMES already project as secret — the toolkit `gui-schema` will emit `secret:true` for them automatically once wired, and the GUI mirror must set `secret:true` to match. **No `flag_is_secret` change needed.**
  - xprv path is passphrase-independent — **ACCURATE.** `resolve_master_xpriv` branch 1 (`:92` `Xpriv::from_str`) returns the master directly, no `derive_master_seed`. If `--passphrase` is supplied WITH an xprv/tprv input, it has no effect → design must warn (mirror `convert`'s "--passphrase ignored on this edge" advisory).
- **Action for brainstorm spec:** add `--passphrase` (String, secret) + `--passphrase-stdin` (bool) as a mutually-exclusive ArgGroup (mirror the `convert` passphrase pattern); resolve once (argv-leak warning for inline; mlock-pin); pass into BOTH `derive_master_seed` calls (`:86`,`:112`); warn-and-ignore on the xprv path; empty default preserves v0.35.0 output. Net-new flag NAMES on `silent-payment` ⇒ GUI `schema_mirror` + manual lockstep MANDATORY. Cite SHA `6100d85`.

### `silent-payment-change-address-m0`
- **WHAT:** the cycle refuses `--label 0` (reserved BIP-352 change label, never publish) and defers emitting it. Add a dedicated guarded flag that emits the m=0 change address, clearly tagged "do not publish."
- **Citations:**
  - `cmd/silent_payment.rs` `--label 0` refusal — **ACCURATE.** `:138` `if args.label.contains(&0)` → `:140` error "the reserved BIP-352 change label and must never be published; use m≥1". (Doc at `:48`.)
  - `silent_payment.rs::labeled_spend_key` "already handles any m" — **ACCURATE.** `:45` `pub fn labeled_spend_key<C: Verification>(...)` → `:51` `bip0352_label_hash(b_scan, m)`; `bip0352_label_hash` (`:33`) takes `m: u32` with no m≥1 restriction. `labeled_spend_key(secp, &b_scan, b_spend_pub, 0)` computes the m=0 address with no code change.
  - **PRIMARY SOURCE (BIP-352):** m=0 IS the reserved change label; `B_m = B_spend + hash_BIP0352/Label(ser_256(b_scan) ‖ ser_32(m))·G` is defined for m=0 (`ser_32(0)=0x00000000`). The receiver computes the m=0 address and sends their OWN change to it; it is for internal change-detection and must NOT be handed to third parties. The FOLLOWUP framing ("never-publish-labeled", footgun-guard) is CORRECT.
- **Action for brainstorm spec:** add a dedicated flag (e.g. `--change-address`, bool) that emits the m=0 labeled address via `labeled_spend_key(secp, &b_scan, b_spend_pub, 0)` + `encode_sp_address`, rendered with an UNMISTAKABLE "change — internal use, DO NOT hand out as a receiving address" tag (human + a distinct JSON field). Keep the `--label 0` refusal (that path stays refused; `--change-address` is the deliberate, guarded route). Net-new flag NAME ⇒ GUI `schema_mirror` + manual lockstep MANDATORY. Cite SHA `6100d85`.

---

## Cross-cutting observations
1. **No drift, no structural errors.** All 7 citations ACCURATE; both crypto claims confirmed against primary source (BIP-352 label formula handles m=0; BIP-39 passphrase is the standard PBKDF2 salt). This is the cleanest cycle-prep in recent memory — the slugs were filed accurately at v0.35.0 ship time with live citations.
2. **Shared surface ⇒ combine.** Both slugs touch the SAME files: `cmd/silent_payment.rs` (`SilentPaymentArgs` + `run` + `resolve_master_xpriv`), the GUI `SILENT_PAYMENT_FLAGS` const, and the manual `## mnemonic silent-payment` chapter. Doing them separately = two consecutive GUI+manual lockstep round-trips on one subcommand; combining = one.
3. **Secret-projection is already covered for passphrase** (`flag_is_secret` true) — so the GUI lockstep for `--passphrase`/`--passphrase-stdin` is flag-NAME-set + `secret:true` mirroring only; no `secrets.rs`/taxonomy change. `--change-address` is NOT secret (it's a public address, like the base/labeled addresses).
4. **Composition:** `--change-address` derives from the same `b_scan`/`b_spend` as the base address, so it correctly inherits `--passphrase` + `--account` + `--network`. The two features compose with no special-casing.
5. **Tier label** on both is `v0.35+`; current shipped version is `0.36.0`, so the next release is `v0.36.1`.

---

## Recommended brainstorm-session scope
- **ONE combined cycle** — both slugs are additive flags on `mnemonic silent-payment`, same files + same lockstep surface.
- **SemVer: PATCH → v0.36.1.** Additive flags on an existing subcommand (project convention: new top-level subcommand = MINOR; additive flags/values = PATCH). Three net-new flag NAMES: `--passphrase`, `--passphrase-stdin`, `--change-address`.
- **Mandatory locksteps:** (a) GUI `schema_mirror` — add the 3 flags to `mnemonic-gui/src/schema/mnemonic.rs::SILENT_PAYMENT_FLAGS` (`--passphrase`/`--passphrase-stdin` `secret:true`; `--change-address` bool, non-secret) + toolkit pin bump → paired GUI release; (b) manual — extend `docs/manual/src/40-cli-reference/41-mnemonic.md` `## mnemonic silent-payment` flag table + `cli-subcommands.list` already lists `mnemonic silent-payment` (no new line). No sibling-codec companions (toolkit-only feature).
- **Sizing:** small (~120–200 LOC + tests). passphrase = 2 flags + ArgGroup + resolve (mirror `convert` passphrase handling) + thread into 2 call sites + xprv-ignored warning + tests (passphrase changes the derived address → new regression vector). change-address = 1 flag + m=0 emit + footgun-tagged output + test (m=0 address differs from base + from any m≥1).
- **Ordering within the cycle:** passphrase first (it changes the derivation that change-address also consumes), then change-address, then GUI+manual lockstep, then ship. No hard inter-slug dependency, but passphrase-before-change-address keeps the derivation-affecting change first.
- **Open design questions for the brainstorm/architect (R0):** (1) `--passphrase` + xprv input → warn-and-ignore vs hard error? (2) `--change-address` flag name + whether it composes with `--json` as a distinct field vs a `change_address` key alongside `address`; (3) footgun-guard wording for the m=0 output; (4) does `--change-address` also imply emitting the base address, or only the change address?
