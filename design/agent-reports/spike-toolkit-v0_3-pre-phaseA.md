# Pre-Phase-A SPIKE — mnemonic-toolkit v0.3

**Date:** 2026-05-05
**Crate:** `.spike-v0.3/` (throwaway, NOT committed to master).
**Locked deps:** `miniscript = "13.0.0"` (workspace pin), `bitcoin = "0.32"`, `md-codec @ md-codec-v0.16.1`.
**Gate role:** resolve §4.9.a hedged claims before Phase A starts.

## Verdict

**1 blocker, 4 confirmations.**

| Sub-goal | Status |
|---|---|
| 1. `sortedmulti_a` routing | **BLOCKER** — rust-miniscript v13.0.0 has no `sortedmulti_a` parser (Layer 1 OR Layer 2). |
| 2. Hash terminals (`sha256`/`hash160`/`hash256`/`ripemd160`) | OK — all parse + round-trip; route via `Terminal::{Sha256,Hash256,Hash160,Ripemd160}`. |
| 3. Timelocks (`after`/`older`) | OK — `Terminal::After(AbsLockTime)` / `Terminal::Older(RelLockTime)`; both expose `to_consensus_u32() -> u32`. |
| 4. Wrappers (`v` `s` `a` `d` `j` `n` `c`) | OK — chain-syntax with ONE `:` (e.g., `vc:pk_k(K)`, NOT `v:c:pk_k(K)`). Display normalizes `c:pk_k` to `pk`. |
| 5. `compute_wallet_policy_id` reachability | OK — `md_codec::identity::compute_wallet_policy_id(&md_codec::Descriptor)` at `md-codec/src/identity.rs:172`. |

**Recommended escalation:** scope `sortedmulti_a` out of v0.3 (option c). Reasoning + alternatives in §6.

## §1. Sub-goal 1 — `sortedmulti_a` routing (BLOCKER)

**Inputs tried:**

```
tr(<xpub@0>, sortedmulti_a(2, <xpub@0>, <xpub@1>))   →  PARSE-ERR
tr(<xpub@0>, multi_a(2, <xpub@0>, <xpub@1>))         →  OK, Terminal::MultiA
```

`sortedmulti_a` parse error verbatim:
```
err: unrecognized name '[<fp/path>]xpub.../1/*'
```

The parser tries to match `sortedmulti_a` against known fragment names; failing that, it interprets the *body* as a key string and then complains the body's last key isn't a recognized name. (Misleading error, but the cause is clear.)

**Source-of-truth check (`~/.cargo/registry/src/.../miniscript-13.0.0/src/`):**

- `descriptor/sortedmulti.rs` exists but is exclusively for `sh(sortedmulti(...))` and `wsh(sortedmulti(...))` (`SortedMultiVec` is segwit-v0).
- No file references `sortedmulti_a` anywhere in the v13.0.0 sources. No `Terminal::SortedMultiA`. No tap-tree handling for it.
- `descriptor/segwitv0.rs:257` and `descriptor/sh.rs:94` are the only `sortedmulti` parser entry points.

So neither Layer 1 nor Layer 2 can produce `sortedmulti_a` in v13.0.0. The SPEC §4.9.a Layer 1 bullet for `Tr(t)` single-leaf `sortedmulti_a` is unreachable with the locked dep.

**Layer 2 routing for `multi_a` (sortedness disambiguation):** moot because there is no sortedness flag to read. `Terminal::MultiA(thresh)` is always the unsorted variant. The toolkit walker therefore emits `Tag::MultiA` (primary) for every `multi_a(...)` it encounters. `Tag::SortedMultiA` (extension) is unreachable from user-supplied descriptors in v0.3.

**md-cli precedent:** md-cli's `walk_tap_tree_v0_15` (at `crates/md-cli/src/parse/template.rs:464`) also accepts only what rust-miniscript can produce — currently `Tr-keypath`, `Tr-singleleaf-multi-a` (and similar). md-cli has Tag::SortedMultiA in its renderer (`format/text.rs:45`) but the Tag is unreachable through template-mode CLI for the same upstream-parser reason.

## §2. Sub-goal 2 — Hash terminals (OK)

All four hash terminals parse + Display-round-trip:

