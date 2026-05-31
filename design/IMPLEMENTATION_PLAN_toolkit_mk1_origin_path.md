# mk1 origin-path consistency (re-pin to mk-codec 0.4.0) — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: superpowers:subagent-driven-development or executing-plans. Steps use checkbox (`- [ ]`).

**Goal:** Re-pin `mnemonic-toolkit` to mk-codec 0.4.0 and make every mk1 card's `origin_path` round-trip the xpub it carries (the encode-guard's invariant), via a centralized `mk1_origin_path` helper + a redesigned verify-bundle cross-check that compares the mk1 xpub against md1's depth-`d` *prefix*.

**Architecture:** Toolkit-only. Helper at the 4 live `KeyCard::new` sites (md1 keeps the full origin independently). Cross-checks rekey off the xpub's own depth `d`. PATCH 0.37.9 → 0.37.10; no GUI/manual lockstep.

**Source SPEC (R0 GREEN):** `design/SPEC_toolkit_mk1_origin_path.md` (R1 0C/0I). Base `master` `a255060`; consumes published `mk-codec 0.4.0`.

---

## Phase 0 — helper + call sites + reject removal

### Task 0.1 — `mk1_origin_path` helper + `ChildNumber` import + unit tests

**Files:** Modify `crates/mnemonic-toolkit/src/synthesize.rs` (import `:12`, add helper, test mod).

- [ ] **Step 1 — Add `ChildNumber` to the import (`:12`).** `use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpriv, Xpub};`

- [ ] **Step 2 — Write the helper** (place near `derivation_path_to_origin_path`, ~`:51`):

```rust
/// Derive the mk1 card's `origin_path` so it round-trips the xpub it carries.
///
/// mk-codec compact-73 reconstructs `depth := component_count(origin_path)` and
/// `child_number := last_component(origin_path)` (or `Normal{0}` empty); mk-codec
/// 0.4.0 rejects any card whose xpub depth/child disagree. The DESCRIPTOR origin
/// (carried independently by md1's path_decl) may be deeper (account xpub + BIP-48
/// leaf path), shallower (a leaf xpub re-annotated with an account origin), or
/// absent. We build a path of length `xpub.depth` whose terminal equals
/// `xpub.child_number`, reusing the descriptor path's leading components for the
/// (non-load-bearing, informational) intermediates.
pub(crate) fn mk1_origin_path(xpub: &Xpub, descriptor_path: &DerivationPath) -> DerivationPath {
    let depth = xpub.depth as usize;
    if depth == 0 {
        return DerivationPath::master(); // empty — no-path / depth-0 key (e.g. a WIF)
    }
    let comps: Vec<ChildNumber> = descriptor_path.into_iter().copied().collect();
    let mut out: Vec<ChildNumber> = Vec::with_capacity(depth);
    for i in 0..(depth - 1) {
        out.push(comps.get(i).copied().unwrap_or(ChildNumber::Normal { index: 0 }));
    }
    out.push(xpub.child_number);
    DerivationPath::from(out)
}
```

- [ ] **Step 3 — Unit tests** (synthesize.rs test mod). Build xpubs by deriving from a fixed seed; assert the round-trip invariant + that the resulting card encodes:

