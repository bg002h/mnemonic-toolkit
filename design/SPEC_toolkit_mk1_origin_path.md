# SPEC — toolkit mk1 origin-path consistency (re-pin to mk-codec 0.4.0)

**Branch:** `toolkit-mk-codec-0.4.0-repin` (off `master` `a255060`)
**Crate:** `mnemonic-toolkit` 0.37.9 → **0.37.10** (PATCH — binary-private correctness fix; no CLI surface change)
**Dep:** re-pin `mk-codec 0.3.1` → **0.4.0** (already applied on branch).
**Supersedes:** `design/IMPLEMENTATION_PLAN_toolkit_mk_codec_0_4_0_repin.md` (the 2-fixture plan — measurement-incomplete; it ran `--lib` only and never saw the 72 integration failures).
**Resolves:** toolkit `mk1-wif-bundle-depth0-invalid-card`; companion `mnemonic-key` `mk1-no-path-depth0-support`. Updates `mk1-depth-child-compensating-check-watch`.

---

## §1. Problem

mk-codec's compact-73 wire form **drops** `xpub.depth` + `xpub.child_number` and **reconstructs** them on decode from the card's `origin_path`:
`depth := component_count(origin_path)`, `child_number := last_component(origin_path)` (or `Normal{0}` when empty). mk-codec **0.4.0** enforces an encode-time guard `Error::XpubOriginPathMismatch` rejecting any card whose `xpub.depth`/`child_number` disagree with `origin_path` (such a card would decode to a different-metadata xpub). The guard is correct by design and is the exact inverse of `reconstruct_xpub`.

The toolkit pinned `mk-codec 0.3.1` (NO guard) and has been building mk1 cards as `KeyCard::new(stubs, fp, **s.path**, s.xpub)` — pairing the **descriptor's** origin path with the bound **xpub**, which routinely disagree:

- A watch-only multisig cosigner is a **depth-3 account xpub** (`m/48'/0'/0'`, the universal hardware-wallet export) but the descriptor origin is a **depth-4 BIP-48 path** (`m/48'/0'/0'/2'`). The `/2'` is hardened → a watch-only holder can't derive the depth-4 key.
- A watch-only single-sig xpub with no declared origin defaults `s.path` to the **empty** path (`bind_watch_only_singlesig:1062` `unwrap_or(DerivationPath::master())`) while the xpub is depth-3.
- Some flows pair a **depth-4** xpub with a **depth-3** path (taproot multisig import; verify_bundle test fixtures).

Re-pinning to 0.4.0 surfaces **74 failing tests** (157 `XpubOriginPathMismatch` instances) — pre-existing flows that on 0.3.1 silently emitted **wrong-metadata mk1 cards** (chain_code/pubkey correct → addresses derive, but the reconstructed BIP-32 depth/child wrong). Signature census (fresh, full `cargo test -p mnemonic-toolkit`):

| `xpub_depth → path_depth` | instances | cause |
|---|---|---|
| 3 → 4 | 97 | depth-3 account xpub + depth-4 BIP-48 descriptor origin (multisig import/export/BSMS) |
| 4 → 3 | 43 | depth-4 xpub + depth-3 path (taproot multisig import; verify_bundle fixtures; bip48-xpub-with-bip84-template) |
| 3 → 0 | 14 | depth-3 xpub + empty path (watch-only single-sig, no declared origin) |
| 3→1, 3→2, 4→4 | 3 | other depth/terminal-child disagreements |

**Root cause:** the mk1 card's `origin_path` is being set to the **descriptor origin** when it must instead **round-trip the xpub the card carries**. These are two different things: the mk1 origin path's only semantic load is to reconstruct the xpub's depth/child; the authoritative **full** descriptor origin is carried *independently* by md1's `path_decl` (`synthesize.rs:710-717`, built from `s.path`) and the JSON envelope (`origin_path`/`origin_paths`). So deriving the mk1 path from the xpub loses nothing at the bundle level.

---

## §2. Source ground-truth (verified @ `a255060` + branch re-pin)

