# SPEC — toolkit mk1 origin-path consistency (re-pin to mk-codec 0.4.0)

**Branch:** `toolkit-mk-codec-0.4.0-repin` (off `master` `a255060`)
**Crate:** `mnemonic-toolkit` 0.37.9 → **0.37.10** (PATCH — binary-private correctness fix; no CLI surface change)
**Dep:** re-pin `mk-codec 0.3.1` → **0.4.0** (applied).
**Recon:** `cycle-prep-recon-mk1-card-origin-path-vs-xpub-depth-consistency.md` (census verified).
**Design resolution:** opus architect (Q1 cross-check redesign / Q2 4→3 root cause / Q3 3→0 / Q4 verdict) — folded below.
**R0 history:** v1 RED 2C/3I/3M (`design/agent-reports/toolkit-mk1-origin-path-spec-R0-review.md`); this v2 folds C1/C2/I1/I2/I3 + the architect resolution + recon corrections.
**Resolves:** toolkit `mk1-wif-bundle-depth0-invalid-card`, `mk1-depth-child-compensating-check-watch`; companion `mnemonic-key mk1-no-path-depth0-support`.

---

## §1. Problem

mk-codec compact-73 **drops** `xpub.depth`/`child_number` and **reconstructs** them on decode from `origin_path` (`depth := component_count`, `child := last_component`, or `Normal{0}` empty). mk-codec **0.4.0** enforces an encode guard `Error::XpubOriginPathMismatch` rejecting any card whose xpub depth/child disagree with `origin_path` — correct by design, the exact inverse of `reconstruct_xpub`.

The toolkit pinned `mk-codec 0.3.1` (NO guard) and builds every mk1 card as `KeyCard::new(stubs, fp, **s.path/c.path**, xpub)` — pairing the **descriptor's** origin path with an xpub at a *different* depth. Re-pinning to 0.4.0 surfaces **74 failing tests / 157 instances**. These were write-side-only: on 0.3.1 the toolkit silently emitted **wrong-metadata mk1 cards** (chain_code/pubkey correct → addresses derive, but the reconstructed BIP-32 depth/child wrong; detectable only at a Wallet-Instance-ID check). The guard exposes them.

**Why the depths disagree** — foreign multisig formats export an **account-level** key with a **full-path** annotation. E.g. a Coldcard/Sparrow/Specter BIP-48 cosigner is the depth-3 account xpub `m/48'/0'/0'` (child `0'`) but the descriptor origin is the depth-4 `m/48'/0'/0'/2'` (child `2'`). md1's `path_decl` correctly keeps the full depth-4 origin (`synthesize.rs:710-717`); only the mk1 card's path was wrong.

### Census (verified, full `cargo test -p mnemonic-toolkit --no-fail-fast`)

| signature `xpub_depth→path_depth` | #tests | nature |
|---|---|---|
| **3→4** | 33 | account xpub (depth-3) + BIP-48 descriptor origin (depth-4). The dominant class. |
| **4→3** | 20 | a genuinely depth-4 leaf key re-annotated with a shallower origin (one shared fixture key `xpub6FQya…`/`xpub6DnEB…` annotated `87'/0'/0'`, `84'/0'/0'`, `86'/0'/0'`; its true origin is `48'/0'/0'/2'`) + 2 verify_bundle test-helper fixtures. |
| **3→0** | 7 | depth-3 xpub + empty path (watch-only / zpub, no declared origin). |
| **3→1, 3→2** | 2 | the two intentionally-tampered verify-bundle cross-check fixtures. |
| **4→4** | 1 | terminal-child disagreement (`c3_multisig_…_converges`). |
| OTHER | 11 | collateral (p11 export-wallet / blob-network / name-override; downstream of the mismatch). |

---

## §2. Source ground-truth (verified @ `a255060` + applied re-pin)

