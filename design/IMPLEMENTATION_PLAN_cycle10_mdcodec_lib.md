# IMPLEMENTATION PLAN — cycle-10: md-codec LIBRARY cluster (M3 + L14/L15/L17 + L6)

**Status:** DESIGN ONLY (no code). This plan-doc feeds its OWN mandatory opus-architect **R0 loop → 0 Critical / 0 Important** before ANY implementation begins (CLAUDE.md Conventions, first bullet). No code, no implementer dispatch, no SemVer bump, no publish/tag while any Critical or Important finding is open.
**Date:** 2026-06-21.
**Spec (R0-GREEN, 0C/0I):** `design/BRAINSTORM_cycle10_mdcodec_lib.md`.
**Spec R0 review:** `design/agent-reports/cycle10-spec-r0-round1-review.md` (Round 1, GREEN; its two Minor notes M-1/M-2 are folded into §1 and §3 of this plan).
**Target repo:** `descriptor-mnemonic` (md-codec + md-cli). NOT this toolkit repo (only a transitive PATCH pin-bump touches the toolkit; §6).

---

## 1. Source-of-truth SHA and the branch-base GOTCHA (folds R0 Minor M-1)

**Repo:** `descriptor-mnemonic`. **Source-of-truth = `origin/main = 1a4b322618e3831fdbb2578bc6f98c7a23bc58e3`** (`release: md-cli 0.9.0 — cycle-9`). All line numbers in this plan were **re-grepped against `1a4b322` at write time** (2026-06-21); see the citation table in §2.

**⚠️ BRANCH-BASE GOTCHA (R0 Minor M-1 — load-bearing for the pin-bump base version):**
- The descriptor-mnemonic working-tree HEAD is `836faf87`, which is **one commit BEHIND `origin/main`** and shows **md-cli `0.8.1`** in `crates/md-cli/Cargo.toml:3`.
- The md-codec **source** is byte-identical between `836faf87` and `1a4b322` (the only delta is in `crates/md-cli/`), so cited md-codec line numbers are authoritative either way — BUT the **md-cli base version differs**.
- **The implementer MUST create the worktree off `origin/main` (`1a4b322`), NOT off the working-tree HEAD.** Branching off HEAD would set the md-cli pin-bump base to `0.8.1` instead of the correct `0.9.0`, corrupting the publish-version chain (§6 expects `md-cli 0.9.0 → 0.9.1`).
- Concretely:
  ```sh
  cd /scratch/code/shibboleth/descriptor-mnemonic
  git fetch origin
  git worktree add -b feature/cycle10-mdcodec-lib <worktree-path> 1a4b322
  # VERIFY before any edit:
  git -C <worktree-path> show HEAD:crates/md-codec/Cargo.toml | grep '^version'   # MUST be 0.38.0
  git -C <worktree-path> show HEAD:crates/md-cli/Cargo.toml   | grep '^version'   # MUST be 0.9.0
  ```
  If either assertion fails, STOP — the base is wrong.

---

## 2. Citation table — re-grepped against `1a4b322` (live line numbers)

