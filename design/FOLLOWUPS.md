# Follow-up tracker

Single source of truth for items that surfaced during a review or implementation pass but were not fixed in the same commit. Mirrors the conventions of the sibling `descriptor-mnemonic`, `mnemonic-key`, and `mnemonic-secret` repos.

## How to use this file

**Format for each entry:**

```markdown
### `<short-id>` — <one-line title>

- **Surfaced:** Phase X review of commit <SHA>, or "inline TODO at <file>:<line>"
- **Where:** `<file>:<line>` or "design — SPEC §X"
- **What:** 1–3 sentences describing the gap or improvement opportunity
- **Why deferred:** the reason it didn't ship in the original commit
- **Status:** `open` | `resolved <COMMIT>` | `wont-fix — <one-line reason>`
- **Tier:** `v0.1-blocker` | `v0.1-nice-to-have` | `v0.2` | `cross-repo` | `v1+` | `external`
```

Reference the `<short-id>` from commit messages when closing: `closes FOLLOWUPS.md <short-id>`.

## Tiers (definitions)

- **`v0.1-blocker`**: must fix before tagging `mnemonic-toolkit-v0.1.0`. (Empty after release.)
- **`v0.1-nice-to-have`**: should fix before v0.1 if time permits, but won't block release. Documented in v0.1's CHANGELOG if shipped.
- **`v0.2`**: explicitly deferred to v0.2 (multisig templates, non-zero account, K-of-N share bundles).
- **`v0.2-nice-to-have`**: surfaced during v0.2 review; non-blocking. Documented in v0.2's CHANGELOG if shipped.
- **`v0.3`**: explicitly deferred to v0.3 (user-supplied descriptor passthrough; resolve during v0.3 cycle).
- **`v0.3-nice-to-have`**: surfaced during v0.3 review; non-blocking.
- **`v0.4-cross-repo`**: deferred to v0.4 AND requires coordination with sibling repos.
- **`v0.4-nice-to-have`**: surfaced during v0.4 review; non-blocking. Documented in v0.4's CHANGELOG if shipped.
- **`v0.4.1`**: explicitly deferred from v0.4.0 to a v0.4.1 follow-on patch (typically scope-safety deferrals).
- **`v0.4.2`**: explicitly deferred from v0.4.1 to a v0.4.2 follow-on patch.
- **`v0.4.2-nice-to-have`**: surfaced during v0.4.1 review; non-blocking. Documented in v0.4.2's CHANGELOG if shipped.
- **`v0.4.3`**: explicitly deferred to a v0.4.3 follow-on patch.
- **`v0.4.3-nice-to-have`**: surfaced during v0.4.2 review; non-blocking.
- **`v0.4.4`**: explicitly deferred to a v0.4.4 follow-on patch.
- **`v0.4.4-nice-to-have`**: surfaced during v0.4.3 review; non-blocking.
- **`v0.5`**: explicitly deferred to a v0.5 minor release (typically scope too large for a v0.4.x patch).
- **`cross-repo`**: depends on coordination with sibling repos (`descriptor-mnemonic`, `mnemonic-key`, `mnemonic-secret`). Mirrored by a companion entry in the affected sibling's tracker; both cite each other.
- **`v1+`**: deferred indefinitely.
- **`external`**: depends on upstream work (e.g., a sibling crate exposing a helper).

---

## Open items

### `resolved-slot-entropy-zeroizing-field` — change `ResolvedSlot.entropy` to `Option<Zeroizing<Vec<u8>>>`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN (deferred from in-cycle landing due to 19-site cascade).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:582` — `pub entropy: Option<Vec<u8>>` field on the `ResolvedSlot` (private) struct. 19 read/write sites cascade through bundle.rs, verify_bundle.rs, parse_descriptor.rs (incl. test mods).
- **What:** Per plan §"Phase 2 — Impl" step 4, ResolvedSlot.entropy was scheduled to become `Option<Zeroizing<Vec<u8>>>` so the field-resident entropy scrubs on drop. Phase 2 GREEN landed local-wrap discipline at every producer + consumer site (entropy entering the field is `Zeroizing` at construction; reads clone to a local `Zeroizing`) but left the field type as `Option<Vec<u8>>` — so the field-resident copy itself is unwrapped during its lifetime.
- **Why deferred:** 19-site cascade across 3 files + test mods is mechanically large and not representative of the per-row wrap discipline the Phase 2 zeroize-lint is enforcing. The local-wrap discipline at producer + consumer sites covers the value's transit; only the brief field-resident lifetime is unwrapped. A separate small commit can complete the field type change in one shot.
- **Status:** `superseded by resolved-slot-derived-account-zeroizing-field` (2026-05-13, Phase 3a R0 v3-fold RESCOPE per `~/.claude/plans/2026-05-13-cycle-b-phase-3a-rescope-proposal.md`). The R0 v2 LOCK that bundled this migration with the Cycle B Phase 3a `_entropy_pin` apply work was reviewed and re-scoped to Path B-lite, which carves out the field-type migration to a focused v0.10.1 patch. The new entry [[resolved-slot-derived-account-zeroizing-field]] takes broader scope (covers both `ResolvedSlot.entropy` AND `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>`, plus `impl Drop for DerivedAccount` deletion, plus `into_parts` body change, plus the lint anchor relabel + new row, plus CHANGELOG entry).
- **Tier:** `v0.10.1-patch` (escalated from `v0.9.2-nice-to-have`; broader scope under the superseding entry)

### `resolved-slot-derived-account-zeroizing-field` — migrate Cycle-A `Vec<u8>` entropy fields to `Zeroizing<Vec<u8>>` (supersedes [[resolved-slot-entropy-zeroizing-field]])