```rust
#[test]
fn mk1_origin_path_round_trips_every_class() {
    use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
    use bitcoin::secp256k1::Secp256k1;
    use std::str::FromStr;
    let secp = Secp256k1::new();
    let seed = [7u8; 32];
    let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
    let xpub_at = |p: &str| {
        let path = DerivationPath::from_str(p).unwrap();
        Xpub::from_priv(&secp, &master.derive_priv(&secp, &path).unwrap())
    };
    // case → (xpub, descriptor_path)
    let cases: &[(Xpub, &str)] = &[
        (xpub_at("m/84'/0'/0'"), "m/84'/0'/0'"),       // consistent 3→3
        (xpub_at("m/48'/0'/0'"), "m/48'/0'/0'/2'"),    // 3→4 truncate
        (xpub_at("m/48'/0'/0'/2'"), "m/87'/0'/0'"),    // 4→3 extend
        (xpub_at("m/84'/0'/0'"), "m"),                 // 3→0 pad
        (xpub_at("m/0'"), "m/0'"),                     // depth-1
    ];
    for (xpub, dpath) in cases {
        let out = mk1_origin_path(xpub, &DerivationPath::from_str(dpath).unwrap());
        let comps: Vec<_> = out.into_iter().copied().collect();
        assert_eq!(comps.len(), xpub.depth as usize, "len==depth for {dpath}");
        assert_eq!(*comps.last().unwrap(), xpub.child_number, "last==child for {dpath}");
        // The card must now ENCODE (no XpubOriginPathMismatch).
        let card = mk_codec::KeyCard::new(vec![[0xAAu8; 4]], None, out, *xpub);
        assert!(mk_codec::encode_with_chunk_set_id(&card, 0).is_ok(), "encodes for {dpath}");
    }
}

#[test]
fn mk1_origin_path_depth0_is_empty() {
    // A WIF-style depth-0 xpub → empty path.
    let secp = bitcoin::secp256k1::Secp256k1::new();
    let sk = bitcoin::PrivateKey::from_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn").unwrap();
    let xpub = bitcoin::bip32::Xpub {
        network: bitcoin::NetworkKind::Main, depth: 0,
        parent_fingerprint: Default::default(),
        child_number: bitcoin::bip32::ChildNumber::Normal { index: 0 },
        public_key: sk.public_key(&secp).inner,
        chain_code: bitcoin::bip32::ChainCode::from([0u8; 32]),
    };
    let out = mk1_origin_path(&xpub, &bitcoin::bip32::DerivationPath::master());
    assert_eq!(out.into_iter().count(), 0);
}
```

- [ ] **Step 4 — Run** `cargo test -p mnemonic-toolkit --lib synthesize::tests::mk1_origin_path` → PASS.

### Task 0.2 — Wire the 4 live + 4 test `KeyCard::new` sites; remove reject loop

**Files:** Modify `crates/mnemonic-toolkit/src/synthesize.rs`.

- [ ] **Step 1 — Replace the path argument at all 8 sites.** At `:137,:168,:221,:238,:386,:556,:773,:789`, change the `KeyCard::new(..., <path>, <xpub>)` path argument from `<path>.clone()` to `mk1_origin_path(&<xpub>, &<path>)`. (e.g. `:780`/`:796` in `synthesize_unified`: `s.path.clone()` → `mk1_origin_path(&s.xpub, &s.path)`; `:221`/`:238` in `synthesize_descriptor`: `c.path.clone()` → `mk1_origin_path(&c.xpub, &c.path)`.) **Do NOT touch the md1 `path_decl` build at `:710-717` or the per-cosigner `paths`/`origin_paths` used for md1.**

- [ ] **Step 2 — Remove the reject loop** at `synthesize_multisig_watch_only:494-506` (the `// 4. SPEC §4.5 path/xpub depth consistency check.` for-loop that returns `CosignerSpec`). The helper makes the card consistent-by-construction.

- [ ] **Step 3 — Run** `cargo test -p mnemonic-toolkit --lib synthesize` → the synthesize lib tests pass (helper is a no-op for their consistent fixtures).

- [ ] **Step 4 — Commit Phase 0.** `git add crates/mnemonic-toolkit/src/synthesize.rs && git commit -m "feat(toolkit): mk1_origin_path helper — derive card path from the xpub it carries"`

---

## Phase 1 — verify-bundle cross-check redesign (load-bearing)

### Task 1.1 — `emit_watch_only_xpub_path_cross_check` (§3.5a)

**Files:** Modify `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:2116-2199` (the per-card Checks 1/2/3 block inside the `for (i, card) in &mk_cards` loop).