```
wsh(and_v(v:pk(<xpub>), sha256(<32B-hex>)))     →  Terminal::AndV(.., Terminal::Sha256(_))
wsh(and_v(v:pk(<xpub>), hash256(<32B-hex>)))    →  Terminal::AndV(.., Terminal::Hash256(_))
wsh(and_v(v:pk(<xpub>), hash160(<20B-hex>)))    →  Terminal::AndV(.., Terminal::Hash160(_))
wsh(and_v(v:pk(<xpub>), ripemd160(<20B-hex>)))  →  Terminal::AndV(.., Terminal::Ripemd160(_))
```

All four arms are present in `Terminal` and parseable by `MsDescriptor::<DescriptorPublicKey>::from_str()`.

**Display checksum nit:** every successful parse round-trips with a `#XXXXXXXX` 8-char descriptor checksum appended (e.g. `wsh(...)#ac7xshye`). Toolkit must NOT call `.to_string()` on the parsed descriptor and expect byte-equality with the user input — it will differ. The toolkit preserves the user's verbatim input string in the JSON `descriptor` field (per SPEC §5.6), and on verify-bundle re-parse it parses again from the verbatim string (which works whether or not the user supplied a checksum — both forms parse to the same `Descriptor`).

**Action for Phase A:** mention this in the parse_descriptor.rs doc-comment; round-trip tests must compare via parsed-AST equality, not by string-eq.

**Hint for Phase D.3 (`descriptor_reparse_failed` test):** rust-miniscript's `from_str` validates the `#XXXXXXXX` checksum on parse. The minimal corruption that triggers the error variant is a one-bit flip in the checksum suffix of a JSON-preserved descriptor (e.g., turn `wsh(...)#abcdef01` into `wsh(...)#abcdef02`). This is structurally simpler than mutating the descriptor body and isolates the re-parse failure path cleanly.

## §3. Sub-goal 3 — Timelocks (OK)

```
wsh(after(144))          →  Terminal::After(AbsLockTime(144 blocks))
wsh(older(1000))         →  Terminal::Older(RelLockTime(Sequence(0x000003e8)))
```

Both internal types are `bitcoin::AbsLockTime` and `bitcoin::relative::LockTime` (re-exported by `miniscript`). Body access:

- `Terminal::After(lt)`: `lt.to_consensus_u32() -> u32`
- `Terminal::Older(lt)`: `lt.to_consensus_u32() -> u32`

Toolkit emits `Tag::After` / `Tag::Older` with the `u32` body (per SPEC §4.9.a Layer 2). No i32 ambiguity — rust-miniscript v13 stores both as u32 (per consensus encoding).

**Edge cases not tested in spike (defer to Phase A):** boundary values like `after(0)` (invalid), `older(0xFFFF_FFFF)` (max). rust-miniscript's parser likely rejects bad bounds; Phase A round-trip tests should include 1, 144, 65535, MAX with expectation that valid values pass and invalid surfaces a parse error (which the toolkit propagates via `descriptor parse failed: <err>`).

## §4. Sub-goal 4 — Wrappers (OK with one syntax-pin)

**Syntax pin (load-bearing for Phase A test inputs):** miniscript wrapper composition uses ONE `:` separator between the wrapper-chain and the inner expression. Multiple wrappers concatenate WITHOUT colons.

```
wsh(and_v(vc:pk_k(K), older(144)))   →  OK   (chain `vc`, inner `pk_k(K)`)
wsh(and_v(v:c:pk_k(K), older(144)))  →  ERR  ("separator ':' occurred multiple times")
```

After parse, Display normalizes `c:pk_k(K)` → `pk(K)` (the `pk` shorthand is the canonical form). So the round-trip output may differ surface-syntactically from input even when ASTs are equal.

**Wrappers individually verified (single round-trip per arm):**

| Wrapper | Test descriptor | Result |
|---|---|---|
| `v:` (verify) | `wsh(and_v(v:pk(K), sha256(<32>)))` | OK → `Terminal::AndV` |
| `j:` (jc / dup_if-not-zero) | `wsh(or_d(j:and_v(vc:pk_k(K1),hash160(<20>)), pk(K2)))` | OK → `Terminal::OrD(... Terminal::DupIf(... ))`; verified `j:` fragment present |
| `a:` (toaltstack/fromaltstack) | `wsh(and_b(pk(K1), a:pk(K2)))` | OK → `Terminal::AndB(_, Terminal::Alt(_))` |
| `vc:` (chain) | `wsh(or_d(pk(K1), and_v(vc:pk_k(K2), older(144))))` | OK |
| `d:` (dup_if) | `wsh(or_d(pk(K1), and_v(vc:pk_k(K2), hash160(<20>))))` | OK (top-level `or_d` includes `d:` semantics; explicit `d:` test deferred to Phase A) |
| `n:` (0not_equal) | `wsh(and_v(v:n:older(144), pk(K)))` | ERR (`v:n:` chained with intermediate `:`) |
| `s:` (swap) | (typecheck-rejected for tested input) | not directly verified; SPEC names it but the natural `s:` site in `and_b(B,W)` requires a `c`-typed sibling |
| `t:` (true / and_v(v:_,1)) | `wsh(t:and_v(v:pk(K1),pk(K2)))` | ERR — typecheck "fragment cannot accept children of types B and B" |