- **Surfaced:** 2026-05-13, Cycle B Phase 3a R0 v3-fold RESCOPE (Path B-lite). Carved out from the R0 v2 LOCK (commit `9be0f0f`) which had bundled this migration with the `_entropy_pin` apply work; the rescope keeps the pin work in Phase 3a (toolkit `v0.10.0`) and defers the field-type migration to a focused v0.10.1 patch.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:585` (`pub entropy: Option<Vec<u8>>`); `crates/mnemonic-toolkit/src/derive.rs:21,49` (`pub entropy: Vec<u8>` + `impl Drop for DerivedAccount`); `crates/mnemonic-toolkit/src/derive.rs:37` (`into_parts()` body); `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` (row labels at lines 14-16, 109-113); `CHANGELOG.md`.
- **What** (7 deliverables, all in one v0.10.1 patch):
  1. `ResolvedSlot.entropy: Option<Vec<u8>>` → `Option<Zeroizing<Vec<u8>>>` at `synthesize.rs:585`. Cascade: 6 ctor sites (`cmd/bundle.rs:{348,449,1065}` real-pin sites + `:417,491` watch-only None sites + `synthesize.rs:1184` test) wrap construction in `Zeroizing::new(...)`. ~6 read-site edits become Deref-through-Zeroizing.
  2. `DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>` at `derive.rs:21`. 1 ctor site (`derive_slot.rs:77`) wraps in `Zeroizing::new(...)`.
  3. DELETE `impl Drop for DerivedAccount` at `derive.rs:49-58` (Zeroizing's Drop carries the scrub responsibility; `pub-struct-drop-semver-risk-monitor` FOLLOWUP exit also closes here).
  4. `into_parts()` body change at `derive.rs:37`: `mem::take(&mut self.entropy)` → `mem::take(&mut *self.entropy)` (Deref through Zeroizing). Outward signature returning `Vec<u8>` is preserved.
  5. `tests/lint_zeroize_discipline.rs` row "DerivedAccount impl Drop scrubs entropy on drop" relabeled to "DerivedAccount entropy field is `Zeroizing<Vec<u8>>`" with new evidence; lint lines 109-113 deferred-FOLLOWUP comment block DELETED; new row "ResolvedSlot entropy field is `Option<Zeroizing<Vec<u8>>>`" with evidence `pub entropy: Option<Zeroizing<Vec<u8>>>` against `src/synthesize.rs`.
  6. `CHANGELOG.md` v0.10.1 entry: "Field-type migration: `ResolvedSlot.entropy` and `DerivedAccount.entropy` to `Zeroizing<Vec<u8>>`; deletes `impl Drop for DerivedAccount` (Zeroizing carries scrub). Closes deferred FOLLOWUP `resolved-slot-entropy-zeroizing-field`. Closes monitoring FOLLOWUP `pub-struct-drop-semver-risk-monitor`."
  7. R1 Opus review per `feedback_opus_primary_review_agent`; report at `design/agent-reports/v0_10_1-zeroizing-field-migration-r1.md`.
- **Why deferred (separated from Cycle B Phase 3a):** Carving the field-type migration out of Phase 3a removes audit-trail entanglement (Cycle A's `lint_zeroize_discipline.rs` stays untouched in Cycle B), eliminates the Arc-wrap design from being conflated with the Zeroizing migration in reviewer eyes, and lets v0.10.1 ship as a clean focused patch with no concurrent mlock work. The cascade cost is roughly equal whether bundled or split (each ctor site takes 1 edit per landing; combined = 14 edits in one PR; split = 7 + 7 edits across two PRs). The structural-discipline gap (human-maintained `impl Drop` scrub vs. structurally-guaranteed Zeroizing field-type) stays at Cycle-A levels through Cycle B; v0.10.1 closes it.
- **Status:** `open` (will close with the v0.10.1 patch ship).
- **Tier:** `v0.10.1-patch`.
- **Companion:** N/A (toolkit-only patch — no cross-repo work; ms-cli `v0.3.0` ships in Cycle B PE without coordination).

### `rust-secp256k1-secretkey-zeroize-upstream` — `secp256k1::SecretKey` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 R1 (Opus, finding I-2). The lint at `lint_safety_third_party_blocked.rs` now scans for `SecretKey::from_slice` patterns in addition to the original Mnemonic/Xpriv anchors.
- **Where:** Upstream crate `bitcoin = "0.32"` (transitive `secp256k1`). Affects every `SecretKey::from_slice` construction in `crates/mnemonic-toolkit/src/{bip85,parse_descriptor,cmd/convert}.rs` — 5 production call sites. Each carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `secp256k1::SecretKey` is stack-bound, provides `non_secure_erase()` (which is best-effort and compiler-defeatable, per the upstream's own doc) but does NOT implement Drop with Zeroize. The toolkit's mitigation is lifetime minimization + SAFETY-anchored doc-comments at the construction sites; the residual gap is that the 32-byte scalar lives in stack memory until function exit unscrubbed. Closes when upstream `rust-secp256k1` ships a Drop+Zeroize impl for SecretKey (or when the toolkit migrates to a different curve library that does).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bip39-mnemonic-zeroize-upstream` — `bip39::Mnemonic` has no Drop+Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Mnemonic::parse_in` / `Mnemonic::from_entropy_in` call site in this repo.
- **Where:** Upstream crate `bip39 = "2"`. Affects every `Mnemonic` construction in `crates/mnemonic-toolkit/src/{bip85,derive,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — 25 production call sites enumerated by `lint_safety_third_party_blocked.rs::SCAN_FILES`. Each site carries a `SAFETY: third-party-blocked` doc-comment pointing at this FOLLOWUP.
- **What:** `bip39::Mnemonic` holds the phrase + internal entropy buffer but does not implement `Drop` with `Zeroize::zeroize`. The toolkit's mitigation is lifetime minimization (construct → `to_entropy()` / `to_seed()` into `Zeroizing` → immediate drop), but a residual gap remains: the secret bytes inside `Mnemonic` are not actively scrubbed before deallocation. Closes when upstream `bip39` adds `impl Drop` + `zeroize` dep.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `rust-bitcoin-xpriv-zeroize-upstream` — `bitcoin::bip32::Xpriv` is Copy + no Drop + no Zeroize

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 2 GREEN — surfaced while landing the `SAFETY: third-party-blocked` doc-comment discipline at every `Xpriv::new_master` / `Xpriv::derive_priv` call site in this repo.
- **Where:** Upstream crate `bitcoin = "0.32"`. Affects every `Xpriv` construction in `crates/mnemonic-toolkit/src/{bip85,derive_slot,synthesize,parse_descriptor,cmd/{bundle,convert,derive_child}}.rs` — also enumerated by `lint_safety_third_party_blocked.rs`.
- **What:** `bitcoin::bip32::Xpriv` is `Copy` and has no Drop hook upstream. Phase 0 R3 C-R3-3 verified that `drop(xpriv)` on a `Copy` type is a no-op for memory cleanup (the value bitwise-copies into `drop()` and the original binding remains untouched; every `derive_priv` call leaves a fresh stack copy). Closes when upstream `bitcoin` removes `Copy` from `Xpriv` + adds `impl Drop` + `zeroize` dep — a coordinated breaking change requiring downstream migration at every `Xpriv` call site.
- **Status:** `open` (upstream-blocked; non-trivial breaking change for upstream)
- **Tier:** `external`

### `convert-minikey-stdout-redaction` — widen `NodeType::is_secret_bearing` to cover Casascius MiniKey on the stdout-redaction pathway

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 1 R1 review (Opus 4.7, finding N-2 partial — surfaced while folding the wider-tag method lift onto `NodeType`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` — `NodeType::is_secret_bearing` (around `convert.rs:85-96`) excludes `MiniKey` from the existing redaction + secret-on-stdout pathways at `convert.rs:769` (`from_value` redaction in `--from <secret>=` echo) and `convert.rs:796` (`secret-on-stdout` warning). Phase 1 added a wider `NodeType::is_argv_secret_bearing` method (around `convert.rs:98-110`) that DOES include MiniKey for argv-leakage advisory purposes; the narrower predicate is preserved to avoid expanding Phase 1's scope.
- **What:** MiniKey (Casascius mini-key — a private-key encoding) is a private-key carrier per survey §5 row "convert --from minikey=" but is currently NOT redacted in the `from_value` echo path and does NOT fire the `secret-on-stdout` warning on convert edges that emit a MiniKey value to stdout. Tightening: either widen `is_secret_bearing` to include MiniKey, or change the two call sites to use the wider `is_argv_secret_bearing` predicate. Either approach is small and additive.
- **Why deferred:** Phase 1 scope (argv-leakage closure) ships in lockstep with SPEC v0.9.0 §1 item 1; widening the existing secret-on-stdout warning is a separate user-facing behavior change that would entrain additional fixture updates in `tests/cli_convert_minikey.rs` (currently no advisory is expected) and warrants its own SPEC/disposition pass.
- **Status:** `open`
- **Tier:** `v0.9.1-nice-to-have` (small mechanical fix; can ship in a Phase E cycle-close patch or in Cycle B planning).

### `argv-overwrite-after-parse` — rewrite `argv[]` post-clap to clear secret bytes from `/proc/$PID/cmdline`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-2` (/proc/self/cmdline post-parse overwrite class).
- **Where:** Hypothetical new module `crates/mnemonic-toolkit/src/argv_overwrite.rs` (does not yet exist). Touches every binary entry-point (`mnemonic`, `md`, `mk`, `ms`) to invoke the overwrite shim immediately after `clap::Parser::parse()` returns. The kernel-owned mirror lives in `/proc/$PID/cmdline`; on Linux a raw FFI write into the original `argv[][i]` byte ranges (via `libc::__progname`-adjacent pointer arithmetic, or the `set_proctitle`-style trick) is the only path that actually mutates the in-kernel copy.
- **What:** Phase 1 added a stderr advisory whenever a secret is detected on argv but did NOT mutate argv. The residual gap: an attacker reading `/proc/$PID/cmdline` (same-UID; or any UID without `PR_SET_DUMPABLE=0`) sees the secret bytes for the lifetime of the process. Real fix is to (a) zero-overwrite the in-place argv slots immediately after clap consumes them, OR (b) call `prctl(PR_SET_DUMPABLE, 0)` to deny `/proc/$PID/cmdline` reads to other UIDs (narrower mitigation — does not protect same-UID reads or core dumps). Both are FFI-heavy and platform-specific.
- **Why deferred:** Phase 1's `--*-stdin` paired-flag + `=-` route closes argv-leakage for documented usage; the residual covers users who ignore the warning. SPEC §3 explicitly defers this to a future cycle pending the raw-FFI route.
- **Status:** `open`
- **Tier:** `v1+`

### `clap-argv-pre-parse-residue` — libc `OsString` heap copies of `argv[]` live un-scrubbed before clap parses

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-libc-osstring` class.
- **Where:** `std::env::args_os() -> Vec<OsString>` materialization, called by Rust startup before user code runs. The toolkit cannot intercept earlier than `main()` without replacing libc rt0 (or per-invocation raw-FFI argv intercept). Mirrors the `mnemonic-gui/secret_widget.rs` doc-comment caveat for cross-repo parity.
- **What:** Phase 2's `std::mem::take(&mut args.phrase)` + `Zeroizing::new(...)` scrubs the clap-created `String` allocation but cannot reach the prior `OsString` heap allocation that libc materialized BEFORE clap parsed. Those `OsString` buffers drop un-scrubbed. Addressable only by libc replacement (e.g., musl + custom rt0) or per-invocation raw-FFI argv intercept before clap.
- **Why deferred:** Outside the toolkit's reach (kernel/libc layer). Mirrors mnemonic-gui caveat for parity.
- **Status:** `open`
- **Tier:** `v1+`

### `allocator-pool-residue` — `Zeroizing<Vec<u8>>` drop-time scrub may be defeated by custom-allocator page retention

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-allocator-residue` class.
- **Where:** Allocator-layer concern — applies to every `Zeroizing<Vec<u8>>` / `Zeroizing<String>` / `Zeroizing<Box<[u8]>>` allocation in the workspace when the binary is built against a non-default allocator (e.g., `jemallocator` with cache retention, `mimalloc` with retention, `tcmalloc`). The Cycle A test environment uses the system allocator (glibc malloc); custom allocators are NOT in scope.
- **What:** When a `Zeroizing<Vec<u8>>` drops, the bytes are zeroed in-place by `zeroize::Zeroize` BEFORE the deallocation call returns the page to the allocator. With the system allocator this is sound — the zeroed pages are returned to the OS or to a free-list with the zeros intact. With a retention-class custom allocator (jemalloc with `lg_dirty_mult=-1` or `dirty_decay_ms=-1`, mimalloc with retain pools, etc.), the allocator may *re-zero* or *re-use* the page for an unrelated allocation in ways that briefly expose the secret bytes to in-process readers. Mitigation requires a secret-class-aware page management discipline (custom allocator hook) or a dedicated mmap'd secret arena ([[dedicated-secret-arena]]).
- **Why deferred:** Defense-in-depth class; system allocator (default for `mnemonic`, `md`, `mk`, `ms` binaries) is sound. Custom allocators are an opt-in build configuration not in Cycle A scope.
- **Status:** `open`
- **Tier:** `v1+`

### `pub-struct-drop-semver-risk-monitor` — `impl Drop` on `DerivedAccount` breaks move-out destructure for external library users

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-pub-struct-drop` class.
- **Where:** `crates/mnemonic-toolkit/src/derive.rs:14-66` — `pub struct DerivedAccount` with `impl Drop` (Phase 2 prereq landing at commit `16f64d7`). Rust E0509 forbids move-out destructuring (`let DerivedAccount { entropy, .. } = derived;` and `let entropy = derived.entropy;`) when the enclosing struct has `impl Drop`. Field-borrow access (`&derived.entropy`) and Deref-via-method-call (`derived.entropy.as_slice()`) remain compatible.
- **What:** Cycle A chose `impl Drop` over changing the field type to `Zeroizing<Vec<u8>>` because the latter would force a v0.10.0 minor bump per the pre-1.0 SemVer convention. The trade-off is that any external library user with a move-out destructure pattern on `DerivedAccount` will get an E0509 compile error post-fold. We treat this as patch-tag-compatible (move-out is uncommon in external use; field-borrow is the typical access pattern). Monitor: if downstream library users surface complaints, the cycle re-tags retroactively as a minor bump (v0.10.0).
- **Why deferred:** The fold itself shipped in Phase 2; this entry is the monitoring artifact for the residual semver risk.
- **Status:** `open` (monitoring; closes if no downstream complaints land before v0.10.0 cycle, or with retroactive re-tag if complaints arrive)
- **Tier:** `v1+`

### `dedicated-secret-arena` — mmap-allocated page-aligned secret arena for secret-class allocations

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-secret-arena` class.
- **Where:** Hypothetical new module pattern — would replace the default heap allocator for `Zeroizing`-wrapped allocations with a dedicated mmap'd region (à la rust `secrecy` / `secure_string` crates). Touches the GlobalAlloc surface or requires a typed allocator parameter on `Zeroizing<T, A>`-style wrappers.
- **What:** `mlock(2)` pins pages, not bytes; future maintainers may need a dedicated mmap-allocated secret arena for page-aligned heap placement of secret-class allocations. This avoids both the page-vs-byte granularity trap (Cycle B mlock will hit this) and the allocator-pool residue ([[allocator-pool-residue]] becomes addressable via a dedicated allocator path). Out of scope for Cycles A and B both — this is the third-pass design class.
- **Why deferred:** Design class — Cycle A is "first-pass at OWNED-buffer hygiene"; Cycle B is mlock; arena is the natural third cycle.
- **Status:** `open`
- **Tier:** `v1+`

### `sha3-shake256-zeroize-upstream` — `sha3::Shake256` XOF reader state has no `Zeroize`

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `sha3 = "0.10"`. Affects `crates/mnemonic-toolkit/src/bip85.rs` callees that use `Shake256::default()` + `.update()` + `.finalize_xof()` to derive BIP-85 child entropy. The XOF reader holds Keccak sponge state with BIP-85 child entropy mixed in until the reader drops.
- **What:** `sha3::digest::ExtendableOutput` returns a `Shake256Reader` that reads the XOF stream lazily. The reader's internal Keccak state carries the absorbed entropy (the child secret) until the reader drops. Upstream `sha3` does not implement `Zeroize` on the Keccak state or the XOF reader. Toolkit mitigation: minimize XOF reader lifetime — call `.read(...)` into a Zeroizing<[u8; N]> output immediately and drop the reader. Residual gap: the Keccak state inside the reader is not actively scrubbed before deallocation. Closes when upstream `sha3` adds `impl Zeroize` on its core state types.
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `bip38-crate-internal-zeroize-upstream` — `bip38` crate's scrypt intermediate state is not zeroize-aware

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 carry-over from survey §3.
- **Where:** Upstream crate `bip38 = "1"`. Affects `crates/mnemonic-toolkit/src/cmd/convert.rs::Bip38` decrypt arm at `convert.rs:1135` (and the V3 NULL-byte-passphrase path closed in Phase 1).
- **What:** The `bip38` crate's internal scrypt KDF intermediate state and the AES round buffers are not Zeroize-wrapped. Toolkit can `Zeroizing`-wrap the *returned* `(privkey_bytes, compressed_flag)` tuple but cannot reach into the crate's stack frames during decrypt. Residual gap: scrypt intermediate state (~1 MiB by default cost factor) lives un-scrubbed on the stack/heap during the decrypt call. Closes when upstream `bip38` adds Zeroize discipline (or when the toolkit replaces with an internally-controlled scrypt + AES implementation).
- **Status:** `open` (upstream-blocked)
- **Tier:** `external`

### `secret-memory-hygiene-cycle-b` — `mlock(2)` / `VirtualLock` page-pinning infrastructure (Cycle B)

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review SPLIT-CYCLE recommendation; opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1).
- **Where:** Hypothetical new module `crates/mnemonic-toolkit/src/mlock.rs` (toolkit-only — ms-secret has no syscall layer; per SPEC §3 OOS-mlock-cycle-b). Touches the 5 mlock candidates named in SPEC §3:
  1. `BundleArgs.passphrase` / `slot[i].value` and the equivalent clap fields on `VerifyBundleArgs`, `ConvertArgs`, `DeriveChildArgs`, `EncodeArgs`, `VerifyArgs`.
  2. `ResolvedSlot.entropy` Vec in `synthesize.rs:582`.
  3. `DerivedAccount.entropy` Vec in `derive.rs:14-66`.
  4. `bip85::derive_entropy` returned `[u8; 64]` (requires heap-promotion first).
  5. ms-cli `read_stdin()` String buffer in `parse.rs:45`.
- **What:** Cycle B will add a cross-platform mlock module with capability-aware soft-fail (Linux mlock + macOS mlock + Windows VirtualLock, with `EPERM`/permission-denied → log-and-continue semantics). mlock is applied to the Zeroizing-wrapped heap buffers from Cycle A — Cycle A is a prerequisite. SPEC for Cycle B will be drafted after Cycle A ships.
- **Why deferred:** R3 SPLIT-CYCLE finding — combining mlock with Zeroizing would have doubled Cycle A's review surface; splitting keeps each cycle's blast radius reviewable.
- **Status:** `open` (Cycle B SPEC drafting starts post-Cycle-A Phase E ship)
- **Tier:** `v0.9.x`

### `cycle-b-pre-spec-questions` — pre-SPEC scoping questions blocking Cycle B drafting

- **Surfaced:** 2026-05-13, v1.0 roadmap-survey Bucket 1 drill-down (Opus scoping read-out, atop `mnemonic-toolkit-v0.9.2` ship). Companion to `secret-memory-hygiene-cycle-b`.
- **Where:** Resolves into the eventual `design/SPEC_secret_memory_hygiene_v0_9_B.md` Phase 0 (not yet drafted). Source artifacts surveyed: `design/FOLLOWUPS.md` Cycle B entry; `design/SPEC_secret_memory_hygiene_v0_9_0.md` §3 OOS-mlock-cycle-b (lines 271-305); `design/agent-reports/v0_9_0-secret-memory-survey.md` §4 (lines 161-210); `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md` §4 (lines 247-269); `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (R3 SPLIT-CYCLE + R1 mlock-module-shape seed).
- **What:** Cycle B has enough material to start SPEC drafting, but 4 open questions + 1 architectural trap + ~4 unaddressed items must resolve before / during Phase 0 SPEC. Drafting paused until these are dispositioned.
  1. **Canonical 5-site list reconciliation.** SPEC §3 OOS-mlock-cycle-b lists `{clap-args, ResolvedSlot.entropy, DerivedAccount.entropy, bip85 [u8;64], ms-cli stdin String}`. Hygiene-matrix §4 substitutes `secp256k1::SecretKey` + `bip39::Mnemonic` interiors for slots 2 and 3 (defense-in-depth reframing). The two lists overlap by ~3 sites only. SPEC list is canonical per R4 I-R4-4 fold; Cycle B SPEC must explicitly pick one enumeration and document the matrix's alternative as OOS or supplementary coverage.
  2. **Toolkit-only scope vs ms-cli site #5.** Cycle B is framed "toolkit-only" (FOLLOWUPS + hygiene-matrix §4) but SPEC site #5 lives at `mnemonic-secret/.../ms-cli/parse.rs:45`. Either drop site #5 or revise scope to "toolkit + ms-cli stdin" (which makes Cycle B cross-repo, with a companion entry needed in `mnemonic-secret/design/FOLLOWUPS.md`).
  3. **bip85 `[u8; 64]` heap-promotion ordering.** SPEC site #4 is stack-resident `[u8; 64]` today. Survey §4 lines 206-210 says "heap-promote first; mlock those *if* they get heap-promoted." Either Cycle B absorbs heap-promotion as a precursor Phase 1, or a separate predecessor cycle does it first. Affects Cycle B's plan shape.
  4. **Platform commitment scope.** FOLLOWUPS commits to 3 platforms (Linux mlock + macOS mlock + Windows VirtualLock). Hygiene-matrix §4 narrows to "libc / Linux-specific." Pick one — the soft-fail abstraction shape (single backend with platform-gates vs three backends behind a trait) depends on this.

  **Architectural trap on record (R3 I-R3-2, Phase 0 R1 report lines 188-260):** The Phase 0 R1 prototype `try_mlock_region(&[u8])` byte-slice API "traps callers into page-vs-byte granularity wastefulness." `mlock(2)` pins pages, not bytes; SPEC §3 OOS-secret-arena defers proper page-aligned allocation to a future Cycle C (`dedicated-secret-arena`). Cycle B accepts residual page-residue from co-allocated non-secret data on locked pages; SPEC must document this and pick a signature shape that doesn't pretend byte-granularity is real.

  **Items not addressed in existing artifacts** (Phase 0 design decisions): soft-fail logging channel / level / format; `RLIMIT_MEMLOCK` exhaustion semantics (no soft-fail story beyond `EPERM` today); `CAP_IPC_LOCK` probe-up-front vs fail-per-call; cgroup memory limits.

  **Resolutions (2026-05-13 session, user decisions):**
  - **Q1 resolved:** SPEC §3 OOS-mlock-cycle-b 5-site list is canonical. Matrix's `secp256k1::SecretKey` + `bip39::Mnemonic` substitutions are out-of-Cycle-B supplementary coverage (filed in Cycle B SPEC §3 as `OOS-upstream-zeroize-mlock`); revisit when those upstreams gain Drop+Zeroize.
  - **Q2 resolved (toward cross-repo):** ms-cli site #5 stays IN Cycle B's target list. Cycle B becomes cross-repo (toolkit + ms-cli). Companion FOLLOWUP `secret-memory-hygiene-cycle-b` to be filed in `mnemonic-secret/design/FOLLOWUPS.md` at P0 SPEC ship. The "toolkit-only" framing in earlier artifacts is superseded by this SPEC's §5 cross-repo coordination.
  - **Q3 resolved:** Cycle B absorbs bip85 `[u8; 64]` heap-promotion as Phase 1 (P1 toolkit-only precursor refactor; P2 builds mlock module; P3a applies at toolkit sites; P3b applies at ms-cli; PE rollup).
  - **Q4 resolved:** Linux + macOS (POSIX path) committed for Cycle B. Windows `VirtualLock` deferred to a separate future cycle once the POSIX soft-fail abstraction has settled. Filed in Cycle B SPEC §3 as `OOS-windows-virtuallock`.

  **Architectural trap resolved (R3 I-R3-2):** Cycle B's `pin_pages_for(&[u8]) -> PinnedPageRange` returns the actual page range pinned (page-granularity explicit in the return type), NOT the byte-slice fiction. SPEC §3 `OOS-page-residue-elimination` documents that co-resident non-secret data on locked pages is incidentally pinned; full isolation deferred to Cycle C `dedicated-secret-arena`.

  **Brainstorming-session resolutions (5 additional Qs, 2026-05-13):**
  - **API shape:** hybrid — `MlockedZeroizing<T>` wrapper (sites 2/3/4) + `pin_pages_for(&[u8])` slice fn (sites 1/5). Matches libsodium's two-tier API.
  - **Capability detection:** try-and-soft-fail per call (no upfront probe). `MlockState` process-static singleton aggregates failures into a single 2-line stderr summary at end of process via `report_at_exit()`.
  - **Logging:** stderr plain-text, 2 lines, no suppression flag/env-var.
  - **Errno discipline:** all errnos soft-fail in release; `debug_assert!` on unreachable `EINVAL` in debug builds.
  - **Cross-repo sharing:** inline copy of `pin_pages_for` in both repos; CI invariant test diffs the two implementations (normalized) and fails on drift. No shared `mnemonic-mlock` crate; constellation stays at 4 crates.
- **Why deferred:** v1.0 roadmap pass; user direction is to capture pre-SPEC scope state so a future SPEC-drafting session starts cold-but-informed rather than re-discovering the discrepancies.
- **Status:** `resolved by P0 ship (commit 0c02247, 2026-05-13) — Cycle B SPEC at design/SPEC_secret_memory_hygiene_v0_9_B.md; reviewer-loop CLEAR 0C/0I across R1 (design/agent-reports/v0_9_B-phase-0-spec-r1.md: 2C/3I folded) and R2 (design/agent-reports/v0_9_B-phase-0-spec-r2.md: 0C/0I confirmed). All 4 pre-SPEC questions plus 5 brainstorming-session questions dispositioned; resolutions inlined in the What block above. Companion FOLLOWUP secret-memory-hygiene-cycle-b filed in mnemonic-secret at P0 close per SPEC §5.`
- **Tier:** `v0.9.x`
- **Companion:** `secret-memory-hygiene-cycle-b` (parent cycle entry) at `design/FOLLOWUPS.md`. If Q2 resolves toward "ms-cli stdin is in scope," a companion entry in `mnemonic-secret/design/FOLLOWUPS.md` is needed at SPEC drafting time.

### `md-mk-private-key-surface-watch` — reopen md/mk Cycle A participation if either repo grows a private-key surface

- **Surfaced:** 2026-05-13, v0.9.0 Cycle A Phase 0 R3 architect-review I-R3-4 fold (drop md/mk symmetry-stubs); opened as standalone tracker entry per Phase 3 hygiene-matrix R1 (Opus, finding C-1). SPEC §3 `OOS-md-mk` class.
- **Where:** `descriptor-mnemonic` repo (md-codec + md-cli) and `mnemonic-key` repo (mk-codec + mk-cli). Currently both hold xpub-only / descriptor-only material with no private-key buffer.
- **What:** Cycle A drops the no-scope-symmetry matrix stubs originally planned for md/mk repos because they have no secret material to audit. If either repo later gains a private-key surface (e.g., a future md-codec descriptor-binding with embedded xprv, or an mk-codec xprv passthrough), this FOLLOWUP fires and Cycle A's hygiene discipline (Zeroizing + SAFETY anchors + matrix delta) reopens for the affected sibling.
- **Why deferred:** No secret material to audit today.
- **Status:** `open` (monitoring)
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md` — same `md-mk-private-key-surface-watch` short-id.

### `secret-memory-hygiene-v0_9-cycle-a` — cross-repo cycle: OWNED-buffer secret-memory hygiene v0.9.0 Cycle A

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_secret_memory_hygiene_v0_9_0.md`. Plan at `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`. Survey precursor at `design/agent-reports/v0_9_0-secret-memory-survey.md`. R1+R2+R3+R4+R5 architect-review disposition at `design/agent-reports/v0_9_0-phase-0-spec-plan-r1.md` (5 rounds: Sonnet/Sonnet/Opus/Opus/Sonnet, cleared CLEAR 0C/0I after R3 SPLIT-CYCLE pushback + user decisions on impl-Drop approach + drop md/mk symmetry-stubs).
- **Where:** mnemonic-toolkit Phases 1 (argv close: 9 toolkit flag-rows + 5 distinct impl changes), 2 (zeroize discipline: ~30 toolkit OWNED rows + `derive_master_seed` seed-step helper + `impl Drop for DerivedAccount` with `into_parts()` migration of 3 internal move-out sites at `bundle.rs:325-329`, `bundle.rs:421-425`, `synthesize.rs:741-744`), 3 (hygiene matrix file), E (rollup). Sibling participation: mnemonic-secret Phase 2 (ms-cli + ms-codec zeroize, 4 + 10 OWNED rows) + Phase 3 (matrix file).
- **What:** OWNED-buffer first-pass at secret-memory hygiene. Closes argv leakage on toolkit's `bundle` / `verify-bundle` / `derive-child` / `convert --bip38-passphrase` flags (via new `--*-stdin` flags + `slot_input.rs` `=-` parser extension). Adds zeroize-on-drop semantics to every OWNED secret allocation in ms-codec + ms-cli + mnemonic-toolkit. Cycle B (mlock infrastructure) is a separate post-Cycle-A cycle per R3 SPLIT-CYCLE finding.
- **Status:** `resolved 9035656` — `mnemonic-toolkit-v0.9.2` tag pushed 2026-05-13. Sibling-repo tags shipped in lockstep: `ms-codec-v0.1.3` (mnemonic-secret `b1694e2`), `ms-cli-v0.2.2` (mnemonic-secret `ab8c73f`). All 6 SPEC §6 gates satisfied; cycle B (mlock) deferred to a separate cycle.
- **Tier:** `cross-repo`
- **Companion:** `mnemonic-secret/design/FOLLOWUPS.md` — same `secret-memory-hygiene-v0_9-cycle-a` short-id. md / mk repos do NOT receive a companion entry this cycle (xpub-only material; SPEC §3 OOS-md-mk + R3 I-R3-4 fold).

### `bip-vector-adoption-v0_8` — cross-repo cycle: BIP-vector adoption v0.8.0

- **Surfaced:** 2026-05-13. Cycle SPEC at `design/SPEC_test_vector_audit_v0_8_0.md`. Plan at `/home/bcg/.claude/plans/v0_8_0-bip-vector-adoption.md`. R1 review at `design/agent-reports/v0_8_0-phase-0-spec-plan-r1.md`.
- **Where:** mnemonic-toolkit Phase 3 = BIP-85 v85.3 (24-word BIP-39) cell at `crates/mnemonic-toolkit/tests/cli_derive_child.rs::cell_2b_bip39_24_words_reference_vector`. BIP-39 Trezor English fill (the other v0.7.1 §5 carry-over named in SPEC §2) was already closed by `feat(v0.8-phase-8)` commit `85694b2` *before* this cycle started; SPEC §2 row updated to record this. Net new for this cycle from the toolkit side: +1 vector (v85.3) plus Phase 4 audit-matrix cross-repo lift + Phase 0 SPEC + plan landed at `d0e6afc`.
- **What:** This repo's contribution to the v0.8.0 cross-repo vectors-only patch cycle. Closes when the cycle's audit-matrix successor doc lands at `design/agent-reports/v0_8_0-bip-test-vector-audit-matrix.md` (Phase 4) and the patch tag is cut at Phase E. Sibling-repo phases: descriptor-mnemonic Phase 1 (BIP-341, committed `b464f3f`); mnemonic-secret Phase 2 (BIP-93, committed `7101c16`).
- **Status:** `resolved f036737` — mnemonic-toolkit-v0.9.1 tag pushed; cycle close PR #15 merged. Companion sibling-repo tags: ms-codec-v0.1.2 (mnemonic-secret 527c9c7), md-codec-v0.32.1 (descriptor-mnemonic ef00e07), mnemonic-key PR #10 (6d43115, docs-only no tag).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, `mnemonic-key/design/FOLLOWUPS.md` — same `bip-vector-adoption-v0_8` short-id in each.

### `bip340-schnorr-signing-surface-evaluation` — BIP-340 Schnorr test vectors deferred pending a signing surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** No file; cross-repo classification. No signing surface exists in any of the four sibling crates (grep for `schnorr` / `sign` / `signing_key` returns zero matches across `descriptor-mnemonic`, `mnemonic-toolkit`, `mnemonic-secret`, `mnemonic-key`).
- **What:** BIP-340 ships a sidecar CSV at `bip-0340/test-vectors.csv` with Schnorr signature test vectors. None of the four sibling crates exposes a signing surface, so BIP-340 is OUT-OF-SCOPE-PER-LAYER for the v0.8.0 vectors-only cycle. If a future cycle introduces signing (e.g., a `mnemonic sign-message` BIP-322 surface, or hardware-signer integration), this FOLLOWUP closes by mirroring the CSV into the relevant repo.
- **Status:** `open` (deferred until signing surface lands).
- **Tier:** `v1+`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `bip341-keypath-signing-vector-coverage` (the BIP-341 `keyPathSpending` corpus has the same gating).

### `bip39-japanese-wordlist-support` — BIP-39 Japanese vectors require JP wordlist surface

- **Surfaced:** 2026-05-13, v0.8.0 Phase 0 (SPEC §3 OUT-OF-SCOPE classification).
- **Where:** `ms-codec` and `mnemonic-toolkit` are English-only at v0.8.x. The Trezor python-mnemonic repo also publishes a Japanese vector corpus at `https://github.com/bip32JP/bip32JP.github.io/blob/master/test_JP_BIP39.json`.
- **What:** Extending BIP-39 support to the Japanese wordlist would add ~24 more Trezor-style vectors. Out-of-scope-per-product at v0.8.x; if a future cycle adds JP support, this FOLLOWUP closes by mirroring the JP vector file into `tests/`.
- **Status:** `open` (deferred; product decision, not a regression).
- **Tier:** `v1+`
- **Companion:** None (single-repo concern; `ms-codec` carries the wordlist plumbing).

### `md-cli-unspendable-key-v0.19-error-string-stale-companion` — companion: md-cli error string still references "v0.19+"

- **Surfaced:** 2026-05-11, toolkit-repo Phase 0.B audit review r1 (commit `713178c`). Surfaced from this repo's audit pass but the fix lives in the sibling `descriptor-mnemonic` repo.
- **Where:** `bg002h/descriptor-mnemonic/crates/md-cli/src/main.rs:224`.
- **What:** Companion entry — see the primary entry in `descriptor-mnemonic/design/FOLLOWUPS.md` (`md-cli-unspendable-key-v0.19-error-string-stale`) for the full action item. No toolkit-side action; closure will happen when the md1-repo entry resolves.
- **Status:** `resolved` (2026-05-11 alongside the primary md1-side fix; see the md1 FOLLOWUP entry for the resolving commit).
- **Tier:** `cross-repo`
- **Companion:** `descriptor-mnemonic/design/FOLLOWUPS.md` — `md-cli-unspendable-key-v0.19-error-string-stale`

### `manual-v0.18-stale-md1-scenario-phrases` — quickstart/workflow chapters carry v0.17-era md1 phrases that no longer round-trip under v0.18

- **Surfaced:** 2026-05-09, PR #11 architect review (manual-mirror for descriptor-mnemonic v0.18 cycle).
- **Where:** `docs/manual/src/30-workflows/31-singlesig-steel.md` (lines 87–89), `docs/manual/src/20-quickstart/22-first-bundle.md` (~line 63), `docs/manual/src/20-quickstart/23-verify.md` (~line 24), `docs/manual/src/40-cli-reference/44-mk-cli.md` (~line 54). All reference the v0.17-era 3-chunk `md1zsxdspqqqpm6jzzqq...` scenario phrase set (3-of-3 multisig).
- **What:** descriptor-mnemonic v0.18 is a wire-format break (`Tag::TrUnspendable` removed, `key_index_width` formula changed). v0.17 phrases now reject under v0.18 with `Error::UnknownExtensionTag(0x05)`. PR #11 limited scope to CLI surface (per `manual-cli-surface-mirror` invariant). The scenario phrases need regenerating from source (run the `mnemonic` derivation pipeline against the abandon mnemonic with v0.18 binaries) and the chapters re-published.
- **Why deferred:** PR #11 maintains narrow CLI-surface-mirror scope. Scenario-content refresh is a separate concern that involves running the full 4-format pipeline and regenerating multiple chunked phrases. Local-only impact; toolkit CI runs `make lint` not `make verify-examples`.
- **Status:** `open`
- **Tier:** `v0.2` (next minor; non-blocking for descriptor-mnemonic v0.18 release).

### `lint-md-flag-coverage-vacuous-with-md_bin-true` — CI flag-coverage step skipped for md/ms/mnemonic via `MD_BIN=true` substitution

- **Surfaced:** 2026-05-09, PR #11 architect review.
- **Where:** `.github/workflows/manual.yml` invokes `make lint MNEMONIC_BIN=true MD_BIN=true MS_BIN=true MK_BIN=mk`. The `flag-coverage` step in `tests/lint.sh` runs `eval "$cmd <subcommand> --help"`; when `cmd=true`, the shell builtin `true` ignores all args and emits no flags, triggering the `warn "no flags parsed"` skip path.
- **What:** Only `mk` actually executes flag-coverage in CI (mk is `cargo install`'d in the workflow). md/ms/mnemonic flag-coverage is silently vacuous. Pre-existing gap; not introduced by PR #11. The `lint` claim "every CLI flag is documented" is therefore not enforced for 3 of 4 binaries in CI — manual `make lint MD_BIN=/path/to/md` runs catch flag drift, but no CI gate.
- **What to fix:** install `md`, `ms`, and `mnemonic` binaries in the manual.yml workflow (similar to how `mk` is installed) and pass them to `make lint` instead of `=true`.
- **Why deferred:** orthogonal to PR #11's CLI-surface-mirror scope; pre-existing infrastructure gap.
- **Status:** `open`
- **Tier:** `v0.2` (CI hardening; non-blocking).

### `manual-cli-surface-mirror` — manual mirrors the four-format CLI/API surface

- **Surfaced:** 2026-05-07, m-format-star user manual v0.1 release (`manual-v0.1.0` tag; PR #1).
- **Where:** `docs/manual/src/40-cli-reference/` (`41-mnemonic.md`, `42-md.md`, `43-ms.md`, `44-mk-codec-rust.md`); CI gate at `docs/manual/tests/lint.sh` `flag-coverage` step (per-`<binary, subcommand>` pair).
- **What:** v0.1 of the manual mirrors `mnemonic` (this repo), `md-cli` (`descriptor-mnemonic`), `ms-cli` (`mnemonic-secret`), and the `mk-codec` Rust API (`mnemonic-key`) verbatim against toolkit v0.8.0. **Any flag addition or removal in any of those four surfaces must touch `docs/manual/src/40-cli-reference/` in lockstep with the implementing PR**; the manual's `flag-coverage` lint step gates on missing flags. Companion entries: `manual-cli-surface-mirror` in `descriptor-mnemonic/design/FOLLOWUPS.md`, `mnemonic-secret/design/FOLLOWUPS.md`, and `mnemonic-key/design/FOLLOWUPS.md`.
- **Why filed:** the manual is a separate artifact (independent `manual-v*` versioning); without an explicit cross-repo mirror invariant, sibling-side flag changes would silently drift the manual.
- **Status:** `open` (mirror invariant active for the lifetime of `docs/manual/`)
- **Tier:** `cross-repo`

### `spec-5-5-kind-enum-gap` — SPEC §5.5 `kind` enum table omits `NetworkMismatch` and `FutureFormat`

- **Surfaced:** Phase 1 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §5.5.
- **What:** SPEC §5.5 enumerates `"kind"` JSON values as `"BadInput" | "Bip39" | … | "ModeViolation"` but doesn't list `NetworkMismatch` and `FutureFormat`. The implementation correctly returns those discriminants; the SPEC prose is just incomplete.
- **Why deferred:** SPEC-prose-only; no code change required. Update during the next SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — NetworkMismatch + FutureFormat added to §5.5 kind enum.`
- **Tier:** `v0.1-nice-to-have`

### `mk-codec-chunked-visual-grouping-helper` — mk-codec lacks a per-string visual grouping helper

- **Surfaced:** Phase 1 spike memo + Phase 1 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::chunk_mk1` (cross-repo: would consume a new `mk_codec::encode::render_grouped` if it existed).
- **What:** md-codec exposes `render_codex32_grouped(s, 5)` for engraving-friendly hyphenated 5-char groups; mk-codec has no equivalent. Toolkit's `chunk_mk1` falls back to space-separated 5-char groups via `chunk_5char`. v0.1 fixtures pin the space-separated behavior.
- **Why deferred:** non-blocking; functionally equivalent fallback. Library-API gap in mk-codec.
- **Status:** `open`
- **Tier:** `cross-repo`

### `plan-spike-md-codec-filler-bug` — IMPLEMENTATION_PLAN's `spike_md_codec.rs` snippet uses invalid SEC1 filler

- **Surfaced:** Phase 1 review r1 (Nit-1) + Task 1.1 spike memo.
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 1.1, `spike_md_codec.rs` snippet (~line 232–260).
- **What:** Plan-given snippet uses `[0x42; 65]` as `tlv.pubkeys` filler, which violates the SEC1-compressed-pubkey prefix invariant (must be 0x02/0x03) and panics with `InvalidXpubBytes`. Spike memo documents the working filler `[0x11; 32] || 0x02 || [0x22; 32]` from `md_codec::identity::deterministic_xpub`. Plan source not patched — future readers running the snippet verbatim will trip the same panic.
- **Why deferred:** spike memo supersedes plan source; cosmetic plan-source bug.
- **Status:** `resolved (this commit, 2026-05-13) — Task 1.1 snippet now uses 32B 0x11 chain_code || 0x02 SEC1 prefix || 32B 0x22 x-coordinate.`
- **Tier:** `v0.1-nice-to-have`

### `plan-trezor-24-fingerprint-stale` — IMPLEMENTATION_PLAN has wrong 24-word zero-entropy master fingerprint

- **Surfaced:** Task 2.1 implementer (verified via spike harness `/tmp/toolkit-spike/spike_trezor_fp.rs`).
- **Where:** `design/IMPLEMENTATION_PLAN_mnemonic_toolkit_v0_1.md` Task 2.1 test assertion (~line 1540) + Task 2.3 commit-message body.
- **What:** Plan asserts `73c5da0a` as the Trezor 24-word "abandon × 23 art" master fingerprint. That value is the **12-word** "abandon × 11 about" vector's fingerprint (rust-miniscript test corpus). Correct 24-word fingerprint is `5436d724`. Handoff doc was corrected during execution; plan source unpatched.
- **Why deferred:** test code uses correct value; only plan documentation is stale.
- **Status:** `resolved (this commit, 2026-05-13) — plan now references 5436d724 (24-word fingerprint) in all 3 sites.`
- **Tier:** `v0.1-nice-to-have`

### `friendly-mk-codec-mixedcase-wording` — `friendly_mk_codec` `MixedCase` text word-order differs from SPEC §6.4.4

- **Surfaced:** Phase 3 review r1 (L-1).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs:friendly_mk_codec` (`MixedCase` arm).
- **What:** SPEC §6.4.4 row says `"mixed case in mk1 input string"`. Code says `"mk1 mixed case in input string"`. Functionally equivalent; word order differs.
- **Why deferred:** no integration test pins the byte-exact text yet; cosmetic.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `bundle-emit-bypasses-chunk-mk1-alias` — `bundle.rs::emit()` calls `chunk_5char` directly for mk1; `chunk_mk1` alias dead

- **Surfaced:** Phase 3 review r1 (L-2) + Phase 5 review r1 (L-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/format.rs::chunk_mk1`.
- **What:** `chunk_mk1` is a reserved alias for `chunk_5char`, retained against the future mk-codec grouping helper (see `mk-codec-chunked-visual-grouping-helper`). `bundle.rs::emit` calls `chunk_5char` directly, leaving `chunk_mk1` flagged as dead code. Switch the call site to `chunk_mk1` so the swap point is single-edit.
- **Why deferred:** functionally identical; one-line cleanup.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `watch-only-stderr-warning-suborder` — depth advisory ordering vs account-index hazard unspecified

- **Surfaced:** Phase 3 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_watch_only`.
- **What:** Watch-only path emits the conditional depth advisory before the unconditional account-index hazard. SPEC §5.2 lists "watch-only mode warning" as item 3 without specifying the sub-order between these two. Phase 5 fixtures don't cover stderr ordering.
- **Why deferred:** SPEC-ambiguous; Phase 5 doesn't pin the ordering.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `spec-2-2-2-vs-5-4-checks-count-prose` — SPEC §2.2.2 prose says "four checks" but §5.4 schema mandates 9-element array

- **Surfaced:** Phase 4 review r1 (L-1).
- **Where:** `design/SPEC_mnemonic_toolkit_v0_1.md` §2.2.2 vs §5.4.
- **What:** §2.2.2 lists 4 substantive watch-only checks; §5.4 schema (line 552) requires all 9 check-name slots populated, with `skipped` for non-applicable. Implementation follows §5.4 (correct). §2.2.2 prose should clarify "4 substantive (5 of the 9 schema slots are `skipped` per §5.4)".
- **Why deferred:** SPEC-internal inconsistency; implementation behavior is correct per the schema.
- **Status:** `resolved (this commit, 2026-05-13) — §2.2.2 prose clarified: "four substantive checks" with explicit §5.4 9-slot schema reference.`
- **Tier:** `v0.1-nice-to-have`

### `bundle-mismatch-card-static-str-constraint` — `BundleMismatch.card: &'static str` constrains future runtime-id callers

- **Surfaced:** Phase 4 review r1 (L-2). Confirmed as Phase 0 mandatory fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-1).
- **Where:** `crates/mnemonic-toolkit/src/error.rs::ToolkitError::BundleMismatch`.
- **What:** Field type was `&'static str`. v0.2 multisig emits per-cosigner card identifiers like `"mk1[0]"` that are runtime-formatted; `&'static str` would force a breaking field-type change mid-v0.2-cycle. Resolved as part of v0.2 Phase 0.
- **Status:** `resolved 9396a58 — field changed to String; test construction sites updated to .into(); doc-comment clarified.`
- **Tier:** `v0.2`

### `verify-bundle-text-mode-trailing-space` — `"{}: {} {}"` produces trailing space when `detail` is empty

- **Surfaced:** Phase 4 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run` text-mode output.
- **What:** Skipped checks with empty `detail` render as `"md1_xpub_match: skipped "` (trailing space). SPEC §5.4 only pins JSON byte-exact; text mode is unpinned.
- **Why deferred:** cosmetic; not test-covered.
- **Status:** `resolved by v0.5.0 Phase F (commit 85c678b) — branch on detail.is_empty() at 3 emit sites`
- **Tier:** `v0.1-nice-to-have`

### `error-allow-comments-staleness` — `error::Result<T>` and `BundleMismatch` doc-comments will rot

- **Surfaced:** Phase 4 review r1 (N-1, N-2) + Phase 5 review r1 (N-2). Bundled into Phase 0 fixup by 2026-05-05 v0.1 audit (`design/audit-v0_1-for-v0_2-extension.md` IMP-2).
- **Where:** `crates/mnemonic-toolkit/src/error.rs` `Result` alias + `BundleMismatch` variant doc.
- **What:** `Result<T>` allow-comment said "reserved for in-crate use" but the type is `pub type` (exported). `BundleMismatch` doc-comment said "Constructed by integration tests in Phase 5" — stale once v0.2 wires the variant as a live runtime error.
- **Status:** `resolved 9396a58 — Result<T> comment now reads "Convenience alias; exported for downstream-crate use." BundleMismatch comment now reads "Exit-4 verify-bundle mismatch variant; card identifies the mismatching card (e.g., mk1, md1, or mk1[N] for multisig cosigner N)."`
- **Tier:** `v0.1-nice-to-have`

### `cli-watch-only-test-hardcodes-fingerprint` — `cli_bundle_watch_only.rs` hardcodes `5436d724` rather than reading from decoded mk1

- **Surfaced:** Phase 5 review r1 (L-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_bundle_watch_only.rs`.
- **What:** Test extracts the xpub from the mk1 fixture via `mk_codec::decode` (correct), but passes `"5436d724"` as the master-fingerprint argument literally. Works because the Trezor 24-word zero vector's fingerprint is constant; future vector swap requires updating the fingerprint in two places. Read it from `card.origin_fingerprint` instead.
- **Why deferred:** works; two-place edit risk only.
- **Status:** `resolved (this commit, 2026-05-13) — both tests read fp_hex from card.origin_fingerprint via .to_string(); cargo test --test cli_bundle_watch_only passes.`
- **Tier:** `v0.1-nice-to-have`

### `changelog-sha-pin-no-reproduction-command` — CHANGELOG SHA pin doesn't document how to reproduce it

- **Surfaced:** Phase 5 review r1 (N-1).
- **Where:** `CHANGELOG.md` Wire-format SHA pin section.
- **What:** SHA `81828299c927783d915108f32c9752b3dbf815c1caba5b6f6e7ce7b810ddcbf6` is documented as `sha256(crates/mnemonic-toolkit/tests/vectors/v0_1/)` but doesn't specify the exact reproduction command (`shasum -a 256 *.txt | sort | shasum -a 256`). Verifiers may need to guess.
- **Why deferred:** verifiers can re-derive; doc-only clarity gap.
- **Status:** `resolved (2026-05-13 survey verified moot) — CHANGELOG.md v0.2 section (lines 1296-1301) already includes the reproduction command \`shasum -a 256 ... | sort | shasum -a 256\` with explicit "(resolves v0.1 FOLLOWUPS N-1)" attribution; this FOLLOWUPS entry's status update was missed in the v0.2 cycle.`
- **Tier:** `v0.1-nice-to-have`

### `cli-mode-violations-byte-exact-naming` — test names say "byte_exact" but use `str::contains`

- **Surfaced:** Phase 5 review r1 (N-3).
- **Where:** `crates/mnemonic-toolkit/tests/cli_mode_violations.rs`.
- **What:** Several test names use the suffix `_byte_exact` but the assertions use `predicate::str::contains(...)` (substring match). Tests are correct; naming overstates assertion strictness. Either rename to `_substring` or tighten the assertions to full-stderr equality (and pin the byte-exact stderr in fixtures).
- **Why deferred:** assertion strength is sufficient for current SPEC pinning; naming is the only mismatch.
- **Status:** `resolved (2026-05-13 survey verified moot) — \`tests/cli_mode_violations.rs\` no longer exists; the \`_byte_exact\` naming pattern is now distributed across per-feature test files (cli_bundle_full, cli_export_wallet_*, etc.) where the original test's scope was absorbed.`
- **Tier:** `v0.1-nice-to-have`

### `phase-2-review-byte-determinism-blind-spot` — process: byte-determinism invariants need a spike, not just a review

- **Surfaced:** Phase 5 implementer caught the bug; Phase 2 r1 + r2 reviews missed it.
- **Where:** Process / `feedback_spike_before_locking_wire_format` memory rule.
- **What:** Phase 2 reviews looked at code correctness against SPEC §4 but didn't run encode twice and diff the bytes. The result: `mk_codec::encode` drew `chunk_set_id` from CSPRNG, which broke v0.1's byte-reproducible-output contract. The fix (`derive_mk1_chunk_set_id` + `encode_with_chunk_set_id`) shipped in the Phase 5 release commit (`f2bd20a`). Process improvement: when a phase locks wire-format invariants that downstream phases will SHA-pin, the per-phase review checklist should include "encode twice, assert identical bytes".
- **Why deferred:** post-mortem item; resolved via the v0.1.0 release fix. Lesson worth carrying forward.
- **Status:** `resolved f2bd20a — Phase 5 fix shipped; process lesson captured here.`
- **Tier:** `v0.1-nice-to-have`

### `mk1-bip-chunk-set-id-determinism-guidance` — mk1 BIP recommendation for deterministic encoders

- **Surfaced:** Phase 5 byte-determinism fix (`f2bd20a`) — the toolkit-side derivation needs lifting into the mk1 BIP so other implementations producing reproducible corpora reach the same wire bits. Companion: same-id entry in `mnemonic-key/bip/bip-mnemonic-key.mediawiki`.
- **Where:** `bip/bip-mnemonic-key.mediawiki` String-layer header section in `mnemonic-key`.
- **What:** Toolkit shipped a `derive_mk1_chunk_set_id(&policy_id_stub)` helper deriving 20 bits from the leading bytes of the policy_id_stub. mk1 BIP edited to recommend this pattern (with the explicit formula `(stub[0] << 12) | (stub[1] << 4) | (stub[2] >> 4)`) and clarify decoders MUST accept any 20-bit value.
- **Why deferred:** mk1 BIP is a sibling-repo asset; toolkit's fix landed first.
- **Status:** `resolved 87bbc11 (mnemonic-key@main) — mk1 BIP §"String-layer header" updated 2026-05-04 with deterministic-encoder guidance + decoder-acceptance clarification. Pushed to bg002h/mnemonic-key.`
- **Tier:** `cross-repo`

### `dead-assert-tautological` — `synthesize.rs` invariant 1 debug-assert is tautological by construction

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs:99` (`debug_assert_eq!(&card.policy_id_stubs[0], &stub)`).
- **What:** `stub` is computed from `policy_id.as_bytes()[..4]` and immediately passed as `policy_id_stubs[0]`. The assertion can never fail at the construction site. Phase 2 r1 originally flagged this as L-4. Pre-existing; meaningful assertion is invariant 2 (`is_wallet_policy()`).
- **Why deferred:** v0.2 multisig will need a meaningful assertion that loops over all per-cosigner stubs; resolve as part of v0.2 Phase C.
- **Status:** `resolved (this commit, 2026-05-13) — tautological debug_assert_eq removed from both single-sig synthesize paths in synthesize.rs (now lines 139, 171); the meaningful is_wallet_policy assert is retained. v0.2 multisig's proper looped assertion never materialized; removing the dead code rather than waiting indefinitely.`
- **Tier:** `v0.2`

### `dead-inner-guard-bundle-watch-only` — redundant `--xpub`-needs-`--master-fingerprint` guard inside `bundle_watch_only`

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:200` (inside `bundle_watch_only`).
- **What:** A redundant guard exists that would emit `BadInput` (exit 1) if `--master-fingerprint` is missing. Unreachable in practice — the mode-violation pre-check at `cmd/bundle.rs:93` rejects the same condition earlier with exit 2 + byte-exact §6.6 text. Future-refactor inconsistency risk.
- **Why deferred:** not currently triggered; v0.2 will refactor mode dispatch and naturally clean this up.
- **Status:** `open`
- **Tier:** `v0.2`

### `friendly-mapper-unit-test-gaps` — friendly-mapper unit tests cover only 3 of ~70 match arms

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-3).
- **Where:** `crates/mnemonic-toolkit/src/friendly.rs::tests`.
- **What:** Unit tests cover `friendly_bip39::UnknownWord`, `friendly_ms_codec::WrongHrp`, `friendly_mk_codec::PathTooDeep`. Untested at unit level: 4 of 5 `friendly_bip39`, all 3 `friendly_bitcoin`, 8 of 9 `friendly_ms_codec`, 21 of 22 `friendly_mk_codec`, all 41 `friendly_md_codec`. Integration tests likely exercise some paths end-to-end but unit isolation is thin.
- **Why deferred:** v0.2 will add new error paths through these mappers; expand the tests in lockstep with v0.2 Phase E.
- **Status:** `open`
- **Tier:** `v0.2-nice-to-have`

### `hex-dep-unused` — `hex = "0.4"` declared in Cargo.toml but unused in non-test source

- **Surfaced:** v0.1 audit 2026-05-05 (LOW-4).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml:27`.
- **What:** No `use hex` statement in any source module. Inert dependency carried from ms-cli precedent or SPEC §10.3 dep list.
- **Why deferred:** user's `feedback_dont_drop_reserved_deps` rule applies — confirm with user before removal. v0.2 may use `hex` for new error-message formatting (e.g., printing fingerprints in mode-violation output), in which case the dep activates naturally.
- **Status:** `open`
- **Tier:** `v0.1-nice-to-have`

### `parse_template-regex-line-ref` — SPEC v0.3 §4.9 step 2 cites wrong line range

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §4.9 step 2.
- **What:** Step 2 cites `descriptor-mnemonic/crates/md-cli/src/parse/template.rs:19-27` for the placeholder regex; the actual `Regex::new` call is at `:25-27` (line 19-24 are imports/doc-comments). Docs-only nit — implementation will read the actual regex from the source.
- **Why deferred:** non-blocking; can be patched alongside any v0.3 SPEC revision.
- **Status:** `resolved (this commit, 2026-05-13) — §4.9 step 2 line range updated to \`parse/template.rs:25-27\`.`
- **Tier:** `v0.3-nice-to-have`

### `unsupported-fragment-error-style` — SPEC v0.3 §6.8 error message text is verbose

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §6.8 (error message wording).
- **What:** The message reads `unsupported miniscript fragment: <fragment-string>; v0.3 walker covers BIP-388 surface modulo multi-leaf tap trees (deferred to v0.4)`. This is verbose for a CLI error; a tighter form (e.g. drop the parenthetical) would be friendlier.
- **Why deferred:** SPEC pins the message for byte-exactness; can be revisited at impl time if friendlier wording surfaces. Not blocking.
- **Status:** `open`
- **Tier:** `v0.3-nice-to-have`

### `walker-backport-to-md-cli` — toolkit's expanded walker should be backported to md-cli

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** cross-repo: `mnemonic-toolkit/crates/mnemonic-toolkit/src/parse_descriptor.rs` ↔ `descriptor-mnemonic/crates/md-cli/src/parse/template.rs`.
- **What:** v0.3 toolkit ships an expanded `walk_miniscript_node` covering all 24 v0.3-NEW `Terminal` arms (hash terminals, timelocks, wrappers, AND/OR/Thresh). md-cli's walker is the inspiration but currently rejects all of these. Backporting (or extracting both into a shared crate `descriptor-walker`) avoids divergence.
- **Why deferred:** scope of v0.3 is toolkit-only by user direction. Cross-repo coordination cycle in v0.4.
- **Status:** `open`
- **Tier:** `v0.4-cross-repo`

### `spike-report-citation` — v0.3 SPEC §9 Q2 closure should cite SPIKE report

- **Surfaced:** v0.3 SPEC architect review r2 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §9 Q2 closure.
- **What:** §9 Q2 declared "moot — v0.3 implements its own walker arms for hash terminals." Pre-Phase-A SPIKE produced `design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §2 confirming hash-terminal round-trip. §9 Q2 updated to cite the report.
- **Status:** `resolved 2026-05-05` (closed inline with SPIKE report patches).
- **Tier:** `v0.3`

### `synthesize-descriptor-fn-naming` — single-vs-split synthesize entry-point decision

- **Surfaced:** v0.3 SPEC § resolved at IMPLEMENTATION_PLAN drafting 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs` (Phase C of v0.3 plan).
- **What:** v0.3 SPEC §10 originally named `synthesize_descriptor_full` / `synthesize_descriptor_watch_only` (mirroring v0.2's two-function shape). v0.3 plan resolves to a single `synthesize_descriptor` entry point that dispatches single-sig vs multisig internally. This is slightly asymmetric with v0.2's pattern.
- **Why deferred:** flagged for Phase C reviewer to confirm the single-entry-point shape doesn't regress code clarity. Not a blocker.
- **Status:** `resolved by IMPLEMENTATION_PLAN_v0_3 Phase C.1` (single entry point chosen)
- **Tier:** `v0.3`

### `v0.2-spec-§8-tier-citation` — v0.3 SPEC §8 citation against v0.2 SPEC §8

- **Surfaced:** v0.3 SPEC architect review r3 2026-05-05.
- **Where:** `design/SPEC_mnemonic_toolkit_v0_3.md` §8 deferred-items table (K-of-N row).
- **What:** §8 cites v0.2 tier of K-of-N share encoding as "v0.3 (gates on ms-codec v0.2)". Verify against v0.2 SPEC §8 verbatim language at impl time for citation accuracy.
- **Why deferred:** non-blocking; doc-only.
- **Status:** `resolved (2026-05-13 survey verified) — v0.3 SPEC §8 line 313 cites v0.2 tier of K-of-N as "v0.3 (gates on ms-codec v0.2)", matching v0.2 SPEC §8 line 582 outcome wording verbatim. Citation is accurate.`
- **Tier:** `v0.3-nice-to-have`

### `ctx-for-descriptor-heuristic-misroutes` — Phase A `ctx_for_descriptor` is string-prefix heuristic

- **Surfaced:** v0.3 Phase A end-of-phase architect review I-2 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Replaced string-prefix heuristic with post-resolve n-based classification inside `parse_descriptor`: `n == 1 → SingleSig`, `n ≥ 2 → MultiSig`. The dead `ctx_for_descriptor` function was removed.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `parse-descriptor-allow-dead-code-audit` — module-level `#![allow(dead_code)]` audit

- **Surfaced:** v0.3 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Resolved:** v0.3 Phase C end-of-phase r1 (2026-05-05). Lifted module-level `#![allow(dead_code)]`. Two items remained dead at the binary-compile boundary (`DescriptorMode` enum + `determine_mode` fn, used only in tests + Phase D verify-bundle re-parse path); both received per-item `#[allow(dead_code)]`.
- **Status:** `resolved by Phase C.6 r2 (2026-05-05)`
- **Tier:** `v0.3`

### `descriptor-mode-engraving-card` — engraving card omitted in descriptor mode

- **Surfaced:** v0.3 Phase C end-of-phase architect review L-5 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` `descriptor_mode_emit` (Phase C.6).
- **What:** `engraving_card: None` for descriptor mode. The existing `engraving_card()` builder takes a `CliTemplate` + path-family + `EngravingMode`, which descriptor mode lacks. v0.3 ships without a descriptor-mode card; v0.4 should add a descriptor-aware engraving card (custom text including the descriptor string + per-cosigner xpub origins).
- **Why deferred:** out of v0.3 scope; engraving card logic is template-coupled.
- **Status:** `open`
- **Tier:** `v0.4`

### `engraving-card-unified-1-master-card` — Phase E unified engraving card deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase E scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/format.rs::engraving_card` + `EngravingMode` enum + `crates/mnemonic-toolkit/src/cmd/bundle.rs` per-mode card emit sites.
- **What:** SPEC §5.5 specifies a single unified `BundleInputForCard` shape + `engraving_card_unified` render function emitting one master card per bundle (in place of v0.2/v0.3's per-mode `EngravingMode` variants). Phase E was originally scoped to land this in v0.4.0 with deprecation of `EngravingMode::*`; deferred to v0.4.1 because it is tightly coupled to the BundleJson schema-4 cutover and the multi-source synthesis path (the unified card needs `MsField` + per-slot blocks). Will land in lockstep with `bundle-json-schema-4-cutover`.
- **Why deferred:** scope-coupling to schema-4 cutover; foundation-only Phase D made standalone Phase E delivery low-value.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `verify-bundle-9-3plus6n-forensics` — Phase G verify-bundle 9/3+6N parity + per-cell forensics deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase G scope decision 2026-05-05 (autonomous-mode time risk).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_*` + `crates/mnemonic-toolkit/src/format.rs::VerifyCheck`.
- **What:** SPEC §5.7 specifies (a) descriptor-mode emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder), (b) `VerifyCheck` gains four forensic fields (`expected`, `actual`, `diff_byte_offset`, `decode_error`), (c) verify-bundle dispatches on `schema_version` for schema-4 BundleJson with per-slot `MsField` array. All three sub-deliverables depend on the schema-4 cutover landing first. Bip388-distinctness symmetric enforcement (SPEC §4.11.c) IS shipping in v0.4.0 (Phase A wired `Bip388VerifyDistinctness` into `descriptor_mode_verify_run`).
- **Why deferred:** depends on `bundle-json-schema-4-cutover`; will land in lockstep with v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bundle-json-schema-4-cutover` — full BundleJson schema-4 cutover deferred from v0.4.0 to v0.4.1

- **Surfaced:** v0.4 Phase D scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/format.rs::BundleJson` + `crates/mnemonic-toolkit/src/cmd/bundle.rs::emit` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` + `crates/mnemonic-toolkit/src/synthesize.rs::Bundle`.
- **What:** v0.4.0 ships the `MsField = Vec<String>` type alias + multi-source synthesis primitives as a foundation, but DEFERS the full `BundleJson.ms1: Option<String>` → `ms1: MsField` migration + `schema_version: "3" → "4"` bump + verify-bundle schema-4 dispatch to v0.4.1. v0.4.0 retains the schema-3 envelope so all existing v0.2/v0.3 fixtures + JSON integration tests pass byte-identically. v0.4.1 lands the cutover with: (a) BundleJson.ms1 → MsField; (b) Bundle.ms1 → Vec<String>; (c) all integration test JSON assertions updated; (d) verify-bundle schema_version dispatch (read schema_version FIRST per SPEC §5.6); (e) regenerate or update v0.2/v0.3 carry-forward tests under the new envelope shape per SPEC §5.6 cross-schema invariant; (f) synthesize_multisig_multisource + synthesize_multisig_hybrid wired into bundle::run via BundleMode dispatch (Phase C foundation already in place); (g) **bundle::run top-level dispatch rewiring**: in v0.4.0 `args.slot` is parsed by clap into `BundleArgs.slot: Vec<SlotInput>` but `bundle::run` itself never reads it. v0.4.1 must wire `expand_legacy_to_slots(args.slot, ...)` → `validate_slot_set(&slots)?` → `detect_bundle_mode(&slots)?` → match-arm dispatch into the new `synthesize_multisig_multisource` / `synthesize_multisig_hybrid` paths AND rewrite the legacy `bundle_full` / `bundle_watch_only` / `bundle_multisig_*` calls to flow through the same SlotInput-driven path. This is a top-level surgery in `cmd/bundle.rs::run` itself, not just additions to the synthesis helper crate.
- **Why deferred:** scope risk in autonomous v0.4.0 release window — full surgery touches ≥10 source files + ~15 test assertions + fixture envelopes; landing without user oversight risks bugs the foundation-only approach avoids.
- **Status:** `open`
- **Tier:** `v0.4.1`

### `bip388-distinctness-path-normalization-phase-b-decision` — typed-vs-raw path semantics in check_key_vector_distinctness

- **Surfaced:** v0.4 Phase A end-of-phase architect review L-1 (2026-05-05).
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs:1049` (`check_key_vector_distinctness`); SPEC `design/SPEC_mnemonic_toolkit_v0_4.md` §4.11.b.
- **What:** Phase A compares `cs[i].path.to_string()` on typed `bitcoin::bip32::DerivationPath`. The bitcoin library normalizes `48h/0h/0h/2h` ↔ `48'/0'/0'/2'` at `from_str` time, so collision detection is normalization-aware. SPEC §4.11.b says "raw user-supplied path string ... no path canonicalization". In Phase A this is safe because all paths arrive through the typed lex/cosigner parser; in Phase B the `--slot @N.path=` raw string flows into the binding directly. Phase B must lock whether `CosignerKeyInfo.path` stores typed `DerivationPath` (normalizing) or raw `String` (preserving), then update SPEC §4.11.b's normalization-domain paragraph in lockstep.
- **Why deferred:** Phase A's typed approach is correct under the v0.3 binding model; the decision is a Phase B design choice (slot input parsing).
- **Status:** `resolved by v0.5.0 Phase C.1 (commit 4a650aa) — typed DerivationPath equality replaces raw-string in check_key_vector_distinctness`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-and-full-forensics-rollout-v0.4.4` — Phase P.1-P.5 deferred from v0.4.3 to v0.4.4 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-call-sites-rollout-v0.4.5 (2026-05-06)`. v0.4.4 P.1+P.2 landed the `emit_verify_checks` helper foundation (#[allow(dead_code)] with 4 unit tests + SuppliedCards struct + watch-only short-circuit + multisig TODO stub). The ~78-site call-site refactors (run_full / run_multisig / descriptor_mode_verify_run consolidation + descriptor-mode 9/3+6N parity + watch-only test migration) deferred again to v0.4.5 per the v0.4.4 plan scope reduction.

- **Surfaced:** v0.4.3 Phase P scope decision 2026-05-06 (P.0 struct shape correction landed; P.1-P.5 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites + new `emit_verify_checks` helper + descriptor-mode 9/3+6N parity refactor).
- **What:** v0.4.3 P.0 corrected the VerifyCheck struct shape per SPEC §5.7 (`result: &'static str` → `passed: bool`). The full SPEC §5.7 rollout — `emit_verify_checks` helper + refactor of run_full / run_multisig / descriptor_mode_verify_run + per-cell forensic field population at every push site + descriptor-mode 9/3+6N parity (closes `verify-bundle-9-3plus6n-descriptor-mode-parity` simultaneously) + skipped-check decode_error population — is deferred to v0.4.4. v0.4.3 ships passing checks with `passed: false` set on failures but forensic fields (expected/actual/diff_byte_offset/decode_error) only populated at the one v0.4.1 J.7 proof-of-shape site.
- **Why deferred:** scope-safety in v0.4.3 release window. Full helper + refactor estimated at ~800-1000 lines deleted in verify_bundle.rs alongside ~70 push-site updates.
- **Tier:** `v0.4.4`

### `verify-bundle-multisig-helper-full-mode-unit-test` — add unit-level coverage for emit_multisig_checks full-mode ms1 branch

- **Surfaced:** v0.4.5 final cross-phase review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` helper_tests mod.
- **What:** v0.4.5 ships `helper_multisig_watch_only_emits_3plus6n_checks_in_spec_order` (renamed from `_full_` after review confirmed the fixture exercises watch-only synthesis with empty `expected.ms1`). The full-mode multisig ms1 branch (`emit_multisig_checks` lines ~1096-1159: substantive ms1_decode + ms1_entropy_match per cosigner) has end-to-end coverage via `cli_bundle_multisig.rs` integration tests but no isolated unit-level test. Add a companion `helper_multisig_full_emits_3plus6n_checks_in_spec_order` that uses `synthesize_multisig_full` (or constructs a synthetic Bundle with non-empty `expected.ms1` strings) to exercise the substantive ms1 path.
- **Why deferred:** integration coverage is sufficient for v0.4.5; the unit-level gap is test isolation hygiene, not behavior.
- **Status:** `resolved by v0.5.0 Phase B.1 (commit 9f1a4e7) — helper_multisig_full_emits_3plus6n_checks_in_spec_order added`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-positional-fallback-condition-cosmetic` — cosmetic dead `unwrap_or(false)` in card_for_cosigner positional fallback

- **Surfaced:** v0.4.5 final cross-phase review L-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` positional fallback condition).
- **What:** Condition `supplied_md_decoded.is_err() || supplied_md_decoded.as_ref().map(|d| d.tlv.pubkeys.is_none()).unwrap_or(false)` — the `.map().unwrap_or(false)` chain is unreachable when `supplied_md_decoded.is_err()` short-circuits OR semantically dead inside the Ok branch. Refactor to `match` for clarity.
- **Why deferred:** cosmetic; no logic impact.
- **Status:** `resolved by v0.5.0 Phase B.2 (commit 9f1a4e7) — refactored to clean match expression`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-md1-xpub-match-set-equality` — md1_xpub_match uses ordered Vec equality

- **Surfaced:** v0.4.5 Phase P.4 review I-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (md1_xpub_match arm).
- **What:** Helper compares `expected_md1.tlv.pubkeys` and `supplied_md1.tlv.pubkeys` as ordered `Vec<[u8; 65]>` via `==`. SPEC §5.7 line 103 says the shared `md1_xpub_match` confirms "all N pubkeys match expected" — semantics are arguably set-equality (the script-level pubkey set must be identical), not ordered. Template-mode synthesis preserves cosigner-index order, so ordered equality is correct for that path. Descriptor-mode verify-bundle (P.5) where the user supplies a descriptor with arbitrary `@N` placement could false-fail under ordered equality even when the logical pubkey set is identical.
- **Why deferred:** template-mode P.4 doesn't trigger this; descriptor-mode P.5 lands in v0.4.5 but the SPEC clarification needed to choose set-vs-ordered semantics is itself open. Re-evaluate after P.5 implementation surfaces real-world cases.
- **Status:** `resolved by v0.5.0 Phase B.3 (commit 9f1a4e7) — sort-then-compare multiset equality`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-cosigner-mapping-diagnostic` — distinguish "card not supplied" from "xpub not in policy"

- **Surfaced:** v0.4.5 Phase P.4 review I-2 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` (`card_for_cosigner` mapping + `mk1_decode[i]` emission).
- **What:** When supplied md1 decodes successfully and pubkeys-TLV is present but a supplied mk1 card's xpub matches no entry, `card_for_cosigner[i]` stays `None` and `mk1_decode[i]` emits "skipped: mk1[i] not supplied or decode failed". This conflates two distinct failure modes:
  1. User forgot to supply --mk1 for cosigner i.
  2. User supplied an mk1 card whose xpub doesn't appear in the descriptor's pubkey set (wrong-key attack scenario).
- **Why deferred:** diagnostic clarity, not correctness. Could split into two distinct check names or add a per-card "policy-membership" field.
- **Status:** `resolved by v0.5.0 Phase B.4 (commit 9f1a4e7) — MappingFailure enum with precedence XpubNotInPolicy > DecodeFailed > NotSupplied`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-multisig-missing-ms1-passes-true` — full-mode multisig with no --ms1 supplied reports passed=true

- **Surfaced:** v0.4.5 Phase P.4 review N-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_multisig_checks` ("Expected substantive but supplied missing/empty" branch).
- **What:** When `expected.ms1[i]` is non-empty (full-mode) but the caller supplies no corresponding --ms1 value, `ms1_decode[i]` and `ms1_entropy_match[i]` are emitted with `passed: true, decode_error: "skipped: ms1[i] not supplied"`. A full-mode multisig bundle verified without supplying any ms1 cards thus reports `result: ok` if mk1+md1 match. SPEC §5.7 line 104 specifies "skipped: watch-only slot" semantics ONLY for `ms1[i] == ""` (watch-only sentinel); the missing-but-expected case is unspecified.
- **Why deferred:** policy decision — should missing-but-expected ms1 be a hard fail (like missing mk1[i])? Or stays as soft skip (current behavior)? Defer for SPEC clarification.
- **Status:** `resolved by v0.5.0 Phase B.5 (commit 9f1a4e7) — SPEC §5.7 four-case table, case 4 passed=false on missing-but-expected ms1`
- **Tier:** `v0.4.5-nice-to-have`

### `verify-bundle-watch-only-spurious-ms1-handling` — watch-only with user-supplied --ms1 produces ms1_entropy_match: fail

- **Surfaced:** v0.4.5 Phase P.3 review L-1 (2026-05-06).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_watch_only` + `emit_verify_checks` watch-only short-circuit.
- **What:** Pre-v0.4.5 `watch_only_checks` ignored `args.ms1` (always emitted "watch-only mode: no entropy known to toolkit" passing-vacuously). Post-v0.4.5 P.3 wire-up: run_watch_only synthesizes the watch-only Bundle (`ms1: vec![""]`) and the helper compares supplied vs expected. If user spuriously supplies `--ms1 <non-empty>` in watch-only mode, `ms1_decode` runs against the supplied string, then `ms1_entropy_match` fails because `expected="" ≠ supplied=non-empty`. Behavior change vs v0.4.4: arguably more useful (tool flags the user's mistake) but not formally specified.
- **Why deferred:** non-blocking; SPEC §5.7 doesn't address this edge. Decide whether to short-circuit in run_watch_only (ignore args.ms1, force-empty SuppliedCards.ms1) or document the behavior in SPEC §2.2.2.
- **Status:** `resolved by v0.5.0 Phase C.2 (commit 4a650aa) — SPEC §5.7 case 1 codification + integration test`
- **Tier:** `v0.4-nice-to-have`

### `verify-bundle-helper-foundation-cleanup-v0.4.5` — 2 Low/Nit cleanups from v0.4.4 final cross-phase review

- **Surfaced:** v0.4.4 final cross-phase review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::emit_verify_checks` (and surrounding helper code).
- **What:**
  - **L-1** — Doc-comment in `emit_verify_checks` cites SPEC §5.8 for the watch-only sentinel discrimination (`expected.ms1[i].is_empty()`); the watch-only short-circuit logic actually lives in §5.7. §5.8 is the MsField wire-format definition. Fix: change `§5.8` → `§5.7` in the doc-comment near `verify_bundle.rs:1882`.
  - **L-2** — `MkField::Multi` arm in single-sig branch returns early with potentially fewer than 9 checks; this path is unreachable in production (single-sig bundles always have `MkField::Single`) and is documented with a comment, but the early return is an implicit invariant assumption. Fix: replace early return with `unreachable!("single-sig branch reached MkField::Multi — invariant violation")` or `debug_assert!(false, ...)`. Land alongside P.3 wiring in v0.4.5.
- **Why deferred:** non-blocking nits; helper is `#[allow(dead_code)]` so no runtime exposure. Bundle with the v0.4.5 P.3-P.7 call-site rollout.
- **Status:** `resolved by v0.4.5 Phase L (commit 40638c8)` — L-1 §5.7 cited; L-2 `unreachable!()` invariant assertion in place.
- **Tier:** `v0.4.5`

### `verify-bundle-helper-call-sites-rollout-v0.4.5` — Phase P.3-P.7 call-site rollout deferred from v0.4.4 to v0.4.5

- **Surfaced:** v0.4.4 Phase P scope decision 2026-05-06 (P.1+P.2 helper foundation landed; P.3-P.7 deferred for scope safety).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::run_full` + `run_multisig` + `descriptor_mode_verify_run` + `crates/mnemonic-toolkit/tests/watch_only_tests.rs` + new integration tests for full forensic rollout.
- **What:** v0.4.4 P.1+P.2 shipped `emit_verify_checks` (single-sig 9-check shape per SPEC §5.7 ordering), `SuppliedCards<'a>` struct, `emit_md1_checks` shared md1 helper, watch-only short-circuit (passed=true + decode_error="skipped: watch-only slot"), multisig TODO stub returning `[VerifyCheck { name: "TODO_multisig_v0_4_5", passed: false, ... }]`, and 4 helper unit tests. The helper is `#[allow(dead_code)]`; v0.4.5 wires it up:
  - **P.3** — `run_full` (single-sig template-mode) calls `emit_verify_checks(SuppliedCards::singlesig(...), false)` and replaces ~30 push sites.
  - **P.4** — `run_multisig` (template-mode multisig) replaces TODO stub with the 3-shared-checks + 6N-per-cosigner pattern; emits real forensics.
  - **P.5** — `descriptor_mode_verify_run` emits the 9 / 3+6N schema (closes `verify-bundle-9-3plus6n-descriptor-mode-parity`) via the helper.
  - **P.6** — `watch_only_tests.rs` migrates to the new shape (`passed` + forensic field assertions).
  - **P.7** — Add integration tests for full forensic field population: tampered-cell roundtrips that assert `expected`/`actual`/`diff_byte_offset` populated; skipped checks assert `decode_error` populated.
- **Why deferred:** scope-safety in v0.4.4 release window. The helper-foundation pattern is the right shape; consolidating ~78 call sites at the same time was estimated at ~800-1000 lines deleted plus ~70 push-site updates and risked release timeline.
- **Status:** `resolved by v0.4.5 commits 679ded7 (P.3+P.6) + d3207dd (P.4) + 57f62eb (P.5) + 40638c8 (L+P.7)` — all 5 sub-phases shipped; net cmd/verify_bundle.rs delete ~660 lines; 3 forensic integration tests added.
- **Tier:** `v0.4.5`

### `verify-bundle-emit-checks-helper-and-full-forensics-rollout` — Phase J.2 + J.3 + full forensic field rollout deferred from v0.4.1 to v0.4.2 — SUPERSEDED

- **Status:** `superseded by verify-bundle-helper-and-full-forensics-rollout-v0.4.4 (2026-05-06)`. v0.4.3 P.0 landed the struct shape correction; the helper + full rollout deferred again to v0.4.4.

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs` (~78 VerifyCheck push sites) + new `emit_verify_checks` helper.
- **What:** v0.4.1 ships the structural pieces of SPEC §5.7: VerifyCheck struct gains `expected` / `actual` / `diff_byte_offset` / `decode_error` Option fields with Default impl + serde skip_serializing_if (J.1), and the `--ms1` CLI repeating-flag migration (J.5). Forensic fields are populated on ONE prominent failure path (descriptor-mode `ms1_entropy_match` mismatch — proof-of-shape in cmd/verify_bundle.rs:1456-1469); the remaining ~70 push sites continue to default to `None` for forensic fields. The `emit_verify_checks` helper (J.2) and the run_full / run_multisig / descriptor_mode_verify_run refactor (J.3) to use it are deferred. Full per-cell forensics rollout requires the helper to land first; otherwise duplicating the population logic at every push site is unmaintainable.
- **Why deferred:** scope-safety in v0.4.1 release window. The 78-site refactor is mechanical but error-prone; helper-first approach is the right shape and lands cleanly in v0.4.2.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `verify-bundle-9-3plus6n-descriptor-mode-parity` — Phase G/J descriptor-mode 9/3+6N parity deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase J scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::descriptor_mode_verify_run`.
- **What:** SPEC §5.7 specifies descriptor-mode verify-bundle emits the same 9 / 3+6N check schema as template-mode (replacing v0.3's 3-element coarse ladder). v0.4.1 retains the v0.3 coarse ladder (cmd/verify_bundle.rs:1361 onward) with the H.1 shim for the schema-4 ms1 vec. v0.4.2 lands the parity refactor atomically with the `emit_verify_checks` helper (FOLLOWUP `verify-bundle-emit-checks-helper-and-full-forensics-rollout`).
- **Why deferred:** depends on the helper; bundled with the same v0.4.2 cycle.
- **Status:** `resolved by v0.4.5 Phase P.5 (commit 57f62eb)` — descriptor_mode_verify_run dispatches to emit_verify_checks(... is_multisig: descriptor.n > 1); single-sig descriptors emit the 9 schema, multisig descriptors emit 3+6N.
- **Tier:** `v0.4.2`

### `legacy-cli-flag-deletion` — delete --phrase / --xpub / --cosigner / --master-fingerprint / --cosigner-count / --cosigners-file CLI flags entirely

- **Surfaced:** v0.4.2 cycle planning 2026-05-06 (user-confirmed during scope brainstorm).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::BundleArgs` + `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs`; consumer test files under `crates/mnemonic-toolkit/tests/`.
- **What:** v0.4.2 lands the unified `--slot @N.<subkey>=<value>` dispatch and routes legacy CLI flags through `expand_legacy_to_slots` (option (a) per the v0.4.2 brainstorm). v0.5 takes the next step: delete the legacy CLI flags entirely from `BundleArgs` + `VerifyBundleArgs`. Estimated cost: rewrite ~25 integration tests (~1500 lines of test churn) to use `--slot` syntax. The unified path itself is unchanged; only the CLI surface contracts.
- **Why deferred:** the user accepted the bigger v0.4.2 scope (legacy-flag-deprecation under option a) but routes the cleaner-CLI-surface end-state to v0.5 to amortize the test-rewrite churn against a separate cycle. Captured as a follow-on after v0.4.2 ships.
- **Status:** `resolved by v0.5.1 commit d782a2d` — 6 legacy fields deleted from both `BundleArgs` and `VerifyBundleArgs`; `bundle_args_to_slots` + `expand_legacy_to_slots` shims deleted; 9 mode-violation guards + 11 mode-text consts removed; 3 retained guards covered by new `cli_mode_violations_v0_5.rs`. `bundle::resolve_slots` refactored to take an explicit args-tuple + promoted to `pub(crate)`; `verify_bundle.rs` dispatch reshaped to consume slots. 13 consumer test files rewritten per the v0.5.0 mapping table.
- **Tier:** `v0.5.1`

### `engraving-card-unified-legacy-migration` — migrate 4 legacy engraving_card() call sites to engraving_card_unified

- **Surfaced:** v0.4.1 Phase I scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` 4 legacy call sites (bundle_full, bundle_watch_only, bundle_multisig_full, bundle_multisig_watch_only) + `crates/mnemonic-toolkit/src/format.rs` legacy `engraving_card` + `EngravingMode` enum.
- **What:** v0.4.1 ships `engraving_card_unified` + `BundleInputForCard` per SPEC §5.5 and wires only the new `bundle_run_unified` (--slot-driven) path through it. Migrating the 4 legacy call sites to the unified card requires removing 3 byte-exact format.rs unit tests for `EngravingMode::*` variants and verifying integration tests still pass with the new card layout. v0.4.2 lands the migration + drops `EngravingMode`.
- **Why deferred:** scope-safety in v0.4.1 release window; legacy call sites work unchanged via the existing `engraving_card` function.
- **Status:** `resolved by v0.5.0 Phase A.3 (commit 456c878) — BundleJson.engraving_card field deleted; doc-comment rewritten`
- **Tier:** `v0.4.2`

### `unified-slot-xpub-missing-path-origin-path-null` — origin_path empty-string vs null divergence

- **Surfaced:** v0.4.1 Phase H r1 review L-1.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` (xpub branch) + `emit_unified` (single-sig N=1 origin_path emission).
- **What:** When `--slot @0.xpub=X` is supplied without `--slot @0.path=`, `emit_unified` emits `"origin_path": ""` in the JSON envelope. Legacy `emit` for the equivalent `--xpub X` (no path) invocation emits `"origin_path": null`. SPEC §4.11.b defines `""` as the absent-path sentinel for collision purposes but does not govern the JSON envelope value. Two paths diverge for semantically equivalent inputs.
- **Why deferred:** non-blocking; tooling that reads the envelope can treat `""` and `null` as equivalent. v0.4.2 unifies emission to `null`.
- **Status:** `resolved by v0.5.0 Phase E (commit 990ccad) — origin_path_for_json helper emits null on empty path_raw`
- **Tier:** `v0.4.2-nice-to-have`

### `unified-slot-additional-subkey-shapes` — entropy / xprv / wif / partial-xpub-only resolution deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.1's unified `--slot` dispatch (`bundle_run_unified`) supports two slot subkey shapes: `{phrase}` (BIP-39 → derived xpub) and `{xpub, fingerprint, path}` (watch-only with full origin metadata). The remaining SPEC §6.6.b shapes (`{entropy}` raw entropy → ms-codec ENTR; `{xprv}` xpriv-direct; `{wif}` degenerate single-key; `{xpub}` alone; `{xpub, fingerprint}`; `{xpub, path}`) return BadInput with a pointer to this FOLLOWUP. v0.4.2 lands the resolution logic for each shape + integration tests per shape.
- **Why deferred:** scope-safety in v0.4.1 release window; the two supported shapes cover the headline multi-source-secrets and watch-only-multisig use cases.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `unified-slot-descriptor-mode-support` — descriptor mode under unified --slot dispatch deferred from v0.4.1 to v0.4.2

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified`.
- **What:** v0.4.1's unified `--slot` dispatch supports `--template` only; supplying `--descriptor` alongside `--slot` is rejected with a pointer to this FOLLOWUP. Legacy descriptor-mode dispatch (no `--slot`) continues to work via `descriptor_mode_run`. v0.4.2 unifies the two paths so `--slot` works with both `--template` and `--descriptor`, including descriptor-mode multi-source via per-`@N` slot binding.
- **Why deferred:** scope-safety; the legacy descriptor-mode path remains the recommended invocation for descriptor-driven workflows in v0.4.1.
- **Status:** `open`
- **Tier:** `v0.4.2`

### `descriptor-binding-entropy-field-redundant` — DescriptorBinding.entropy field is redundant after v0.4.3 N

- **Surfaced:** v0.4.3 Phase N (CosignerKeyInfo type alias merge) 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/parse_descriptor.rs::DescriptorBinding`.
- **What:** v0.4.3 N merged CosignerKeyInfo into ResolvedSlot via type alias; ResolvedSlot has per-slot `entropy: Option<Vec<u8>>`. The bundle-level `DescriptorBinding.entropy: Option<Vec<u8>>` field is now semantically redundant with `binding.cosigners[0].entropy`. v0.4.4 retires the field; ~10 call sites (parse_descriptor.rs tests, verify_bundle.rs, bundle.rs::bundle_run_unified_descriptor) update to read `binding.cosigners[0].entropy.as_deref()` instead.
- **Why deferred:** non-blocking; harmless redundancy.
- **Status:** `resolved by v0.4.4 Phase S (commit c99a78b)` — DescriptorBinding.entropy field deleted; `entropy_at_0()` helper method (Option<&[u8]>) reads `cosigners[0].entropy`; bind_full_mode sets `cosigners[0].entropy` before construction; all readers migrated; 244 tests pass.
- **Tier:** `v0.4.4`

### `bundle-json-cli-flag-and-dispatch` — `--bundle-json <file>` verify-bundle intake + schema-version dispatch

- **Surfaced:** v0.4.1 Phase J.4 scope decision 2026-05-05 (per impl plan r1 review I2).
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::VerifyBundleArgs` + new JSON-intake handler.
- **What:** SPEC §6.7 reserves `--bundle-json <file>` as a verify-bundle flag for round-tripping a `bundle --json` envelope. v0.4.3 added the CLI flag + the `serde_json::Value` peek-then-typed-decode dispatch on `schema_version` (schema-4 only; schema-2/3 retro-compat tracked at NEW FOLLOWUP `bundle-json-schema-2-3-retro-compat` at v0.4.4+).
- **Status:** `resolved by v0.4.3 Phase Q (commit pending)` — clap flag with `conflicts_with_all = ["ms1", "mk1", "md1"]`; `load_bundle_json_into_args` synthesizes a VerifyBundleArgs with extracted card vecs; rest of run() unchanged. 3 integration tests in `cli_bundle_json_intake.rs`.
- **Tier:** `v0.4.2` (target met)

### `cosigner-keyinfo-resolved-slot-merge` — retire CosignerKeyInfo into ResolvedSlot

- **Surfaced:** v0.4.1 Phase H.6 (impl plan r1 review I1).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::CosignerKeyInfo` + `ResolvedSlot`.
- **What:** v0.4.1 carried two near-identical typed shapes; v0.4.3 N merged them via `pub type CosignerKeyInfo = ResolvedSlot;` alias. ResolvedSlot is now the sole binding type. CosignerKeyInfo retained as a #[allow(dead_code)] alias for source-compat.
- **Status:** `resolved by v0.4.3 Phase N (commit 25581f3)` — type alias merge; per-slot entropy lives on ResolvedSlot; legacy DescriptorBinding.entropy field retained but redundant (tracked at NEW FOLLOWUP `descriptor-binding-entropy-field-redundant` at v0.4.4).
- **Tier:** `v0.4.2` (target met)

### `bundle-json-schema-2-3-retro-compat` — `--bundle-json` schema-2/3 retro-compat intake

- **Surfaced:** v0.4.3 Phase Q scope decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs::load_bundle_json_into_args`.
- **What:** v0.4.3 ships schema-4-only intake. Schema-2/3 envelopes (theoretical; no real-world bundles exist since v0.4.1) error with byte-exact stderr pointing at this FOLLOWUP. v0.4.4+ adds schema-2/3 typed dispatch IF a real-world need surfaces.
- **Why deferred:** speculative; no real bundles to consume.
- **Status:** `resolved by v0.5.0 Phase D (commit 6e4b87e) — placeholder rejection branch deleted; schema-mismatch fails at field extraction`
- **Tier:** `v0.4.4-nice-to-have`

### `wif-multisig-resolution` — wif slots in multisig contexts

- **Surfaced:** v0.4.2 Phase K.3 (single-sig-only guard introduced; multisig deferred).
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots`.
- **What:** v0.4.3 R lifted the single-sig-only guard. Wif slots in multisig produce ResolvedSlots with the wif's pubkey + zero chain code + empty path. BIP-388 distinctness applies normally (same WIF twice → row 13 collision).
- **Status:** `resolved by v0.4.3 Phase R (commit 610bef6)` — 3 new integration tests cover hybrid 2-of-3 + pure 2-of-2 + same-WIF-twice collision.
- **Tier:** `v0.4.3` (target met)

### `legacy-flag-deprecation` — full migration of --phrase / --xpub / --cosigner to alias-only deferred from v0.4.1 to v0.5+

- **Surfaced:** v0.4.1 Phase H.5 scope decision 2026-05-05.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::run` legacy dispatch path.
- **What:** SPEC §9 v0.4 promises that legacy `--phrase` / `--xpub` / `--cosigner` flags become deprecation aliases that auto-expand into `--slot` form. v0.4.1 ships unified `--slot` as opt-in alongside the unchanged legacy dispatch. v0.5+ (a future BREAKING release) deletes the legacy dispatch entirely and routes everything through `bundle_run_unified` via `expand_legacy_to_slots`.
- **Why deferred:** would force fixture regeneration of 16+ v0.1 byte-exact fixture files + v0.2 carry-forward fixtures; too large for v0.4.1 release window.
- **Status:** `resolved by v0.5.1 commit d782a2d` — superseded by `legacy-cli-flag-deletion`. Legacy dispatch path is deleted entirely; `--slot` is the sole input shape.
- **Tier:** `v0.5.1`

### `bundle-removed-subcommand-trap-positional-eq-bypass` — `bundle multisig-full=value` token bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-2 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** Trap matches `argv[i+1] == "multisig-full"` with exact string equality. A token like `multisig-full=value` would not match and would fall through to clap's generic "unexpected argument" error rather than the byte-exact §6.6 row 1 message. Positional args do not idiomatically take `=value` form in shells, so this is essentially theoretical.
- **Why deferred:** no realistic user invocation produces this argv shape; a post-trap fallback in clap already rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.3 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `bundle-removed-subcommand-trap-double-dash-bypass` — `mnemonic bundle -- multisig-full` bypasses pre-clap trap

- **Surfaced:** v0.4 Phase 2 SPIKE r1 architect review L-3 (2026-05-05).
- **Where:** Phase C.1 `detect_removed_subcommand` (locked SPIKE shape at `design/agent-reports/spike-toolkit-v0_4-pre-phaseA.md` SPIKE-2).
- **What:** With a `--` separator inserted between `bundle` and `multisig-full`, the trap reads `argv[i+1] == "--"` and skips. Clap then processes `multisig-full` as a positional after `--` and emits a generic "unexpected argument" error rather than the byte-exact §6.6 row 1 text. UX difference matters only if a user intentionally inserts `--` before a removed subcommand name — not a realistic migration-error path.
- **Why deferred:** vanishingly unlikely user error; clap's fallback still rejects with exit 2.
- **Status:** `resolved by v0.5.0 Phase C.4 (commit 4a650aa) — entire detect_removed_subcommand trap deleted`
- **Tier:** `v0.4-nice-to-have`

### `tr-sortedmulti-a-via-upstream` — toolkit-side resolved in v0.3.1; v0.3.2 is the cleanup release

- **Surfaced:** v0.3 pre-Phase-A SPIKE 2026-05-05 (`design/agent-reports/spike-toolkit-v0_3-pre-phaseA.md` §1).
- **Resolution timeline:**
  - 2026-04-03: rust-miniscript PR #910 ("Add support for sortedmulti_a") merged; closed issue #320.
  - 2026-04-04: PR #915 ("refactor: remove SortedMultiVec and use Terminal::SortedMulti") merged.
  - 2026-05-05: upstream search confirmed both PRs on master rev `95fdd1c5773bd918c574d2225787973f63e16a66`; no published crate release contains them.
  - 2026-05-05: v0.3.1 adopted via `[patch.crates-io] miniscript = { git = ..., rev = "95fdd1c..." }` after a read-only build experiment confirmed feasibility; walker refactored for the post-#915 API; SPEC §4.9.a Layer 1+2 patched; new `Terminal::SortedMulti` + `Terminal::SortedMultiA` arms added; wire-bit-identical regression test passes (descriptor-mode `tr(@0, sortedmulti_a(...))` md1 == template-mode `--template tr-sortedmulti-a` md1).
- **Where:** `crates/mnemonic-toolkit/Cargo.toml` (`[patch.crates-io]` entry); `crates/mnemonic-toolkit/src/parse_descriptor.rs` (walker arms); `descriptor-mnemonic/crates/md-cli/src/parse/template.rs` (md-cli still pre-#910 — separate FOLLOWUP `walker-backport-to-md-cli`).
- **Toolkit-side status:** `partially resolved by v0.3.1` — `tr(K, sortedmulti_a(...))` works end-to-end via the master `[patch]`. md-cli divergence is the remaining cross-repo concern (FOLLOWUP `walker-backport-to-md-cli`).
- **v0.3.2 cleanup release** (mechanical, when miniscript crates.io publishes a post-#910+#915 release):
  1. Drop the `[patch.crates-io]` entry from `Cargo.toml`.
  2. Bump `miniscript` version in `crates/mnemonic-toolkit/Cargo.toml` to the new release.
  3. Update CHANGELOG; tag `mnemonic-toolkit-v0.3.2`.
  4. No code, SPEC, or test changes expected — the patched master and the new published release should be wire-identical for the surface this toolkit uses.
  5. Watch via `gh api repos/rust-bitcoin/rust-miniscript/tags --jq '.[].name' | grep -E 'miniscript-(13\.[1-9]|14|15)'`.
- **Status:** `partially resolved by v0.3.1; v0.3.2 cleanup pending miniscript crates.io release`
- **Tier:** `v0.3.2` (toolkit-side; was `v0.4-cross-repo` until v0.3.1 shipped)

### `secret-on-stdout-warning-bundle-retrofit` — apply convert's §7 secret-on-stdout warning to bundle

- **Surfaced:** v0.6.0 SPEC architect review r1 C-2 + impl decision 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs` emit_unified ms1 emission paths.
- **What:** v0.6.0 introduces a stderr warning `"warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')"` when convert emits secret-bearing material to stdout (phrase/entropy/xprv/wif/ms1). The bundle subcommand also emits secret-bearing ms1 strings to stdout but does NOT have the warning. Retrofit for cross-tool consistency.
- **Why deferred:** convert was the natural place to introduce the convention (ad-hoc one-shot operations where stdout-redirect-discipline is most likely overlooked); bundle retrofit is a separate scope-bounded change.
- **Status:** `resolved 66ff7c0` (v0.6.1 Phase D — `bundle.rs::emit_unified` emits the warning when `Bundle::any_secret_bearing()` returns true; SPEC §5.5.a; +1 positive (text mode) +1 positive (JSON mode) +2 negative (watch-only single + multisig) test assertions).
- **Tier:** `v0.6.1`

### `convert-seed-and-raw-privkey-nodes` — add seed / raw_privkey / xprv-via-ms1 / seed-via-ms1 nodes to convert when ms-codec v0.2 ships

- **Surfaced:** v0.6.0 SPEC §1 deferral 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType` + edge table + SPEC §1.
- **What:** ms-codec v0.1.0's `SEED`, `XPRV`, `PRVK` tags are `RESERVED_NOT_EMITTED_V01`. v0.6.0 SPEC §1 documents `seed` and `raw_privkey` as deferred-not-rejected nodes. When ms-codec ships v0.2 with the reserved tags activated, add these nodes + their edges to convert (and update SPEC §1 / §2 accordingly).
- **Why deferred:** upstream codec library limit; additive.
- **Status:** `open`
- **Tier:** `cross-repo`

### `convert-phrase-to-leaf-wif` — implement phrase/entropy → wif (path-to-leaf-WIF derivation)

- **Surfaced:** v0.6.0 SPEC §10 deferral 2026-05-06 + impl r1 review.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` Phrase|Entropy arm.
- **What:** v0.6.0 SPEC §2 lists `phrase/entropy → wif` as not directly defined; impl returns `BadInput` with deferral message. Implementing requires a leaf-depth BIP-32 path (`m/<purpose>'/<coin>'/<account>'/<chain>/<index>`, depth 5) and serializing the leaf privkey to WIF. v0.6.1+ adds the missing edge.
- **Why deferred:** scope-safety in v0.6.0; the headline conversion graph nodes were prioritized.
- **Status:** `resolved 62b4f23` (v0.6.1 Phase B — SPEC-A in `SPEC_convert_v0_6.md` §2 + §8; sibling helper `derive_slot::derive_bip32_at_path` for path-driven derivation; `bitcoin::PrivateKey { compressed: true, network, inner: leaf_xpriv.private_key }.to_wif()`; explicit `--path` REQUIRED with byte-exact `ConvertRefusal` stderr (exit 2) when absent; `edge_uses_pbkdf2` extended to include `Wif` so `--passphrase` does not spuriously fire the ignored-warning).
- **Tier:** `v0.6.1`

### `convert-slip0132-prefix-support` — accept zpub/ypub on input + emit modes (consolidated v0.6.1)

- **Surfaced:** v0.6.0 post-release UX audit 2026-05-06 (user prompt about SLIP-0132 prefix interpretation).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` (input normalization + new edge); possibly cross-cutting into `crates/mnemonic-toolkit/src/cmd/bundle.rs::resolve_slots` Xpub branch (if input normalization is repo-wide rather than convert-only).
- **What:** SLIP-0132 (`ypub`/`Ypub`/`zpub`/`Zpub` mainnet, plus `tpub`/`upub`/`vpub`/`Upub`/`Vpub` testnet) extended-key prefixes encode the intended script type (BIP-49 single/multi, BIP-84 single/multi) in the version bytes. Bitcoin Core + rust-miniscript + BIP-388 wallet policies reject non-`xpub` prefixes; the canonical modern path is descriptor-native xpub + descriptor wrapper. v0.6.0 currently fails at `Xpub::from_str` for a SLIP-0132 prefix. Both directions ship together in v0.6.1:

  **Permissive input (mechanical):** add a SLIP-0132 → xpub normalizer (`src/slip0132.rs` helper or inline). On input, detect non-`xpub` prefix; recompute version bytes to the matching `xpub`/`tpub` neutral prefix and re-base58check. The 78 payload bytes are byte-identical across SLIP-0132 variants, so no ECC work — pure prefix swap. Applies to:
    - `convert --from xpub=<zpub-string>`
    - `convert --from xpub=<ypub-string>` (etc.)
    - Cross-cutting: `bundle --slot @0.xpub=<zpub>` and `verify-bundle --slot @0.xpub=<zpub>` normalize identically for input symmetry across the toolkit.

  **Expressive output (design fork — resolve early in the cycle):** add output-side SLIP-0132 emission. Two grammar shapes to choose between via a SPEC amendment + one architect review round at the start of the v0.6.1 cycle:
    - (a) New target nodes `ypub`/`zpub`/`Ypub`/`Zpub` plus testnet `upub`/`vpub`/`Upub`/`Vpub`. Adds 8 nodes to NodeType. Edges: `xpub → ypub` etc. (pure prefix swap; no derivation).
    - (b) Existing `--to xpub` plus a `--xpub-prefix <neutral|y|z|Y|Z>` modifier flag. Single new flag; no new nodes.
   Option (b) is grammar-lighter and preserves the convention that SLIP-0132 variants are *encodings of the same xpub*, not different artifact classes. Lock the choice before implementation begins.

- **Why deferred:** v0.6.0 prioritized the headline single-format conversion graph; SLIP-0132 is a UX-convenience layer over BIP-32 + BIP-388 descriptors. Both directions ship together in v0.6.1 to close the SLIP-0132 story in one release cycle.
- **Status:** `resolved bb77164` (v0.6.1 Phase C — Option (b) selected per architect convergence: `--xpub-prefix <variant>` modifier flag with 5 case-sensitive values (`xpub`/`ypub`/`Ypub`/`zpub`/`Zpub`) per SPEC §11.a; testnet variants are network-context-derived via `--network` (no separate flag values); `--network` REQUIRED when `--xpub-prefix` is non-default. Input normalizer in new `src/slip0132.rs` handles all 8 SLIP-0132 prefixes (4 mainnet + 4 testnet); cross-cut wired at `convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`. New `(xpub, xpub)` edge in §2 for the §11.a round-trip primitive).
- **Tier:** `v0.6.1`

### `convert-test-coverage-tightening` — close convert subcommand test gaps (6 direct-edge + 2 deferral + 3 round-trip tests)

- **Surfaced:** v0.6.0 post-release coverage audit 2026-05-06 (user-prompted enumeration of supported edges vs. test coverage).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_happy_paths.rs`.
- **What:** v0.6.0 ships 23 convert tests covering 14 of 20 supported direct edges. Three coverage gaps to close in v0.6.1:
  1. **6 untested supported direct edges** — add at least one happy-path test each:
     - `phrase → ms1`
     - `entropy → xpub`
     - `entropy → xprv`
     - `entropy → fingerprint`
     - `xprv → fingerprint`
     - `wif → fingerprint`
  2. **2 deferral-message negative tests** — assert the v0.6.0 BadInput stderr ("not yet supported in v0.6 (path-to-leaf-WIF derivation deferred)") for:
     - `phrase → wif`
     - `entropy → wif`
     These tests pin the deferral text byte-exactly so the v0.6.1+ implementation of `convert-phrase-to-leaf-wif` will need to update them in lockstep (intentional: forces the deferral-→-implementation transition to be explicit).
  3. **3 explicit round-trip loop tests** (A→B→A) for the supported bidirectional pairs:
     - `phrase ↔ entropy` — assert `phrase → entropy → phrase` produces the canonical phrase byte-for-byte.
     - `entropy ↔ ms1` — assert `entropy → ms1 → entropy` produces identical entropy bytes.
     - `phrase ↔ ms1` (via entropy intermediate) — assert `phrase → ms1 → phrase` produces the canonical phrase. v0.6.0 has one-direction tests on each leg but no full-loop assertion.
- **Why deferred:** v0.6.0 prioritized headline-edge coverage and refusal-taxonomy correctness; the missing tests are tightening, not net-new functionality. The 6 uncovered edges are exercised indirectly through the JSON envelope test (#3 in `cli_convert_json.rs`) and the v0.5.2 16-cell parametric byte-identity test, but lack explicit asserts.
- **Status:** `resolved 59140c5` (v0.6.1 Phase E — 6 direct-edge tests added to `cli_convert_happy_paths.rs`; 3 round-trip loop tests added in new `cli_convert_round_trips.rs`. The 2 deferral-message tests are explicitly NOT written — Phase B (62b4f23) implemented `phrase/entropy → wif` so the deferrals no longer exist).
- **Tier:** `v0.6.1`

### `convert-run-step-numbering-duplicate-8` — `cmd::convert::run` has duplicate `// 8)` step labels

- **Surfaced:** Phase B code-reviewer r1 (Nit, deferred — predates Phase B).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs:382` and `:385`.
- **What:** The dispatch in `convert::run` numbers its steps `// 1)` through `// 9)`; both "Compute outputs" and "Emit" are labeled `// 8)`. The second should be `// 9)` to keep the comment numbering monotonic. Comment-only nit; no behavioral effect.
- **Why deferred:** Pre-existed Phase B; out of scope for the SPEC-A `phrase/entropy → wif` commit. Cleanly fixable in the next convert-touching patch.
- **Status:** `resolved a52b2aa` — v0.6.2 cosmetic micro-commit (Phase 5). `// 8) Emit.` → `// 9) Emit.`; `// 9) §7 secret-on-stdout warning.` → `// 10)`. Comment-only.
- **Tier:** `v0.6.2-nice-to-have`

### `slip0132-input-normalization-stderr-info` — emit a one-line stderr note when SLIP-0132 input is silently normalized

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new helper at `crates/mnemonic-toolkit/src/slip0132.rs`; emitter at the 3 production cross-cut sites (`convert.rs:515`, `bundle.rs:327`, `bundle.rs:853`).
- **What:** v0.6.1 Phase C silently normalizes SLIP-0132 prefix variants (zpub/ypub/Zpub/Ypub mainnet + uvpub/UVpub testnet) to neutral xpub/tpub on input. The user gets correct math but loses the intent signal their prefix carried (BIP-49 vs BIP-84, single-sig vs multisig). Add a one-line stderr informational note when normalization actually fires — pattern:
  ```
  info: normalized <variant> input to neutral <xpub|tpub> (encoding-only; no key change). Re-emit with --xpub-prefix <variant> if you need the SLIP-0132 form.
  ```
  Suppressed when input is already neutral. Quiet for users who already understand the normalization; informative for users discovering the round-trip primitive. The emitter must thread `&mut dyn Write` for stderr to all 3 cross-cut sites OR be implemented as an out-parameter on `normalize_xpub_prefix` so the caller decides where to write. Implementation tip: if `normalize_xpub_prefix` returns `Result<(String, Option<&'static str> /* variant-name */), ToolkitError>`, callers can match on `Some(_)` and emit per their stderr convention.
- **Why deferred:** v0.6.1 shipped the silent-normalization MVP intentionally (smaller blast radius; no new stderr bytes that could break byte-exact tests; Phase D's stderr-ordering invariant stays simple). UX-improvement work fits a v0.6.2 patch.
- **Caveat:** new stderr lines at the 3 cross-cut sites must NOT break the Phase D §5.5.a "secret-on-stdout warning is the LAST stderr write" invariant. Either fire the info note BEFORE the engraving card / before the secret-on-stdout warning, or relax the §5.5.a SPEC clause. Spike before SPEC-amending.
- **Status:** `resolved e4fedd7` — v0.6.2 lean cycle. SLIP-0132 input-normalization stderr info-line shipped; SPEC §5.5.a relaxation + multi-slot ordering + `--json` / `--no-engraving-card` independence locked. Phase 1 RED scaffold (`38c4272` + `740a917`), Phase 2 helper signature (`11c8edb` + `957db16`), Phase 3 emission (`e4fedd7` + `7bf1f1e` review-fix DRY refactor), Phase 4 SPEC + CHANGELOG (`39fa359` + `42561f3`), Phase 5 cosmetic step-numbering (`a52b2aa` + `96c2e3b`), Phase 6 release (`1fddf3b`).
- **Tier:** `v0.6.2`

### `slip0132-info-line-spec-text-not-byte-pinned` — SPEC §11 info-line wording isn't programmatically locked to the production format string

- **Surfaced:** v0.6.2 final cumulative review 2026-05-06.
- **Where:** `design/SPEC_convert_v0_6.md` §11 (canonical info-line paragraph); `crates/mnemonic-toolkit/src/slip0132.rs::render_slip0132_info_line` (production helper); `crates/mnemonic-toolkit/src/slip0132.rs::tests::render_slip0132_info_line_pins_canonical_text` (existing pin test, locks production ↔ slip0132 internal only).
- **What:** v0.6.2 introduced `render_slip0132_info_line(variant)` as the single production source of truth for the info-line text, with a unit test pinning the byte sequence for representative variants. The SPEC body in §11 carries the canonical text but as a templated example with `<variant>` and `<xpub|tpub>` placeholders. There is no test that asserts the SPEC body matches the production format-string structure. A future editor "improving" SPEC §11 prose (e.g., changing "Re-emit with" to "Re-encode with") would silently desync the SPEC from shipped behavior; CI catches nothing.
- **Why deferred:** v0.6.2 lean cycle scope; not a correctness bug. Test-side helpers in `tests/cli_*_slip0132_info.rs` provide bidirectional locking against production already (any production-text drift fails the integration tests), so the practical drift hazard is bounded — this entry is about catching SPEC-prose drift specifically.
- **Possible fix:** add a doc-test or unit test that grep-matches the SPEC §11 paragraph against a structural pattern, OR convert SPEC §11's example block into a fenced ```text block whose canonical form is read at test time. The first option is lower-overhead.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — `slip0132::tests::spec_info_line_template_matches_production_render` reads `SPEC_convert_v0_6.md` §11 via `include_str!`, slices between HTML markers, and asserts byte-equality against `render_slip0132_info_line` for all 8 SLIP-0132 variants. SPEC↔production drift now CI-locked.
- **Tier:** `v0.7-nice-to-have`

### `verify-bundle-discards-slip0132-input-variant-asymmetry` — `verify-bundle` silently drops the SLIP-0132 input-normalization signal across 4 callsites

- **Surfaced:** v0.6.2 Phase 3 implementation (`e4fedd7`); confirmed in v0.6.2 final cumulative review 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/verify_bundle.rs:209`, `:261`, `:337`, `:407` — each callsite destructures `let (resolved, _slip0132_signals) = resolve_slots(...)?;` and discards the signal with the comment `// verify-bundle does not surface SLIP-0132 input-normalization signals.`
- **What:** v0.6.2 made `mnemonic convert` and `mnemonic bundle` emit a stderr informational line when a user-supplied SLIP-0132 prefix is silently normalized. `mnemonic verify-bundle` calls the same `pub(crate) resolve_slots` helper but discards the signal. Result: a user who pastes a `zpub` to `bundle` gets the info-line; pasting the same `zpub` to `verify-bundle` does not. The discard is semantically correct for v0.6.2's scope (verify-bundle is structurally a checker that emits check-pass/fail status, not a renderer of user inputs), but it creates a UX asymmetry within the toolkit.
- **Why deferred:** parity decision is its own UX policy question (does verify-bundle want to also emit the info-line? Or remain silent on stderr by design?). v0.6.2 lean cycle did not litigate this — the discard was the no-op-on-verify-bundle choice that minimized blast radius.
- **Possible fix (v0.7+ brainstorm):** decide whether `verify-bundle` should also emit the info-line for symmetry. If yes, thread `slip0132_signals` to a stderr emitter near each of the 4 callsites; SPEC §5.5.a's stderr-ordering invariant applies (notes precede any conditional warnings). If no, document the asymmetry intentionally in `SPEC_convert_v0_6.md` §11 / verify-bundle SPEC.
- **Status:** `resolved 354c945`
- **Resolution:** v0.7 Phase 7 — Option B locked per architect R1-I8. The 4 callsite-comments at `verify_bundle.rs:208/:261/:336/:406` gain a SPEC §11 v0.7 amendment cross-pointer; verify-bundle remains silent on SLIP-0132 input-normalization signals as intentional checker semantics. Zero new emission code.
- **Tier:** `v0.7-nice-to-have`

### `bip38-distinct-passphrase-flag` — split composite `(Phrase|Entropy, Bip38)` passphrase into two channels

- **Surfaced:** v0.7 Phase 1 code-quality review (commit `c3d0a85`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` composite arm + `convert::ConvertArgs` clap struct; SPEC §12.b reference.
- **What:** v0.7 ships dual-purpose `--passphrase` for composite paths flowing `phrase → wif → bip38` (or `entropy → wif → bip38`). One passphrase value drives both BIP-39 PBKDF2 mnemonic extension and BIP-38 Scrypt encryption. A user wanting distinct values must invoke `convert` twice. v0.8 may add `--bip38-passphrase` as a distinct flag so a single composite invocation can use different passphrases per layer. Implementation: thread the new flag through `compute_outputs`'s composite arms; if `--bip38-passphrase` is supplied, use it for the Scrypt step and use `--passphrase` (or `""` if absent) for the PBKDF2 step.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--bip38-passphrase` flag added with locked R1-I3 semantics (composite arm: independent passphrases, no fallback, BREAKING change from v0.7's dual-purpose dispatch; direct arm: fallback to `--passphrase`). CHANGELOG `[0.8.0]` migration sentence pinned. SPEC v0.8 §12.b amendment.
- **Tier:** `v0.8`

### `bip38-encrypted-wif` — accept + emit BIP-38 passphrase-encrypted privkeys (`6P...`)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `BipEncryptedWif` (or `Bip38`) `NodeType` variant in `convert.rs`; new edges from/to `wif`; SPEC §1 + §2 amendments.
- **What:** BIP-38 (`6P...`, base58check, 58 chars) is a passphrase-encrypted WIF format — widely used for paper wallets and key handoff. Two pieces: non-EC-multiplied form (encrypts an existing privkey under a passphrase via Scrypt) and EC-multiplied form (generates new privkey from passphrase + intermediate code; less common). Add as a new convert node so users can decrypt `6P → wif` (with `--passphrase`) and encrypt `wif → 6P` (with `--passphrase`); composite edges `phrase → 6P` follow naturally. Refusal class for `6P → 6P` and any cross-format pivot.
- **Why deferred:** v0.6.x focused on BIP-39 + BIP-32 + SLIP-0132 graph completeness; BIP-38 is its own well-defined Scrypt-backed format and merits a dedicated phase. Implementation likely uses the `bip38` crate or hand-rolled Scrypt against `secp256k1` primitives already in the dep tree.
- **Status:** `resolved c3d0a85`
- **Resolution:** v0.7 Phase 1 — `Wif↔Bip38` edges + composite paths shipped via `bip38 = "1.1"` crate (Apache-2.0). Security review at `design/agent-reports/v0_7-phase-1-bip38-security-review.md`. SPEC §12.
- **Tier:** `v0.7`

### `casascius-mini-private-key` — accept Casascius mini-key (`S...`, 22/26/30 chars) on input

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `MiniKey` `NodeType` (or absorb into `Wif` arm with prefix-detect) in `convert.rs`; SPEC §1 + §2.
- **What:** Casascius mini private key is a compact base-58 alphabet encoding starting with capital `S` (22, 26, or 30 chars) used historically on physical Bitcoin coins. Format: `S` + N chars; SHA256 of the full string + `?` must hash to `0x00` prefix (typo-checksum). Decoding: SHA256 of the mini-key string yields a 32-byte privkey scalar. One-way edge `mini-key → wif` (encoding-only; no key change). No `wif → mini-key` (mini-key generation requires a search for the typo-checksum-passing string; not deterministic from a given privkey). Refusal class: encode direction is a `§3.b lossy compression barrier` (the typo-checksum embedded in the mini-key string is not recoverable from a raw privkey).
- **Why deferred:** small but distinct format with its own checksum spec (Casascius's typo-check rule); fits a v0.7 grab-bag of less-common formats.
- **Status:** `resolved 89d29ab`
- **Resolution:** v0.7 Phase 2 — `(MiniKey, Wif)` decode-only edge shipped; SHA256 self-checksum rule enforced; encode direction refused as one-way (§3.b lossy-compression barrier). SPEC §13.
- **Tier:** `v0.7`

### `bip85-deterministic-entropy` — derive child seeds from a BIP-32 master

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new top-level `mnemonic derive-child` subcommand OR new edge `xprv → entropy` (with `--bip85-path` modifier); SPEC §1 / new top-level SPEC.
- **What:** BIP-85 derives deterministic child entropy from a BIP-32 master xpriv via HMAC-SHA512 at the path `m/83696968'/<application>'/<index>'`. Use cases: managing many seeds from one master; per-application sub-seeds; password derivation; WIF derivation. Standard application codes per BIP-85: `39'` (BIP-39 entropy of length L words), `2'` (HD-seed), `32'` (xprv child), `128169'` (hex bytes), `707764'` (passwords). Output node depends on the application code: `entropy` for `39'`, `xprv` for `32'`, etc. Grammar lean: `mnemonic derive-child --from xprv=<master> --application <bip39|hd-seed|xprv|hex|password> --length <N> --index <N>` to keep convert's edge-table model untouched (BIP-85 is a *derivation* operation, not a *single-format conversion*).
- **Why deferred:** BIP-85 is a useful but narrow derivation utility; doesn't fit `convert`'s "single-format conversion" framing cleanly. Likely wants its own subcommand. SPEC question to resolve at brainstorm: subcommand vs. extending convert.
- **Status:** `resolved 965cc3e`
- **Resolution:** v0.7 Phase 6 — new `mnemonic derive-child` subcommand shipped with 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`). RSA / RSA-GPG / DICE refused with v0.8 deferral stubs. New SPEC `design/SPEC_derive_child_v0_7.md`.
- **Tier:** `v0.7`

### `slip39-shamir-secret-sharing` — SLIP-39 Trezor Shamir backup format

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `Slip39` `NodeType` (or new top-level subcommand `mnemonic slip39`); SPEC additions or new SPEC document.
- **What:** SLIP-39 (Trezor's standard) is a K-of-N Shamir-secret-sharing scheme for BIP-39 entropy with its own wordlist (1024 words, distinct from BIP-39's 2048). Shares carry: identifier, iteration exponent, group threshold, group count, member threshold, member index, share value, checksum (Reed-Solomon). Two-level scheme: groups × shares-within-group. Used by Trezor Model T for its native backup. Edges: `entropy → slip39-shares` (split; takes `--group-threshold` + per-group `--member-threshold`); `slip39-shares → entropy` (combine; needs ≥ K shares). Composite via entropy intermediate: `phrase → slip39-shares` etc. The 1024-word SLIP-39 wordlist must be embedded.
- **Why deferred:** Largest single addition in the queue — SLIP-39 is essentially an alternative to BIP-39's wordlist + a Shamir layer. The toolkit's secret-material slot would gain a second-class citizen alongside BIP-39 entropy. Significant SPEC + impl work. Trezor's `python-shamir-mnemonic` library is the reference impl. Note: the planned `mnemonic-secret` v0.2 cycle (sibling repo) is shipping K-of-N share encoding for ms1 codex32 — that may obviate the need for SLIP-39 in this toolkit, depending on user priorities. Brainstorm should resolve "do we want SLIP-39 *and* ms1-shares, or just ms1-shares?" before any impl. v0.7 cycle resolved to defer (lib audit returned hand-roll-required; no maintained Rust crate). v0.8 cycle re-tiered to v1+: scope is too large for a v0.8 minor cycle alongside the locked BIP-38/BIP-85/Electrum/export-wallet menu, and ms1-shares may obviate.
- **Status:** `open`
- **Tier:** `v1+`

### `electrum-native-seed-format` — Electrum seed wordlist + version-prefix checksum

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new `ElectrumPhrase` `NodeType` in `convert.rs` (or distinct from BIP-39's `Phrase`); SPEC §1 + §2.
- **What:** Electrum's seed format is its own wordlist + checksum scheme distinct from BIP-39. The seed validates by HMAC-SHA512 of the phrase prefixed with `"Seed version"`; the resulting hash's hex prefix encodes the seed-type (`01` = standard, `100` = segwit, `101` = 2FA standard, `102` = 2FA segwit). Conversion: `electrum-phrase → entropy` (different mapping than BIP-39); `electrum-phrase → seed → master xpriv`. Edges symmetric to BIP-39's. Wordlist embedding required (Electrum English wordlist is similar to BIP-39's but differs).
- **Why deferred:** medium scope — own wordlist + checksum + seed-version dispatch. Used by Electrum users transitioning to / from BIP-39-based wallets. Less urgent than BIP-38 / BIP-85 because most Electrum users can re-derive into BIP-39 via the wallet. Brainstorm should weigh user demand.
- **Status:** `resolved 892139c`
- **Resolution:** v0.7 Phase 3 — `ElectrumPhrase ↔ Entropy` edges shipped with 4-version HMAC-SHA512 prefix dispatch (`01`/`100`/`101`/`102`); 2FA versions (`101`/`102`) refused. Corpus spike at `design/agent-reports/v0_7-phase-3-electrum-corpus-spike.md`. SPEC §14.
- **Tier:** `v0.7`

### `miniscript-beyond-bip388` — accept full miniscript policies beyond BIP-388's descriptor-template subset

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `crates/mnemonic-toolkit/src/cmd/bundle.rs::bundle_run_unified_descriptor`; descriptor-input handling in `parse_descriptor.rs`.
- **What:** v0.5+ accepts BIP-388-conformant descriptors (placeholder-template form `wpkh(@0/<0;1>/*)`, multipath, sortedmulti, etc.). The full miniscript language has many additional policies the toolkit doesn't surface as supported wallet types: `andor`, `thresh`, `pk_h`, time-locked branches via `older` / `after`, hash-locked `hash160` / `sha256` / `ripemd160`, `multi_a` (taproot multi without sortedness), arbitrary `tr` taproot trees with multi-leaf miniscript. Rust-miniscript supports parsing these; the gap is the toolkit's wallet-policy validation and the engraving-card / verify-bundle UX.
- **Why deferred:** Open-ended scope. Each new miniscript shape may have its own UX implications (verify-bundle parity check counts; engraving-card rendering; BIP-388 distinct-key extensions). Brainstorm should pick a small subset (e.g., `thresh`, `andor`, `tr` with single-leaf miniscript) rather than open-ended "all of miniscript." v0.7 cycle did not pick this up; v0.8 cycle re-tiered to v1+: doesn't fit alongside the locked BIP-38/BIP-85/Electrum/export-wallet menu and the open-ended scope deserves its own dedicated brainstorm.
- **Status:** `open`
- **Tier:** `v1+`

### `vault-construction-covenant-based` — accept covenant-based vault descriptors (CTV / OP_CAT / OP_VAULT)

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** `parse_descriptor.rs` (descriptor parser extension); SPEC additions.
- **What:** Vault constructions use covenant opcodes (BIP-119 `OP_CHECKTEMPLATEVERIFY`, BIP-348 `OP_CAT` re-enable, BIP-345 `OP_VAULT`) to enforce spending paths beyond what current Bitcoin script allows: time-delayed spends, recovery paths, batch authorizations, etc. None of these opcodes are activated on mainnet today. When/if activated, vault descriptors become a wallet-type class distinct from current single-sig/multisig descriptors.
- **Why deferred:** **Gated on Bitcoin consensus activation.** No mainnet support today; signet test-cases exist for some of these. Re-evaluate when the relevant BIP advances to mainnet. The plumbing in this toolkit (descriptor parsing, mk1 xpub binding, md1 wallet-policy encoding) generalizes to vaults, so the impl gap is small once the script-side BIP activates — but speculative until then.
- **Status:** `open`
- **Tier:** `v1+`

### `address-derivation-from-xpub-path` — xpub + path + script-type → bitcoin address

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06; in-scope confirmed 2026-05-06 (SPEC §10 exclusion struck through, see `SPEC_convert_v0_6.md` §10 v0.6.1 amendment).
- **Where:** new edge `(xpub, address)` in `convert.rs::is_supported_direct_edge`; new `Address` `NodeType`; SPEC §1 + §2 amendments.
- **What:** Edge: `xpub` source + `--path` (or `--address-index N` + `--chain receive|change`) + script-type inferred from `--template` (or explicit `--script-type p2wpkh|p2sh-p2wpkh|p2tr|...`) → bech32 / bech32m / base58 address string. Composite from `phrase` / `entropy` via the existing BIP-32 derivation pipeline. Refusal classes: address → anything (one-way; addresses are hash160/SHA256 of pubkeys). Read-only display only — does NOT extend to PSBT / signing (PSBT remains out-of-scope per `bip174-psbt-signing` v1+).
- **Why deferred:** Useful but not blocking; v0.7 cycle slot. SPEC §10 amendment from "out of scope" to "in scope, deferred to v0.7" was committed alongside this entry update.
- **Status:** `resolved 940ec0b`
- **Resolution:** v0.7 Phase 4 — `(Xpub, Address)` edge shipped with `--path` mandatory + `--script-type` inferred from `--template` for BIP-44/49/84/86 → P2PKH/P2SH-P2WPKH/P2WPKH/P2TR. Composite paths via the existing BIP-32 derivation pipeline. SPEC §10.a.
- **Tier:** `v0.7`

### `bip327-musig2-collective-keys` — MuSig2 collective-key wallet-policy support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** descriptor parser + new wallet-policy class.
- **What:** BIP-327 (MuSig2) defines a 2-round Schnorr multi-signature scheme that produces a single aggregated public key from N participants. Wallet-policy formats for MuSig2 collective keys are still maturing — there's no settled "BIP-388-equivalent" for MuSig2 wallets yet. When the spec settles, add support for MuSig2 collective keys as a wallet-policy variant alongside multisig (sortedmulti) and single-sig.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; rust-miniscript support is partial; hardware-wallet vendor adoption is preliminary. Re-evaluate when the wallet-policy spec for MuSig2 stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `bip174-psbt-signing` — Partially Signed Bitcoin Transactions support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** would be an entirely new subcommand surface area.
- **What:** BIP-174 PSBT (Partially Signed Bitcoin Transactions) is the standard format for unsigned/partially-signed transactions exchanged between wallets and signers. Adding PSBT support would expand the toolkit from "key/wallet management" into "transaction signing" — a fundamentally different problem class.
- **Why deferred:** **Out of scope per `convert` SPEC §10:** "different problem class." This toolkit is explicitly about key/wallet info, not transaction signing. PSBT belongs in a distinct tool (or in a separate signer subcommand of this toolkit if scope is reframed at v1+).
- **Status:** `open`
- **Tier:** `v1+`

### `frost-threshold-keys` — FROST threshold signature scheme support

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new wallet-policy class; cryptographic primitive set extension.
- **What:** FROST (Flexible Round-Optimized Schnorr Threshold signatures) is a K-of-N threshold signature scheme that produces a single aggregated Schnorr signature without requiring a trusted dealer. Distinct from MuSig2 in that it's threshold (K-of-N) rather than n-of-n. Wallet-policy and key-aggregation formats are still being standardized.
- **Why deferred:** **Standards-maturity gate.** No settled wallet-policy spec; cryptographic primitives not yet in `bitcoin` / `secp256k1` crates; hardware-wallet adoption preliminary. Re-evaluate when the spec stabilizes.
- **Status:** `open`
- **Tier:** `v1+`

### `liquid-confidential-extended-keys` — Liquid sidechain extended-key formats

- **Surfaced:** v0.6.1 post-release wallet-types audit 2026-05-06.
- **Where:** new network variant in `network.rs` (`Liquid` / `LiquidTestnet`); xpub/xprv version-byte table extension; SPEC §11 SLIP-0132 swap table additions.
- **What:** Elements/Liquid sidechain uses its own xpub/xprv version bytes plus blinding-key extensions for confidential transactions. Asset-blinding keys, value-blinding keys, and the address-blinding-key derivation (SLIP-0077) are Liquid-specific cryptographic primitives. Adding Liquid support would require: (a) network variant + version-byte table; (b) blinding-key derivation primitives; (c) Confidential Address derivation (different from mainnet bech32 addresses).
- **Why deferred:** Different chain context. Liquid is a federated sidechain with its own wallet ecosystem (Blockstream Green, Elements). The toolkit's primary user base is Bitcoin mainnet/testnet; Liquid users typically use Liquid-specific tooling. Re-evaluate if there's demand from cross-chain users.
- **Status:** `open`
- **Tier:** `v1+`

### `wallet-export-industry-formats` — `mnemonic export-wallet` (or `bundle --wallet-export <format>`) for Bitcoin Core / Sparrow / Specter / BIP-388 import

- **Surfaced:** v0.6.1 post-release UX discussion 2026-05-06.
- **Where:** new subcommand `mnemonic export-wallet` OR new flag on `bundle`; output formatters under `crates/mnemonic-toolkit/src/wallet_export.rs` (new module).
- **What:** Today the canonical "all wallet info, no secret" representation IS `mnemonic bundle --json` in watch-only mode (per SPEC §5.8: ms1 omitted-or-empty-sentinel; mk1 carries xpub bindings; md1 carries the descriptor/template). It is correct and complete BUT only the toolkit can re-ingest it. Users who want to feed the watch-only artifact to another wallet (Bitcoin Core, Sparrow, Specter, hardware-wallet HWI flows) must hand-translate. Add an industry-format export layer with at least:
  - **Bitcoin Core `importdescriptors` JSON** — `{"desc": "wpkh([fp/path]xpub.../{0,1}/*)#checksum", "active": true, "internal": false, "range": [0, 999], "timestamp": "now"}` per descriptor (one for receive, one for change; or `<0;1>` multipath split). Matches Bitcoin Core 25+ descriptor-wallet expectations.
  - **BIP-388 wallet policy** — formal `wallet_policy` JSON with `name`, `description_template`, `keys_info` array. Matches Ledger / hardware-wallet vendors that follow BIP-388.
  - **Sparrow / Specter wallet JSON** (optional; format is per-wallet). Lower priority — both can ingest output descriptors directly via the Bitcoin Core format.
  - **HWI signer JSON** (optional) — for cosigner export.
  Grammar lean: `mnemonic export-wallet --format <bitcoin-core|bip388|sparrow|specter> --output <path-or-->` with the same `--slot @N.<subkey>=<value>` input shape as `bundle`. Refuses if any slot supplies entropy/phrase (export-wallet is watch-only by definition). SPEC question to resolve at brainstorm: does this live as a new top-level subcommand OR a `bundle --wallet-export` flag? Lean: new subcommand because the input grammar is a strict subset of bundle (no entropy/phrase) and the output is a different wire format from `BundleJson`.
- **Why deferred:** v0.6.1 was a polish patch for `convert` + `bundle` UX. New subcommand or new bundle flag is its own minor scope. Brainstorm should resolve the format priority list (Bitcoin Core first vs BIP-388 first), the subcommand-vs-flag fork, and whether `range`/`timestamp` defaults need to be configurable.
- **Status:** `resolved 3821f66`
- **Resolution:** v0.7 Phase 5 — new `mnemonic export-wallet` subcommand shipped. Bitcoin Core `importdescriptors` JSON (default) + BIP-388 `wallet_policy` JSON. Sparrow / Specter formats stubbed (refuse with v0.8 deferral). `--range` / `--timestamp` / `--bitcoin-core-version` overrides. Watch-only enforced (refuses entropy/phrase slot input). New SPEC `design/SPEC_export_wallet_v0_7.md`.
- **Resolution-extended (v0.8.1 Phase 1):** Coldcard generic JSON skeleton (singlesig bip44/bip49/bip84) + Coldcard multisig text (wsh / sh-wsh, sorted and unsorted) + Blockstream Jade multisig text (byte-identical to Coldcard's, delegated emitter) shipped. New `wallet_export/{coldcard,jade}.rs`. `CliExportFormat::Coldcard` + `CliExportFormat::Jade` variants. `--wallet-name <STRING>` clap flag for formats publishing wallet names (Coldcard generic JSON, Sparrow / Specter / Electrum land in subsequent phases). New slot subkey `@N.master_xpub=` (depth-0 root xpub, optional, watch-only-class). Coverage now 2/8 → 4/8 of the SPEC §11 priority list; Sparrow/Specter/Electrum/Green land in Phases 2-5.
- **Resolution-extended (v0.8.1 Phase 5):** All six new vendor formats now shipped — `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`. The complete v0.8.1 SPEC §11 priority list (8 formats: `bitcoin-core`, `bip388`, `coldcard`, `jade`, `sparrow`, `specter`, `electrum`, `green`) is fully realized. New emitter modules: `wallet_export/{sparrow,specter,electrum,green}.rs`. New `CliExportFormat` variants: `Sparrow`, `Specter`, `Electrum`, `Green`. Per-format pinned byte-exact fixtures + SPEC §4 missing-info refusal channel exercised by Sparrow (Threshold) and Specter (WalletName). Status remains `resolved`.
- **Tier:** `v0.6.2`

### `coldcard-master-xpub-plumbing-pending` — `@N.master_xpub=` slot subkey parses but is dropped before reaching the Coldcard emitter

- **Surfaced:** v0.8.1 Phase 1 R1 reviewer-loop fold (I-2).
- **Where:** `crates/mnemonic-toolkit/src/synthesize.rs::ResolvedSlot` + `crates/mnemonic-toolkit/src/wallet_export/mod.rs::EmitInputs` + `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** SPEC §2 + §5.1 ship two normative claims: (a) the slot grammar accepts `@N.master_xpub=<base58>` (shipped in Phase 1.1 via `SlotSubkey::MasterXpub`); (b) the Coldcard generic-JSON top-level `xpub` field is emitted iff `@0.master_xpub=` was supplied. Phase 1 shipped (a) but not (b); Phase 1.9.R1 added a refuse-on-supply guard in `cmd::export_wallet::run` so the gap was not silent.
- **Status:** `resolved` (v0.8.2 plumbing cycle).
- **Resolution:** v0.8.2 follow-up — `ResolvedSlot` gained a `master_xpub: Option<Xpub>` field populated in the `{Xpub, ...}` arm of `resolve_slots` via `crate::slip0132::normalize_xpub_prefix` + `Xpub::from_str`. `EmitInputs` gained `master_xpub_at_0: Option<Xpub>` plumbed from `resolved_slots[0].master_xpub` in `cmd::export_wallet::run`. `emit_coldcard_generic_json` now emits the top-level `xpub` field conditionally (`Some(x) → x.to_string()`, `None → field omitted via `#[serde(skip_serializing_if = "Option::is_none")]`). The Phase 1.9.R1 refuse-on-supply guard at `cmd::export_wallet::run:182-197` was retired. New byte-exact fixture `tests/export_wallet/coldcard_generic_bip84_mainnet_with_master_xpub.json`; new test cells `cell_8_coldcard_master_xpub_plumbing_byte_exact` (supplied case) and `cell_9_coldcard_master_xpub_absent_omits_top_level_xpub` (absent case). All other resolution arms (Phrase / Entropy / Wif / synthesize-test-helper / verify-bundle-rebuild) set `master_xpub: None` since master_xpub semantically only exists on user-supplied watch-only xpub slots.
- **Tier:** `v0.8.2`

### `coldcard-bip86-generic-export-pending-firmware` — `--template bip86 --format coldcard` refuses (BIP-86 not in upstream schema)

- **Surfaced:** v0.8.1 Phase 1 (SPEC R1-I2 reviewer-loop fold).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_generic_json`.
- **What:** Coldcard's canonical `generic-wallet-export.md` (upstream master) documents only `bip44` / `bip49` / `bip84` sub-objects. BIP-86 (P2TR singlesig) has no slot in the schema. The toolkit refuses `--template bip86 --format coldcard` with the SPEC §5.1 byte-exact pointer until Coldcard firmware extends the schema. Workaround: use `--format bitcoin-core` (descriptor passthrough) or `--format sparrow` (native P2TR support).
- **Status:** open (pending Coldcard firmware). Last upstream-checked **2026-05-12**: `gh api repos/Coldcard/firmware/contents/docs/generic-wallet-export.md` — no `bip86` / `p2tr` / `taproot` mentions. `releases/ChangeLog.md` — no taproot / schnorr / bip86 entries.
- **Tier:** `v1+`

### `coldcard-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format coldcard`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs::emit_coldcard_multisig_text`.
- **What:** Coldcard's multisig text emitter ingests only `P2WSH` / `P2SH-P2WSH` / `P2SH` formats per the SPEC §5.2 `Format` field. Taproot-multisig (tr-multi-a / tr-sortedmulti-a) is not in the firmware's import surface. The toolkit refuses with a pointer at `--format bitcoin-core` (descriptor) / `--format sparrow` for taproot multisig watch-only setup. Companion: Jade has the same gap (`jade-tr-multi-a-pending-firmware` below).
- **Status:** open (pending Coldcard firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `releases/ChangeLog.md` — no taproot / schnorr entries; firmware most recent commit `ca06dfd2` 2026-04-25 is unrelated regression fix.
- **Tier:** `v1+`

### `jade-tr-multi-a-pending-firmware` — `--template tr-multi-a` / `tr-sortedmulti-a` refuses under `--format jade`

- **Surfaced:** v0.8.1 Phase 1.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/jade.rs::JadeEmitter::emit`.
- **What:** Blockstream Jade's `register_multisig.multisig_file` accepts the Coldcard §5.2 multisig text shape; taproot multisig is not yet in that surface. The toolkit refuses with a pointer at `--format bitcoin-core` / `--format sparrow` for taproot multisig. Companion: `coldcard-tr-multi-a-pending-firmware` above (Jade shares the schema; once Coldcard ships, Jade follows).
- **Status:** open (pending Blockstream Jade firmware taproot-multisig support). Last upstream-checked **2026-05-12**: `Blockstream/Jade:CHANGELOG.md` — singlesig BIP-86 P2TR SHIPPED (`Add support for signing bip86 single-key p2tr inputs and for registering bip86 p2tr(key) descriptors`); taproot **multisig** (`multi_a` / `sortedmulti_a`) NOT yet shipped. Entry remains accurately open for the multisig case; singlesig P2TR is already a separate emitter path (Sparrow `tr(@0/**)`).
- **Tier:** `v1+`

### `electrum-non-latin-wordlists` — Electrum native seed format hard-codes the English wordlist

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs` (wordlist embedding + lookup).
- **What:** Electrum supports 9 wordlists across English, Japanese, Spanish, Chinese (simplified/traditional), Korean, Italian, Portuguese, Dutch, etc. v0.7 Phase 3 ships only the English wordlist; non-English Electrum users cannot decode their phrases through `mnemonic convert`. Add a `--language` parameter mirroring BIP-39 + bundle the additional embedded wordlists.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — embedded 4 non-English Electrum wordlists (zh-Hans, ja, pt, es) from `spesmilo/electrum` upstream commit `e1099925e30d91dd033815b512f00582a8795d25`. Plan correction noted: upstream Electrum has 5 total wordlists, not 9 (zh-Hant, German, French, Italian are NOT upstream). Separate `--electrum-language` flag distinct from `--language` (R1-I2 lock); `--electrum-language` wins on Electrum arms (R2-L2 lock). Portuguese is base-1626 (Monero copyright header); base-N arithmetic correctly parameterized.
- **Tier:** `v0.8`

### `electrum-encode-iteration-bound` — encode mining loop has no upper iteration cap

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::encode_phrase` (or equivalent encode-from-entropy path).
- **What:** Electrum's encode direction iterates by incrementing a nonce until the HMAC-SHA512 prefix matches the requested SeedVersion (`01` standard / `100` segwit). The loop has no iteration bound; on adversarially-chosen entropy or for rare versions, the search may run unboundedly long. Add a sane upper bound (e.g., `2^24` iterations) with a byte-exact stderr refusal on exhaustion.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `MAX_ENCODE_ITERATIONS = 1<<20` cap on `entropy_to_phrase` rejection-search loop. New `ElectrumError::EncodeIterationBoundExceeded` mapped to user-visible refusal.
- **Tier:** `v0.8`

### `electrum-version-info-stderr` — decode emits the detected SeedVersion silently

- **Surfaced:** v0.7 Phase 3 review (commit `69ac560`).
- **Where:** `crates/mnemonic-toolkit/src/electrum.rs::decode_phrase` + `cmd::convert::run` electrum arm.
- **What:** When `mnemonic convert --from electrum-phrase=... --to entropy` decodes a phrase, the toolkit dispatches via the SeedVersion prefix (`01` / `100` / `101` / `102`) but does not surface which version it detected. Adding a stderr info-line (e.g., `info: Electrum SeedVersion=01 (standard)`) parallel to the SLIP-0132 input-normalization note (SPEC §11) would help users confirm the dispatch matches their wallet's expectation.
- **Status:** `resolved 5dc83eb` (v0.8 Phase 2).
- **Resolution:** v0.8 Phase 2 — `note: detected Electrum SeedVersion <01|100> (<standard|segwit>)` emitted to stderr on decode arms. `compute_outputs` extended to triple-tuple return surfacing the detected SeedVersion.
- **Tier:** `v0.8-nice-to-have`

### `tr-multi-a-tr-sortedmulti-a-export-wallet-support` — `mnemonic export-wallet` refuses taproot multisig templates

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (descriptor pipeline + taproot-multisig validator).
- **What:** `mnemonic export-wallet` refuses `--template tr-multi-a` and `--template tr-sortedmulti-a` at runtime with an error pointing at this v0.8 deferral. Reason: taproot multisig descriptors require an internal-key designation (NUMS point or shared key) plus the script-path tree; the export-wallet pipeline doesn't yet thread the internal-key choice through to Bitcoin Core / BIP-388 formatters. Single-leaf `tr` (BIP-86) IS supported.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `--taproot-internal-key <nums|@N>` flag designates the BIP-341 internal key. NUMS uses the canonical reference `50929b74...0ac0` x-only point; `@N` makes cosigner N the key-path key (removed from multi_a leaf set). v0.7 stub refusal replaced with flag-pointing message. Bounds-checked + n=1 degenerate refusal.
- **Tier:** `v0.8`

### `export-wallet-descriptor-bip388-interop` — `--descriptor` mode + `--format bip388` is refused

- **Surfaced:** v0.7 Phase 5 code-quality review (commit `f8369d3`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export.rs` (format dispatch + descriptor-mode validator).
- **What:** `mnemonic export-wallet --descriptor <user-supplied> --format bip388` is refused at runtime; `--descriptor` mode currently only supports `--format bitcoin-core`. Reason: user-supplied descriptors arrive as opaque strings; converting them to BIP-388 `wallet_policy` requires re-parsing into the placeholder-template form (`@0/<0;1>/*`), which the watch-only template-mode pipeline already does but the descriptor-mode pipeline skips.
- **Status:** `resolved 86647ca` (v0.8 Phase 3).
- **Resolution:** v0.8 Phase 3 — new `descriptor_to_bip388_wallet_policy` helper parses canonical descriptor via miniscript, iterates `iter_pk()` to collect `[fp/path]xpub` keys (stripping `/<0;1>/*`), strips `#checksum`, and replaces each full key-expression with `@N/**` placeholder via longest-first substitution. Refused for non-multipath descriptors.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-dice-applications` — RSA / RSA-GPG / DICE BIP-85 applications deferred

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` application dispatch + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** BIP-85 application codes `828365'` (RSA), `67797633'` (RSA-GPG), `89101'` (DICE) are refused with v0.8 deferral stubs. Reason: RSA derivation requires an RSA crate not currently in the dep tree; DICE is a niche application (deterministic dice-roll output) with limited demand. The 6 in-scope applications (`bip39`, `hd-seed`, `xprv`, `hex`, `password-base64`, `password-base85`) cover the primary use cases.
- **Status:** `split-resolved 1dde4dc` (v0.8 Phase 7); split into `bip85-dice-application` (resolved) + `bip85-rsa-rsa-gpg-applications` (re-tiered).
- **Resolution:** v0.8 Phase 7 — DICE shipped with BIP85-DRNG-SHAKE256 + rejection sampling per BIP-85 v1.3.0 §"DICE". Spec reference vector (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`) pinned. New `--dice-sides` flag. New `sha3 = "0.10"` direct dep. RSA + RSA-GPG re-tiered to v0.9 / pending-rsa-crate-stability per Phase 6 SPIKE (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`): RUSTSEC-2023-0071 Marvin-attack timing sidechannel is **unpatched** (`patched = []`); rsa crate is in extended pre-release (`v0.10.0-rc.18`). Reopen criteria: rsa crate publishes patched stable release OR user requests with stated downstream use case.
- **Tier:** `v0.8` (DICE) / `v0.9` (RSA + RSA-GPG)

### `bip85-passphrase-protected-master` — `--from phrase=` + `--passphrase` direct path

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface.
- **What:** `mnemonic derive-child --from xprv=...` requires xprv input. A user with a passphrase-protected BIP-39 phrase must currently route through `mnemonic convert --from phrase=... --passphrase ... --to xprv` first, then pipe the xprv to `derive-child`. A direct `--from phrase=... --passphrase ...` path on `derive-child` would be more ergonomic.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `--from phrase=...` accepted; internal `phrase → seed → master xprv` (mainnet, BIP-85-network-agnostic) before BIP-85 derivation. New `--passphrase` for BIP-39 mnemonic extension.
- **Tier:** `v0.8-nice-to-have`

### `bip85-non-english-bip39-language-codes` — `--language` flag inert for BIP-39 application

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_bip39`.
- **What:** `--language` is plumbed through clap on `mnemonic derive-child` but ignored for BIP-85's `bip39` application. BIP-85 supports 9 wordlists (`0'` English through `8'` Czech) for the BIP-39 application via the language-index sub-path component. v0.7 hardcodes English (`0'`).
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `resolve_bip85_language` maps `CliLanguage` → (BIP-85 path code, `bip39::Language`). 9 BIP-85-coded languages supported. Portuguese refused (no BIP-85 code assigned).
- **Tier:** `v0.8`

### `bip85-testnet-emission` — `--network` flag inert for hd-seed / xprv applications

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs` clap surface + `bip85::derive_hd_seed` / `derive_xprv`.
- **What:** `--network` is plumbed through clap but unused. v0.7 hardcodes mainnet WIF / xprv emission for `--application hd-seed` and `--application xprv`. Testnet users must post-process via `mnemonic convert` to swap version bytes.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `format_hd_seed_wif` + `format_xprv_child` now take `NetworkKind` parameter. Testnet emits `c…` WIF / `tprv…` xprv. Driven by `--network` flag (default mainnet to match BIP-85 spec test vectors).
- **Tier:** `v0.8`

### `bip85-spec-prose-byte-formula-clarification` — SPEC §3 prose vs. worked-example formula mismatch

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §3 (BIP-39 byte slicing).
- **What:** SPEC §3 prose says BIP-39 byte slicing uses `2 * length_in_words / 3`; the worked examples (12 words → 16 bytes, 24 words → 32 bytes) match the correct formula `words * 4 / 3`. The two are equivalent for word counts divisible by 3 but the prose formula is not the canonical BIP-39 form. Pure SPEC text fix — implementation is correct.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — `2 * length_in_words / 3` → `length_in_words * 4 / 3` in SPEC §3.
- **Tier:** `v0.7-nice-to-have`

### `bip85-stdin-master-xprv` — `--from xprv=-` parses but does not read stdin

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `crates/mnemonic-toolkit/src/cmd/derive_child.rs::run`.
- **What:** `mnemonic derive-child --from xprv=-` parses through clap (the `=-` sentinel is recognized) but `derive_child::run` does not read stdin to populate the xprv value. `mnemonic convert` does honor `=-`. Add stdin-read parity for cross-subcommand consistency.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — `derive_child::run` now reads stdin when `args.from.value == "-"` via `crate::cmd::convert::read_stdin_to_string` (made `pub(crate)`). Works for both `xprv=-` and `phrase=-`.
- **Tier:** `v0.8`

### `derive-child-spec-2-grammar-uniformity-tension` — SPEC §2 prose-internal contradiction on `--length` mandatoriness

- **Surfaced:** v0.7 Phase 6 review (commit `edaa959`).
- **Where:** `design/SPEC_derive_child_v0_7.md` §2.
- **What:** SPEC §2 has internal tension: it says `--length` is mandatory at clap level AND that `--length` is refused for `--application hd-seed` / `--application xprv`. Phase 6 implementation adopted a sentinel-0 convention: clap requires `--length`, and `hd-seed`/`xprv` arms refuse only when the supplied value is non-zero (`0` is treated as sentinel-absent). The SPEC text should be edited to reflect this; current prose reads as a contradiction.
- **Status:** `resolved 4dfea5a` (v0.8 Phase 0).
- **Resolution:** v0.8 Phase 0 — SPEC §2 + §4 prose updated to document the sentinel-0 convention canonically.
- **Tier:** `v0.7-nice-to-have`

### `bip38-ec-multiplied-encrypt-mode-support` — emit BIP-38 EC-multiplied form via intermediate codes

- **Surfaced:** v0.7.1 Phase 3 (BIP test vector audit cycle); rescoped from `bip38-ec-multiplied-mode-support` after Phase 3 forensics.
- **Where:** `crates/mnemonic-toolkit/src/cmd/convert.rs` `(Wif, Bip38)` arm; `bip38 = "1.1"` crate.
- **What:** v0.7.1 supports BIP-38 EC-multiplied DECRYPT transparently (4 spec vectors pinned). ENCRYPT to EC-multiplied form requires the intermediate-code workflow per BIP-38 §"Generation of intermediate code": the passphrase owner generates a passphrase code; a third party combines it with random entropy to derive the encrypted privkey + the corresponding bitcoin address. Implementation: new subcommand `mnemonic intermediate-code` (or `--passphrase-code <code>` flag on the `(Wif, Bip38)` arm). Out of scope for v0.7.1 vectors-only audit.
- **Why deferred:** v0.8 Phase 4 SPIKE returned DEFER verdict (`design/agent-reports/v0_8-phase-4-bip38-ec-mult-encrypt-spike.md`). The `bip38 v1.1.1` `Generate` trait covers owner-only path only with internal `rand::thread_rng()` (non-deterministic) and exposes no intermediate-code workflow + no confirmation code. Hand-rolling spec-compliant API costs ~155 LOC of cryptographic code (AES + scrypt + secp256k1 + Unicode normalization). Marginal user value (paper-wallet niche). Re-tiered to `v0.8.1+`.
- **Status:** `open`
- **Tier:** `v0.8.1+`

### `bip38-spec-section-12-ec-multiplied-erratum` — SPEC §12 incorrectly claimed EC-multiplied was refused

- **Surfaced:** v0.7.1 Phase 3 (audit cycle).
- **Where:** `design/SPEC_convert_v0_6.md` §12.
- **What:** The v0.7.0 SPEC §12 stated the `bip38` crate's `Decrypt` impl rejected EC-multiplied codes. Empirical testing in Phase 3 disconfirmed: all 4 EC-multiplied spec vectors decrypt correctly. SPEC §12 corrected in this cycle (commit pinned in matrix). Filed for cross-referencing the erratum source: the v0.7 Phase 1 security review report at `design/agent-reports/v0_7-phase-1-bip38-security-review.md` likely contains the source claim — re-read on next sec-review touch.
- **Why deferred:** documentation-only; closed in this cycle. Filed for audit history continuity.
- **Status:** `resolved 2c59b27`
- **Tier:** `v0.7.1`

### `bip85-dice-application` — BIP-85 `89101'` dice rolls (split product of `bip85-rsa-rsa-gpg-dice-applications`)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision.
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs::format_dice_rolls` + `crates/mnemonic-toolkit/src/cmd/derive_child.rs` dispatch.
- **What:** BIP-85 §"DICE" deterministic dice rolls via SHAKE256 BIP85-DRNG + rejection sampling. Spec at BIP-85 v1.3.0 §"DICE".
- **Status:** `resolved 1dde4dc` (v0.8 Phase 7).
- **Resolution:** v0.8 Phase 7 — `--application dice` + new `--dice-sides <N>` flag. Spec reference vector pinned (`m/83696968'/89101'/6'/10'/0'` → `1,0,0,2,0,1,5,5,2,4`). New `sha3 = "0.10"` direct dep.
- **Tier:** `v0.8`

### `bip85-rsa-rsa-gpg-applications` — BIP-85 RSA + RSA-GPG (split product, deferred)

- **Surfaced:** v0.8 Phase 6 SPIKE split decision (`design/agent-reports/v0_8-phase-6-rsa-crate-security-review.md`).
- **Where:** `crates/mnemonic-toolkit/src/bip85.rs` (would need new app dispatchers); `crates/mnemonic-toolkit/src/cmd/derive_child.rs` (would lift `rsa` / `rsa-gpg` from out-of-scope refusal).
- **What:** BIP-85 application codes `828365'` (RSA) + `67797633'` (RSA-GPG) generate RSA keys deterministically from BIP-85 entropy. Implementation requires the `rsa` crate.
- **Why deferred:** v0.8 Phase 6 SPIKE returned DEFER verdict. RUSTSEC-2023-0071 (Marvin attack: timing sidechannel against PKCS#1 v1.5 decryption) is **unpatched** as of 2026-05-07 (`patched = []`). `rsa` crate is in extended pre-release (`v0.10.0-rc.18`). Adding it as direct dep would import an open advisory into mnemonic-toolkit's `cargo audit` output. BIP-85 RSA / RSA-GPG demand signal is absent.
- **Reopen criteria:** rsa crate publishes patched stable release (`patched = ["X.Y.Z"]` in advisory) OR a user requests BIP-85 RSA / RSA-GPG with a stated downstream use case.
- **Status:** `open`
- **Tier:** `v0.9 / pending-rsa-crate-stability`

### `18-remaining-bip39-trezor-corpus-vectors` — pin remaining 18 of 24 Trezor english corpus cells

- **Surfaced:** v0.7.1 Phase 1.B (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip39_vectors.rs`.
- **What:** v0.7.1 Phase 1.B pinned 6 of 24 BIP-39 §"Test Vectors" English Trezor corpus cells via hand-rolled tests; the remaining 18 stayed MISSING per the v0.7.1 audit matrix. v0.8 lifts to a parametric loop over the full corpus.
- **Status:** `resolved 85694b2` (v0.8 Phase 8).
- **Resolution:** v0.8 Phase 8 — refactored `cli_convert_bip39_vectors.rs` to a single `bip39_trezor_english_corpus_full` test that loops over all 24 english entries via vendored `tests/bip39_trezor_vectors.json` (Trezor `python-mnemonic` SHA `b57a5ad77a981e743f4167ab2f7927a55c1e82a8`). Audit-matrix coverage 6/24 → 24/24 ✓.
- **Tier:** `v0.7.1-carry`

### `bip38-spec-vector-3-null-byte-passphrase` — V3 Unicode passphrase contains U+0000; not representable via argv

- **Surfaced:** v0.7.1 Phase 3.A (BIP test vector audit cycle).
- **Where:** `crates/mnemonic-toolkit/tests/cli_convert_bip38.rs::{encrypt,decrypt}_..._spec_vector3_unicode_nfc_passphrase` (`#[ignore]`'d); `crates/mnemonic-toolkit/src/cmd/convert.rs` passphrase input plumbing.
- **What:** BIP-38 §"Test vectors" vector 3 specifies a passphrase of 5 codepoints (U+03D2 + U+0301 + U+0000 + U+10400 + U+1F4A9). The U+0000 NULL byte cannot be passed via argv (POSIX `execve` truncates at NULL); the existing `--passphrase=-` stdin path also fails because `read_stdin_to_string` calls `.trim()`. To exercise this vector end-to-end the toolkit needs a NULL-safe input channel — e.g. `--passphrase-bytes-hex <hex>` accepting the raw byte sequence, or a stdin path that reads bytes verbatim (no trim, no UTF-8 reinterpretation). The `bip38` crate itself NFC-normalizes whatever string slice it receives; the gap is purely at the toolkit's input plumbing.
- **Status:** `resolved 2eef44b` (v0.8 Phase 1).
- **Resolution:** v0.8 Phase 1 — new `--passphrase-stdin` flag with line-ending-only trim (preserves leading/trailing spaces + internal NULL). Both V3 ignored tests unignored and now active. Phase 1 review I1 added a separate `read_stdin_passphrase` helper distinct from `read_stdin_to_string` to prevent the trim issue.
- **Tier:** `v0.8`

### `electrum-seed-version-spike-pending` — Phase 4 step 0 interactive spike

- **Surfaced:** v0.8.1 Phase 4 (`design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs:33` — `ELECTRUM_SEED_VERSION_PIN = 17`.
- **What:** SPEC v0.8 §9 + IMPL_PLAN Phase 4 step 0 mandate an interactive spike against current Electrum (>= 4.5.x) to lock `ELECTRUM_SEED_VERSION_PIN` to a verified-cleanly-imports value.
- **Status:** `resolved` (2026-05-12 spike against Electrum 4.5.5).
- **Resolution:** Spike executed against Electrum 4.5.5 in `/tmp/electrum-spike-venv/`. Empirical result: a toolkit-emitted wallet file with `seed_version: 17` loads cleanly via `electrum --offline -w <file> listaddresses` (returns the expected BIP-84 receive set; Electrum migrates the in-memory state to FINAL_SEED_VERSION=59 on save). Source-code cross-check at `wallet_db.py:1195-1211` confirms `seed_version >= 12 → return seed_version` with no rejection at 17. Pin retained at 17 (the SPEC's "minimum cleanly-imports" specification matches 17; 59 is what Electrum WRITES, not the minimum it ACCEPTS). Full report: `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`.
- **Tier:** `v0.8.2`

### `electrum-tr-multi-a-pending-libsecp-taproot` — `--template tr-multi-a` refuses under `--format electrum`

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit()` guard; refusal fixture `tests/export_wallet/electrum_tr_multi_a_refusal.stderr`.
- **What:** Electrum's `wallet_db.py` does not currently ingest taproot multisig wallet shapes (pending libsecp-taproot integration in Electrum's signer surface). `--format electrum --template tr-multi-a` (or `tr-sortedmulti-a`) emits a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow` (which supports taproot multisig via descriptor-passthrough).
- **Status:** `open` (last upstream-checked 2026-05-12 against Electrum 4.5.5 source; `grep -E "'p2tr'|p2tr" electrum/transaction.py` returns no matches in the script-type enum, confirming taproot script type not yet wired).
- **Tier:** `v1+ / pending-electrum-firmware`

### `electrum-final-seed-version-drift` — track Electrum FINAL_SEED_VERSION upstream

- **Surfaced:** v0.8.1 Phase 4.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` — `ELECTRUM_SEED_VERSION_PIN` doc-comment.
- **What:** Electrum's `wallet_db.py` `FINAL_SEED_VERSION` drifts upward over releases (4.5.5 = 59; the v0.8.1 SPEC §9 cited 71 from master at SPEC-write time). Toolkit pins to 17 (minimum cleanly-imports) and relies on Electrum's migration loader to walk forward. Track in case the loader ever drops support for old migration paths.
- **Status:** `open` (no fix scheduled; tracking only).
- **Tier:** `v1+ / informational`

### `electrum-root-fingerprint-roundtrip-quirk` — Electrum nulls `root_fingerprint` on load

- **Surfaced:** v0.8.1 Phase 4 step 0 spike (2026-05-12, Electrum 4.5.5).
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/electrum.rs` `emit_electrum_standard_json` + `electrum/keystore.py` `BIP32_KeyStore`.
- **What:** The toolkit emits `keystore.root_fingerprint` per SPEC §9.1 (e.g., `"5436d724"`). Electrum 4.5.5's loader successfully imports the wallet, derives the correct BIP-84 addresses, but its re-serialized form has `"root_fingerprint": null` — the `_root_fingerprint` private attribute on the in-memory `BIP32_KeyStore` is not populated from the on-disk JSON field. Functionally inert for watch-only address derivation; required only for PSBT-with-origin flows. Likely an Electrum-side bug or intentional drop; cross-check against current master may surface a fix.
- **Status:** `open` (informational).
- **Tier:** `v1+ / informational`

### `green-native-multisig-pending-server-support` — `--format green` refuses multisig

- **Surfaced:** v0.8.1 Phase 5.
- **Where:** `crates/mnemonic-toolkit/src/wallet_export/green.rs` `emit()` guard; refusal fixture `tests/export_wallet/green_multisig_refusal.stderr`.
- **What:** Blockstream Green's multisig surface is server-mediated (Green Multisig Shield + Liquid), not a direct file-import shape. `--format green` is therefore singlesig-only; multisig templates return a byte-exact refusal with pointer to `--format bitcoin-core` (descriptor) or `--format sparrow`. Resolves once Green publishes a self-custody multisig file-import format.
- **Status:** `open`. Last upstream-checked **2026-05-12**: Green Help Center article `19340800530713-Set-up-watch-only-wallet` returns HTTP 403 to programmatic fetchers (Zendesk-hosted, browser-only). Status cannot be verified autonomously; entry remains open pending manual browser check.
- **Tier:** `v1+ / pending-green-server-support`

### `mnemonic-gui-schema-mirror` — companion to `bg002h/mnemonic-gui` schema gate

- **Companion:** `bg002h/mnemonic-gui` `FOLLOWUPS.md` entry `mnemonic-gui-schema-mirror`; CI gate at `.github/workflows/schema-mirror.yml`.
- **Where:** This CLI's clap-derive `Args` blocks (currently `cmd/{bundle,verify_bundle,convert,export_wallet,derive_child}.rs`); the introspection subcommand at `cmd/gui_schema.rs`.
- **What:** The `mnemonic-gui` GUI mirrors this CLI's clap-derive flag surface at pinned tag `mnemonic-toolkit-v0.9.0` (was `v0.8.1` pre-v0.2). Any flag add / remove / rename / `conflicts_with` / `required_unless_present_any` change in this repo's CLI surface must land in lockstep with a companion `mnemonic-gui` PR that bumps the schema + the `pinned-upstream.toml` tag for this CLI. The `mnemonic-gui` CI gate runs `cargo install --locked --git <this-repo> --tag <pin>` + `cargo test --test schema_mirror`, so drift surfaces as a CI failure. Additionally, the GUI's `build.rs` codegen reads `crates/mnemonic-toolkit/src/cmd/convert.rs::NodeType::is_secret_bearing()` and `crates/mnemonic-toolkit/src/slot_input.rs::SlotSubkey::is_secret_bearing()` via `syn::parse_file` to populate its `SECRET_*` constants — drift in those impls is also caught by a runtime source-audit test in the GUI repo.
- **v0.2 update (2026-05-12, mnemonic-toolkit v0.9.0):** `mnemonic gui-schema` introspection subcommand shipped (SPEC §7 contract). The GUI consumes its JSON output instead of (or alongside) the `syn` codegen path. `cli_gui_schema.rs` (16 tests) pins the SPEC §7 contract on this side. The companion `mnemonic-gui` v0.2 Phase C.2 PR consumes the schema via `cargo run -p mnemonic-toolkit -- gui-schema` at build time.
- **Status:** `open` (mirror-invariant; tracking only — every flag-surface PR carries this lockstep work).
- **Tier:** `v1 / mirror-invariant`
