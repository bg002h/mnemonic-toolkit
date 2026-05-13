# cross-repo SPEC — v0.9.0 secret-memory hygiene cycle (Cycle A: OWNED-buffer first-pass)

**Cycle name:** v0.9.0 secret-memory-hygiene **Cycle A** — OWNED-buffer
first-pass: argv-leakage close + Zeroizing discipline on owned secret
buffers in CLIs.
**Companion cycle (deferred):** v0.9.0 Cycle B — `mlock(2)` /
`VirtualLock` infrastructure for long-lived heap-allocated secrets.
Cycle B SPEC will be written after Cycle A ships; survey precursor
shared with Cycle A (this cycle).
**Ship tags:** `ms-codec-v0.1.3` + `mnemonic-toolkit-v0.9.2` (patch
tags per impl-Drop approach — see §3 OOS-pub-struct-drop).
**Date:** 2026-05-13
**Status:** DRAFT R3-folded — awaiting R4 architect re-review.
**Predecessor cycle:** v0.8.0 (BIP-vector adoption);
[`SPEC_test_vector_audit_v0_8_0.md`](SPEC_test_vector_audit_v0_8_0.md).
**Survey precursor:** [`agent-reports/v0_9_0-secret-memory-survey.md`](agent-reports/v0_9_0-secret-memory-survey.md).
**Architect review history:** R1 (Sonnet, 2C/3I/2N) → R2 (Sonnet,
0C/0I/1N) → R3 (Opus, 3C/4I/2N, SPLIT-CYCLE) → user decision (split
cycle + impl Drop + drop md/mk stubs) → this revision → R4 pending.
Persisted: [`agent-reports/v0_9_0-phase-0-spec-plan-r1.md`](agent-reports/v0_9_0-phase-0-spec-plan-r1.md).

**Cross-repo coordination:** per CLAUDE.md mirror-invariant. Two
touched repos (`mnemonic-toolkit`, `mnemonic-secret`) carry companion
`secret-memory-hygiene-v0_9-cycle-a` FOLLOWUPS entries. `descriptor-mnemonic`
and `mnemonic-key` are out of scope (no private-key material) and
get no symmetry-stubs (R3 I-R3-4 fold).

## §1 Purpose

Cycle A is the OWNED-buffer first-pass at secret-memory hygiene for
the m-format constellation CLIs. It closes two orthogonal gaps
surfaced by the cross-repo secret-memory surface survey
([`agent-reports/v0_9_0-secret-memory-survey.md`](agent-reports/v0_9_0-secret-memory-survey.md)).
A third gap (`mlock(2)` infrastructure) was originally scoped into
the cycle but split out to a separate Cycle B per R3 architect-review
recommendation (cycle was overscoped at ~3-4 weeks; Phase 2+3 sequential
with cross-platform FFI introducing new regression-class risk). No new
product feature; no wire-format change.

Cycle A delivers:

1. **Closes argv leakage on every secret-bearing CLI flag** that
   currently lacks a stdin alternative. The survey §5 enumerates
   exactly which flags route their values into `/proc/N/cmdline`
   today. Toolkit's `bundle` / `verify-bundle` / `derive-child`
   passphrase + slot-secret flags have no stdin escape;
   `convert --bip38-passphrase` has none either. Add `=-` /
   `--*-stdin` escapes; add a runtime warning when an inline secret
   is detected and a stdin alternative exists; add a lint-test that
   fails CI if any new clap-derived secret-bearing field is added
   without a stdin escape registered.