| Finding | File @ `1a4b322` | Live lines | Anchor |
|---|---|---|---|
| **M3** gate | `crates/md-codec/src/derive.rs` | `92` (`pub fn derive_address`), **`110`** (`if let Some(alts) = &self.use_site_path.multipath`), **`117`** (`} else if chain != 0 {`), `118-121` (`Err(ChainIndexOutOfRange { chain, alt_count: 0 })`), `124` (`to_miniscript_descriptor(self, chain)?`) | gate reads ONLY `self.use_site_path.multipath`; never `self.tlv.use_site_path_overrides` |
| M3 override field | `crates/md-codec/src/tlv.rs` | `26` | `pub use_site_path_overrides: Option<Vec<(u8, UseSitePath)>>` |
| M3 per-key authority | `crates/md-codec/src/to_miniscript.rs` | `54` (`pub fn to_miniscript_descriptor`), `58` (`let expanded = expand_per_at_n(d)?;`), `277` (`fn use_site_to_derivation_path`), `282` (`.ok_or(Error::ChainIndexOutOfRange …)`) | per-key path resolves `chain` against EACH key's own (override-aware) multipath; still fail-closes |
| M3 regression test | `crates/md-codec/src/derive.rs` | `241` (`fn derive_address_chain_out_of_range`), `263-264` (`chain: 5, alt_count: 2`) | no overrides → `max_alts == baseline == 2` → unchanged |
| **L14** fix locus | `crates/md-codec/src/identity.rs` | **`193`** (`e.origin_path.write(&mut path_scratch)?;`) inside the per-record loop (`189` `for e in &expanded`) | empty/elided origin hashed verbatim |
| L14 fill source | `crates/md-codec/src/canonical_origin.rs` | `45` (`pub fn canonical_origin(tree: &Node) -> Option<OriginPath>`) | wrapper→canonical table |
| L14 policy-id fn | `crates/md-codec/src/identity.rs` | `172` (`pub fn compute_wallet_policy_id`), `176` (`canonicalize_placeholder_indices(&mut d_canonical)?`) | policy-id already canonicalizes placeholder order |
| **L15** WDT-id fn | `crates/md-codec/src/identity.rs` | `71` (`pub fn compute_wallet_descriptor_template_id`), `74` (`let mut w = BitWriter::new();` — fn-body top), `83` (`sub.write_bits(u64::from(*idx), …)`) | NO `canonicalize_placeholder_indices` call; writes raw `*idx` |
| L15 import (already present) | `crates/md-codec/src/identity.rs` | `4` (`use crate::canonicalize::{canonicalize_placeholder_indices, expand_per_at_n};`) | `canonicalize_placeholder_indices` in scope |
| **L17** vacuous test | `crates/md-codec/src/identity.rs` | `572` (`fn walletpolicyid_stable_across_origin_elision`), body `573-591` (builds `cell_7_wpkh_descriptor` + `origin_path_overrides` byte-identical to the `Shared(BIP84)` baseline) | never constructs an empty `path_decl` |
| L17 sibling (keep) | `crates/md-codec/src/identity.rs` | `593` (`fn walletpolicyid_stable_across_use_site_elision`) | non-vacuous (real `None`-baseline + `Some` override) — leave as-is |
| L17 fixture | `crates/md-codec/src/identity.rs` | `385` (`fn cell_7_wpkh_descriptor`) — `n:1`, `path_decl Shared(84'/0'/0')`, `tlv.pubkeys = Some([(0, xpub)])`, `tlv.fingerprints = Some([(0, …)])` | the explicit operand; `@0` pubkey present so the id resolves |
| **L6** panic site | `crates/md-codec/src/canonicalize.rs` | `168` (`pub fn canonicalize_placeholder_indices`), `169` (`let n = d.n as usize;`), `200` (identity fast-path `return Ok(());`), **`206`** (`if let PathDeclPaths::Divergent(paths) = &mut d.path_decl.paths {`), `216` (`new_paths.push(old_paths[inverse[new_idx] as usize].clone());` — OOB site) | NO `paths.len()==n` guard |
| L6 sibling guard (mirror) | `crates/md-codec/src/canonicalize.rs` | `425` (`if let PathDeclPaths::Divergent(paths) = &d.path_decl.paths {`), `426-432` (`if paths.len() != d.n as usize { return Err(Error::DivergentPathCountMismatch { n, got }) }`) | the exact guard to mirror |
| L6 error variant (exists) | `crates/md-codec/src/error.rs` | `66-72` | `DivergentPathCountMismatch { n: u8, got: usize }` — NO new variant |
| `PathDecl` / `PathDeclPaths` shape | `crates/md-codec/src/origin_path.rs` | `82` (`pub struct PathDecl { pub n: u8, pub paths: PathDeclPaths }`), `91` (`pub enum PathDeclPaths { Shared(OriginPath), Divergent(Vec<OriginPath>) }`), `47-49` (`OriginPath { pub components: Vec<PathComponent> }`) | for the L17 elided-fixture construction |
| wire does NOT embed id (SemVer determinant) | `crates/md-codec/src/encode.rs` | grep `policy_id\|template_id\|compute_wallet` → ∅ | `encode_md1_string` emits no id → MINOR not MAJOR |
| CI fmt gate | `.github/workflows/ci.yml` | `49` (`fmt:` job), `59` (`- run: cargo fmt --all --check`) | md-codec/md-cli ARE fmt-gated (folds M-2) |
| version sites | `crates/md-codec/Cargo.toml:3` (`0.38.0`); `crates/md-cli/Cargo.toml:3` (`0.9.0`), `:28` (`md-codec = { path="../md-codec", version="=0.38.0" }`); root `Cargo.lock:500` (`md-codec 0.38.0`); `fuzz/Cargo.lock:169` (**stale `0.35.1`** — regen); `fuzz/Cargo.toml:22-23` (path-only, no literal); `CHANGELOG.md` (single top-level file, entries prefixed `## md-codec [x]` / `## md-cli [x]`; current top `## md-cli [0.9.0]`) | — |
| toolkit pin (transitive, §6) | `crates/mnemonic-toolkit/Cargo.toml:36` (`md-codec = "0.38"`); `Cargo.lock:677` (`md-codec 0.38.0`) | toolkit PATCH pin-bump only (toolkit already consumes 0.38.0 from cycle-4) |
| bughunt report tick targets | `design/agent-reports/constellation-bughunt-2026-06-20.md` (toolkit master) | M3 `### - [ ]` @ `245`; L6 @ `368`; L14 @ `593`; L15 @ `603`; L17 @ `621` (re-grepped live; round-1 plan-R0 citation-decay fix) | flip `- [ ]` → `- [x]` in the toolkit ship commit |