- [ ] **Step 1 — Replace Checks 1/2/3** (current `:2116-2199`, from `// Check 1: depth.` through the deeper-paths skip comment) with Checks A/B/C keyed off `d = card.xpub.depth`:

```rust
        let d = card.xpub.depth as usize;
        let md_depth = md_path.components.len();
        let (xpub_idx, xpub_hardened) = match card.xpub.child_number {
            bitcoin::bip32::ChildNumber::Normal { index } => (index, false),
            bitcoin::bip32::ChildNumber::Hardened { index } => (index, true),
        };

        // Check A: the card must not claim a node DEEPER than md1's declared
        // origin. d <= md_depth is fine (3→4 account-xpub truncation); d > md_depth
        // means md1 cannot be this key's full origin.
        if d > md_depth {
            writeln!(
                stderr,
                "warning: cosigner[{}] mk1 xpub depth ({}) is deeper than md1 origin-path length ({}); the card claims a node below the declared origin — cards are internally inconsistent",
                i, d, md_depth
            ).ok();
        }

        // Check B: the xpub's child_number must equal md1's component at index d-1
        // (md1 TRUNCATED to the xpub's depth, terminal). For d < md_depth this
        // compares against the ACCOUNT-level component, NOT md1's leaf terminal —
        // the change that dissolves the 3→4 false-positive.
        if d >= 1 && d <= md_depth {
            let md_at = md_path.components[d - 1];
            if xpub_idx != md_at.value || xpub_hardened != md_at.hardened {
                writeln!(
                    stderr,
                    "warning: cosigner[{}] mk1 xpub child_number ({}{}) does not match md1 origin-path component #{} ({}{}); cards are internally inconsistent",
                    i, xpub_idx, if xpub_hardened { "'" } else { "" },
                    d, md_at.value, if md_at.hardened { "'" } else { "" },
                ).ok();
            }
        }

        // Check C: parent_fingerprint structural sanity, keyed off the xpub's own
        // depth d (NOT md_depth). Depth >= 2 is verified by
        // emit_full_path_parent_fingerprint_check (needs ms1 to derive the parent).
        let pfp = card.xpub.parent_fingerprint.to_bytes();
        if d == 0 {
            if pfp != [0u8; 4] {
                writeln!(
                    stderr,
                    "warning: cosigner[{}] mk1 xpub parent_fingerprint ({}) is non-zero at depth 0 (expected 00000000); cards are internally inconsistent",
                    i, hex::encode(pfp)
                ).ok();
            }
        } else if d == 1 {
            let claimed_master_fp = md_fp_for(*i)
                .or_else(|| card.origin_fingerprint.map(|f| f.to_bytes()));
            if let Some(master_fp) = claimed_master_fp {
                if pfp != master_fp {
                    writeln!(
                        stderr,
                        "warning: cosigner[{}] mk1 xpub parent_fingerprint ({}) does not match claimed master fingerprint ({}) at depth 1; cards are internally inconsistent",
                        i, hex::encode(pfp), hex::encode(master_fp),
                    ).ok();
                }
            }
        }
```

(M1 fold: the `split_child` from the SPEC is inlined as the `match` above — no helper. M2 fold: Check C preserves the depth-1 `claimed_master_fp` fallback, now gated on `d` not `md_depth`.)

### Task 1.2 — `emit_full_path_parent_fingerprint_check` (§3.5b)

**Files:** Modify `verify_bundle.rs:2323-2402`.

- [ ] **Step 1 — Move the depth gate to `d` (`:2328`).** Replace `if md_path.components.len() < 2 { … continue; }` with, at the top of the `for (i, card)` loop body (after `md_path` is bound):
```rust
        let d = card.xpub.depth as usize;
        if d < 2 {
            // depth 0/1 handled by emit_watch_only_xpub_path_cross_check's Check C.
            continue;
        }
```