2. **Adds `zeroize` discipline across owned secret buffers** in
   `ms-codec` + `ms-cli` + `mnemonic-toolkit`. Every survey-§1 row
   tagged OWNED + zeroize=none gets scrub-on-drop semantics at its
   originating allocation site. Two patterns are used depending on
   the call site's API constraints:

   - **Local `Zeroizing<...>` wrappers** for function-local secret
     allocations — straight `Zeroizing::new(...)` wrap.
   - **`impl Drop`** for owned-secret fields in `pub` structs
     (e.g., `DerivedAccount.entropy`) — preserves public field
     types (no breaking change to library API) while scrubbing on
     drop. Trade-off: blocks move-out destructuring patterns on
     affected structs; we accept this as a narrow break (residual
     semver risk documented at §3 OOS-pub-struct-drop).

   The five sites that derive a BIP-32 master seed from a BIP-39
   mnemonic (survey §6 bullet 2) share a new
   `derive_slot::derive_master_seed(&Mnemonic, &str) -> Zeroizing<[u8; 64]>`
   helper for the seed-derivation step. Per-site master/account/leaf
   derivation remains site-specific (each site differs on input
   type, network handling, derivation-path source, and return
   shape; full spine consolidation is not straight-line-deliverable
   per R3 C-R3-2 finding, and was not the original intent of the
   helper).

   Third-party `bip39::Mnemonic` and `bitcoin::bip32::Xpriv` are
   upstream-blocked (survey §3); the cycle minimizes their lifetime
   and documents the residual gap honestly (no false reassurance
   that "drop after last use" tightens anything — `Xpriv: Copy`
   means `drop(xpriv)` is a memory-no-op).

**What Cycle A does NOT close** (honest scope):

- `bitcoin::bip32::Xpriv` interior 32-byte private-key scalar +
  chain code residue (Copy-bound; upstream-blocked).
- `bip39::Mnemonic` wordlist-resolved phrase residue
  (upstream-blocked).
- `mlock(2)` / page-pinning against swap (deferred to Cycle B).
- libc `OsString` / `env::args_os()` pre-clap copy of argv (kernel
  + libc layer; addressable only by `prctl(PR_SET_DUMPABLE)` or
  raw FFI argv rewrite).
- Allocator-pool page residue after `Zeroizing` drop (glibc /
  mimalloc layer; addressable only by custom allocator or
  dedicated secret-arena placement).

These deferrals are explicit OOS entries (§3). Cycle A's positioning
is "scrubbed-on-drop for OWNED buffers we allocate; argv-resistant
for CLI-facing flags." It is **not** a full secret-memory mitigation;
it is the OWNED-buffer baseline on which Cycle B (mlock) and
upstream-PR follow-ons build.

## §2 Coverage deltas

Per repo, post-v0.9.0 vs the v0.8.0 baseline. Counts derived by
walking the survey tables row-by-row at Phase 0 close. **Counts
corrected post-R1** ([`agent-reports/v0_9_0-phase-0-spec-plan-r1.md`](agent-reports/v0_9_0-phase-0-spec-plan-r1.md))
where the initial draft had argv-flag count + ms-cli OWNED-row
miscounts (R1 C-1 + C-2 folds).

| Repo | Hygiene class | v0.8.0 covered | v0.9.0 target | Delta |
|---|---|---|---|---|
| `mnemonic-toolkit` | argv leakage close | 11 / 20 inline-secret flag-rows have stdin escape | 20 / 20 | +9 (via 5 distinct implementation changes — see plan Phase 1) |
| `mnemonic-toolkit` | zeroize discipline on survey-§1 OWNED rows | 0 / ~30 toolkit rows with OWNED component | full coverage | every OWNED allocation in scope; verified at Phase 2 R1 |
| `ms-codec` | zeroize discipline on survey-§1 OWNED rows | 0 / 4 ms-codec production OWNED rows | 4 / 4 | +4 (internal-only — see §3 OOS-public-payload) |
| `ms-cli` | argv leakage close | 5 / 5 inline-secret flag-rows already have stdin escape | 5 / 5 | +0 (ms-cli is Phase 2-only) |
| `ms-cli` | zeroize discipline on survey-§1 OWNED rows | 0 / 10 ms-cli OWNED rows | 10 / 10 | +10 (incl. 3 clap-field rows added post-R1) |
| `mnemonic-toolkit` | mlock on long-lived heap secrets | 0 / 3 survey-§4 candidates | (deferred to Cycle B) | — (Cycle B scope) |