---

## 3. Execution model (binding for the implementer)

1. **Single implementer subagent**, single worktree off `origin/main = 1a4b322` (§1). NOT parallel re-implementations (CLAUDE.md per-phase pattern step 3).
2. **Strict TDD, RED before GREEN.** Each phase: write the RED test(s) FIRST, confirm they fail/panic for the documented reason, THEN implement, THEN confirm GREEN.
3. **Per-phase FULL-suite gate (NOT targeted `--test` targets — per MEMORY `feedback_r0_review_run_full_package_suite`):**
   ```sh
   cargo test -p md-codec           # full package suite, every phase
   cargo test -p md-cli             # full package suite, every phase (no code change, but must stay GREEN)
   cargo clippy --workspace --all-targets -- -D warnings
   ```
4. **`cargo fmt` BEFORE the green-suite/tag gate (folds R0 Minor M-2).** This repo's CI enforces `cargo fmt --all --check` (`ci.yml:59`). **Contrast with the toolkit, which is fmt-EXEMPT (mlock g6) — md-codec/md-cli are NOT.** Run, at minimum once before P4 ship and ideally at each phase close:
   ```sh
   cargo fmt --all
   cargo fmt --all --check        # must exit 0 (matches CI)
   ```
   Do NOT use `cargo fmt --all` carelessly across unrelated files — but here the whole repo IS the fmt domain, so `--all` is correct.
5. **No CLI-surface delta** ⇒ no `schema_mirror`, no manual-mirror (library-internal; spec D10). The implementer touches NO clap derive, NO `docs/manual/`.
6. **Mandatory post-implementation whole-diff adversarial review** over the entire P1-P4 diff (CLAUDE.md per-phase pattern step 4; persisted to `design/agent-reports/`) BEFORE publish/tag. See §5.
7. All agent review outputs persist verbatim to `design/agent-reports/cycle10-phase-N-<round>-review.md` BEFORE the fold-and-commit step (CLAUDE.md persist-before-fold convention).
8. **Stage paths explicitly** (no `git add -A`).

---

## 4. Phased plan

### Phase P1 — M3 chain-gate widening (`derive.rs`) + funds-availability tests