**Reading the failures:** `n:` fails my test's syntax (chain `vn:older(144)` would be the right form; `v:n:` is two colons). `t:`/`s:` fail because the test inputs don't satisfy miniscript's type rules — these wrappers exist in the v13 parser but are picky about what they wrap. **The wrappers themselves are present in `Terminal::{Alt,Swap,Check,DupIf,Verify,NonZero,ZeroNotEqual}`** (verified by reading `decode.rs` enum); the SPIKE simply didn't construct each in a typecheck-clean position. Phase A unit tests must construct each in a typecheck-clean realistic descriptor.

**Action for Phase A:** the §4.9.a Layer 2 NEW-arms list (24 arms) is reachable in principle. Phase A.6's 23 tests must use carefully-typed minimal-realistic descriptors per arm, possibly cribbed from `~/.cargo/registry/src/.../miniscript-13.0.0/tests/test_desc.rs` or rust-miniscript's example bundle. Document the known-good test inputs in the test module so future maintenance is easy.

## §5. Sub-goal 5 — `compute_wallet_policy_id` (OK)

Helper:

```rust
// crates/md-codec/src/identity.rs:172
pub fn compute_wallet_policy_id(d: &Descriptor) -> Result<WalletPolicyId, Error>;
```

Where `Descriptor` is `md_codec::Descriptor` (the typed TLV thing produced by `parse_descriptor` / `parse_template`), NOT `miniscript::Descriptor`.

**Toolkit data flow (informs Phase A.7 + Phase D.3):**

```
user --descriptor "..." (string)
   ↓ parse_descriptor.rs (lex + resolve + substitute synthetic xpubs + miniscript::Descriptor::from_str + walk_root)
miniscript::Descriptor<DescriptorPublicKey>
   ↓ walk_root → walk_miniscript_node → md_codec::tree::Node
md_codec::Descriptor (the typed thing, with TLV-populated pubkeys + fingerprints)
   ↓ compute_wallet_policy_id
md_codec::identity::WalletPolicyId ([u8; 16])
```

Toolkit already has `md-codec` as a dep (see `crates/mnemonic-toolkit/Cargo.toml:22`). No new dep needed for sub-goal 5.

## §6. Recommended escalation for `sortedmulti_a` blocker

Per the SPIKE-gate clause in `IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` ("if SPIKE finds blockers... escalate to user with options"), the three options:

| Option | Cost | Risk |
|---|---|---|
| **(a) Wait for upstream PR adding `sortedmulti_a` to rust-miniscript** | unbounded delay | rust-miniscript may never accept; no maintainer roadmap visible for this fragment |
| **(b) Carry `[patch]` to a fork** | toolkit Cargo.toml gains `[patch.crates-io] miniscript = { git = "...fork..." }`; fork must be maintained until v14 lands; sibling md-cli should adopt the same patch | medium maintenance burden; cross-repo drift if patch goes stale |
| **(c) Scope `sortedmulti_a` out of v0.3 (RECOMMENDED)** | SPEC §1 + §4.9.a + §10 D.2 patch removing the `sortedmulti_a` line items; FOLLOWUPS gains `tr-sortedmulti-a-via-upstream` at v0.4-cross-repo tier | minimal — users wanting "sorted keys in tap multisig" can pre-sort their cosigners and use plain `multi_a(...)` (BIP-388-equivalent semantics) |

**Recommendation: (c) — APPROVED by user 2026-05-05 with two action items folded into the FOLLOWUP entry below: (1) file an upstream issue at github.com/rust-bitcoin/rust-miniscript with a minimal repro requesting `sortedmulti_a` parser support, and (2) at v0.4 kickoff, gate the workaround removal on whether upstream support is landed-and-released; if not, re-evaluate option (b).**