**Toolkit argv count derivation (deterministic re-walk of survey §5
toolkit table):** 20 total flag-rows. 11 carry "YES" (have stdin):
8 `convert --from <node>=<value>` variants
(phrase/entropy/xprv/wif/ms1/bip38/minikey/electrum-phrase),
`convert --passphrase`, `derive-child --from xprv`,
`derive-child --from phrase`. 9 carry "NO" (need closure):
`bundle --passphrase`, `bundle --slot @N.{phrase,entropy,wif,xprv}=`
(4 rows, all routed through one parser extension),
`verify-bundle --passphrase`, `verify-bundle --slot @N.<secret>=`
(1 grouped row, same parser extension), `convert --bip38-passphrase`,
`derive-child --passphrase`. 5 distinct implementation changes close
all 9: one `slot_input.rs::parse_slot_input` `=-` parser extension
(covers 5 flag-rows), four new `--*-stdin` flags (one each for
bundle/verify-bundle/derive-child `--passphrase` and one for
`convert --bip38-passphrase`).

**ms-cli argv count derivation:** survey §5 marks all 5 ms-cli
flag-rows YES — both `ms encode/verify --phrase` and `--hex` have
`--phrase -` / `--hex -` stdin routes; `ms decode <MS1>` and
`ms verify <MS1>` positional flags route through `-`. ms-cli has
no argv-closure work in Phase 1 (R1 I-2 fold).

**ms-cli OWNED-row count derivation:** 11 survey-§1 ms-cli data
rows; 10 carry OWNED component including the 3 clap-field rows
(`encode.rs:30` `EncodeArgs::phrase`, `encode.rs:34` `EncodeArgs::hex`,
`verify.rs:27` `VerifyArgs::phrase`) that the initial plan draft
missed (R1 C-2 fold). The remaining row (`decode.rs:67-94`,
`emit_json`/`emit_text` for the decode output) is primarily
STDOUT-LEAK and OOS-per-design (see §3 OOS-7).

**Toolkit Zeroizing count:** survey §1 toolkit table has 39 data
rows. Approximately 30 carry an OWNED component (rows with primary
disposition starting with "OWNED" — counted by filtering Zeroize
column). Phase 2 walks all survey-§1 toolkit rows row-by-row; every
OWNED allocation gets `Zeroizing` discipline at its originating
site. The "~30" is indicative — the deterministic acceptance gate
is "every OWNED allocation in production code from survey §1
carries Zeroizing discipline," verified at Phase 2 R1 by direct
survey walk (R1 N-1 fold: explicit "OWNED" qualifier added).

