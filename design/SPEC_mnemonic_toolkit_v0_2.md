# SPEC — `mnemonic-toolkit` v0.2

**Status:** Round 1 — pending architect review
**Date:** 2026-05-05
**Predecessor:** `SPEC_mnemonic_toolkit_v0_1.md` (936 lines; canonical for v0.1 invocations)
**Brainstorm:** `BRAINSTORM_mnemonic_toolkit_v0_2.md` (architect-converged at r2; 0C/0I)
**Pre-SPEC spike memo:** `agent-reports/spike-toolkit-v0_2-pre-spec.md` (GATE GREEN; 3 multisig + privacy + divergent-path round-trips verified)

This SPEC extends v0.1 with the 5 v0.2 features. v0.1 SPEC remains canonical for backwards-compatible single-sig invocations under a v0.2 binary (per §9.4 closure). Sections below specify **deltas** from v0.1 unless explicitly re-stated; v0.1 §-text not addressed here is unchanged.

---

## §1. Scope (delta)

v0.2 adds 5 features to v0.1's surface:

1. **Multisig templates** — pure BIP-388 multisig wrappers across segwit + taproot families.
2. **Non-zero account index** via `--account` flag.
3. **Multi-cosigner watch-only mode** via `--cosigner` (repeatable) or `--cosigners-file`.
4. **Privacy-preserving mk1 emission** via `--privacy-preserving` flag (whole-bundle).
5. **Bundle self-check** via `--self-check` flag (bundle subcommand only).

**Still in scope** (single-sig from v0.1, unchanged): `--phrase` + `--template bip{44,49,84,86}` full mode; `--xpub --master-fingerprint` watch-only mode; `--language`; `--passphrase`; `--no-engraving-card`; `--json`.

**Explicitly deferred to v0.3+** (per Q11 brainstorm closure):
- K-of-N share encoding (multi-string ms1 + multi-card-set mk1/md1; gates on ms-codec v0.2).
- User-supplied descriptor passthrough (arbitrary descriptors, including hash-locked / hybrid miniscripts).
- `--output <dir>` (write per-card files instead of stdout sections).
- Recovery flow (3 strings → wallet artifact).
- Color / interactive prompts (forbidden indefinitely per v0.1 §3.3).

## §1.1 (no changes — engraving as load-bearing user persona)

## §2. Command surface

### §2.1 `mnemonic bundle` — multisig + privacy + self-check additions

New v0.2 flags (added to `BundleArgs`):

| Flag | Type | Default | Mode applicability |
|---|---|---|---|
| `--account <N>` | `u32` | `0` | both full + watch-only |
| `--cosigner <xpub>:<fp>:<path>` | repeatable | none | watch-only multisig |
| `--cosigners-file <path>` | path | none | watch-only multisig (bulk) |
| `--multisig-path-family <bip48\|bip87>` | enum | `bip87` | multisig (full + watch-only) |
| `--privacy-preserving` | bool | `false` | both |
| `--self-check` | bool | `false` | bundle only (verify-bundle is the explicit verify command) |

**Locked invariants:**
- v0.1's `--xpub <X> --master-fingerprint <FP>` continues to work for single-sig watch-only. Single-sig invocations under a v0.2 binary produce the same encoded card strings as under v0.1 (per §9.4 closure).
- `--cosigner` and `--cosigners-file` are **mutually exclusive** (runtime pre-check; exit 2 + byte-exact §6.6 text).
- `--xpub` is **mutually exclusive** with `--cosigner` and `--cosigners-file` (runtime pre-check; exit 2 + byte-exact §6.6 text). Use `--xpub` for single-sig watch-only; `--cosigner`/`--cosigners-file` for multisig watch-only.
- `--account` defaults to `0` everywhere; non-default values produce `PathDeclPaths::Divergent` md1 declarations (per §4.6 delta).
- `--multisig-path-family` is informational for full mode (toolkit derives all cosigners from a single seed; the family selects the path template); for watch-only mode it's the default for cosigners whose specs omit `<path>`. Per-cosigner explicit paths in `--cosigner=<xpub>:<fp>:<path>` always override the family default.

### §2.1.1 Full mode (delta)

Single-sig full mode: unchanged from v0.1.

**Multisig full mode** (NEW):
- One `--phrase` (single seed); toolkit derives N cosigner xpubs from the same seed at distinct paths.
- `--template <wsh-multi|wsh-sortedmulti|sh-wsh-multi|sh-wsh-sortedmulti|tr-multi-a|tr-sortedmulti-a>` selects the multisig wrapper.
- `--threshold <K>` (REQUIRED for multisig templates; 1 ≤ K ≤ N; N ≤ 16).
- `--cosigner-count <N>` (REQUIRED for full multisig; toolkit derives N xpubs at the same `--account` from the single seed).
- `--multisig-path-family` selects between BIP-48 and BIP-87 derivation paths.

**Note on full multisig semantics:** the N cosigners derived from one seed produce a "self-multisig" — useful for testing, demo, or single-user-multi-device patterns where the user holds all N keys themselves. Production multisig wallets use distinct seeds per cosigner (watch-only mode below).

**Self-multisig stderr warning** (resolves I2 from r1 review): full-mode multisig with `--cosigner-count > 1` derives all N cosigner xpubs from one seed at one path; **all N xpubs are byte-identical**, all N master fingerprints are identical, and all N mk1 cards are interchangeable. This is a degenerate self-multisig backup, not a true multi-device multisig. Toolkit emits a non-suppressible stderr warning (byte-exact, pinned in §5.2):

```
warning: full-mode multisig (--cosigner-count > 1) derives all N cosigner xpubs from one
warning: seed at one path; all N cosigner cards are byte-identical interchangeable copies.
warning: For production multi-device multisig, use --cosigner watch-only mode with distinct
warning: cosigner xpubs from distinct seeds.
```

The warning fires once per `bundle` invocation in full multisig mode regardless of `--no-engraving-card` (the engraving card and this warning are orthogonal — the warning is a safety advisory; the engraving card is a record). **Emission ordering** (resolves N-1 from r2 review): the stderr safety warning is emitted BEFORE the bundle stdout block, so pipeline consumers reading stdout-while-monitoring-stderr see the warning before processing the bundle.

### §2.1.2 Watch-only mode (delta)

Single-sig watch-only: unchanged from v0.1.