- **All 8 `mk_codec::KeyCard::new` sites are in `synthesize.rs`:** `:137` (`synthesize_full`), `:168` (`synthesize_watch_only`), `:221`+`:238` (`synthesize_descriptor`), `:386` (`synthesize_multisig_full`), `:556` (`synthesize_multisig_watch_only`), `:773`+`:789` (`synthesize_unified` — the dominant import-json/descriptor/BSMS path). Each takes `s.path.clone()` (or `c.path`/template path) + `s.xpub` (or `c.xpub`).
- **The ONLY depth-consistency check** is `synthesize_multisig_watch_only:494-503` (`path_depth != c.xpub.depth → CosignerSpec` error). `synthesize_unified` has none — which is why the import flows reach `KeyCard::new` and the 0.4.0 guard fires.
- **md1 path_decl is independent:** `synthesize.rs:710-717` builds `PathDeclPaths` from `derivation_path_to_origin_path(&s.path)` — the FULL descriptor origin, untouched by this fix.
- **`bind_watch_only_singlesig:1062`** — `path_from_decl(resolved, 0).unwrap_or(DerivationPath::master())`; `master()` = empty path. (`bind_watch_only_multisig:1091` already *errors* if path absent — the single-sig asymmetry is the `3→0` source.)
- **WIF slot** (`cmd/bundle.rs:655-674`): depth-0 xpub, child `Normal{0}`, empty path — already consistent; 0.4.0 accepts it (depth-0 → empty path → the case `mk-codec 0.4.0` was built for). No change.
- **mk-codec 0.4.0** (`~/.cargo/registry/.../mk-codec-0.4.0/src/`): `bytecode/encode.rs` guard = `xpub.depth as usize != path_depth || xpub.child_number != path_child.unwrap_or(Normal{0})`; `bytecode/xpub_compact.rs` `reconstruct_xpub` = `depth := count`, `child := last.unwrap_or(Normal{0})`. `Error::XpubOriginPathMismatch { xpub_depth: u8, path_depth: u8, xpub_child: ChildNumber, path_child: Option<ChildNumber> }` (enum `#[non_exhaustive]`; variant fields public + nameable).
- **Toolkit error-mirrors** (both `_ =>`-fallback safe, so re-pin compiles): `friendly.rs:124` (`friendly_mk_codec`), `error.rs:393` (`mk_codec_exit_code`).

---

## §3. Design

### 3.1 The principle

The mk1 card's `origin_path` must **round-trip the xpub it carries** — nothing more. mk-codec reconstructs `depth := component_count(path)` and `child := last_component(path)`; the *intermediate* path components are purely informational (the reconstructed `Xpub` is byte-identical regardless of them — only the count and the terminal are load-bearing). So derive the mk1 path **from the xpub's own `depth`/`child_number`**, using the descriptor path for the informational prefix.

### 3.2 The helper (single source of truth)

Add to `synthesize.rs`:

```rust
/// Derive the mk1 card's `origin_path` so it round-trips the xpub it carries.
///
/// mk-codec compact-73 reconstructs `depth := component_count(origin_path)` and
/// `child_number := last_component(origin_path)` (or `Normal{0}` when empty), and
/// mk-codec 0.4.0 rejects any card whose xpub depth/child disagree
/// (`Error::XpubOriginPathMismatch`). The DESCRIPTOR origin (which may be deeper —
/// a depth-4 BIP-48 path for a depth-3 account xpub — or shallower, or absent) is
/// carried independently by md1's `path_decl`; the mk1 origin path's sole job is to
/// encode the xpub's own depth/child. We therefore build a path of length
/// `xpub.depth` whose terminal is `xpub.child_number`, reusing the descriptor path's
/// leading components for the (purely informational) intermediates.
pub(crate) fn mk1_origin_path(xpub: &Xpub, descriptor_path: &DerivationPath) -> DerivationPath {
    let depth = xpub.depth as usize;
    if depth == 0 {
        return DerivationPath::master(); // empty — no-path / depth-0 key (e.g. a WIF)
    }
    let comps: Vec<ChildNumber> = descriptor_path.into_iter().copied().collect();
    let mut out: Vec<ChildNumber> = Vec::with_capacity(depth);
    for i in 0..(depth - 1) {
        // Intermediate components are informational only (not used to reconstruct
        // the xpub). Reuse the descriptor path where available; pad short/absent
        // paths with Hardened{0}. The authoritative full origin lives in md1.
        out.push(comps.get(i).copied().unwrap_or(ChildNumber::Hardened { index: 0 }));
    }
    out.push(xpub.child_number); // terminal MUST equal the xpub's child (round-trip)
    DerivationPath::from(out)
}
```