Net new hardening cells across Cycle A: 9 (argv) + (~44 zeroize
wraps: ~30 toolkit + 4 ms-codec + 10 ms-cli) ≈ **53** survey-derived
closures, plus the OOS-per-§3 deferrals (14 entries post-R3+R4, see
§3). Cycle B will add the 5 survey-§4 mlock candidates (#1-3
top-priority + #4-5 lower-priority, all named in §3 OOS-mlock-cycle-b).
Per-phase test counts will be reported at phase close.

## §3 Out-of-scope (filed for explicit closure)

The following hygiene improvements surfaced during the survey but are
deferred to a future cycle. Each gets a FOLLOWUP entry rather than a
silent skip.

- **Upstream `bitcoin::bip32::Xpriv` Copy + Zeroize bridge.** Per
  survey §3, `Xpriv` inherits `Copy` from `secp256k1::SecretKey`.
  Wrapping each binding in `Zeroizing<Xpriv>` only scrubs *that*
  binding's bytes; every `xpriv.derive_priv()` call leaves a fresh
  unscrubbed copy. Real fix requires upstream `rust-bitcoin` (and
  `rust-secp256k1`) to remove `Copy` and implement `Zeroize`.
  FOLLOWUP: `rust-bitcoin-xpriv-zeroize-upstream` (toolkit).

- **`bip39::Mnemonic` interior buffer.** Per survey §3, the
  `Mnemonic` type holds its wordlist-resolved phrase verbatim and
  drops un-scrubbed. The cycle minimizes its lifetime (construct →
  `to_entropy()`/`to_seed()` into a `Zeroizing` wrapper → drop ASAP)
  but cannot fully fix without an upstream PR. FOLLOWUP:
  `rust-bip39-mnemonic-zeroize-upstream` (toolkit).

- **`/proc/self/cmdline` post-parse overwrite.** mlock cannot
  retroactively cover the kernel-owned copy of argv in
  `/proc/N/cmdline`. The only mitigations are (a)
  `prctl(PR_SET_DUMPABLE, 0)` to deny `/proc/N/cmdline` reads to
  other UIDs, or (b) rewriting `argv[]` post-parse via the
  `crate clap_complete`/raw FFI route. v0.9.0 adds a warning when
  an inline secret is detected but does not rewrite argv. FOLLOWUP:
  `argv-overwrite-after-parse` (toolkit).

- **OOS-public-payload — `ms-codec` `Payload::Entr` public-API
  shape.** The codec's `Payload::Entr(Vec<u8>)` is the public-API
  shape. Adding `impl Drop for Payload` to scrub the variant
  on drop blocks move-out destructuring patterns
  (`let Payload::Entr(v) = payload` move) and is therefore a
  breaking change for external library consumers. Cycle A keeps
  `Payload::Entr(Vec<u8>)` shape AND no Drop impl on Payload;
  internal callers in `ms-codec` are tightened to use Zeroizing
  *behind* the public surface (encode/decode helpers' intermediate
  buffers); the public variant continues to be caller-managed
  (callers responsible for Zeroizing-wrapping the returned Vec).
  A future cycle can decide to break the API for a hardened
  `Payload`. FOLLOWUP: `ms-codec-payload-zeroize-public-api`
  (ms-codec).

- **OOS-pub-struct-drop — `impl Drop` adds a narrow break for
  move-out destructuring patterns.** For `pub struct` fields in
  `mnemonic-toolkit` holding owned secret material (chiefly
  `DerivedAccount.entropy: Vec<u8>` at `derive.rs:14`), Cycle A
  adds `impl Drop` to scrub on drop rather than changing the
  field type to `Zeroizing<Vec<u8>>` (the alternative would force
  a v0.10.0 minor bump per project's pre-1.0 SemVer convention).
  Trade-off: external code patterns like
  `let DerivedAccount { entropy, .. } = derived;` (move-out
  destructure) or `let entropy = derived.entropy;` will no longer
  compile post-fold (Rust disallows move-out of fields when the
  enclosing struct has `impl Drop`, errno E0509). Field-access
  via borrow (`&derived.entropy`) and Deref-via-method-call remain
  unchanged.

  **Phase 2 internal-callsite migration is required** (R4 C-R4-1
  fold; this is a Phase 2 prerequisite, not residual external
  risk). Three toolkit-internal sites currently move fields out
  of `DerivedAccount`:
  - `cmd/bundle.rs:325-329` (Phrase arm)
  - `cmd/bundle.rs:421-425` (entropy arm)
  - `synthesize.rs:741-744` (test move-out)

  Plan Phase 2 step 4 (DerivedAccount sub-bullet) describes the
  remediation: add a `DerivedAccount::into_parts(self) -> (...)`
  consuming method that `mem::take`s entropy and clones the Copy
  fields, then migrate the three sites to use `into_parts()`.

  We accept the *external* move-out break as a narrow patch-tag-
  compatible risk: external library users typically access secret
  fields via borrow, not move-out. **Residual semver risk:** if
  downstream library users surface move-out break complaints,
  the cycle re-tags retroactively as a minor bump. FOLLOWUP:
  `pub-struct-drop-semver-risk-monitor` (toolkit).

- **OOS-libc-osstring — pre-clap libc `OsString` residue.** The
  `std::env::args_os() -> Vec<OsString>` materialization in libc
  rt0 / Rust std allocates `OsString` heap buffers BEFORE
  `clap::Parser::parse()` is called. Those buffers are dropped
  un-scrubbed (the `OsString` allocation is distinct from the
  `String` allocation clap creates from it). Cycle A's
  `std::mem::take(&mut args.phrase)` scrubs clap's String but
  cannot reach the prior `OsString` copy. Addressable only by
  libc replacement or per-invocation raw-FFI argv intercept
  before clap. Mirrors mnemonic-gui's `secret_widget.rs`
  doc-comment caveat for parity. FOLLOWUP:
  `clap-argv-pre-parse-residue` (toolkit).

- **OOS-allocator-residue — allocator-pool page residue.** When
  a `Zeroizing<Vec<u8>>` drops, its bytes are zeroed but the
  underlying allocator page may be reused later for unrelated
  allocations; depending on glibc / jemalloc / mimalloc behavior,
  the zeroed bytes may persist or be overwritten by the next
  allocation. Mitigation requires a custom allocator with
  secret-class-aware page management or a dedicated secret arena
  (see OOS-secret-arena below). FOLLOWUP:
  `allocator-pool-residue` (toolkit, defense-in-depth class).

- **OOS-mlock-cycle-b — `mlock(2)` / `VirtualLock` infrastructure
  deferred to Cycle B.** Per R3 architect-review SPLIT-CYCLE
  recommendation, the originally-planned Phase 3 (cross-platform
  mlock module with capability-aware soft-fail) is deferred to a
  separate Cycle B. Cycle B SPEC will be written after Cycle A
  ships; survey precursor (`agent-reports/v0_9_0-secret-memory-survey.md`)
  shared with Cycle A. Cycle A's `Zeroizing` discipline (Phase 2)
  is a prerequisite for Cycle B's mlock — mlock is applied to
  Zeroizing-wrapped heap buffers from Cycle A.

  **Cycle B target set** (R4 I-R4-4 fold; all 5 survey-§4 mlock
  candidates explicitly named here):
  - **#1 (top-priority)** — `clap::Args`-derived passphrase /
    phrase / slot-value fields (`BundleArgs.passphrase`,
    `BundleArgs.slot[i].value`, `VerifyBundleArgs.passphrase`,
    `VerifyBundleArgs.slot[i].value`, `ConvertArgs.from[i].value`,
    `ConvertArgs.passphrase`, `ConvertArgs.bip38_passphrase`,
    `DeriveChildArgs.from.value`, `DeriveChildArgs.passphrase`,
    `EncodeArgs.phrase`, `EncodeArgs.hex`, `VerifyArgs.phrase`).
  - **#2 (top-priority)** — `ResolvedSlot.entropy: Option<Vec<u8>>`
    in `synthesize.rs:569-582`.
  - **#3 (top-priority)** — `DerivedAccount.entropy` Vec<u8>
    heap allocation in `derive.rs:14-20`.
  - **#4 (lower-priority)** — `bip85::derive_entropy` returned
    `[u8; 64]` — requires heap-promotion first (currently stack);
    touches 6 `format_*` callees in `bip85.rs`. Cycle A wraps
    with `Zeroizing<[u8; 64]>` in Phase 2 but defers
    heap-promotion + mlock to Cycle B.
  - **#5 (lower-priority)** — ms-cli `read_stdin()` String
    buffer (`parse.rs:45`). Cycle A wraps with `Zeroizing<String>`
    in Phase 2; Cycle B considers mlock for the stdin-buffer
    pre-trim window.

  FOLLOWUP: `secret-memory-hygiene-cycle-b` (toolkit) covers all 5
  candidates; Cycle B SPEC will further classify by priority.

- **OOS-secret-arena — dedicated mmap'd secret-arena placement.**
  `mlock(2)` pins pages, not bytes; future maintainers may need
  a dedicated mmap-allocated secret arena (à la rust `secrecy` /
  `secure_string` crates) for page-aligned heap placement of
  secret-class allocations. This avoids both the page-vs-byte
  granularity trap and the allocator-pool residue (OOS-allocator-residue
  becomes addressable via a dedicated allocator path). Out of
  scope for Cycles A and B both. FOLLOWUP:
  `dedicated-secret-arena` (toolkit, design class).

- **BIP-85 SHAKE256 state scrubbing.** Per survey §3, `sha3::Shake256`'s
  XOF reader state can carry BIP-85 child entropy until drop.
  Upstream `sha3` does not implement `Zeroize`. FOLLOWUP:
  `sha3-shake256-zeroize-upstream` (toolkit).

- **`bip38` crate internal decrypt buffers.** Per survey §3, the
  crate's internal scrypt intermediate state is not zeroize-aware.
  Toolkit can `Zeroizing`-wrap the *returned* tuple but not
  internals. FOLLOWUP: `bip38-crate-internal-zeroize-upstream`
  (toolkit).

- **OOS-7: ms-codec `lib.rs:18-19,29-30` doc-example row.** The
  survey §1 ms-codec table includes the public doc-test example
  carrying a literal entropy value. Doc-tests are not production
  secret material — the literal is a synthetic vector chosen for
  documentation, not a real secret. Wrapping it would add
  visual noise to the public API's documentation example without
  any security benefit. R1 I-1 fold. FOLLOWUP:
  `ms-codec-doc-example-zeroize-consistency` (ms-codec) — optional
  future cycle to apply Zeroizing in the doc-test for pattern
  consistency only.

- **OOS-decode-stdout — ms-cli `decode.rs:67-94` STDOUT-LEAK row.** The
  `emit_json`/`emit_text` paths in ms-cli decode are
  primarily STDOUT-LEAK: the values go to stdout by design (that
  is the command's purpose). Wrapping the intermediate `String`
  before flush is theoretically possible but adds machinery for
  zero practical benefit — the entropy and phrase land on stdout
  one syscall later. R1 C-2 fold (OWNED-row counting). FOLLOWUP:
  `ms-cli-decode-emit-zeroize-intermediate` (ms-cli) — optional
  future cycle.

- **OOS-md-mk — `mnemonic-key` / `descriptor-mnemonic` participation.**
  Both hold xpub-only / descriptor-only material; no private-key
  buffer in either crate to scrub. Cycle A drops the no-scope-symmetry
  matrix stubs originally planned for these repos (R3 I-R3-4 fold:
  symmetry-stubs add no engineering value when the repos have no
  secret material to audit). md / mk get no Cycle A artifact at
  all; the toolkit and ms-secret matrices cite "md, mk: out-of-scope
  per private-key-material check." If md or mk later gains a
  private-key surface (e.g., a future md-codec descriptor-binding
  with embedded xprv), participation reopens; FOLLOWUP:
  `md-mk-private-key-surface-watch` (cross-repo).

## §4 Phase structure (cross-ref to plan)

Detailed phase-by-phase TDD breakdown lives in
`/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`. High-level
shape:

- **Phase 0** — SPEC + plan + survey landed; architect-reviewed;
  0C/0I gate.
- **Phase 1** — argv-leakage close (Option 3). Scope: 9 toolkit
  flag-rows identified in survey §5 as "no stdin alternative" (no
  ms-cli argv work this phase per R1 I-2 fold — all 5 ms-cli
  flag-rows already have stdin route). Add `=-` parser extension
  to `slot_input.rs` (covers 5 toolkit flag-rows); add
  `--bip38-passphrase-stdin` (toolkit); add stdin escapes for
  `bundle --passphrase`, `verify-bundle --passphrase`,
  `derive-child --passphrase`. Add a runtime warning when clap
  parses an inline secret while a stdin alternative exists. Add a
  lint-test (`tests/lint_argv_secret_flags.rs`) that enumerates all
  clap-derived secret-bearing fields and asserts each has either a
  paired `*-stdin` flag or a `=-` escape registered.
- **Phase 2** — zeroize discipline (Option 1). Scope: add `zeroize`
  dep to `ms-codec/Cargo.toml`, `ms-cli/Cargo.toml`,
  `mnemonic-toolkit/Cargo.toml`. Wrap every survey-§1 OWNED row at
  its originating allocation: local function-scope allocations use
  `Zeroizing<...>` wrappers; `pub struct` fields holding owned
  secrets (chiefly `DerivedAccount.entropy`) use `impl Drop` on
  their enclosing struct to keep public field types unchanged
  (no breaking API change; patch-tag-compatible per §3
  OOS-pub-struct-drop). Add a new
  `derive_slot::derive_master_seed(&Mnemonic, &str) -> Zeroizing<[u8; 64]>`
  helper for the seed-derivation step (1 line × 5 sites
  deduplication) and migrate the five call sites. Document that
  `bitcoin::bip32::Xpriv` is upstream-blocked (Copy + no Drop +
  no Zeroize); the cycle does NOT attempt in-cycle Xpriv mitigation
  beyond lifetime minimization. `ms-codec` zeroize is internal-only;
  `Payload::Entr(Vec<u8>)` public-API unchanged (see §3
  OOS-public-payload).
- **Phase 3** (was Phase 4 pre-R3) — secret-memory-hygiene audit
  matrix at `<each-repo>/design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`,
  modeled on v0.8.0 BIP-vector audit matrix shape. Each survey-§1
  row + survey-§5 flag gets a CLEAR / PARTIAL / OUT-OF-SCOPE-PER-*
  status. Two matrix files only (toolkit + ms-secret); md / mk
  get no symmetry-stub per R3 I-R3-4 fold. Matrix carries a §0.5
  "what this cycle does NOT close" prose section in plain language
  per R3 I-R3-3 (cycle naming overstates closure if presented as
  categorical).
- **Phase E** — Release rollup. Coordinated tags across the two
  touched crates: `ms-codec-v0.1.3` (patch, internal-only zeroize)
  + `ms-cli-v0.1.X+1` (patch, bumps exact-pin to ms-codec-v0.1.3)
  + `mnemonic-toolkit-v0.9.2` (patch, impl-Drop approach keeps
  public API stable per §3 OOS-pub-struct-drop; semver risk
  monitored). CHANGELOG entries cite this SPEC + the plan and
  acknowledge OOS scope explicitly. FOLLOWUPS entries flip to
  `resolved <merge-sha>` in lockstep. Cycle B SPEC drafting is a
  Cycle A post-ship follow-on (not blocking Phase E).

## §5 Cross-repo coordination

Per CLAUDE.md mirror-invariant. Two touched repos
(`mnemonic-toolkit`, `mnemonic-secret`) each open a
`secret-memory-hygiene-v0_9-cycle-a` FOLLOWUPS entry at Phase 0
close. The entry body:

> **Companion:** Cycle SPEC at
> `mnemonic-toolkit/design/SPEC_secret_memory_hygiene_v0_9_0.md`;
> cycle plan at `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`;
> survey at `mnemonic-toolkit/design/agent-reports/v0_9_0-secret-memory-survey.md`.
>
> v0.9.0 secret-memory-hygiene **Cycle A** (OWNED-buffer first-pass:
> argv + zeroize). This repo's phase: [Phase 1 + Phase 2 + Phase 3
> for toolkit | Phase 2 + Phase 3 for ms-secret]. Closes when the
> hygiene-matrix successor doc lands in this repo at
> `design/agent-reports/v0_9_0-secret-memory-hygiene-matrix.md`
> and the patch tag is cut. Companion Cycle B (mlock) deferred to
> a separate cycle per R3 SPLIT-CYCLE finding.

Sibling-repo participation:

- `mnemonic-toolkit` — primary; Phases 1 + 2 + 3.
- `mnemonic-secret` — Phase 2 (ms-cli + ms-codec zeroize) + Phase 3
  (matrix file).
- `descriptor-mnemonic` — **no participation** (xpub-only material;
  no symmetry stub per R3 I-R3-4 fold).
- `mnemonic-key` — **no participation** (xpub-only material; no
  symmetry stub).

If md / mk later acquire a private-key surface, participation
reopens via the `md-mk-private-key-surface-watch` cross-repo
FOLLOWUP (see §3 OOS-md-mk).

## §6 Acceptance gates

Cycle A is shippable when all six hold:

1. All four phase reports persist at
   `<repo>/design/agent-reports/v0_9_0-phase-{0,1,2,3}-*.md` at
   0C/0I.
2. Every survey-§1 OWNED row in scope carries scrub-on-drop
   semantics at its originating allocation in production code —
   either `Zeroizing<...>` local wrap or `impl Drop` on the
   enclosing pub struct. Verified by architect-review against the
   survey table.
3. Every survey-§5 inline-secret CLI flag has either a paired
   `*-stdin` flag or a `=-` escape syntax. Verified by
   `cargo test --test lint_argv_secret_flags` (added in Phase 1).
4. `cargo test --workspace` green in `mnemonic-toolkit` and
   `mnemonic-secret`.
5. `cargo clippy --workspace --all-targets -- -D warnings` clean
   in both repos. (No new clippy surface from `zeroize` dep.)
6. Each touched repo's `v0_9_0-secret-memory-hygiene-matrix.md`
   exists and passes self-consistency check (every survey-§1 row
   tagged CLEAR / PARTIAL / OUT-OF-SCOPE-PER-* with citation back
   to the phase that closed it). Matrix carries §0.5 "what is NOT
   closed" prose section.

**Cycle B trigger (separate cycle, not a Cycle A gate):** Cycle B
SPEC drafting begins after Cycle A Phase E close. The
`secret-memory-hygiene-cycle-b` FOLLOWUP (§3 OOS-mlock-cycle-b)
tracks this.

## §7 Cross-refs

- v0.8.0 predecessor SPEC: [`SPEC_test_vector_audit_v0_8_0.md`](SPEC_test_vector_audit_v0_8_0.md).
- Survey precursor (shared with future Cycle B):
  [`agent-reports/v0_9_0-secret-memory-survey.md`](agent-reports/v0_9_0-secret-memory-survey.md).
- Plan: `/home/bcg/.claude/plans/v0_9_0-secret-memory-hygiene.md`.
- R1+R2+R3 architect-review disposition:
  [`agent-reports/v0_9_0-phase-0-spec-plan-r1.md`](agent-reports/v0_9_0-phase-0-spec-plan-r1.md).
- Existing zeroize-discipline reference (GUI):
  `/scratch/code/shibboleth/mnemonic-gui/src/form/secret_widget.rs`
  (`SecretLineEdit` with `Zeroizing<Vec<u8>>` buffer) +
  `mnemonic-gui/src/secrets.rs::zeroize_form_state`. The CLI
  hardening in this cycle mirrors the GUI's Phase-B.1 discipline
  at the clap-parsed-arg boundary. The GUI's allocator-residue
  caveat (secret_widget.rs:9-19 doc-comment) is mirrored in this
  cycle's §3 OOS-allocator-residue entry per R3 I-R3-1.
- `secret-on-stdout` warning precedent: `bundle.rs:697`,
  `convert.rs:799`, `derive_child.rs:205`. Phase 1's
  `secret-in-argv` warning matches its shape byte-for-byte.
- Cycle B (mlock infrastructure) — SPEC to be written post-Cycle-A
  ship; will cite the same survey precursor as authoritative.