**Multisig watch-only** (NEW):
- One or more `--cosigner <xpub>:<fp>:<path>` flags (repeatable; minimum N for K-of-N) OR a single `--cosigners-file <path>` JSON file.
- `--threshold <K>` (REQUIRED).
- Cosigner ordering follows flag order (or JSON array order). Sorting (per `sortedmulti`/`sortedmulti_a`) is performed at descriptor-construction time on the canonical xpub bytes; **flag order does NOT determine on-wire ordering** for sorted variants.
- For `--cosigner` parsing: `<xpub>` is the base58check-encoded extended public key; `<fp>` is 8 hex chars (case-insensitive) — **REQUIRED, may not be empty** (resolves L2 from r1 review); `<path>` is optional (BIP-32-format, e.g. `m/87'/0'/0'`); colon delimiter (xpubs / fingerprints / paths contain no `:`). Empty `<fp>` (`--cosigner=<xpub>::<path>`) emits exit 1 `CosignerSpec` with a diagnostic. The `--privacy-preserving` flag suppresses fingerprints from mk1 *output*, but the user must still supply them for the cross-binding cross-check at md1's `tlv.fingerprints`. If the cosigner's master fingerprint is genuinely unknown to the user, the bundle cannot be assembled — this is by design (md1's fingerprint binding is part of the wallet-policy identity).
- For `--cosigners-file`: JSON array of `{ "xpub": "...", "master_fingerprint": "...", "path": "..." }`. Path is optional per-entry.

#### §2.1.2.1 `--cosigners-file` JSON schema (NEW)

```json
[
  { "xpub": "xpub6...", "master_fingerprint": "deadbeef", "path": "m/87'/0'/0'" },
  { "xpub": "xpub6...", "master_fingerprint": "cafef00d" },
  { "xpub": "xpub6...", "master_fingerprint": "12345678", "path": "m/48'/0'/3'/2'" }
]
```

- `master_fingerprint` is **REQUIRED** for every entry (resolves L-A from r2 review); missing or empty value emits exit 1 `CosignersFile { message: "cosigner index N: master_fingerprint required" }`. Same rationale as the `--cosigner` flag (per §2.1.2): privacy mode suppresses the value from mk1 output but the user must still supply it for cross-binding into md1's `tlv.fingerprints`.
- `xpub` is **REQUIRED** for every entry; missing/empty rejected the same way.
- `path` is optional (omit → use `--multisig-path-family` default).
- Order in array determines `@N` placeholder index in the descriptor (before sorting for sortedmulti).
- Strict JSON; trailing commas + comments rejected.

### §2.1.3 `--template` enum (delta)

v0.1 enum (unchanged for single-sig): `bip44 | bip49 | bip84 | bip86`.

v0.2 adds 6 multisig variants:

| Template | Wrapper | md-codec Tag |
|---|---|---|
| `wsh-multi` | `wsh(multi(K, @0, ..., @N-1))` | `Tag::Wsh` + `Body::Children([Tag::Multi])` |
| `wsh-sortedmulti` | `wsh(sortedmulti(K, @0, ..., @N-1))` | `Tag::Wsh` + `Body::Children([Tag::SortedMulti])` |
| `sh-wsh-multi` | `sh(wsh(multi(K, @0, ..., @N-1)))` | `Tag::Sh` + `Body::Children([Tag::Wsh, ...])` |
| `sh-wsh-sortedmulti` | `sh(wsh(sortedmulti(K, @0, ..., @N-1)))` | as above with sortedmulti |
| `tr-multi-a` | `tr(multi_a(K, @0, ..., @N-1))` | `Tag::Tr` + `Body::Tr{key_index: 0, tree: Some(multi_a_node)}` (note: requires md-codec wrapper composition; verified by Phase 1.5 spike) |
| `tr-sortedmulti-a` | `tr(sortedmulti_a(K, @0, ..., @N-1))` | as above with `Tag::SortedMultiA` |

All 6 multisig variants set `Body::Variable { k: K, children: <N PkK leaves> }` for the inner `multi`/`sortedmulti`/`multi_a`/`sortedmulti_a` node, where each child is `Node { tag: Tag::PkK, body: Body::KeyArg { index: i } }` (verified by spike).

### §2.1.4 `--network` enum (no changes)

### §2.1.5 `--master-fingerprint` format (no changes for single-sig)

For multisig watch-only mode, `--master-fingerprint` is NOT used; per-cosigner fingerprints come from `--cosigner=<xpub>:<fp>:<path>` or `--cosigners-file`.

### §2.1.6 `--passphrase` (no changes; full-mode only; mutually exclusive with `--xpub` AND with multisig watch-only flags)

### §2.1.7 `--account <N>` (NEW)

- `u32`. Default `0` (preserves v0.1 wire-bit-identical for single-sig).
- For full mode: replaces the hardcoded `0'` in the BIP-32 derivation path's account component.
- For watch-only mode: encoded into md1's `PathDecl` for cross-checking; the toolkit does NOT re-derive xpubs (caller-supplied), but the `--account` value MUST be consistent with the cosigner-supplied paths or the toolkit emits exit 2 mode-violation.
- Multi-cosigner consistency check: when `--cosigner` paths carry differing accounts, toolkit emits md1 with `PathDeclPaths::Divergent(...)` containing each cosigner's path verbatim. `--account` becomes informational but is still consistency-checked (it must equal the account of any cosigner whose `<path>` is omitted from the cosigner spec — i.e., the cosigners using the family default).

### §2.1.8 `--privacy-preserving` (NEW)

- Whole-bundle boolean. When set, all emitted mk1 cards have `KeyCard.origin_fingerprint = None` per `mk-codec v0.2.1` (verified by spike).
- ms1 + md1 emission unchanged (privacy mode applies only to mk1).
- Stderr engraving card: `master fingerprint:` line replaced with `master fingerprint: (suppressed by --privacy-preserving)`.
- Per-cosigner mk1 cards in multisig privacy mode: ALL cosigner cards omit `origin_fingerprint`; the cosigner-specific fingerprints from `--cosigner=<xpub>:<fp>:<path>` are NOT emitted into mk1.
- mk1 decode under privacy mode: `decoded.origin_fingerprint == None` (verified by spike).
- Cross-binding (verify-bundle): privacy mode RELAXES the `mk1_fingerprint_match` check — emits `result: skipped` with detail `privacy-preserving mode; fingerprint suppressed` instead of comparing.

### §2.1.9 `--self-check` (NEW)

- Bundle-subcommand-only flag. Verify-bundle is the explicit verify command; `--self-check` is for emitter sanity.
- After `synthesize_*` returns, toolkit invokes verify-bundle's 9-check logic on the emitted bundle internally. Any check returning `result: fail` triggers `Err(ToolkitError::BundleMismatch{ card, message })` from `bundle::run`, exiting 4.
- The `card` field of `BundleMismatch` (now `String` per Phase 0 fixup) is set to `format!("self-check[{}]", check_name)` so users can distinguish self-check failures from external verify-bundle failures by inspecting the message.
- Implementation: factor verify-bundle's check logic into a reusable lib helper (function name + signature decided at IMPLEMENTATION_PLAN drafting; not SPEC-pinned).
- Privacy-mode interaction: self-check honors the same `mk1_fingerprint_match: skipped` relaxation as external verify-bundle.

### §2.2 `mnemonic verify-bundle` (delta)

#### §2.2.1 Full-mode verify-bundle (delta)

Single-sig: unchanged from v0.1 (5 substantive checks; 9-element `checks` array per SPEC v0.1 §5.4).

**Multisig full-mode verify** (NEW):
- Inputs: `--phrase` + `--multisig-path-family` + `--cosigner-count <N>` + `--threshold <K>` + `--account <A>` + `--ms1` + `--mk1 ...` (repeatable, all chunks of all cosigners) + `--md1 ...`.
- Toolkit derives N cosigner xpubs from the seed at the family-selected paths with the given account.

**`--mk1` grouping semantics** (resolves I1 from r1 review; r2 amendments for I-A + I-B):

The user passes ALL mk1 chunks across ALL cosigners as repeated `--mk1 <chunk>` flags, in any order (transcribers don't have to remember per-cosigner ordering). Internally:

1. Toolkit extracts each chunk's chunked-header (per mk1 BIP §"String-layer header") via `mk_codec::decode_string` (BCH-correction step) followed by `mk_codec::StringLayerHeader::from_5bit_symbols` to read the variant. Chunks may carry one of two header variants:
   - `Chunked { version, chunk_set_id, total_chunks, chunk_index }` — multi-chunk card-set; `chunk_set_id` groups chunks of the same set.
   - `SingleString { version }` — single-string card; no `chunk_set_id`. Treat each `SingleString` chunk as its own group with synthetic group-key = the string itself (each is a complete cosigner card-set on its own).
2. Toolkit groups chunks accordingly — `Chunked` chunks by their `chunk_set_id`; `SingleString` chunks each in their own group.
3. Each group is passed to `mk_codec::decode(&[&str])` independently; decode rejects mixed-chunk_set_id within a group (mk-codec invariant).
4. Each decoded `KeyCard` has a `policy_id_stubs: Vec<[u8;4]>` listing all N cosigners' stubs (per §4.5 delta).
5. Group-count check: if grouping yields fewer than N or more than N groups, emit exit 4 `BundleMismatch { card: "mk1", message: "expected N cosigner card-sets; got M" }`.
5b. Stub-list consistency check (resolves I-A from r2 review): if the N decoded cards' `policy_id_stubs` lists are not byte-identical across all N decoded cards, emit exit 4 `BundleMismatch { card: "mk1", message: "policy_id_stubs lists differ across cards; mixed bundle" }`. This catches the case where a transcriber accidentally pulled chunks from two distinct bundles whose card counts coincidentally match.
6. Cosigner association is determined by matching each card's xpub against `tlv.pubkeys` entries in md1 (after cards pass step 5b, all share the same stub list — only xpub disambiguates which cosigner is which).

This grouping is performed BEFORE invoking mk-codec at the multi-card level; mk-codec itself sees only per-cosigner slices. The user-facing CLI is one flat `--mk1 <chunk>` repetition; per-cosigner grouping is an internal toolkit responsibility.

- **9-element `checks` array per cosigner** (multiplied: `3 + 6N` total checks for N cosigners; matches v0.1's 9 for N=1). Schema in §5.4 delta below.

#### §2.2.2 Watch-only-mode verify-bundle (delta)

Single-sig: unchanged from v0.1 (4 substantive checks; mandatory stderr warning at `run_watch_only` start).

**Multisig watch-only verify** (NEW):
- Inputs: `--cosigner` (repeatable) or `--cosigners-file` + `--threshold` + `--multisig-path-family` + `--account` + `--mk1 ...` + `--md1 ...`.
- `--ms1` not applicable in watch-only.
- Stderr warning emitted at start (mirror of v0.1's §2.2.2 watch-only warning, but adapted for multisig context):
  ```
  warning: watch-only multisig verify-bundle does not verify --cosigner xpubs are at the
  warning: claimed BIP path (no per-cosigner master seed available for re-derivation).
  warning: Use --phrase mode for end-to-end verification of self-multisig backups.
  ```
- 4 substantive checks per cosigner; 9-element schema slots per cosigner (some `skipped`).

### §2.3 `--help` text (delta)

v0.1 top-level help unchanged. New flag listings in `bundle --help` and `verify-bundle --help` per §2.1 / §2.2 deltas.

---

## §3. Input/output discipline (no changes from v0.1)

§3.1, §3.2, §3.3 unchanged. `--cosigners-file` reads a file path, not stdin (no new stdin-reading flag).

---

## §4. Bundle synthesis rules

### §4.1 Derivation in full mode (delta)

Single-sig: unchanged.

**Multisig full mode** (NEW):
- Compute master xpriv from seed + passphrase as in v0.1 §4.1.
- For each cosigner index `i ∈ [0, N)`:
  - Compute path = `--multisig-path-family` template applied with `account = --account` (BIP-87: `m/87'/<coin>'/<account>'`; BIP-48: `m/48'/<coin>'/<account>'/<script_type>'`).
  - Derive xpub at that path. Note: BIP-48/BIP-87 paths give ONE xpub per `(family, coin, account, script_type)` — N "self-multisig" cosigners derived from one seed will share the same xpub at the same path. **This is intentional for v0.2 self-multisig demo/test mode**; production multisig requires distinct seeds (watch-only mode).
- Per-cosigner xpub bytes assembled into `tlv.pubkeys` at indices 0..N-1.
- Per-cosigner master fingerprint computed once (single seed, single fingerprint), assembled into `tlv.fingerprints` at indices 0..N-1. All N entries equal in self-multisig.

### §4.2 Origin paths per (template, network, account) (delta)

v0.1 single-sig: unchanged (account hardcoded `0`; v0.2 with `--account 0` produces identical output).

v0.2 single-sig with `--account N` where N > 0:
- Origin path: `m/<purpose>'/<coin>'/<N>'` (replaces v0.1's `m/<purpose>'/<coin>'/0'`).
- md1 `PathDecl.paths`: stays `PathDeclPaths::Shared(origin_path)` (single cosigner).

v0.2 multisig:
- Per-cosigner origin path computed per §4.1 deltas above.
- md1 `PathDecl.paths`:
  - If all cosigners share an origin path (full mode self-multisig OR watch-only with all cosigners using identical paths): `PathDeclPaths::Shared(origin_path)`.
  - If cosigners diverge (watch-only with different `<path>` per cosigner spec, or different accounts): `PathDeclPaths::Divergent(vec![path_0, path_1, ..., path_N-1])` (verified by spike).

### §4.3 Network ↔ xpub-version cross-check (delta)

For multisig watch-only mode: each cosigner's xpub network must match `--network`; mismatch on ANY cosigner emits exit 2 `NetworkMismatch{ xpub_network, expected }` with the **first** mismatching cosigner's index in the message (e.g., `"cosigner @1 xpub network mainnet does not match --network testnet"`).

### §4.4 ms1 card synthesis (no changes; full mode only; omitted in multisig watch-only)

ms1 in self-multisig full mode is single-string, encoding the seed entropy once (the seed is shared across N cosigners by definition of self-multisig).

### §4.5 mk1 card synthesis (delta)

Single-sig: unchanged.

**Multisig** (NEW):
- One `KeyCard` per cosigner. For N cosigners, N `KeyCard` instances → N independent encode calls → N card-sets (each card-set may chunk into multiple strings per mk-codec's chunking rules).
- Each `KeyCard.policy_id_stubs` is the FULL list of N stubs (one per cosigner) — the same `Vec<[u8;4]>` across all N cosigner cards. This binds the cards as a coordinated set: any one card's `policy_id_stubs` lists all N members.
- Each `KeyCard.origin_fingerprint`: the cosigner's master fingerprint (full mode: derived from the single seed = same value for all N; watch-only: per-cosigner from `--cosigner` spec). `--privacy-preserving` overrides all to `None`.
- Each `KeyCard.origin_path`: the cosigner's BIP-32 derivation path. Per §4.2 may diverge.
- Each `KeyCard.xpub`: the cosigner's xpub.

**Path/xpub depth consistency** (resolves L3 from r1 review; mirrors Phase 1.5 spike Errata 2): for each cosigner's `KeyCard`, `origin_path` MUST have depth and final-component value matching the xpub's intrinsic `depth` and `child_number`. mk-codec strips these BIP-32 metadata fields on the wire and reconstructs them from `origin_path` on decode; mismatch silently rewrites the xpub's depth/child_number on round-trip. In full mode, toolkit-derived xpubs satisfy this automatically (path is used to derive). In watch-only mode, if a cosigner-supplied path's depth doesn't match the xpub's intrinsic depth, emit exit 1 `CosignerSpec { cosigner_idx, message: "path depth N does not match xpub depth M; xpub at depth M expects path of depth M" }`.

#### §4.5.1 `origin_fingerprint = Some(_)` invariant (RELAXED for v0.2)

v0.1 §4.5.1 required `origin_fingerprint: Some(_)`. v0.2 RELAXES this: under `--privacy-preserving`, `origin_fingerprint = None` is permitted. This is a non-breaking shape change because mk-codec's wire format already supports both (header bit 2 governs presence).

### §4.6 md1 card synthesis (typed-struct construction; delta)

Single-sig: unchanged.

**Multisig** (NEW):
- `Descriptor.n` = N (cosigner count).
- `Descriptor.path_decl.n` = N.
- `Descriptor.path_decl.paths` = `Shared(p)` if all cosigners share a path, else `Divergent(vec![p_0, ..., p_N-1])` per §4.2.
- `Descriptor.tree`:
  - For `wsh-{sorted,}multi`: `Node { tag: Tag::Wsh, body: Body::Children(vec![multisig_inner]) }` where `multisig_inner = Node { tag: Tag::{SortedMulti|Multi}, body: Body::Variable { k: K, children: <N PkK leaves> } }`. Each PkK leaf: `Node { tag: Tag::PkK, body: Body::KeyArg { index: i } }`.
  - For `sh-wsh-{sorted,}multi`: outer `Node { tag: Tag::Sh, body: Body::Children(vec![wsh_node]) }`; inner wsh structure as above.
  - For `tr-{sorted,}multi-a`: `Node { tag: Tag::Tr, body: Body::Tr { key_index: 0, tree: Some(taproot_multi_a_subtree) } }` where the taproot subtree contains the `Tag::{SortedMultiA|MultiA}` node. **Phase 1.5 spike validated only the wsh-sortedmulti case for typed-struct construction; the taproot multisig path requires a Phase A spike-extension to confirm exact tree composition before implementation. SPEC contract: the tree shape MUST round-trip cleanly through `chunk::split` / `chunk::reassemble` for `is_wallet_policy()` to return true; if Phase A finds md-codec needs additional helper APIs, file as cross-repo FOLLOWUPS.**
- `Descriptor.tlv`:
  - `fingerprints`: `Some(vec![(0, fp_0), (1, fp_1), ..., (N-1, fp_N-1)])` per cosigner. (N elements; each unique — even in self-multisig where the master fp is identical, the index distinguishes them.)
  - `pubkeys`: `Some(vec![(0, xpub_0), ..., (N-1, xpub_N-1)])` per cosigner. Each `xpub_i` is the 65-byte chain_code‖compressed_pubkey transform per v0.1 §4.6.1.

#### §4.6.1 xpub byte-format transform (no changes from v0.1)

#### §4.6.2 `PathDeclPaths::Divergent` enabled in v0.2 (delta from v0.1)

v0.1 §4.6.2 explicitly forbade emitting `Divergent`. v0.2 LIFTS this restriction for multisig with per-cosigner divergent paths. Single-sig under v0.2 still emits `Shared` (path is shared with itself, trivially).

#### §4.6.3 Per-template wrapper tags + bodies (delta)

v0.1 single-sig (4 templates): unchanged.

v0.2 multisig (6 templates): see §4.6 delta and §2.1.3 table.

#### §4.6.4 md1 encoding (no changes; uses chunk::split as in v0.1)

### §4.7 Cross-binding invariants (delta)

v0.1 invariants (debug-asserted):
1. `compute_wallet_policy_id(&descriptor).as_bytes()[0..4] == keycard.policy_id_stubs[0]`
2. `descriptor.is_wallet_policy()`

v0.2 multisig adapts invariant 1: ALL N cosigner cards share the same `policy_id_stubs` list (each card lists all N), and `policy_id_stubs[0] == compute_wallet_policy_id(&descriptor).as_bytes()[0..4]`. The assertion loops over all N cards but checks a single derived stub. (Implementation: replace v0.1's tautological per-stub check with a per-card check that the LIST equals the descriptor-derived list.)

v0.2 self-check (NEW invariant 3, per Q7 brainstorm closure): post-synthesis, the bundle decodes/verifies cleanly against the original inputs. Tautological at construction time but catches bugs in the encode→decode round-trip path.

### §4.8 Xpub depth advisory (delta)

v0.1: xpub depth advisory if not 3 (single-sig).

v0.2 multisig: depth advisory per cosigner. BIP-48 paths have depth 4 (`m/48'/coin'/account'/script_type'`); BIP-87 paths have depth 3. Toolkit checks each cosigner xpub's depth against the expected depth-for-family; mismatch emits a per-cosigner stderr warning before bundle output.

---

## §5. Output format

### §5.1 Default text-mode stdout layout (delta)

Single-sig: unchanged.

**Multisig**: stdout adds per-cosigner `# mk1[<i>]` headers when N > 1:

```
# ms1 (entropy, BCH-checksummed)
ms1...

# mk1[0] (cosigner 0 xpub + origin)
mk1...

# mk1[1] (cosigner 1 xpub + origin)
mk1...

[...repeat for N cosigners...]

# md1 (multisig wallet policy)
md1...
```

Single-sig (N=1) keeps v0.1's `# mk1` header (no index suffix) for wire-bit-identical output.

### §5.2 Engraving stderr card (delta — byte-exact updates for multisig)

v0.1 single-sig (4 templates × full|watch-only|with-passphrase modes): unchanged. Byte-exact text remains.

**Multisig additions:**

For full multisig:
```
network: <network>
template: <multisig-template>
account: <N>
threshold: <K> of <N>
cosigner_count: <N>
multisig_path_family: <bip48|bip87>
origin path (cosigner 0): <path>
origin path (cosigner 1): <path>
... (per cosigner; collapses to single 'origin paths: shared' line if all paths identical)
master fingerprint: <fp> | (suppressed by --privacy-preserving)
language: <lang> (BIP-39 checksum valid)
passphrase: not used | USED — not engraved on any card; record separately and never lose it.
SELF-MULTISIG WARNING: all N cosigner xpubs are derived from one seed at one path and
  are byte-identical interchangeable copies. For production multi-device multisig, use
  --cosigner watch-only mode with distinct cosigner xpubs from distinct seeds.
  (this line emitted ONLY for full multisig with --cosigner-count > 1)
HARDWARE WALLET CAVEAT: taproot multisig (multi_a / sortedmulti_a) signing-side support
  is nascent as of v0.2; verify your signing device supports it before engraving.
  (this line emitted ONLY for tr-multi-a / tr-sortedmulti-a templates)
engrave each card on its own plate. record this card alongside.
```

The SELF-MULTISIG WARNING is also emitted as a non-suppressible stderr warning at synthesis time (per §4.1 delta), independent of the engraving card; it appears even with `--no-engraving-card`.

For watch-only multisig:
```
network: <network>
template: <multisig-template>
account: <N>
threshold: <K> of <N>
cosigner_count: <N>
multisig_path_family: <bip48|bip87> (default if cosigners didn't specify path)
origin paths:
  cosigner 0: <path> (fp <fp>) | (fp suppressed)
  cosigner 1: <path> (fp <fp>) | (fp suppressed)
  ...
mode: watch-only multisig (xpub-supplied per cosigner; no entropy known to toolkit)
ms1 card omitted; recover entropy from each cosigner's individual seed backup.
HARDWARE WALLET CAVEAT: ... (as above for taproot)
engrave each card on its own plate. record this card alongside.
```

The hardware-wallet caveat line is byte-exact-pinned for taproot multisig templates (per Q1 + L1 brainstorm closures).

### §5.3 `bundle --json` schema (delta — `schema_version: "2"`)

v0.1's flat `BundleJson`:

```json
{
  "schema_version": "1",
  "mode": "...",
  "network": "...",
  "template": "...",
  "account": 0,
  "origin_path": "...",
  "master_fingerprint": "...",
  "ms1": "...",
  "mk1": ["..."],
  "md1": ["..."],
  "engraving_card": "..."
}
```

v0.2 schema:

```json
{
  "schema_version": "2",
  "mode": "full" | "watch-only",
  "network": "...",
  "template": "...",
  "account": <u32>,
  "origin_path": "...",         // single-sig OR shared-path multisig
  "origin_paths": ["...", ...], // multisig with divergent paths (omits "origin_path")
  "master_fingerprint": "...",  // single-sig only; NULL for multisig OR --privacy-preserving
  "ms1": "..." | null,
  "mk1": [...],                 // shape varies; see below
  "md1": ["..."],
  "engraving_card": "..." | null,
  "multisig": {
    "template": "wsh-sortedmulti" | ...,
    "threshold": <K>,
    "cosigner_count": <N>,
    "path_family": "bip48" | "bip87",
    "cosigners": [
      { "index": 0, "master_fingerprint": "...", "origin_path": "...", "xpub": "xpub6..." },
      ...
    ]
  } | null,
  "privacy_preserving": <bool>
}
```

**`mk1` field shape** (per Q9 brainstorm closure):
- Single-sig (`multisig: null`): flat `Vec<String>` — `["mk1q...", ...]`. Same shape as v0.1.
- Multisig (`multisig != null`): nested `Vec<Vec<String>>` — `[["mk1q...", ...], ["mk1q...", ...], ...]`. Outer = per-cosigner; inner = chunks per cosigner.

**Discriminated-union shape note** (resolves I3 from r1 review): the `mk1` field is a discriminated union keyed on `multisig`. Consumers MUST inspect `multisig` BEFORE deserializing `mk1`. JSON-Schema-style description:

```
mk1: oneOf
  - if multisig == null then mk1: array<string>          // flat, single-sig
  - if multisig != null then mk1: array<array<string>>   // nested, per-cosigner
```

For Rust consumers using `serde`, the implementation typed `BundleJson.mk1` as either `serde_json::Value` (with caller-side branching) or a custom enum with `#[serde(untagged)]`; both are acceptable. Strict consumers expecting v0.1's flat shape MUST guard on `schema_version == "1"` (no v0.2-emitted JSON has this) OR `multisig == null`. Single-sig invocations under v0.2 produce `multisig: null` and `mk1` is flat — consumers handling only single-sig need not change deserialization logic, only schema-version awareness.

**Single-sig invocations under v0.2**:
- `multisig: null`, `privacy_preserving: false`, `cosigners` absent.
- `mk1` is flat (matches v0.1 shape).
- The encoded card strings inside `ms1`/`mk1`/`md1` are byte-identical to v0.1 (per §9.4 closure).
- `schema_version: "2"` always.

**v0.1 fixture corpus retirement**: SHA pin `81828299c9277...` retires at v0.2 release; v0.2 ships a new corpus SHA pin reflecting the new envelope shape. CHANGELOG documents the retirement.

### §5.4 `verify-bundle --json` schema (delta — multisig per-cosigner checks)

v0.1's 9-element `checks` array (single-sig): unchanged for `multisig: null` invocations.

**Multisig**: `checks` array becomes `len(cosigners) × 9` slots, each named with the cosigner index:
- `ms1_entropy_match` (N=1; only one ms1 card per bundle)
- `mk1_decode[0]`, `mk1_decode[1]`, ..., `mk1_decode[N-1]`
- `mk1_xpub_match[0]`, ..., `mk1_xpub_match[N-1]`
- `mk1_fingerprint_match[0]`, ... (skipped if `--privacy-preserving`)
- `mk1_path_match[0]`, ...
- `md1_decode` (N=1; one md1 per bundle)
- `md1_wallet_policy`
- `md1_xpub_match[0]`, ..., `md1_xpub_match[N-1]`
- `stub_linkage[0]`, ..., `stub_linkage[N-1]`

Total: 1 + (4 × N) + 2 + (2 × N) = 3 + 6N checks for multisig. For N=1: 9 (matches v0.1). For N=3: 21.

**Self-check failures** (`bundle --self-check` failure path): the `BundleMismatch.card` field is `format!("self-check[{}]", failed_check_name)` (e.g., `"self-check[mk1_xpub_match[1]]"`).

### §5.5 Error JSON envelope (delta — `kind` enum gains `MultisigViolation`)

v0.1 §5.5 `kind` enum unchanged. v0.2 may add `MultisigViolation` if needed for threshold / cosigner-count errors not subsumed by `BadInput` / `ModeViolation`. Final decision deferred to implementation; SPEC §6.2 below lists the new variants.

---

## §6. Errors and exit codes

### §6.1 Exit-code table (no changes; exit 5 NOT added per Q7 brainstorm closure)

### §6.2 `ToolkitError` enum (delta)

New variants (all `#[non_exhaustive]` already on the enum):

```rust
ToolkitError::MultisigConfig {
    message: String,  // e.g., "threshold 5 exceeds cosigner count 3"
},
ToolkitError::CosignerSpec {
    cosigner_idx: usize,
    message: String,  // parsing errors in --cosigner=<xpub>:<fp>:<path>
},
ToolkitError::CosignersFile {
    message: String,  // JSON parse errors, schema violations
},
```

`exit_code()` mapping: all three new variants → exit **1** (`BadInput`-equivalent; user-input errors).

### §6.3 Exit-code mapping (delta — new variants → exit 1)

### §6.4 Friendly mappers + dispatch tables (delta)

§6.4.0 routing principle: unchanged.

§6.4.1–§6.4.5 per-source mappers: unchanged.

NEW: `friendly_multisig` for the 3 new variants → returns the variant's `message` field directly (no further mapping).

### §6.5 Display rules (no changes)

### §6.6 Mode-violation messages (byte-exact; delta)

v0.1 rows: unchanged.

v0.2 NEW rows (each runtime pre-check, exit 2, byte-exact):

| Trigger | Routing | Message |
|---|---|---|
| `--xpub` with `--cosigner` or `--cosigners-file` | exit 2 | `--xpub cannot be combined with --cosigner or --cosigners-file; pick single-sig (--xpub) or multisig (--cosigner/--cosigners-file) but not both.` |
| `--cosigner` with `--cosigners-file` | exit 2 | `--cosigner cannot be combined with --cosigners-file; supply cosigners via flag-repetition or file, not both.` |
| `--threshold` without multisig template | exit 2 | `--threshold is meaningful only with a multisig --template; single-sig templates ignore threshold.` |
| `--cosigner-count` without multisig template | exit 2 | `--cosigner-count is meaningful only with a multisig --template.` |
| `--multisig-path-family` without multisig template | exit 2 | `--multisig-path-family is meaningful only with a multisig --template.` |
| `--privacy-preserving` with `--xpub` (single-sig watch-only) | exit 2 | `--privacy-preserving with --xpub (single-sig watch-only) has no useful effect: --xpub mode requires --master-fingerprint and the bundle's md1 binds that fingerprint into tlv.fingerprints; suppressing it from mk1 only would produce an inconsistent bundle. Drop --privacy-preserving or switch to multisig watch-only mode.` |
| `--account <N>` with `N > 0` and template that requires path-family-defined account-component-position not satisfied | exit 2 | `--account is incompatible with the selected --template (template lacks an account-position in its standard path).` |
| `--self-check` with `verify-bundle` subcommand | exit 64 (clap; `--self-check` not declared on `verify-bundle`) | clap default text |

All v0.2 mode-violation rows enforced as runtime pre-checks (exit 2) per Q12 brainstorm closure, NOT as clap `conflicts_with` (which would exit 64 with non-byte-exact text).

---

## §7. Engraving guidance

### §7.1 The three-card backup workflow (delta)

v0.1 single-sig: 3 plates (ms1 + mk1 + md1). Unchanged.

**Multisig (NEW):** the plate count grows to **2 + N** (1 ms1 if full mode + N mk1 cards + 1 md1). Each cosigner's mk1 card is a separate plate; in production multisig (watch-only mode) each cosigner is responsible for their own mk1 plate. The coordinator assembling the bundle holds md1 (and ms1 if full mode); cosigners hold their respective mk1.

**Coordinator vs cosigner persona**:
- Coordinator: assembles the bundle from cosigner-supplied xpubs + fingerprints. Holds the descriptor. Their own xpub may or may not be in the cosigner set.
- Cosigner: holds an individual seed. Provides xpub + fingerprint to the coordinator. Receives their own mk1 card to engrave.

### §7.2 Watch-only restoration (delta)

Multisig watch-only restoration: each cosigner's mk1 contributes one xpub + path; reassembling the bundle requires all N cosigner mk1 cards plus the md1 wallet-policy card.

### §7.3 Passphrase hazard (delta — interaction with `--privacy-preserving`)

v0.1 §7.3 unchanged.

v0.2 NEW interaction: `--privacy-preserving` does NOT obscure passphrase usage. If `--passphrase` is set and `--privacy-preserving` is set, the engraving card stderr STILL emits the `passphrase: USED` warning. Privacy mode hides only the master fingerprint, not the existence of a passphrase.

### §7.4 Wordlist-language hazard (no changes)

### §7.5 Hardware-wallet support caveat (NEW)

Taproot multisig (`tr-multi-a` / `tr-sortedmulti-a`) signing-side support is nascent as of v0.2 (per Q1/L1 brainstorm closures). Users emitting a taproot-multisig bundle MUST verify their signing device supports `multi_a` / `sortedmulti_a` Miniscript leaves before engraving — emitting a backup for a wallet whose hardware can't sign with it produces an unusable backup. The §5.2 engraving-card stderr emits a HARDWARE WALLET CAVEAT line for the two taproot-multisig templates.

---

## §8. Out-of-scope items deferred (delta — 5 features promote to "shipped")

| Feature | v0.1 tier | v0.2 tier |
|---|---|---|
| Multisig templates | v0.2 deferred | **shipped** |
| `--account` flag | v0.2 deferred | **shipped** |
| `--xpub`-input multisig | v0.2 deferred | **shipped** |
| `--privacy-preserving` | v0.2+ deferred | **shipped** |
| `--self-check` | v0.2 deferred | **shipped** |
| K-of-N share encoding | v0.2 deferred | **v0.3 (gates on ms-codec v0.2)** |
| `--output <dir>` | v0.3 deferred | v0.3 (unchanged) |
| Recovery flow | v0.3+ deferred | v0.3+ (unchanged) |
| User-supplied descriptor | v0.3+ deferred | **v0.3+** (formerly implicit; now explicit per Q11) |
| Hash-locked / hybrid-miniscript descriptors | implicit v0.3+ | **v0.3+** (subset of user-supplied descriptor) |
| Color / interactive prompts | never | never (unchanged) |

---

## §9. Closures from brainstorm

### §9.1 Q1–Q5 closures (carried from v0.1; no changes)

### §9.2 r1 architect findings (carried from v0.1)

### §9.3 r2 architect findings (carried from v0.1)

### §9.4 v0.2 architect closures (back-filled from §11 revision history)

Concrete findings + resolutions from the SPEC r1 / r2 / r3 architect-review iterations. Verbatim summary distilled from §11 history (cross-reference §11 for the full bullet list); each row below points at the SPEC location where the resolution lives.

- **r1 critical findings:** none. r1 verdict: 0C/3I/4L/3N.
- **r1 important findings (3) — all integrated in r2:**
  - **I1 — multisig `--mk1` grouping syntax:** §2.2.1 clarified that the user passes flat `--mk1` repetitions; the toolkit groups by `chunk_set_id` internally before calling mk-codec decode. Mismatch in chunk-set-id count produces exit 4 BundleMismatch.
  - **I2 — SELF-MULTISIG WARNING text + ordering:** §4.1 + §5.2 added a non-suppressible SELF-MULTISIG WARNING for `--cosigner-count > 1` in full mode (acknowledges all N xpubs are byte-identical; advises watch-only multisig for production). Byte-exact text pinned. r3 N-1 then specified emission ordering (stderr advisory fires BEFORE the bundle stdout block).
  - **I3 — `mk1` JSON discriminated-union shape:** §5.3 added explicit `oneOf` description and Rust serde guidance for the flat-vs-nested mk1 field.
- **r1 low findings integrated in r2:**
  - **L2 — `<fp>` REQUIRED in --cosigner spec:** §2.1.2 — empty fingerprint rejected as exit 1 `CosignerSpec`. Privacy mode suppresses fingerprints from output but the user still supplies them for cross-binding.
  - **L3 — path/xpub depth consistency:** §4.5 — emit exit 1 `CosignerSpec` on depth mismatch in watch-only mode (mirrors Phase 1.5 spike Errata 2).
  - L1, L4 deferred (cosmetic; do not block advancing).
- **r2 critical findings:** none. r2 verdict: 0C/2I/2L/1N.
- **r2 important findings (2) — all integrated in r3:**
  - **I-A — stub-list mismatch across cards:** §2.2.1 step 5b — emit exit 4 `BundleMismatch` when N decoded mk1 cards expose mismatched stub-lists (catches mixed-bundle transcription errors).
  - **I-B — chunk_set_id extraction order:** §2.2.1 step 1 — `mk_codec::decode_string` + `StringLayerHeader::from_5bit_symbols` (BCH-correction first, header parse second); `SingleString`-headed cards each form their own group with synthetic group-key.
- **r2 low findings integrated in r3:**
  - **L-A — `--cosigners-file` REQUIRED fields:** §2.1.2.1 — explicit `master_fingerprint` + `xpub` REQUIRED bullet; missing/empty → exit 1 `CosignersFile`.
  - **L-B — `--privacy-preserving` + `--xpub` advice:** §6.6 — drops the contradictory "drop --master-fingerprint" advice; now correctly says "drop --privacy-preserving or switch to multisig watch-only mode."
- **r2 nit integrated in r3:**
  - **N-1 — SELF-MULTISIG WARNING ordering:** §4.1 — stderr advisory fires BEFORE the bundle stdout block.
- **r3 verdict:** 0C/0I/0L/0N — SPEC frozen for implementation.
- **Brainstorm Q1–Q12 closure proofs:** every brainstorm question's lock has a SPEC location implementing it. Cross-reference table:

| Q | Brainstorm lock | SPEC location |
|---|---|---|
| Q1 | All BIP-388 multisig (6 templates incl. taproot); pure wrappers only | §2.1.3, §4.6 |
| Q2 | BIP-48 + BIP-87 with `--multisig-path-family` flag (default bip87) | §2.1.7, §4.1, §4.2 |
| Q3 | 1 ≤ K ≤ N ≤ 16 | §2.1.1, §6.2 (`MultisigConfig` rejects out-of-range) |
| Q4 | Hybrid: --account global default + per-cosigner override | §2.1.7, §4.1, §4.2 |
| Q5 | --cosigner canonical + --cosigners-file bulk; explicit path overrides family default | §2.1.2, §2.1.2.1, §6.6 |
| Q6 | Whole-bundle privacy boolean | §2.1.8, §4.5.1 |
| Q7 | Reuse exit 4 BundleMismatch with self-check[X] card identifier | §2.1.9, §4.7 |
| Q8 | Separate templates for sorted vs unsorted | §2.1.3, §4.6 |
| Q9 | Wire-bit-identical encoded strings; JSON schema_version=2; mk1 flat for single-sig, nested for multisig | §5.3, §9.4.1 below |
| Q10 | Sparse fixture matrix ~50 cells | (implementation; tracked in IMPLEMENTATION_PLAN) |
| Q11 | K-of-N + user-supplied + hash-locks deferred to v0.3+ | §1, §8 |
| Q12 | --xpub vs --cosigner runtime pre-check; mutually exclusive | §6.6 |

### §9.4.1 Wire-bit-identical claim scope (closure detail)

For a single-sig invocation under a v0.2 binary with all multisig flags omitted: the emitted ms1, mk1, and md1 strings are byte-identical to a v0.1 binary's output for the same input. The JSON envelope differs (`schema_version: "2"`, new optional fields with default values) but the encoded card strings inside do not. v0.1 decoders consuming v0.2-emitted encoded strings work unchanged.

---

## §10. Reference implementation (delta)

### §10.1 Sibling-crate dependencies (no version bumps required)

v0.2 consumes these APIs newly (but at the SAME pinned versions as v0.1):
- `md_codec::Tag::{Multi, SortedMulti, MultiA, SortedMultiA}` (multisig wrappers)
- `md_codec::Body::Variable { k, children }` (multisig body)
- `md_codec::PathDeclPaths::Divergent(Vec<OriginPath>)` (per-cosigner paths)
- `mk_codec::KeyCard::new(stubs, None, ...)` (privacy mode; was always supported, just unexercised in v0.1)

All of the above are present in `md-codec v0.16.1` and `mk-codec v0.2.1` per Phase 0 audit + Phase 1.5 spike. No sibling-crate version bumps required for v0.2.

### §10.2 cargo publish status (unchanged from v0.1)

`cargo publish` of the toolkit remains gated on `ms-codec` / `mk-codec` / `md-codec` reaching crates.io. v0.2 ships via GitHub tag `mnemonic-toolkit-v0.2.0` like v0.1.

### §10.3 Pinned dep versions (no changes)

```toml
ms-codec = { git = "https://github.com/bg002h/mnemonic-secret",      tag = "ms-codec-v0.1.0" }
mk-codec = { git = "https://github.com/bg002h/mnemonic-key",         tag = "mk-codec-v0.2.1" }
md-codec = { git = "https://github.com/bg002h/descriptor-mnemonic",  tag = "md-codec-v0.16.1" }
```

---

## §11. Revision history

- **v0.2.0 r1 (2026-05-05):** initial v0.2 SPEC draft. Brainstorm-architect-r2 0C/0I closures pulled in. Phase 1.5 spike memo's verified API surface ratified into §4.6 / §4.5 deltas. Architect r1 returned 0C/3I/4L/3N.
- **v0.2.0 r2 (2026-05-05):** integrated architect-r1 findings (0C/3I/4L/3N).
  - **I1**: §2.2.1 multisig `--mk1` grouping syntax clarified — user passes flat `--mk1` repetitions; toolkit groups by `chunk_set_id` internally before calling mk-codec decode. Mismatch in chunk-set-id count produces exit 4 BundleMismatch.
  - **I2**: §4.1 + §5.2 add a non-suppressible SELF-MULTISIG WARNING for `--cosigner-count > 1` in full mode (acknowledges all N xpubs are byte-identical; advises users to use watch-only multisig for production). Byte-exact text pinned.
  - **I3**: §5.3 explicit discriminated-union shape note for `mk1` field with JSON-Schema-style `oneOf` description and Rust serde guidance.
  - **L2**: §2.1.2 `<fp>` REQUIRED in `--cosigner` spec; empty rejected as exit 1 `CosignerSpec`. Privacy mode suppresses fingerprints from output but the user still supplies them for cross-binding.
  - **L3**: §4.5 path/xpub depth consistency requirement added (mirrors Phase 1.5 spike Errata 2; emit exit 1 `CosignerSpec` on mismatch in watch-only mode).
  - L1, L4, N1, N2, N3 deferred (cosmetic / process; do not block advancing).
- **v0.2.0 r3 (2026-05-05):** integrated architect-r2 findings (0C/2I/2L/1N).
  - **I-A**: §2.2.1 step 5b added — stub-list mismatch across N decoded cards emits exit 4 `BundleMismatch` (catches mixed-bundle transcription errors).
  - **I-B**: §2.2.1 step 1 specifies `chunk_set_id` extraction via `mk_codec::decode_string` + `StringLayerHeader::from_5bit_symbols` (BCH-correction first, then header parse); `SingleString`-headed cards each form their own group with synthetic group-key.
  - **L-A**: §2.1.2.1 explicit "REQUIRED" bullet for `master_fingerprint` and `xpub` in `--cosigners-file` JSON; missing/empty → exit 1 `CosignersFile`.
  - **L-B**: §6.6 `--privacy-preserving` + `--xpub` row reworded — drops the contradictory "drop --master-fingerprint" advice (single-sig watch-only requires --master-fingerprint always); now correctly says "drop --privacy-preserving or switch to multisig watch-only mode."
  - **N-1**: §4.1 SELF-MULTISIG WARNING emission ordering specified — stderr advisory fires BEFORE the bundle stdout block.

(SPEC review iterations populate §9.4 with findings; final r2/r3+ verdicts append revision-history bullets here.)