**Behavior by class (all become structurally round-trippable):**

| input | `mk1_origin_path` output | note |
|---|---|---|
| consistent (`len==depth`, `last==child`) | the descriptor path unchanged | no-op — passing flows untouched |
| `3→4` (depth-3 xpub, depth-4 path) | descriptor[0..2] + child = `m/48'/0'/0'` | truncate to account |
| `4→3` (depth-4 xpub, depth-3 path) | descriptor[0..3] + child = `…/2'` | extend by script component |
| `3→0` (depth-3 xpub, empty path) | `[H0, H0, child]` | padded intermediates (informational) |
| depth-0 (WIF) | empty | no-path |
| `4→4` child mismatch | descriptor[0..3] + xpub.child | xpub is authoritative for itself |

### 3.3 Apply at all 8 `KeyCard::new` sites

Replace the `path` argument at every site with `mk1_origin_path(&<xpub>, &<path>)`:
- `synthesize_full:137`, `synthesize_watch_only:168` (single-sig — usually consistent → no-op; defensive).
- `synthesize_descriptor:221,238`.
- `synthesize_multisig_full:386`.
- `synthesize_multisig_watch_only:556`.
- `synthesize_unified:773,789` (THE dominant path).

md1 `path_decl` construction (`:710-717`) is **unchanged** — it keeps the full descriptor origin.

### 3.4 The `synthesize_multisig_watch_only:494-503` compensating check

That hard-reject (`path_depth != c.xpub.depth → error`) now **blocks a case the helper handles** (depth-3 xpub + depth-4 path is now representable). **Remove** the reject loop (the helper supersedes it: the card is now consistent-by-construction). Resolves the long-standing `mk1-depth-child-compensating-check-watch` (the mk-codec precondition shipped in 0.4.0).

### 3.5 verify-bundle cross-checks (correctness-critical — must verify)

`verify_bundle.rs` cross-checks decoded mk1 origin paths against md1 paths (the `cross_check_mk1_*` tests). After this fix, an emitted mk1 card's origin path is the **xpub-consistent** path (e.g. depth-3 account), while md1 carries the **full** descriptor origin (e.g. depth-4). The cross-check logic must compare on a basis that tolerates this by-design divergence (e.g. compare the mk1 path as a *prefix* of / consistent-with the md1 path, or compare reconstructed xpub identity, not raw path equality). **Phase R0 + the implementing phase MUST exercise `bundle → verify-bundle` round-trips** and adjust the cross-check comparison if it now false-positives. (Per the `verify-bundle-round-trip-per-phase-r0-scope` discipline.) The two intentionally-tampered cross-check tests (`cross_check_mk1_depth_lt_md1_path_warns`, `cross_check_mk1_parent_fingerprint_mismatch_warns`) currently build their tampered card via `mk_codec::encode(&tampered).expect(...)` reusing a consistent xpub at a shortened path — on 0.4.0 that `encode` now rejects; rebuild those fixtures to tamper at a depth the guard permits (mutate the xpub's `depth`/`child_number` to the shortened path so the *decoded* card carries the intended inconsistency vs md1).

### 3.6 Error-mirror hygiene + WIF round-trip regression

- **`friendly.rs`** — explicit arm before `_ =>`:
  ```rust
  E::XpubOriginPathMismatch { xpub_depth, path_depth, .. } => format!(
      "mk1 xpub/origin-path depth mismatch: xpub depth {} vs origin_path depth {} (toolkit bug — the mk1 card's path must round-trip its xpub)",
      xpub_depth, path_depth,
  ),
  ```
- **`error.rs::mk_codec_exit_code`** — `mk_codec::Error::XpubOriginPathMismatch { .. } => 2,` before `_ => 1`.
- **WIF round-trip regression** (`tests/`): `bundle --slot @0.wif=- --json` → extract `mk1.Single[*]` → `inspect --mk1 …` → assert success + depth-0 card. Proves the WIF round-trip the 0.3.1 pin broke (was write-only).

### 3.7 Snapshot / fixture regeneration

The mk1 byte output changes for every multisig-watch-only / foreign-import / `3→0` flow whose card path was previously inconsistent (now consistent). This is a **correctness change to a previously-wrong card**, not a regression. All pinned mk1 chunk assertions, byte-determinism snapshots, transcripts, and convergence fixtures for the affected flows must be regenerated from the corrected output and re-verified for *semantic* round-trip (decode → correct depth/child), not just byte-pinning.

