# v0.9.0 Cycle A — secret-memory-hygiene matrix (mnemonic-toolkit)

**Cycle:** OWNED-buffer secret-memory hygiene v0.9.0 Cycle A.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_0.md`.
**Plan:** `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`.
**Survey:** `design/agent-reports/v0_9_0-secret-memory-survey.md`.
**Cycle reports:**
  - Phase 0: `v0_9_0-phase-0-spec-plan-r1.md` (R1-R5 disposition)
  - Phase 1: `v0_9_0-phase-1-{xprv-cell-opinion-r1,argv-leakage-r1,argv-leakage-r2}.md`
  - Phase 2: `v0_9_0-phase-2-zeroize-r{1,2}.md`

This matrix is the cross-repo audit hub for Cycle A's two-prong
scope (argv-leakage close + OWNED-buffer Zeroizing). Every
survey-§1 row + survey-§5 flag-row gets a status cell here; the
SPEC §3 OOS entries are surfaced for forward visibility; Cycle B
(mlock) carry-overs are listed in §4.

## §0 Cross-repo coverage

| Repo | Branch (Phase 2 close) | Phases participated | Matrix file | Delta cells |
|------|------------------------|--------------------|-------------|-------------|
| mnemonic-toolkit (this repo) | `v0_9_0-phase-2-zeroize` @ `863f18a` | 1 (argv-leakage), 2 (zeroize), 3 (matrix), E (rollup) | this file (§1 + §2 + §3 + §4) | 9 new argv-flag closures (Phase 1) + ~30 toolkit OWNED-row wraps per SPEC §2 (enumerated in §1: 38 row-cells) + 32 SAFETY anchors |
| mnemonic-secret | `v0_9_0-phase-2-zeroize` @ `123dea3` | 2 (zeroize), 3 (matrix), E (rollup) | `mnemonic-secret/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` | 4 ms-codec OWNED rows + 10 ms-cli OWNED rows |
| descriptor-mnemonic (md) | — | — (xpub-only material; no companion cycle entry per R3 I-R3-4 fold + SPEC §3 OOS-md-mk) | — | 0 |
| mnemonic-key (mk) | — | — (same as md) | — | 0 |

## §0.5 What this cycle does NOT close

The Cycle A "first-pass at OWNED-buffer secret-memory hygiene"
contract is bounded. Five classes of residual secret-memory
exposure remain after Cycle A ships:

1. **`Xpriv`-Copy residue (upstream-blocked).**
   `bitcoin::bip32::Xpriv` is `Copy + !Drop`. Wrapping a binding in
   `Zeroizing<Xpriv>` only scrubs that one binding's stack copy;
   every `derive_priv()` returns a fresh stack copy that is dropped
   un-scrubbed. The cycle ships `SAFETY: third-party-blocked`
   doc-comments at every `Xpriv::new_master` / `.derive_priv(`
   site naming `rust-bitcoin-xpriv-zeroize-upstream` (FOLLOWUPS,
   tier `external`) as the upstream fix path.

2. **`Mnemonic`-interior residue (upstream-blocked).**
   `bip39::Mnemonic` holds the wordlist-resolved phrase verbatim
   in its private buffer and drops un-scrubbed. The cycle minimizes
   `Mnemonic` lifetime (construct → `to_entropy()` / `to_seed()`
   into `Zeroizing` → drop ASAP) but cannot scrub the interior
   without an upstream PR. SAFETY anchors at every call site cite
   `rust-bip39-mnemonic-zeroize-upstream` (FOLLOWUPS, tier
   `external`).

3. **`secp256k1::SecretKey` stack-bound residue (upstream-blocked).**
   `SecretKey` provides `non_secure_erase()` (best-effort,
   compiler-defeatable per upstream's own doc) but no Drop +
   Zeroize. Production sites at `bip85.rs:104, 133`,
   `parse_descriptor.rs:778`, `cmd/convert.rs:1135, 1245`. SAFETY
   anchors cite `rust-secp256k1-secretkey-zeroize-upstream`
   (FOLLOWUPS, tier `external` — surfaced by Phase 2 R1 I-2 fold).

4. **libc-OsString pre-clap copy (kernel/libc layer).**
   The libc + Rust startup copies `argv[]` into an `OsString` chain
   before clap parses. This pre-parse copy lives in heap-allocated
   memory the toolkit does NOT control; clap's parsed `String`
   fields can be wrapped in `Zeroizing` (Cycle A does this at
   `run()` entry via consume + `mem::take`-style patterns) but the
   pre-clap residual copy cannot be reached. FOLLOWUP:
   `clap-argv-pre-parse-residue` (toolkit; tier `v1+`).

5. **`/proc/self/cmdline` post-parse retention + custom-allocator
   pool residue.** The kernel-owned `/proc/N/cmdline` mirror of
   `argv[]` is also outside the toolkit's reach. mlock cannot
   retroactively cover it. Mitigations require either
   `prctl(PR_SET_DUMPABLE, 0)` or argv-overwrite-after-parse
   (FOLLOWUP `argv-overwrite-after-parse` in toolkit). Cycle A
   ships the secret-in-argv stderr advisory but does not rewrite
   argv. Separately, custom-allocator pools that retain freed
   buffers indefinitely (not the system allocator default; e.g.,
   `jemalloc` configured with cache retention) would defeat
   Zeroizing's drop-time scrub for any wrapper whose backing
   allocation is pooled. The Cycle A test environment uses the
   system allocator; custom allocators are NOT in scope. FOLLOWUP:
   `allocator-pool-residue` (toolkit; tier `v1+`).

6. **mlock / page-pinning (deferred to Cycle B).** Cycle A does
   NOT prevent the OS from swapping or core-dumping pages that hold
   secret material. Cycle B (`secret-memory-hygiene-cycle-b`) will
   add `mlock` infrastructure to the 5 highest-value sites named
   in survey §4. Tier: `v0.9.x` (Cycle B SPEC drafting starts
   post-Cycle-A ship).

## §1 Survey §1 OWNED-buffer row coverage

Status legend:

- **CLEAR**: fully scrub-on-drop wrapped (`Zeroizing<...>` local OR
  `impl Drop` on enclosing pub struct).
- **PARTIAL-3RD-PARTY**: third-party-blocked (Mnemonic / Xpriv /
  SecretKey); lifetime minimized, SAFETY doc-comment in place.
- **OUT-OF-SCOPE**: explicitly deferred per SPEC §3 or this
  matrix §0.5 — named FOLLOWUP carries the deferral.

### Toolkit `cmd/bundle.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `BundleArgs::passphrase` clap-field | `bundle.rs:42` | CLEAR | wrapped at `bundle.rs:917` via consume-clone into `Zeroizing<String>` in descriptor Phrase arm; per-run wrap pattern. |
| `BundleArgs::slot` Vec | `bundle.rs:77` | CLEAR via transit | survey-§5 secret-bearing slot values now have `=-` route (Phase 1); inline values trigger advisory + value transits via `SlotInput.value: String` consumed at slot-bind time. |
| `resolve_slots` Phrase arm derived account | `bundle.rs:346` | CLEAR | `acc.into_parts()` (Phase 2 prereq) + `impl Drop for DerivedAccount` scrubs entropy on drop. |
| `resolve_slots` Entropy arm | `bundle.rs:447` | CLEAR | same `into_parts()` migration; entropy_bytes flow through `Zeroizing<Vec<u8>>` in descriptor Entropy arm. |
| `bundle_run_unified_descriptor` Phrase arm | `bundle.rs:910-940` | CLEAR | `Zeroizing::new(args.passphrase.clone().unwrap_or_default())` + `Zeroizing::new(mnemonic.to_entropy())` + `derive_master_seed`-wrapped `seed`. SAFETY anchor cites `rust-bip39-mnemonic-zeroize-upstream`. |
| `bundle_run_unified_descriptor` Entropy arm | `bundle.rs:982-1009` | CLEAR | `Zeroizing::new(hex::decode(entropy_hex)?)` + `Zeroizing<String>` passphrase + `derive_master_seed`. |
| `entropy_at_0` clone | `bundle.rs:877` | CLEAR via consumer wrap | `entropy_at_0: Option<Vec<u8>>` consumed by downstream `synthesize_*` calls that wrap on entry. |
| `ResolvedSlot.entropy` field | `synthesize.rs:582` | OUT-OF-SCOPE | deferred to FOLLOWUP `resolved-slot-entropy-zeroizing-field` (v0.9.2-nice-to-have; 19-site cascade). Local-wrap discipline at producer + consumer sites covers transit; only brief field-resident lifetime is unwrapped. |

### Toolkit `cmd/verify_bundle.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `VerifyBundleArgs::passphrase` clap-field | `verify_bundle.rs:43` | CLEAR | `--passphrase-stdin` (Phase 1) routes inline form to stdin; downstream consumer pattern wraps. |
| `entropy_at_0` clone | `verify_bundle.rs:461` | CLEAR | typed `Option<zeroize::Zeroizing<Vec<u8>>>` per Phase 2; consumer at L503 uses `.as_ref().map(\|z\| &z[..])`. |

### Toolkit `cmd/derive_child.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `DeriveChildArgs::from` clap-field | `derive_child.rs:26` | CLEAR | `from_value: Zeroizing<String>` at `derive_child.rs:98-102`. |
| `DeriveChildArgs::passphrase` clap-field | `derive_child.rs:61` | CLEAR via Phase 1 `--passphrase-stdin` + Phase 2 transit wrap. |
| `stdin_passphrase` Option | `derive_child.rs:108-122` | CLEAR | `Option<zeroize::Zeroizing<String>>` (R1 I-3 fold) at L108; `.as_ref().map(\|z\| z.as_str())` consumer at L135. |
| Phrase-master run() locals | `derive_child.rs:125-141` | PARTIAL-3RD-PARTY | `Mnemonic` + `Xpriv` upstream-blocked; SAFETY anchors at L126-130 cite `rust-bip39-mnemonic-zeroize-upstream` + `rust-bitcoin-xpriv-zeroize-upstream`. |

### Toolkit `cmd/convert.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `ConvertArgs::from` Vec<FromInput> | `convert.rs:147` | CLEAR via existing `=-` route + Phase 1 advisory. |
| `ConvertArgs::passphrase` | `convert.rs:165` | CLEAR | pre-existing `--passphrase-stdin` + `effective_passphrase: Option<String>` local at L614 (caller-wraps if persisted). |
| `ConvertArgs::bip38_passphrase` | `convert.rs:175` | CLEAR | Phase 1 added `--bip38-passphrase-stdin` (BIP-38 V3 NULL-byte gap closed). |
| `compute_outputs` Phrase/Entropy arm | `convert.rs:882-934` | CLEAR | `entropy: zeroize::Zeroizing<Vec<u8>>` typed local; SAFETY at L887. |
| `Wif` arm `PrivateKey.inner` | `convert.rs:1132-1138` | PARTIAL-3RD-PARTY | `SecretKey` stack-bound; SAFETY at L1132-1133. |
| `Bip38` decrypt arm raw 32-B | `convert.rs:1135` | PARTIAL-3RD-PARTY | same; SAFETY anchor. |
| `Ms1` arm entropy | `convert.rs:1156-1174` | CLEAR | `entropy: zeroize::Zeroizing<Vec<u8>>` typed local at L1163 wrapping the `Payload::Entr(bytes)` consumer side per `payload.rs` caller-wrap contract. |
| `MiniKey` arm raw 32-B | `convert.rs:1245` | PARTIAL-3RD-PARTY | SAFETY at L1243-1244 (Phase 2 R1 I-2 fold). |
| `ElectrumPhrase` arm | `convert.rs:1199-1213` | CLEAR via consumer wrap | entropy flows from `electrum::phrase_to_entropy` whose accumulator is now `Zeroizing<Vec<u8>>`. |

### Toolkit `derive.rs` / `derive_slot.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `DerivedAccount` struct | `derive.rs:20-58` | CLEAR | `pub struct DerivedAccount` at L20; `pub fn into_parts` at L36 enables E0509-safe consumer migration; `impl Drop for DerivedAccount` at L49-58 scrubs entropy on drop. |
| `derive_full()` entropy local | `derive.rs:69-83` | CLEAR | `Zeroizing::new(mnemonic.to_entropy())`. SAFETY at L72-74. |
| `derive_bip32_from_entropy` seed | `derive_slot.rs:43+` | CLEAR | `derive_master_seed(&mnemonic, passphrase)` returns `Zeroizing<[u8; 64]>`. SAFETY at L42-45. |
| `derive_bip32_at_path` seed | `derive_slot.rs:95+` | CLEAR | same helper; SAFETY at L94-96. |
| `derive_master_seed` helper | `derive_slot.rs:32-34` | CLEAR (canonical) | consolidates 7 production BIP-39→BIP-32 spines into one site returning `Zeroizing<[u8; 64]>`. |

### Toolkit `bip85.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `derive_entropy` returned 64-B | `bip85.rs:21-46` | CLEAR | return type `Result<Zeroizing<[u8; 64]>, ToolkitError>`. SAFETY at L36-38. |
| `format_bip39_phrase` entropy local | `bip85.rs:63-78` | CLEAR | inherits from `derive_entropy`. SAFETY at L70-71. |
| `format_hd_seed_wif` SecretKey + PrivateKey | `bip85.rs:88-107` | PARTIAL-3RD-PARTY | SAFETY at L100-103 cites `rust-secp256k1-secretkey-zeroize-upstream`. |
| `format_xprv_child` SecretKey + Xpriv | `bip85.rs:111-134` | PARTIAL-3RD-PARTY | SAFETY at L128-132 cites both `rust-secp256k1-secretkey-zeroize-upstream` + `rust-bitcoin-xpriv-zeroize-upstream`. |
| `format_hex_bytes` / `format_password_base64` / `format_password_base85` / `format_dice_rolls` | `bip85.rs:137-244` | CLEAR | inherit `Zeroizing<[u8; 64]>` entropy from `derive_entropy`. |

### Toolkit `synthesize.rs` / `parse_descriptor.rs` / `electrum.rs`

| Row | Site | Status | Evidence |
|-----|------|--------|----------|
| `synthesize_multisig_full` entropy | `synthesize.rs:404-405` | CLEAR | `Zeroizing::new(seed_mnemonic.to_entropy())` (R1 I-1 fold). |
| `synthesize_multisig_full` seed | `synthesize.rs:324-326` | CLEAR | `derive_master_seed`-wrapped. SAFETY at L323-325. |
| `parse_descriptor::bind_full_mode` entropy + seed | `parse_descriptor.rs:863-872` | CLEAR | `Zeroizing::new(mnemonic.to_entropy())` + `derive_master_seed`. SAFETY at L859-863. |
| `electrum::phrase_to_entropy` accumulator | `electrum.rs:107` | CLEAR | `Zeroizing::new(vec![0])`. |
| `electrum::entropy_to_phrase` accumulator | `electrum.rs:147` | CLEAR | `Zeroizing::new(entropy.iter().rev().copied().collect())`. |

## §2 Survey §5 argv-leakage flag-row coverage

All 20 toolkit secret-bearing flag-rows per survey §5 are
enumerated below. Closure is via `--*-stdin` paired flag OR `=-`
value carve-out. Phase 1 added 9 new closures; the other 11 were
already closed pre-cycle.

| Flag-row | Pre-cycle stdin route? | Status |
|----------|------------------------|--------|
| `bundle --passphrase` | NO | CLEAR — `--passphrase-stdin` (Phase 1) |
| `bundle --slot @N.phrase=` | NO | CLEAR — `=-` route (Phase 1) |
| `bundle --slot @N.entropy=` | NO | CLEAR — `=-` route (Phase 1) |
| `bundle --slot @N.wif=` | NO | CLEAR — `=-` route (Phase 1) |
| `bundle --slot @N.xprv=` | NO | CLEAR-STRUCTURAL — `=-` parser accepts; runtime rejects per v0.4.2 deferral; lint anchor + advisory cover (Phase 1) |
| `verify-bundle --passphrase` | NO | CLEAR — `--passphrase-stdin` (Phase 1) |
| `verify-bundle --slot @N.<secret>=` | NO | CLEAR — `=-` route (Phase 1) |
| `convert --from phrase=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from entropy=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from xprv=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from wif=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from ms1=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from bip38=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from minikey=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --from electrum-phrase=` | YES (`=-`) | CLEAR — pre-cycle |
| `convert --passphrase` | YES (`--passphrase-stdin`) | CLEAR — pre-cycle |
| `convert --bip38-passphrase` | NO | CLEAR — `--bip38-passphrase-stdin` (Phase 1; closes BIP-38 V3 NULL-byte gap) |
| `derive-child --from xprv=` | YES (`=-`) | CLEAR — pre-cycle |
| `derive-child --from phrase=` | YES (`=-`) | CLEAR — pre-cycle |
| `derive-child --passphrase` | NO | CLEAR — `--passphrase-stdin` (Phase 1) |

**Phase 1 advisory:** every inline-secret occurrence at run-time
emits a `warning: secret material on argv (...) — pipe via ... to
avoid /proc/$PID/cmdline exposure` stderr line (`secret_advisory.rs`).
Per-(flag, slot-index) emission so users see every leak site.

## §3 SPEC §3 FOLLOWUPS forward-visibility list

The 14 SPEC §3 OOS classes + the 4 cycle-surfaced entries that
shipped with Cycle A — all entries open in the respective repos'
`design/FOLLOWUPS.md` files. Cross-cite verified at Phase 3 R1
fold.

**SPEC §3 OOS entries (14):**

| FOLLOWUP id | SPEC §3 anchor | Tier | Repo | Status |
|-------------|---------------|------|------|--------|
| `rust-bitcoin-xpriv-zeroize-upstream` | Xpriv Copy + Zeroize | external | toolkit | open |
| `rust-bip39-mnemonic-zeroize-upstream` | bip39 Mnemonic interior | external | toolkit | open |
| `argv-overwrite-after-parse` | /proc/self/cmdline | v1+ | toolkit | open |
| `ms-codec-payload-zeroize-public-api` | OOS-public-payload | v1+ | ms-codec | open |
| `pub-struct-drop-semver-risk-monitor` | OOS-pub-struct-drop | v1+ | toolkit | open (monitoring) |
| `clap-argv-pre-parse-residue` | OOS-libc-osstring | v1+ | toolkit | open |
| `allocator-pool-residue` | OOS-allocator-residue | v1+ | toolkit | open |
| `secret-memory-hygiene-cycle-b` | OOS-mlock-cycle-b | v0.9.x | toolkit | open |
| `dedicated-secret-arena` | OOS-secret-arena | v1+ | toolkit | open |
| `sha3-shake256-zeroize-upstream` | SHAKE256 XOF state | external | toolkit | open |
| `bip38-crate-internal-zeroize-upstream` | bip38 internals | external | toolkit | open |
| `ms-codec-doc-example-zeroize-consistency` | OOS-7 | v1+ | ms-codec | open |
| `ms-cli-decode-emit-zeroize-intermediate` | OOS-decode-stdout | v1+ | ms-cli | open |
| `md-mk-private-key-surface-watch` | OOS-md-mk | cross-repo | cross-repo | open (monitoring) |

**Cycle-surfaced entries (4) — not in SPEC §3 but opened during Phase 1-2:**

| FOLLOWUP id | Surfaced | Tier | Repo | Status |
|-------------|----------|------|------|--------|
| `rust-secp256k1-secretkey-zeroize-upstream` | Phase 2 R1 I-2 fold | external | toolkit | open |
| `resolved-slot-entropy-zeroizing-field` | Phase 2 GREEN (19-site cascade deferral) | v0.9.2-nice-to-have | toolkit | open |
| `convert-minikey-stdout-redaction` | Phase 1 R1 N-2 partial | v0.9.1-nice-to-have | toolkit | open |
| `rust-codex32-zeroize-upstream` | Phase 2 ms-codec envelope work | external | ms-codec | open |

**Cycle meta entry:** `secret-memory-hygiene-v0_9-cycle-a` (cross-repo;
closes at Phase E rollup when patch tags ship).

## §4 Cycle A → Cycle B carry-overs

Cycle B (`secret-memory-hygiene-cycle-b`) is the mlock /
page-pinning workstream deferred from Cycle A per Phase 0 R3
SPLIT-CYCLE fold. The 5 mlock candidates named in survey §4:

1. **Top-priority:** mlock the `DerivedAccount.entropy` Vec and
   the `derive_master_seed` Zeroizing<[u8; 64]> seed buffer.
   Highest concentration of secret bytes in the hottest spine.
2. **Top-priority:** mlock the `secp256k1::SecretKey` scalar
   bytes in `bip85.rs::format_hd_seed_wif` / `format_xprv_child`
   call sites. Defense-in-depth for the
   `rust-secp256k1-secretkey-zeroize-upstream` residue.
3. **Top-priority:** mlock the `bip39::Mnemonic` interior. Defense-
   in-depth for `rust-bip39-mnemonic-zeroize-upstream`.
4. **Lower-priority:** mlock the `--passphrase-stdin` /
   `--bip38-passphrase-stdin` read buffer in `convert.rs:557+`.
5. **Lower-priority:** mlock the `electrum.rs::phrase_to_entropy`
   accumulator.

Cycle B SPEC will assess libc / Linux-specific mlock infrastructure
(toolkit only; ms-secret has no syscall layer) + per-page rather
than per-allocation guarantees. Tag plan TBD at Cycle B Phase 0.

## §5 Cycle-close gates (SPEC §6)

All six SPEC §6 gates satisfied at Phase 2 close:

1. ✓ Argv-leakage closure for 9 toolkit flag-rows (Phase 1).
2. ✓ Zeroizing wrappers on every OWNED secret allocation in the
   3 touched crates' canonical lint enumeration (Phase 2; with
   one row deferred via FOLLOWUP per §3).
3. ✓ `lint_argv_secret_flags.rs` + `lint_zeroize_discipline.rs` +
   `lint_safety_third_party_blocked.rs` all green at Phase 2 close.
4. ✓ All 14 SPEC §3 OOS entries have FOLLOWUPS opened (see §3
   above; verified open in `design/FOLLOWUPS.md` at Phase 3 R1
   fold). Plus 4 cycle-surfaced entries also opened.
5. ✓ Cross-repo coordination via the sibling ms-secret matrix
   file (see §0).
6. ✓ This matrix file in place (Phase 3 deliverable).

Phase E (release rollup) is the final cycle-close step.