**RED first** (add to `crates/md-codec/src/derive.rs` `#[cfg(test)]`):
- `derive_address_override_change_chain_derivable` — build a `Descriptor` with `use_site_path.multipath = None` (baseline → alt-count modeled as 1) + `tlv.use_site_path_overrides = Some(vec![(0u8, UseSitePath { multipath: Some(<0;1>), wildcard_hardened: false })])` and a `tlv.pubkeys` entry for `@0`. Assert today: `derive_address(chain=1, index=0, network)` → `Err(ChainIndexOutOfRange { chain: 1, alt_count: 0 })`. Post-fix: derives a valid change address (chain 1). Also assert `chain=0` derives (receive control). Model the legal D5(b) shape from `validate.rs` (the `None`-baseline + `Some`-override mix is legal in both directions).
- `derive_address_override_chain_over_max_still_rejects` (positive control / no over-widening) — same wallet, `derive_address(chain=2)` (beyond the override's 2-alt max) → still `Err(ChainIndexOutOfRange { chain: 2, alt_count: 2 })`.

**Implement** — replace the gate at `derive.rs:110-122` with the MAX-alt-count form (spec §3.1):
```rust
// Pre-flight: chain index in range. Bound by the MAX alt-count across the
// baseline use-site path AND every per-`@N` override (None → alt-count 1,
// i.e. only chain 0). The per-key path (use_site_to_derivation_path) is the
// real authority and STILL fail-closes per key; this coarse gate must not be
// narrower than the widest key, or a valid override change chain is rejected.
let baseline_alts = self.use_site_path.multipath.as_ref().map(|a| a.len()).unwrap_or(1);
let max_alts = self
    .tlv
    .use_site_path_overrides
    .iter()
    .flatten()
    .map(|(_, p)| p.multipath.as_ref().map(|a| a.len()).unwrap_or(1))
    .fold(baseline_alts, std::cmp::max);
if (chain as usize) >= max_alts {
    return Err(Error::ChainIndexOutOfRange { chain, alt_count: max_alts });
}
```
- Do NOT touch `to_miniscript.rs` / `expand_per_at_n` (spec D2; R0-confirmed the per-key path already composes override-over-baseline and re-checks `alts.get(chain)` → fail-closed). `chain as usize` matches the existing comparison form.
- **`chain` is `u32`** in `derive_address`; `(chain as usize)` is the existing pattern — keep it.

**Fail-closed argument (carry into the per-phase review):** `max_alts` is the widest per-key chain count. A request in `[0, max_alts)` passes the coarse gate; a key that genuinely supports the chain derives, a key that lacks it STILL errors `ChainIndexOutOfRange` in `use_site_to_derivation_path` (`to_miniscript.rs:282`). Widening can NEVER produce a wrong subtree — only "derives correctly" or "still errors". `chain ≥ max_alts` still rejected.

**Regression guard (must stay GREEN unchanged):** `derive_address_chain_out_of_range` (`derive.rs:241-267`) — baseline alt-count 2, no overrides, `chain=5` → `alt_count: 2`, because `max_alts == baseline_alts == 2`. Cite as the unchanged-behavior anchor.

**P1 gate:** RED tests fail for the documented reason → implement → `cargo test -p md-codec` (full), `cargo test -p md-cli` (full), `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check` all GREEN. Persist `design/agent-reports/cycle10-phase-1-r0-review.md` (per-phase review) before commit.

---

### Phase P2 — identity-stability sub-fix (L14 + L15 + L17, one atomic `identity.rs` commit)

**RED first** (`crates/md-codec/src/identity.rs` `#[cfg(test)]`):
- **L17 de-vacuify** — REPLACE `walletpolicyid_stable_across_origin_elision` (`:572-591`). New body asserts an ELIDED empty `path_decl` `wpkh(@0)` hashes identically to the explicit `m/84'/0'/0'` form:
  ```rust
  #[test]
  fn walletpolicyid_stable_across_origin_elision() {
      let d_explicit = cell_7_wpkh_descriptor();           // explicit Shared(BIP84)
      let mut d_elided = cell_7_wpkh_descriptor();
      d_elided.path_decl = PathDecl {                       // genuinely ELIDED: empty Shared origin
          n: 1,
          paths: PathDeclPaths::Shared(OriginPath { components: vec![] }),
      };
      let id_explicit = compute_wallet_policy_id(&d_explicit).unwrap();
      let id_elided = compute_wallet_policy_id(&d_elided).unwrap();
      assert_eq!(id_explicit, id_elided);   // RED today; GREEN after the L14 fill
  }
  ```
  (`PathDecl`/`PathDeclPaths`/`OriginPath` shapes verified at `origin_path.rs:82/91/47`. `cell_7_wpkh_descriptor` carries `tlv.pubkeys[@0]` so the id resolves.) Confirm this is RED today (empty path hashes a `0000` length prefix + no components, differing from the explicit component bits).
- **L15 NEW** — `wdt_id_invariant_to_placeholder_ordering`: build a 2-of-2 `wsh(multi(2,@0,@1))` vs `wsh(multi(2,@1,@0))` (same keys, swapped placeholder indices); `compute_wallet_descriptor_template_id` → differs today → equal after the L15 canonicalize. Model on the `pkk(index)` helper pattern in the existing identity tests (`identity.rs:627`/`:793`). Confirm RED today.

**Implement:**
- **L14** — REPLACE the existing two lines `identity.rs:192-193` (the `let mut path_scratch = BitWriter::new();` declaration at `:192` AND the `e.origin_path.write(&mut path_scratch)?;` call at `:193`) with the block below — do NOT insert before `:192`, or you get a duplicate `path_scratch` binding (plan-R0 Minor-3). Canonical-fill an empty resolved origin (spec §3.2):
  ```rust
  // L14: canonical-fill an elided (empty) origin so the policy-id honors its
  // documented "stable across origin-elision" invariant. An empty resolved
  // origin with a canonical wrapper hashes identically to the explicit form.
  // expand_per_at_n already returns explicit paths verbatim, so only the
  // empty case needs the fill; when canonical_origin is None the empty path
  // is structurally precluded upstream (MissingExplicitOrigin), so the
  // unwrap_or_else fallback is unreachable-but-safe.
  let origin_for_hash: OriginPath = if e.origin_path.components.is_empty() {
      crate::canonical_origin::canonical_origin(&d.tree)
          .unwrap_or_else(|| e.origin_path.clone())
  } else {
      e.origin_path.clone()
  };
  let mut path_scratch = BitWriter::new();
  origin_for_hash.write(&mut path_scratch)?;
  ```
  (`canonical_origin` is fully-qualified `crate::canonical_origin::canonical_origin` — NOT imported in `identity.rs` today, so no `use` edit needed. `OriginPath` IS in scope — already used by the fixtures.)
- **L15** — at the top of the `compute_wallet_descriptor_template_id` fn body (`identity.rs:74`, before `let mut w = BitWriter::new();`), canonicalize a clone first (spec §3.2):
  ```rust
  // L15: canonicalize placeholder ordering on a clone first (mirror
  // compute_wallet_policy_id) so the WDT-id is invariant to placeholder
  // index permutation. The identity fast-path leaves already-canonical
  // inputs (the toolkit's @0,@1,… ordering) byte-identical.
  let mut d_canonical = d.clone();
  canonicalize_placeholder_indices(&mut d_canonical)?;
  let d = &d_canonical;
  ```
  (`canonicalize_placeholder_indices` already imported at `identity.rs:4`; returns `Result<…, Error>` so the `?` rides the existing `-> Result<…, Error>` signature — no new error surface. `key_index_width()`/`d.key_index_width()` is a pure fn of `d.n`, preserved by canonicalization → no bitstream desync, R0-confirmed.)

**Atomicity:** L14 + L15 + L17 land as ONE commit — they share the canonicalization-invariant theme; the de-vacuified L17 test is the RED→GREEN gate for L14.

**Regressions that must stay GREEN unchanged:**
- `compute_wallet_policy_id_canonicalizes_first` (`identity.rs:791`) — policy-id canonicalization unchanged.
- `golden_vector_wpkh_cell_7` (the explicit-origin `cell_7` golden, ~`identity.rs:469`) — explicit-origin policy-id is byte-identical (the L14 fill is empty-only).
- `walletpolicyid_stable_across_use_site_elision` (`:593`) — KEEP as-is (non-vacuous, different axis).
- Canonical-ordering WDT-id is byte-identical pre/post L15 (identity fast-path `canonicalize.rs:200`). If a golden WDT-id vector exists, it must stay GREEN; if not, add a one-line note in the test asserting "canonical input ⇒ unchanged via the identity fast-path."

**P2 gate:** same as P1 (full `-p md-codec` + `-p md-cli` suites, clippy `-D warnings`, fmt-check). Persist `cycle10-phase-2-r0-review.md`.

---

### Phase P3 — L6 Divergent length guard (`canonicalize.rs`)

**RED first** (`crates/md-codec/src/canonicalize.rs` `#[cfg(test)]`):
- `canonicalize_short_divergent_returns_typed_error` — hand-build a `Descriptor` with `n=2`, a NON-canonical tree (e.g. a `multi`/`wsh` body referencing `@1` before `@0` so the permutation is non-identity, bypassing the `:200` fast-path), and `path_decl.paths = Divergent(vec![one_path])` (length 1 ≠ n=2). Call `canonicalize_placeholder_indices(&mut d)` → **panics (OOB at `:216`) today** → `Err(Error::DivergentPathCountMismatch { n: 2, got: 1 })` after the guard. (Today the panic IS the RED state; confirm it panics, then the test asserts the typed error post-fix. Model the non-canonical 2-key shape on the existing Divergent fixtures at `canonicalize.rs:654`/`:1204`.)
- `canonicalize_identity_short_divergent_not_reached` (regression / scope-bound) — a CANONICAL-ordering descriptor with a short Divergent vector returns `Ok(())` via the identity fast-path (`:200`) WITHOUT hitting the guard, OR document why the guard fires only on non-identity. (Confirms the guard does not over-reject the fast-path.)

**Implement** — at `canonicalize.rs:206`, inside the non-identity `Divergent` branch, mirror the `expand_per_at_n:426-432` guard (spec §3.3):
```rust
if let PathDeclPaths::Divergent(paths) = &mut d.path_decl.paths {
    // L6: a hand-built Descriptor can carry a short Divergent vector; guard
    // before indexing old_paths[inverse[new_idx]] to surface a typed error
    // instead of an out-of-bounds panic (mirror expand_per_at_n:426-432).
    if paths.len() != n {
        return Err(Error::DivergentPathCountMismatch {
            n: d.n,
            got: paths.len(),
        });
    }
    // …existing reorder loop unchanged…
```
- **Borrow note (spec §3.3):** `n` is the local `usize` already bound at `canonicalize.rs:169` (`let n = d.n as usize;`) — compare `paths.len() != n` against THAT local (no re-borrow of `d`). The error payload field `n: u8` wants the original `d.n` — but `d.path_decl.paths` is mutably borrowed by `paths` here. **Bind `let n_keys = d.n;` immediately before the `if let PathDeclPaths::Divergent(paths) = &mut d.path_decl.paths` line, then use `n: n_keys` in the error construction** so the error build does not re-borrow `d` while `paths` is live. (Equivalent: `n: n as u8`, but the explicit `n_keys = d.n` read avoids the cast-back and is the clearer choice.) The implementer resolves the exact binding to satisfy the borrow checker; design intent = "typed error not panic, mirroring `expand_per_at_n`."
- No new error variant (`DivergentPathCountMismatch { n: u8, got: usize }` exists at `error.rs:66-72`); no public-surface growth.

**P3 gate:** same as P1/P2. Persist `cycle10-phase-3-r0-review.md`.

---

### Phase P4 — publish→pin chain ship (md-codec MINOR → md-cli re-release)

**This is the SemVer + crates.io publish chain. STRICT ORDER (spec D11): md-codec FIRST, then md-cli.**

**P4.0 — pre-ship gates (whole-tree):**
```sh
cargo fmt --all && cargo fmt --all --check   # CI parity (ci.yml:59) — MUST exit 0
cargo test -p md-codec
cargo test -p md-cli
cargo clippy --workspace --all-targets -- -D warnings
```

**P4.1 — md-codec MINOR `0.38.0 → 0.39.0`:**
- `crates/md-codec/Cargo.toml:3` `version = "0.38.0"` → `"0.39.0"`.
- `CHANGELOG.md` — prepend a `## md-codec [0.39.0] — 2026-06-21` entry: M3 chain-gate widening (funds-availability — change addresses now derivable for `None`-baseline + override wallets); L14 policy-id origin-elision canonical-fill; L15 WDT-id placeholder-ordering canonicalization (**LOUD release note: id-VALUE change for NON-canonical / elided inputs — in-memory only, NOT on the wire, no card/persisted-format change**); L6 typed `DivergentPathCountMismatch` guard replacing an OOB panic.
- **`fuzz/Cargo.lock`** — currently STALE at `0.35.1` (`:169`); regen so it resolves md-codec `0.39.0` (`cargo update -p md-codec` within the fuzz workspace, or `cargo build` in `fuzz/` to self-heal the lock). `fuzz/Cargo.toml` is **path-only** (`:22-23`) → no version literal to edit there.
- Root **`Cargo.lock:500`** (`md-codec 0.38.0`) self-heals on the next `cargo build`/`cargo test` — confirm it reads `0.39.0` and stage it.
- **Publish FIRST:** `cargo publish -p md-codec` (dry-run `--dry-run` first; then publish to crates.io).

**P4.2 — md-cli re-release `0.9.0 → 0.9.1` (NO code change):**
- `crates/md-cli/Cargo.toml:3` `version = "0.9.0"` → `"0.9.1"`.
- **BLOCKING hand-edit:** `crates/md-cli/Cargo.toml:28` `md-codec = { path = "../md-codec", version = "=0.38.0" }` → `version = "=0.39.0"`. This is the **ONLY md-codec version literal** in the workspace (verified `1a4b322`); the exact-`=` pin will NOT resolve against md-codec 0.39.0 until this is bumped.
- `CHANGELOG.md` — prepend `## md-cli [0.9.1] — 2026-06-21`: "re-release pinning md-codec 0.39.0 (cycle-10 library cluster); no md-cli code change. `inspect`/`encode` id display reflects the L14/L15 id-value changes (display-only; no flag/output-shape change → no manual-mirror)."
- **No manual-mirror, no new flags** (library-internal; D10). `cargo test -p md-cli` GREEN.
- **Publish AFTER md-codec:** `cargo publish -p md-cli` (after md-codec 0.39.0 is live on crates.io so the `=0.39.0` pin resolves).

**P4.3 — git tags (after both publishes succeed) — tag conventions VERIFIED against existing tags at `1a4b322`:**
- Tag md-codec: **`md-codec-v0.39.0`** (matches the established md-codec convention — prior tags `md-codec-v0.38.0`, `md-codec-v0.37.0`; NOT a `descriptor-mnemonic-` prefix).
- Tag md-cli: **`descriptor-mnemonic-md-cli-v0.9.1`** (matches the established md-cli convention — prior tags `descriptor-mnemonic-md-cli-v0.9.0`, `…-v0.8.1`).
- Confirmed neither `md-codec-v0.39.0` nor `descriptor-mnemonic-md-cli-v0.9.1` exists yet.
- FF `origin/main` per the codec/CLI direct-FF-and-tag convention (NOT a PR; GUI is the PR repo, codecs/toolkit are direct-FF — MEMORY `feedback_autonomous_all_remaining_bugfix_cycles`).

**P4 gate:** both crates published to crates.io in the strict order; tags pushed; `origin/main` fast-forwarded. The whole-diff review (§5) MUST be GREEN before publish/tag.

---

## 5. Mandatory post-implementation whole-diff review (non-deferrable)

After P3 GREEN and BEFORE P4 publish/tag, dispatch an **independent adversarial whole-diff review** (opus architect) over the entire P1-P3 diff (CLAUDE.md per-phase pattern step 4: "R0 = plan correctness; this catches implementation-introduced regressions TDD misses"). It MUST:
1. Re-verify the M3 fail-closed property against the AS-WRITTEN code (not just this plan) — the funds-critical invariant.
2. Confirm L14 fill is empty-only and does not perturb explicit-origin ids; confirm L15 canonicalizes a clone (no mutation of the caller's `d`); confirm the L17 test is genuinely RED-before / GREEN-after.
3. Confirm L6's borrow resolution compiles AND the guard fires only on non-identity short-Divergent.
4. Run the FULL `cargo test -p md-codec` + `cargo test -p md-cli` + clippy `-D warnings` + `cargo fmt --all --check` (full package suites, not targeted — MEMORY `feedback_r0_review_run_full_package_suite`).
5. Confirm NO clap/CLI/manual/schema surface changed (library-internal invariant).

Persist verbatim to `design/agent-reports/cycle10-whole-diff-review.md` (toolkit `design/agent-reports/`, this repo) BEFORE the ship commit. If Agent-API dispatch fails mid-session, FLAG it explicitly and defer the formal review to API recovery — never silently substitute inline self-review (CLAUDE.md step 5).

---

## 6. Toolkit transitive pin-bump (note only — NOT planned here)

A SEPARATE toolkit ship consumes md-codec 0.39.0:
- `crates/mnemonic-toolkit/Cargo.toml:36` `md-codec = "0.38"` → `"0.39"` + `Cargo.lock:677` regen (0.38.0 → 0.39.0). (Toolkit already consumes 0.38.0 from cycle-4.)
- **Toolkit version: PATCH.** M3's widening is a strict capability gain surfaced transitively; the toolkit only ever builds explicit-origin / canonical-ordering descriptors, so its recorded ids are unchanged in practice (a toolkit-built explicit form now correctly MATCHES an md-cli-elided card's id — a fix, not a regression). No toolkit flag/capability gains.
- **⚠️ VERSION-COORDINATION with cycle-11b (toolkit-hygiene lane):** cycle-11b takes the toolkit to **0.65.1** (per the parallel-lane plan). The spec drafted this cycle's transitive bump as v0.65.1 against the v0.65.0 base — but cycle-11b is concurrently consuming that PATCH slot. **Do NOT hard-code the toolkit target version here.** When the toolkit pin-bump is actually authored, reconcile against the LIVE toolkit tag at that moment (MEMORY `feedback_followup_status_discipline` — tracking lags code; verify at decision time): if cycle-11b has already shipped 0.65.1, this transitive bump renumbers to the next PATCH (0.65.2); if not yet, coordinate the merge order. **This plan does NOT plan the toolkit internals — it only flags the dependency and the version-collision risk.**

---

## 7. FOLLOWUP status flips + bughunt report ticks (in the ship commit)

**FOLLOWUP status (plan-R0 Minor-1 — VERIFIED: none of the five slugs below currently exist in `design/FOLLOWUPS.md` in EITHER descriptor-mnemonic or this toolkit repo).** The bughunt report (`constellation-bughunt-2026-06-20.md`) is the system-of-record for these findings, so the bughunt ticks below ARE the canonical closure. Do NOT attempt to flip a non-existent slug. If — at ship time — a companion `FOLLOWUPS.md` entry has since been filed for any of these (re-grep at decision time per MEMORY `feedback_followup_status_discipline`), flip it to RESOLVED in the shipping commit; otherwise file nothing. Candidate slug names (for any newly-filed companion):
- `md-codec-derive-chain-gate-baseline-only-ignores-overrides` (M3) — gate-widening in `derive.rs`.
- `md-codec-walletpolicyid-canonical-fill-origin-elision` (L14).
- `md-codec-wdt-id-canonicalize-placeholder-ordering` (L15). **Release-note: id-VALUE change for non-canonical inputs.**
- `md-codec-walletpolicyid-elision-test-vacuous` (L17).
- `md-codec-canonicalize-divergent-path-decl-unchecked-len-panic` (L6).

**Tick the bughunt report** — in `design/agent-reports/constellation-bughunt-2026-06-20.md` (toolkit master), flip the `### - [ ]` checkboxes → `### - [x]`:
- M3 @ line `245`, L6 @ `368`, L14 @ `593`, L15 @ `603`, L17 @ `621` (re-grepped live against toolkit `origin/master`; round-1 plan-R0 citation-decay fix — re-grep again at ship time as the report grows with concurrent cycle ticks).

**Out-of-scope FOLLOWUPs — keep with explicit disposition** (spec §6; do NOT flip):
- `md-codec-lp4ext-varint-cannot-encode-child-ge-2pow29` (L16) — WON'T-FIX / DOC.
- `md-codec-chunk-split-ignores-37bit-header-budget` (D-md-chunk-budget) — WON'T-FIX.
- `md-codec-chunk-set-id-20bit-crosschunk-bind` (D-mk-crosschunk) — DEFERRED to a separate wire-format cycle (companion mk-codec note warranted when scheduled).

---

## 8. MANDATORY R0 GATE (project standard)

This is a **plan-doc — DESIGN ONLY, NO CODE.** Per CLAUDE.md Conventions (first bullet), before ANY implementation begins this plan-doc MUST pass an **opus-architect R0 review** and the reviewer-loop MUST converge to **0 Critical / 0 Important**: fold findings → persist the review verbatim to `design/agent-reports/cycle10-plan-r0-round-N-review.md` → re-dispatch → repeat until GREEN (the reviewer-loop continues after EVERY fold; folds can introduce drift). No code, no implementer dispatch, no worktree edits, no SemVer bump, no publish, no tag while ANY Critical or Important finding is open. Per-phase execution (P1-P4) then carries its own per-phase R0 + the mandatory post-implementation whole-diff adversarial review (§5).