- **8 `mk_codec::KeyCard::new` sites in `synthesize.rs`:** `:137`, `:168`, `:221`, `:238`, `:386`, `:556`, `:773`, `:789`. **LIVE production emit:** `synthesize_descriptor` (cards `:221`/`:238`; callers `bundle.rs:1414,1649`, `import_wallet.rs:1383`, `verify_bundle.rs:867`) + `synthesize_unified` (cards `:773`/`:789`; callers `bundle.rs:377`, `verify_bundle.rs:374/463/566`). **Test-only helpers:** `synthesize_full` (`:115`, `#[allow(dead_code)]`), `synthesize_watch_only` (`:153`, `#[allow(dead_code)]`), `synthesize_multisig_full` (`:299`), `synthesize_multisig_watch_only` (`:434`) — called only from `#[cfg(test)]`. (Correction to v1 M1: only the first two carry the attr, but all four are test-only by call-graph.)
- **md1 `path_decl` is built independently** from `s.path` (`synthesize.rs:710-717`, `derivation_path_to_origin_path`). Unchanged by this fix — md1 keeps the full descriptor origin.
- **`synthesize_multisig_watch_only:494-503`** — the only depth-consistency reject (`path_depth != c.xpub.depth → CosignerSpec` error). Test-only path; superseded by the helper.
- **verify-bundle cross-checks (C1 sites):** `emit_watch_only_xpub_path_cross_check` (`verify_bundle.rs:2024`; depth check `:2117-2126`, child check `:2129-2163`, parent-fp branch `:2165-2198`) and `emit_full_path_parent_fingerprint_check` (`:2239`; guard `:2328`, parent derivation `:2372-2382`, depth print `:2341/:2343/:2399`). Both run on full-path/watch-only/multisig verify (`:389/:432/:531`). They compare the **decoded supplied mk1** xpub against **md1's** origin path INDEPENDENTLY of any synthesized expectation.
- **Tampered fixtures (C2):** `cross_check_mk1_depth_lt_md1_path_warns` (3→2) + `cross_check_mk1_parent_fingerprint_mismatch_warns` (3→1) in `tests/cli_verify_bundle_watch_only.rs` (`~:246`/`:333`) build a card via `mk_codec::encode(&tampered).expect(...)` → panics on 0.4.0's guard. Working precedent (no re-encode): `cross_check_mk1_child_number_ne_md1_last_warns:290-324`.
- **WIF slot** (`cmd/bundle.rs:655-674`): depth-0 xpub / child `Normal{0}` / empty path — already consistent; 0.4.0 accepts it. No change.
- **`inspect --mk1`** prints `"origin_path: m/{}"` from the decoded `card.origin_path` (`cmd/inspect.rs:222`) — so the mk1 path IS user-visible (drives I2).
- **mk-codec 0.4.0** guard `encode.rs`: `xpub.depth as usize != path_depth || xpub.child_number != path_child.unwrap_or(Normal{0})`; `reconstruct_xpub` uses only `len→depth`, `last→child`; intermediate components are non-load-bearing for the reconstructed `Xpub` but **are preserved on-wire** (`encode_path`/`decode_path`) → surface in `inspect`.
- **`synthesize.rs:12`** imports `{DerivationPath, Fingerprint, Xpriv, Xpub}` — **`ChildNumber` NOT in scope** (I3: add it).
- **Error-mirrors** (fallback-safe; re-pin compiles): `friendly.rs:124` (`friendly_mk_codec`, `_ =>`), `error.rs:393` (`mk_codec_exit_code`, `_ => 1`).

---

## §3. Design

### 3.1 The unifying invariant

After the fix, the mk1 card and md1 are **no longer "equal paths."** The relationship is:

> **The mk1 card's xpub is the BIP-32 node at the `xpub.depth`-length prefix of md1's full origin path, and that prefix's terminal component equals `xpub.child_number`.**

The mk1 `origin_path` exists only to round-trip the xpub's depth/child; the authoritative full origin lives in md1's `path_decl`. Two consequences: (a) the emit side derives the mk1 path from the xpub (§3.2-3.4); (b) the verify side compares the decoded mk1 xpub against md1's depth-`d` *prefix*, not the full path (§3.5).

### 3.2 The helper (single source of truth)

Add to `synthesize.rs` (and add `ChildNumber` to the `bitcoin::bip32` import, I3):