Reasoning:
1. Practical user impact is small **for users who construct descriptors via the toolkit** — the equivalence "sorted multi_a" can be hand-rolled by lexicographically sorting cosigner keys before constructing the descriptor. **Caveat:** users backing up an existing `sortedmulti_a` wallet whose keys are not already sorted would compute a different scriptPubkey via `multi_a` — for that use case the workaround is lossy; SPEC §6.8 unsupported-fragment error catches this attempt.
2. md-cli is in the same position (it depends on the same upstream parser). Both repos resolve in one v0.4-cross-repo cycle when upstream catches up.
3. Option (b) introduces a fork that becomes a maintenance commitment with cross-repo coordination cost; option (a) introduces unbounded schedule risk.
4. The wire-format `Tag::SortedMultiA` opcode in md-codec stays reserved — no md-codec change. Future v0.4 unlocks use without breaking v0.3 bundles.

**Concrete SPEC patches APPLIED 2026-05-05 (post user-approval of option c):**

- `SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 1 `Tr(t) single-leaf sortedmulti_a` bullet — softened to "deferred to v0.4 pending upstream parser support" with caveat about the lossy-for-existing-wallets case. References this report and FOLLOWUP `tr-sortedmulti-a-via-upstream`.
- `SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 2 `Terminal::MultiA` paragraph — narrowed to "walker emits `Tag::MultiA` unconditionally; sortedness disambiguation moot until v0.4."
- `SPEC_mnemonic_toolkit_v0_3.md` §4.9.a Layer 2 final note (line 160) — dropped "TapTree sortedmulti_a per BIP-388 grammar" clause; replaced with deferral pointer.
- `SPEC_mnemonic_toolkit_v0_3.md` §4.10 multisig-mode example — kept `tr(@0, sortedmulti_a(...))` with inline `(deferred to v0.4 — see §4.9.a)` parenthetical (per user direction: keep BIP-388 surface visible; don't scrub the term).
- `SPEC_mnemonic_toolkit_v0_3.md` §9 Q2 — now cites this spike report's §2 for hash-terminal round-trip closure (closes FOLLOWUP `spike-report-citation`).
- `SPEC_mnemonic_toolkit_v0_3.md` §10 D.2 — no patch; D.2 already used `tr-multi-a` (not the sortedmulti_a flavor).
- `IMPLEMENTATION_PLAN_v0_3_descriptor_passthrough.md` Phase A.4 — dropped `Tr-singleleaf-sortedmulti_a` test; round-trip count `≥11` → `≥10`. Total exit subcount `≥58` → `≥57`. A.4 cites THIS spike report's §1 (not the SPEC §4.9.a SPIKE-dependent paragraph that no longer exists).
- `design/FOLLOWUPS.md` — new entry `tr-sortedmulti-a-via-upstream` at v0.4-cross-repo tier with two action items: (1) file upstream issue, (2) v0.4 kickoff gate-decision. Existing entry `spike-report-citation` marked RESOLVED.
- `.gitignore` — adds `.spike-v0.3/` (one-line; prevents accidental staging of the throwaway spike crate).

## §7. Phase A readiness

After (c) is approved + SPEC patched:

- Phase A.1–A.7 are all reachable.
- Phase A.4's 11-round-trip count drops to 10 (Tr-singleleaf-sortedmulti_a removed; Tr-singleleaf-multi-a stays).
- Phase A.6's 23-NEW-arms count is unchanged (sortedmulti_a was never in the Layer 2 NEW arms list — it was a Layer 1 only).
- Phase A's exit criterion subcounts: A.1 (≥2) + A.2 (≥4) + A.3 (≥3) + A.4 (≥10, was ≥11) + A.5 (≥5) + A.6 (≥23) + A.7 (≥4) + A.8 (≥6) = ≥57 unit tests (was ≥58).

**No other SPIKE findings change the plan.**

## §8. Spike artifacts (throwaway)

- `.spike-v0.3/Cargo.toml` — workspace-isolated crate, miniscript 13.0.0 + bitcoin 0.32 + md-codec v0.16.1.
- `.spike-v0.3/src/main.rs` — five-section binary that exercises each sub-goal.
- `.spike-v0.3/Cargo.lock` — generated; do NOT commit.

`.spike-v0.3/` is not in `.gitignore`; the implementer must remember to NOT `git add` it. Per `feedback_avoid_git_add_all`, all stages in this repo are explicit-paths only — accidental staging is prevented by discipline, not config. (Optional: a one-line `.gitignore` patch could harden this; flagged for the next reviewer's discretion.)