---

## §4. SemVer + lockstep

- **PATCH** (0.37.9 → 0.37.10): binary-private correctness fix; the mk1 *wire bytes* change for affected flows but **no CLI flag/subcommand/output-shape/JSON-wire change**. No GUI schema-mirror, no manual lockstep. (mk1 chunk strings are engraving output, not a clap surface.)
- Re-pin `mk-codec = "0.4.0"` (applied) + `Cargo.lock`.

---

## §5. Test plan

1. **Re-baseline:** record the exact failing-test set + per-class census (done: 74 tests / 157 instances).
2. **Helper unit tests** (`synthesize.rs` test mod): `mk1_origin_path` for each class — consistent (no-op), 3→4 (truncate, terminal==child), 4→3 (extend), 3→0 (padded, len==depth, terminal==child), depth-0 (empty). Assert `len == xpub.depth` && `last == xpub.child_number` for every case (the round-trip invariant) + that `mk_codec::encode` of a card built with it never returns `XpubOriginPathMismatch`.
3. **Per-class fix verification:** after applying the helper + the `3.4` removal + the `3.5` cross-check adjustment + the `3.5` tampered-fixture rebuilds, the full suite goes green.
4. **WIF round-trip regression** (§3.6).
5. **FULL-suite gate (the reverted-re-pin lesson — NEVER `--lib` only):** `cargo test -p mnemonic-toolkit` 0 failures; `cargo test --workspace` if siblings consume the toolkit; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` (pre-existing drift handled separately — confirm-pre-existing-via-stash, don't fix here); `cargo +stable fmt -p mnemonic-toolkit -- --check`.
6. **Semantic round-trip spot-checks:** for a representative multisig-watch-only + foreign-import bundle, `bundle → verify-bundle --self-check` (or `inspect`) confirms the mk1 card decodes to the correct xpub identity.

---

## §6. Phases

**Phase 0 — helper + call sites.** Add `mk1_origin_path` + unit tests (TDD); wire all 8 `KeyCard::new` sites; remove the `:494-503` reject loop. Run the synthesize lib tests + the helper unit tests green.

**Phase 1 — cross-check + tampered fixtures.** Adjust the `verify_bundle` mk1↔md1 cross-check to tolerate the by-design path divergence; rebuild the two tampered cross-check fixtures (§3.5). Exercise `bundle → verify-bundle` round-trips.

**Phase 2 — fixture/snapshot regen.** Re-baseline the full suite; regenerate every failing pinned-mk1 assertion/transcript/convergence fixture from corrected output; verify semantic round-trip. Iterate until `cargo test -p mnemonic-toolkit` is 0-fail.

**Phase 3 — error-mirrors + WIF regression + hygiene.** §3.6 arms + the WIF regression test; clippy/fmt gates.

**Phase 4 — version + FOLLOWUPs + ship.** 0.37.9→0.37.10; resolve `mk1-wif-bundle-depth0-invalid-card`, `mk1-depth-child-compensating-check-watch`; end-of-cycle opus R0 → GREEN; clean-tree → ff-merge `master` + push + tag `mnemonic-toolkit-v0.37.10`.

Per-phase: tests before impl where applicable; **full-suite** (not `--lib`) before any commit that claims green.

---

## §7. Risks

- **R1 — the helper's padded `3→0` path is fabricated-intermediate.** Mitigated: intermediates are non-load-bearing (the reconstructed xpub is identical); the full origin lives in md1; documented in the helper. If R0 prefers, the `3→0` watch-only-single-sig default (`bind_watch_only_singlesig:1062`) can instead default to a depth-consistent template path — evaluate at R0.
- **R2 — verify-bundle cross-check false-positives** (§3.5) — the highest-risk item; must be exercised, not assumed.
- **R3 — snapshot-regen masking a real regression.** Mitigation: regenerate only after confirming the new output *semantically* round-trips (decode → correct xpub), never blind-accept byte diffs.
- **R4 — scope creep into md1 path semantics.** This cycle fixes ONLY the mk1 card path. md1 `path_decl` (full origin) is unchanged. If a test asserts md1 path content, that's out of scope (separate concern).