```rust
/// Derive the mk1 card's `origin_path` so it round-trips the xpub it carries.
///
/// mk-codec compact-73 reconstructs `depth := component_count(origin_path)` and
/// `child_number := last_component(origin_path)` (or `Normal{0}` empty); mk-codec
/// 0.4.0 rejects any card whose xpub depth/child disagree. The DESCRIPTOR origin
/// (carried independently by md1's path_decl) may be deeper (account xpub + BIP-48
/// leaf path), shallower (a leaf xpub re-annotated with an account origin), or
/// absent. We build a path of length `xpub.depth` whose terminal is
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
        // Reuse the descriptor path where available; pad absent intermediates with
        // Normal{0} (honest filler — reads as obviously-synthetic in `inspect`).
        out.push(comps.get(i).copied().unwrap_or(ChildNumber::Normal { index: 0 }));
    }
    out.push(xpub.child_number); // terminal MUST equal the xpub's child (round-trip)
    DerivationPath::from(out)
}
```

**Behavior (one formula, all classes — verified `len==depth && last==child` each):**

| class | descriptor path | output | op |
|---|---|---|---|
| consistent 3→3 | `m/84'/0'/0'` | `m/84'/0'/0'` | no-op |
| 3→4 | `m/48'/0'/0'/2'` | `m/48'/0'/0'` | truncate (drop leaf) |
| 4→3 | `m/87'/0'/0'` (xpub child 2') | `m/87'/0'/0'/2'` | extend (append child) |
| 3→0 | `m` (empty) | `m/0/0/<child>` | pad (Normal{0} filler) |
| depth-0 (WIF) | `m` | `m` | empty |
| 4→4 child-mismatch | … (xpub child authoritative) | `…/<xpub child>` | terminal override |

**Append-correctness (Q2):** for 4→3 the helper appends `xpub.child_number` to the descriptor's depth-(d-1) prefix → always `len==depth, last==child`, always encodes. The appended intermediate may be the descriptor's (possibly fictional) prefix, but it is non-load-bearing — the reconstructed `Xpub` is byte-identical. **Provably no wrong-intermediate that breaks round-trip.**

### 3.3 Apply at the live emit sites (+ test sites for hygiene)

Replace the `path` argument at every `KeyCard::new` with `mk1_origin_path(&<xpub>, &<path>)`:
- **Live:** `synthesize_descriptor:221,238`; `synthesize_unified:773,789`.
- **Test-only (no-op for consistent fixtures; hygiene):** `synthesize_full:137`, `synthesize_watch_only:168`, `synthesize_multisig_full:386`, `synthesize_multisig_watch_only:556`.

md1 `path_decl` (`:710-717`) is **unchanged**.

### 3.4 Remove the `synthesize_multisig_watch_only:494-503` reject loop

The helper makes the card consistent-by-construction; the hard reject (`path_depth != xpub.depth`) now blocks a case the helper handles. Remove it (test-only path; the helper supersedes). Resolves `mk1-depth-child-compensating-check-watch`.

### 3.5 verify-bundle cross-check redesign (C1 — the load-bearing piece)

Both cross-checks key off `md_depth` today; after the fix the mk1 xpub is at depth `d ≤ md_depth` (or, for re-annotated leaves, `d` may exceed a shallow md1). **Rekey both off the xpub's own depth `d = card.xpub.depth`, comparing against md1's depth-`d` prefix.**

**3.5a — `emit_watch_only_xpub_path_cross_check` (`:2117-2198`)** — replace Checks 1/2/3 with:

```rust
let d        = card.xpub.depth as usize;
let md_depth = md_path.components.len();

// Check A (replaces old depth-equality Check 1): the card must not claim a node
// DEEPER than md1's declared origin. d <= md_depth is fine (3→4 truncation);
// d > md_depth means md1 cannot be this key's full origin.
if d > md_depth {
    warn!("cosigner[{i}] mk1 xpub depth ({d}) is deeper than md1 origin-path \
           length ({md_depth}); the card claims a node below the declared origin \
           — cards are internally inconsistent");
}

// Check B (replaces old terminal Check 2): the xpub's child_number must equal
// md1's component at index d-1 (md1 TRUNCATED to the xpub's depth, terminal).
// For 3→4 this compares against the ACCOUNT-level component (md_path[2]=0'),
// NOT md_path.last() (=2') — exactly the change that dissolves the false-positive.
if d >= 1 && d <= md_depth {
    let md_at_prefix = md_path.components[d - 1];
    let (xi, xh) = split_child(card.xpub.child_number);
    if xi != md_at_prefix.value || xh != md_at_prefix.hardened {
        warn!("cosigner[{i}] mk1 xpub child_number ({xi}{h}) does not match md1 \
               origin-path component #{d} (…); cards are internally inconsistent");
    }
}

// Check C (replaces old parent-fp Check 3, keyed off d not md_depth):
// d == 0 → master xpub MUST have all-zero parent_fingerprint (existing branch).
// d == 1 → existing pfp-vs-claimed-master logic, gated on d==1.
// d >= 2 → skip (parent needs the seed; watch-only ceiling).
```

**3.5b — `emit_full_path_parent_fingerprint_check` (`:2328`, `:2372-2382`, `:2341/2343/2399`)** — derive the parent at md1 truncated to **`d-1`** (the xpub's parent level), not `md_depth-1`:

```rust
let d = card.xpub.depth as usize;
if d < 2 { /* depth 0/1 handled by the structural branch */ continue; }
if d > full_components.len() { continue; } // Check A above warns this case
let parent_path  = DerivationPath::from(full_components[..d - 1].to_vec());
let parent_xpriv = master.derive_priv(&secp, &parent_path)?;
let parent_xpub  = Xpub::from_priv(&secp, &parent_xpriv);
// compare parent_xpub.fingerprint() vs card.xpub.parent_fingerprint (unchanged)
```
Also gate `:2328` on `d < 2` (not `md_depth < 2`) and report `d` (not `md_depth`) at the depth prints.

**Correctness (verified by the architect):** (a) correct **3→4** — Check A `3>4`? no; Check B `md_path[2]=0'` vs child `0'` equal; Check C `d=3≥2` skip; parent at `full[..2]=m/48'/0'` = correct parent → **PASS, no warning**. (b) correct **4→4** leaf — parent at `full[..3]` = correct → **PASS**. (c) **genuine tampering** — child≠prefix-component, or `d>md_depth`, or wrong parent-fp → **FIRES**. Discriminating power moves from raw-depth-equality (can't tell correct-truncation from tampering) to prefix-terminal consistency (can).

### 3.6 C2 — rebuild the 2 tampered fixtures

On 0.4.0 no public encoder bypasses the guard, so `mk_codec::encode(&tampered).expect(...)` can't construct an inconsistent card. Rebuild both via the **two-internally-consistent-cards-that-disagree** pattern (precedent `cross_check_mk1_child_number_ne_md1_last_warns:290-324`): build a consistent depth-2 mk1 card and pair it with a depth-3 md1 (so the new Check A `d=2 ≤ md_depth=3`, Check B `md_path[1]` ≠ the depth-2 card's child) → fires. Confirm a real depth-2-vs-depth-3 prefix-violating pairing is constructible.

### 3.7 I2 — the 3→0 path: helper-pad (decided)

For a depth-3 watch-only xpub with no declared origin, the helper pads → `m/0/0/<child>`, visible in `inspect --mk1` as e.g. `m/0/0/0'`. **Decision: helper-pad, NOT a bind-site default** — defaulting `s.path` to a template path would ripple into md1's `path_decl` → descriptor string → `compute_wallet_policy_id` → every md1 byte (R4 out-of-scope, wide blast radius). Helper-pad keeps the change mk1-local; the reconstructed `Xpub` is byte-identical. **Pin `inspect --mk1 origin_path` output for a 3→0 case in a new test** so the synthetic value is a documented contract. (Citation correction: the prior SPEC blamed `bind_watch_only_singlesig:1062`, which is legacy/test-only; the live 3→0 empty path originates in the descriptor-mode cosigner resolution / slot resolution — not load-bearing for the fix, which normalizes at the emit site downstream of all sources.)

### 3.8 Error-mirror hygiene + WIF round-trip regression

- **`friendly.rs`** explicit arm before `_ =>`: `E::XpubOriginPathMismatch { xpub_depth, path_depth, .. } => format!("mk1 xpub/origin-path depth mismatch: xpub depth {} vs origin_path depth {} (toolkit bug — the mk1 card's path must round-trip its xpub)", xpub_depth, path_depth)`.
- **`error.rs::mk_codec_exit_code`** — `mk_codec::Error::XpubOriginPathMismatch { .. } => 2,` before `_ => 1`.
- **WIF round-trip regression** (`tests/`): `bundle --slot @0.wif=- --json` → extract `mk1.Single[*]` → `inspect --mk1 …` → assert success + depth-0 card (the round-trip the 0.3.1 pin broke).

---

## §4. SemVer + lockstep

**PATCH** 0.37.9 → 0.37.10: binary-private correctness fix. mk1 chunk *bytes* change for ~40+ flows (correcting previously-wrong cards) but **no clap flag/subcommand/JSON-wire/output-shape change → no GUI schema-mirror, no manual lockstep**. (Spot-check `bundle --json`/envelope shape does not expose the mk1 origin_path string as a gated wire field.) Re-pin `mk-codec = "0.4.0"` + `Cargo.lock`.

---

## §5. Test plan

1. **Helper unit tests** (`synthesize.rs` test mod): `mk1_origin_path` for all 6 classes — assert `out.len() == xpub.depth` && `out.last() == xpub.child_number`, and `mk_codec::encode_with_chunk_set_id` of a card built with it never returns `XpubOriginPathMismatch`.
2. **Cross-check no-false-positive** (NEW integration test): a correct 3→4 multisig watch-only bundle AND a correct 4→4 bundle → `verify-bundle` emits NO `"internally inconsistent"` stderr.
3. **C2** rebuilt tampered fixtures still fire the new Check A/B.
4. **3→0** `inspect --mk1 origin_path` pinned.
5. **WIF round-trip regression** (§3.8).
6. **FULL-suite gate (NEVER `--lib` only — the reverted-re-pin lesson):** `cargo test -p mnemonic-toolkit` 0 failures; `cargo clippy -p mnemonic-toolkit --all-targets -- -D warnings` (pre-existing drift confirmed-pre-existing-via-stash + deferred, not fixed here); `cargo +stable fmt -p mnemonic-toolkit -- --check`.
7. **Snapshot regen** with **semantic round-trip** verification — decode each regenerated mk1 → assert correct depth/child/chain_code/pubkey (never blind byte-accept).

---

## §6. Phases (architect's 5-phase plan)

- **Phase 0 — helper + sites + reject-removal.** `mk1_origin_path` + `ChildNumber` import + 4 live sites (+ 4 test sites, no-op) + remove `synthesize_multisig_watch_only:494-503` + helper unit tests (§5.1).
- **Phase 1 — cross-check redesign (load-bearing, §3.5).** Rewrite both `verify_bundle.rs` cross-checks (Checks A/B/C keyed off `d`; parent-fp at `full[..d-1]`) + the no-false-positive integration test (§5.2). Highest review scrutiny.
- **Phase 2 — C2 tampered-fixture rebuild + 3→0 inspect pin** (§3.6, §3.7).
- **Phase 3 — snapshot/transcript/convergence regen (~40 assertions) with semantic round-trip verification** (§5.7); fix the 2 `helper_multisig_*` test-helper fixtures at source (derive their xpub at the template path). Full-suite gate.
- **Phase 4 — error-mirror arms (§3.8) + WIF regression + version 0.37.10 + FOLLOWUPs + end-of-cycle opus R0 → GREEN + clean-tree → ff-merge `master` + push + tag `mnemonic-toolkit-v0.37.10`.**

Per-phase: tests before impl where applicable; **full-suite** (not `--lib`) before any commit claiming green.

---

## §7. Risks

- **R1 — cross-check redesign is the riskiest piece** (§3.5). Mitigation: the no-false-positive integration test (§5.2) + the architect's verified case analysis (3→4 pass, 4→4 pass, tampering fires). Phase 1 gets highest review scrutiny.
- **R2 — snapshot-regen masking a real regression.** Mitigation: §5.7 semantic round-trip (decode → correct xpub), never blind byte-accept.
- **R3 — the 3→0 fabricated `m/0/0/<child>` is user-visible** (`inspect`). Accepted + pinned (§3.7); Normal{0} filler reads as obviously synthetic.
- **R4 — md1 path semantics OUT OF SCOPE.** This cycle changes only the mk1 card path; md1 keeps the full origin. A 4→3 case where md1 *should* carry a depth-4 origin (under-annotated source) is a separate FOLLOWUP, not this cycle.