- [ ] **Step 2 — Depth-notice print (`:2341-2344`)** — report `d`, not `md_path.components.len()`:
```rust
                "notice: cosigner[{}] mk1 parent_fingerprint at depth {} unverified (requires ms1 to derive parent xpub)",
                i, d
```

- [ ] **Step 3 — Parent prefix (`:2379-2382`)** — derive at `full[..d-1]` with a bounds gate:
```rust
        if d - 1 > full_components.len() {
            // The xpub claims a node more than one level below md1's origin; can't
            // form the parent prefix. Check A (above) already warns this case.
            continue;
        }
        let parent_components: Vec<bitcoin::bip32::ChildNumber> =
            full_components[..d - 1].to_vec();
        let parent_path = bitcoin::bip32::DerivationPath::from(parent_components);
```

- [ ] **Step 4 — Mismatch print (`:2399`)** — report `d`, not `md_path.components.len()`.

### Task 1.3 — No-false-positive integration test

**Files:** Create/extend a test in `crates/mnemonic-toolkit/tests/` (e.g. `cli_verify_bundle_watch_only.rs`).

- [ ] **Step 1 — Write `correct_3to4_multisig_emits_no_internal_inconsistency_warning`:** build a 2-of-2 watch-only multisig bundle with depth-3 account xpubs + a depth-4 BIP-48 md1 origin (use the toolkit's own `bundle`/`synthesize` so the cards are emitted via the fixed helper), run `verify-bundle`, assert stderr does NOT contain `"internally inconsistent"`. Add a correct 4→4 single-leaf case asserting the same.

- [ ] **Step 2 — Run** Phase-1 tests + the affected `cross_check_*` (which still need the Phase-2 rebuild) → the no-false-positive test PASSES; note the 2 tampered fixtures still fail (rebuilt in Phase 2).

- [ ] **Step 3 — Commit Phase 1.** `git add crates/mnemonic-toolkit/src/cmd/verify_bundle.rs crates/mnemonic-toolkit/tests/cli_verify_bundle_watch_only.rs && git commit -m "fix(toolkit): verify-bundle cross-checks key off the xpub's own depth (mk1 path = md1 depth-d prefix)"`

---

## Phase 2 — C2 tampered fixtures + 3→0 inspect pin

### Task 2.1 — Rebuild the 2 tampered fixtures (M3: divergent values)

**Files:** Modify `crates/mnemonic-toolkit/tests/cli_verify_bundle_watch_only.rs` (`cross_check_mk1_depth_lt_md1_path_warns ~:246`, `cross_check_mk1_parent_fingerprint_mismatch_warns ~:333`).

- [ ] **Step 1 — Study the precedent** `cross_check_mk1_child_number_ne_md1_last_warns:290-324` (a consistent mk1 card whose decoded identity disagrees with md1 — no re-encode).
- [ ] **Step 2 — Rebuild `cross_check_mk1_depth_lt_md1_path_warns`:** emit a CONSISTENT depth-2 mk1 card (xpub derived at a depth-2 path, e.g. `m/84'/1'`, child `1'`) and pair it with a depth-3 md1 origin whose component[1] is `0'` (≠ the card's child `1'`). On 0.4.0 the new **Check B** fires (`md_path.components[1]=0'` ≠ xpub child `1'`); assert the `"internally inconsistent"` warning. **Pin the child to differ from md1.components[d-1]** (M3).
- [ ] **Step 3 — Rebuild `cross_check_mk1_parent_fingerprint_mismatch_warns`:** emit a consistent depth-2 mk1 card whose `parent_fingerprint` differs from the value derivable from md1's depth-1 prefix; assert Check C (d==... — actually depth-2 → routed to `emit_full_path_parent_fingerprint_check`) fires when ms1 is supplied. Build so the derived parent fp ≠ the card's parent_fingerprint.
- [ ] **Step 4 — Run** both rebuilt fixtures → PASS (warnings fire as intended).

### Task 2.2 — Pin `inspect --mk1` for the 3→0 case

**Files:** new test in `crates/mnemonic-toolkit/tests/`.

- [ ] **Step 1 — Write `bundle_watch_only_no_origin_xpub_inspect_shows_synthetic_path`:** bundle a depth-3 watch-only xpub with NO declared origin → `inspect --mk1` the emitted card → assert `origin_path: m/0/0/<child>` (the helper-pad output; pin the exact string per the bundled xpub's child). Documents the synthetic-informational contract (SPEC §3.7).

- [ ] **Step 2 — Commit Phase 2.** `git add … && git commit -m "test(toolkit): rebuild tampered cross-check fixtures (0.4.0 guard-safe) + pin 3→0 inspect synthetic path"`

---

## Phase 3 — Snapshot/transcript/convergence regen + helper-fixture fixes

### Task 3.1 — Fix the 2 verify_bundle test-helper fixtures

**Files:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (`helper_multisig_full_*`, `helper_multisig_missing_ms1_*`, ~`:2643`/`:2725`).

- [ ] **Step 1 — Make the fixtures internally consistent:** they derive the xpub at `m/48'/0'/0'/2'` (depth-4) but call `synthesize_full(…, CliTemplate::Bip84, …)` (depth-3 template path). Either (a) derive the xpub at the Bip84 path `m/84'/0'/0'` (depth-3) so it's consistent, OR (b) leave the xpub depth-4 — the helper now normalizes the mk1 path to depth-4 and the test (which reads only `ms1[0]`) passes regardless. Prefer (a) — a consistent fixture is clearer. Run the 2 helper tests → PASS.

### Task 3.2 — Regenerate snapshots with semantic round-trip verification

**Files:** the ~40 failing tests with pinned mk1-chunk / byte-exact / transcript / convergence assertions (the census in the recon).

- [ ] **Step 1 — Run the full suite** `cargo test -p mnemonic-toolkit --no-fail-fast` and list remaining failures (should be only pinned-output mismatches now that the encode succeeds).
- [ ] **Step 2 — For EACH failing pinned-mk1 assertion:** regenerate the expected value from the new (correct) output, BUT first **semantically verify** the new card: decode it (`mk_codec::decode`) and confirm `xpub.depth`/`child_number`/`chain_code`/`public_key` match the intended cosigner identity (NOT a blind byte-accept). Update the fixture only after semantic verification. Work file-by-file; commit in logical groups.
- [ ] **Step 3 — Iterate** until `cargo test -p mnemonic-toolkit --no-fail-fast` is **0 failures**.

- [ ] **Step 4 — Commit Phase 3** (may be several commits). `git add … && git commit -m "test(toolkit): regen mk1 snapshots for the corrected origin-path (semantic round-trip verified)"`

---

## Phase 4 — Mirrors + WIF regression + version + ship

### Task 4.1 — Error-mirror arms

**Files:** `crates/mnemonic-toolkit/src/friendly.rs` (~`:123`), `error.rs::mk_codec_exit_code` (~`:392`).

- [ ] **Step 1 — `friendly.rs`** before `_ =>`:
```rust
        E::XpubOriginPathMismatch { xpub_depth, path_depth, .. } => format!(
            "mk1 xpub/origin-path depth mismatch: xpub depth {} vs origin_path depth {} (toolkit bug — the mk1 card's path must round-trip its xpub)",
            xpub_depth, path_depth,
        ),
```
- [ ] **Step 2 — `error.rs::mk_codec_exit_code`** before `_ => 1`: `mk_codec::Error::XpubOriginPathMismatch { .. } => 2,`
- [ ] **Step 3 — Extend the friendly.rs mk-codec test** (~`:302`) to assert `friendly_mk_codec(&mk_codec::Error::XpubOriginPathMismatch { xpub_depth: 3, path_depth: 4, xpub_child: ChildNumber::Hardened { index: 0 }, path_child: None })` contains `"depth mismatch"` and NOT `"unhandled"`.

### Task 4.2 — WIF round-trip regression

**Files:** `crates/mnemonic-toolkit/tests/` (sibling to `bundle_slot_wif_stdin_succeeds`).

- [ ] **Step 1 — `bundle_wif_mk1_round_trips_via_inspect`:** `bundle --slot @0.wif=- … --no-engraving-card --json` (stdin=`MAINNET_WIF`) → parse JSON `.mk1.Single[*]` → `inspect --mk1 <chunk…>` → assert `.success()` (the round-trip the 0.3.1 pin broke; on 0.4.0 the depth-0 card decodes).

### Task 4.3 — Re-pin commit + version + FOLLOWUPs

**Files:** `crates/mnemonic-toolkit/Cargo.toml` (`:3` version, `:21` pin), `Cargo.lock`, `design/FOLLOWUPS.md`.

- [ ] **Step 1 — Version** `0.37.9` → `0.37.10`; pin already `mk-codec = "0.4.0"` (commit the re-pin + lock here with the version bump).
- [ ] **Step 2 — FOLLOWUPs:** file `mk1-card-origin-path-vs-xpub-depth-consistency` (resolved `<phase-3 SHA>`, companion `mnemonic-key mk1-no-path-depth0-support`); flip `mk1-wif-bundle-depth0-invalid-card` + `mk1-depth-child-compensating-check-watch` to resolved.
- [ ] **Step 3 — Commit.** `git add crates/mnemonic-toolkit/Cargo.toml Cargo.lock crates/mnemonic-toolkit/src/friendly.rs crates/mnemonic-toolkit/src/error.rs crates/mnemonic-toolkit/tests/… design/FOLLOWUPS.md && git commit -m "release(toolkit): v0.37.10 — adopt mk-codec 0.4.0 + mk1 origin-path consistency"`

### Task 4.4 — Full-suite gate + end-of-cycle R0 + ship

- [ ] **Step 1 — FULL gate (NEVER `--lib` only):** `cargo test -p mnemonic-toolkit` 0 failures; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` (pre-existing drift: confirm-pre-existing via `git stash` + re-run on master; if it pre-dates this branch, leave for its own hygiene cycle — do NOT fix here); `cargo +stable fmt -p mnemonic-toolkit -- --check` (authoritative; capture the REAL exit code, not `| tail`).
- [ ] **Step 2 — End-of-cycle opus R0** over the full branch diff → persist to `design/agent-reports/`. Fold to GREEN (0C/0I); re-dispatch after any fold.
- [ ] **Step 3 — Clean-tree check** (`git status --porcelain` empty), then ff-merge `master` + push + tag `mnemonic-toolkit-v0.37.10`.

---

## Self-review
- **Spec coverage:** §3.2 helper→0.1; §3.3 sites→0.2; §3.4 reject-removal→0.2; §3.5a→1.1; §3.5b→1.2; §3.6→2.1; §3.7→2.2; §3.8→4.1/4.2; §6 phases→Phases 0-4. All covered. M1 (inline match)→1.1; M2 (Check C d-keying)→1.1; M3 (divergent fixture values)→2.1.
- **Placeholders:** `<phase-3 SHA>` filled at 4.3, `<child>` in 2.2 pinned to the bundled xpub at impl. No TODO/TBD.
- **Type consistency:** `mk1_origin_path(&xpub, &path)` signature identical across 0.1/0.2; `d = card.xpub.depth as usize` identical across 1.1/1.2; `ChildNumber::Normal { index: 0 }` filler matches the helper + the test pins.
- **The parent-fp bounds gate** is `d - 1 > full.len()` (skip), NOT `d > full.len()` — the `d == full.len()+1` leaf case is valid (`full[..d-1]` = all of full).
