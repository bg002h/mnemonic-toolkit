# Whole-diff adversarial review — `deps/md-codec-0.44.0` (md-codec 0.44.2 adoption)

**Reviewer:** independent agent (opus), 2026-09-19. Did not author the branch.
**Diff under review:** `git diff origin/master...deps/md-codec-0.44.0` — 13 commits, 16 files, +745/−92.
**Baseline for every A/B measurement:** `origin/master` = `9513f754`, branch tip = `8c418666`.
**Method:** both binaries built and run side by side (`master` from a throwaway worktree with its own
`CARGO_TARGET_DIR`; branch from the repo's own `target/debug/mnemonic`). Every claim below is a
measured command, not a reading. Mutation tests were run in a third detached worktree, which was
removed; the repo tree is unmodified by this review.

**The one question:** *does anything in this diff produce a wrong wallet, a wrong address, a card that
cannot be restored, or a check that passes when it should fail?*

**Answer: yes — one Critical.** A canonical descriptor carrying inline origins on *some* placeholders
now fabricates an origin for the others, engraves it, and — for secret-bearing slots — **derives the
key at the fabricated path**, producing a different wallet at a different address than the shipped
binary produces for the identical command line. A card emitted by the shipped v0.101.0 for that shape
now fails `verify-bundle` with `result: mismatch`.

---

## Counts

| Severity | Count |
| --- | --- |
| Critical | 1 |
| Important | 2 |
| Minor | 5 |
| Nit | 2 |

Full suite re-run independently during this review: **`cargo nextest run --locked` → 4037 passed,
0 failed, 20 skipped, 3.945s.** The Critical is invisible to all 4037.

---

# CRITICAL

## C1 — Ungating `bind_descriptor_mode_paths` left the `Divergent` arm's default-inference ungated: a canonical descriptor with PARTIAL inline origins now fabricates an origin, engraves it, and derives the key at it

**File:** `crates/mnemonic-toolkit/src/cmd/bundle.rs:2302-2338` (and symmetrically on the verify side
via `verify_bundle.rs:1514`).

### What the diff did

Commit `41c415fa` removed the early `return` that made `bind_descriptor_mode_paths` a no-op for
canonical descriptors. It then carefully re-gated default-inference inside the `PathDeclPaths::Shared`
arm:

```rust
PathDeclPaths::Shared(op) => {
    if op.components.is_empty() {
        if is_non_canonical {
            defaulted_indices.extend(0..(n as u8));
            (0..n).map(|_| default_path.clone()).collect()
        } else {
            // Canonical with no inline origins: leave them EMPTY here ...
            // Defaulting a canonical shape would invent an origin the
            // descriptor never claimed.
            (0..n).map(|_| OriginPath { components: Vec::new() }).collect()
        }
    } else { ... }
}
```

It did **not** re-gate the `PathDeclPaths::Divergent` arm eight lines below, which is unchanged from
before the ungating:

```rust
PathDeclPaths::Divergent(v) => v
    .iter()
    .enumerate()
    .map(|(i, op)| {
        if op.components.is_empty() {
            defaulted_indices.push(i as u8);
            default_path.clone()          // <-- fires for CANONICAL descriptors now
        } else {
            op.clone()
        }
    })
    .collect(),
```

Before the ungating that arm was unreachable for canonical shapes (the `return` came first). It is
reachable now, and it does exactly what the comment eight lines above forbids: **it invents an origin
the descriptor never claimed**, on a canonical shape.

`resolve_placeholders` produces `Divergent` with one empty entry whenever a canonical descriptor
annotates some `@N` and not others — e.g. `wsh(sortedmulti(2,[fp/48'/0'/0'/2']@0,@1))`. That shape is
confirmed canonical: `--account 1` on it is refused with `DESCRIPTOR_WITH_NONZERO_ACCOUNT`, and that
refusal is gated on `!is_non_canonical`.

### (a) Secret-bearing slots — DIFFERENT KEY, DIFFERENT ADDRESS, DIFFERENT WALLET

`anno_path` (bundle.rs:1514) is read out of `resolved_placeholders.path_decl` *after* the binder has
mutated it, and it is what `Phrase` / `Entropy` / `Ms1` slots derive with
(`master.derive_priv(&secp, &anno_path)`). So the fabricated path is not a label — it is the
derivation path.

Repro (P1 = `abandon`×23 `art`, P2 = `legal winner thank year … title`):

```sh
mnemonic --allow-argv-secret bundle --network mainnet \
  --descriptor "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']@0,@1))" \
  --slot "@0.phrase=$P1" --slot "@1.phrase=$P2" --no-engraving-card
```

| | @1 key on the emitted card | first receive address |
| --- | --- | --- |
| **master (shipped v0.101.0)** | `[0d9250da]` `xpub661MyMwAqRbcGf4bJ6AT4rMVgxJEukeuiEBNo2ujYe3VbKbk59CJi3WqdeEtrKeMeaZUa8sGnC3bDSia2hUTuwTamHvJe5Avm6fYp1EUwJo` (P2's **master** key, empty origin) | `bc1q9ryzzm44z2pfx28a2zx40eztr6l4gmcwzaea6rnz5trr0syms6yqct3w4j` |
| **branch** | `[0d9250da/48'/0'/0'/2']` `xpub6DXuQW1Q2JpZw4v1XW76nro81tTsci36FcwAPrj57hTYmWswiGWUMW9MYFHVzw5MdohEEuxTVqctxYXLW3xucC3rg5qQRnpNcXqft65zUrA` | `bc1qr8hf2l7afk692rm4j638j54egewh7cxykstddr8rgupz58zrnhfqcq3fzw` |

Same command, two different wallets. And the previously-emitted card no longer verifies:

```
$ mnemonic --allow-argv-secret verify-bundle <same args> --bundle-json <master-emitted>.json   # BRANCH binary
mk1_xpub_match[1]: fail cosigner[1] xpub mismatch
mk1_path_match[1]: fail cosigner[1] path mismatch
md1_xpub_match:    fail md1 pubkeys differ from expected set
result: mismatch
```

The same file verifies `result: ok` on the master binary. This is precisely the failure mode the
`verify_bundle.rs` fold comment names as unacceptable — *"a verifier that disagrees with the emitter
about what was emitted … turns a correct backup into a failed check"* — reached across versions, for
a shape the fold's own case enumeration never considered.

### (b) Watch-only (xpub) slots — a FABRICATED origin is engraved

```sh
mnemonic bundle --network mainnet \
  --descriptor "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']@0,@1))" \
  --slot "@0.xpub=$X0" --slot "@1.xpub=$X1" --slot "@1.fingerprint=aaaaaaaa"
```

master restores `…,[aaaaaaaa]xpub661My…` (no path claimed).
branch restores `…,[aaaaaaaa/48'/0'/0'/2']xpub6DXuQ…` — **an origin the operator never supplied and
the descriptor never declared**, cut into metal. Address unchanged, so no address check sees it.

An empty origin says *"unknown"*. A fabricated one says *"the key is at m/48'/0'/0'/2' under
aaaaaaaa"*, which is false. For a backup artifact, engraving a confident lie is worse than engraving
a blank.

### Why nothing caught it

- The fold's own safety argument enumerates three canonical cases — *with* inline origins, with
  *neither* origins nor slot paths, and with slot paths. The **partial** case is absent. I measured
  all three of the enumerated cases and they are indeed byte-identical no-ops (`git`-level: emitted
  `md1`+`mk1` sha256 identical for "both inline origins", "both inline origins + matching slot paths",
  and "no origins, no paths"). Only the un-enumerated fourth case moves.
- No test in the repo exercises a partially-annotated canonical descriptor:
  `grep -rn ']@0,@1)' crates/mnemonic-toolkit/` → 0 hits. The new
  `cli_slot_path_reaches_the_card.rs` is canonical-with-*no*-origins by its own comment.
- The only user-visible signal is an `info:` line on **stderr** that *mislabels the descriptor*:
  `info: non-canonical descriptor; defaulting origin path for @1 to m/48'/0'/0'/2'` — emitted for a
  descriptor that is canonical. Under `--json` it is invisible on stdout. On the **verify** side the
  notice is discarded entirely (`verify_bundle.rs:1514` binds the result to `_defaulted`), so a
  verifier fabricates silently.

### Remedy direction (not authoritative — reproduce, then decide)

Gating the `Divergent` arm the same way the `Shared` arm was gated restores the enumerated safety
argument (`if op.components.is_empty() && is_non_canonical { … } else { op.clone() }`). Whatever is
chosen, the partial-annotation shape needs a test that asserts the **origin as stored**, in the same
style as `cli_slot_path_reaches_the_card.rs`, and a cross-version statement about cards already
engraved from that shape.

---

# IMPORTANT

## I1 — The md-codec 0.44 xpub-header fix does NOT reach the taproot wallet-policy restore arm; two goldens silently kept the pre-fix value and nothing records the exception

**Files:** `crates/mnemonic-toolkit/Cargo.toml:47-50` (the claim);
`crates/mnemonic-toolkit/tests/cli_restore_taproot.rs:108,111` (the two unchanged goldens).

The Cargo.toml comment justifying the git-tag pin says:

> `0.42.0 -> 0.44.0` is what makes `restore`/`bundle` emit a descriptor whose xpub header agrees with
> the origin beside it. Before it, **every** rendered key serialised at depth 0 under a depth-4 origin.

Measured on the branch binary — bundle, then restore, then decode each rendered xpub's depth and
compare to its declared origin's component count:

| shape (all with `--slot @N.path=` supplied) | rendered depth vs origin length |
| --- | --- |
| `wsh(sortedmulti(2,@0,@1))` | 4 / 4 **OK** |
| `wsh(multi(2,@0,@1))` | 4 / 4 **OK** |
| `sh(wsh(sortedmulti(2,@0,@1)))` | 4 / 4 **OK** |
| `wsh(and_v(v:pk(@0),pk(@1)))` (general) | 4 / 4 **OK** |
| `tr(NUMS,and_v(v:pk(@0),older(144)))` (general) | 4 / 4 **OK** |
| **`tr(NUMS,multi_a(2,@0,@1))`** | **0 / 4 MISMATCH** |
| **`tr(NUMS,sortedmulti_a(2,@0,@1))`** | **0 / 4 MISMATCH** |

and the non-NUMS distinct-trunk pair, byte-for-byte identical between master and branch:

```
tr(K2,multi_a(2,K0,K1))        master and branch BOTH restore
  tr([28645006/87'/0'/0']xpub661MyMwAqRbcEdy4jr5Et…   depth 0 under a depth-3 origin
tr(K2,sortedmulti_a(2,K0,K1))  same
```

while its sibling one line up in the same file moved as advertised:

```
tr(K2,and_v(v:pk(K0),older(144)))  master xpub661MyMwAqRbcEdy4jr5Et…  →  branch xpub6BemYiVNp19ZzE9Ek… (depth 3)
```

So the *same key* at the *same declared origin* serialises two different ways inside one binary,
depending on tree shape. The taproot **wallet-policy / template** arm — which is the canonical
taproot multisig this toolkit mints — is unfixed.

Machine-check of all six taproot goldens in the file (chain code + pubkey vs the pre-diff value,
depth/child vs the declared origin, BIP-380 checksum recomputed):

```
GOLDEN_DESC_SINGLE_LEAF         MOVED=True  checksum_ok=True  depth 3 / origin 3  OK   (keymat identical)
GOLDEN_DESC_TWO_LEAF            MOVED=True  checksum_ok=True  depth 3 / origin 3  OK   (keymat identical)
GOLDEN_DESC_MULTI_A_2LEAF       MOVED=True  checksum_ok=True  depth 3 / origin 3  OK   (keymat identical)
GOLDEN_DESC_NON_NUMS_GENERAL    MOVED=True  checksum_ok=True  depth 3 / origin 3  OK   (keymat identical)
GOLDEN_DESC_NON_NUMS_MULTI_A        MOVED=False           depth 0 / origin 3  MISMATCH
GOLDEN_DESC_NON_NUMS_SORTEDMULTI_A  MOVED=False           depth 0 / origin 3  MISMATCH
```

The four that moved are **correct, not merely new** — verified independently, not taken from the
commit message. The two that did not move are asserted live at lines 228 and 253 (not `#[ignore]`d),
which is how the suite stays green while carrying the defect the cycle exists to remove.

**Severity call:** Important, not Critical. The address is unchanged (chain code and point are right),
so no funds are misdirected and no card is unrestorable. What is wrong is that the branch's central
claim is false as written, two goldens silently disagree with their four siblings, and an operator
importing a taproot multisig restore into a wallet that validates `depth == len(origin)` gets a
descriptor the branch believes it fixed. The exception is recorded nowhere — not in the commit
message ("Four taproot goldens re-baselined … Verified the change is serialisation ONLY"), not in
FOLLOWUPS, not in the file header (which still names the sortedmulti_a rendering gap as open).

**To close:** either fix the taproot template arm, or state the exception explicitly next to the
Cargo.toml claim *and* in a FOLLOWUP, and add an assertion that pins the gap's exact shape rather than
leaving two goldens looking like ordinary un-updated pins.

## I2 — The `bundle-descriptor-drops-per-slot-path` follow-up understates the shipped defect: on NON-canonical descriptors the old binary engraves a WRONG origin, not an empty one

**File:** `design/FOLLOWUPS.md:5727` onwards.

The entry says, repeatedly and as its headline: *"every slot gets one shared EMPTY origin"*, *"the
engraved card records where neither key lives"*, *"md-codec renders the origin as `[5436d724/m]` for
both slots"*. That is true for the **canonical** arm. It is not true for the non-canonical arm, which
the entry does not distinguish.

Measured on the shipped master binary:

```sh
mnemonic bundle --network mainnet --descriptor "wsh(and_v(v:pk(@0),pk(@1)))" \
  --slot "@0.xpub=$X0" --slot "@0.fingerprint=aaaaaaaa" --slot "@0.path=m/48'/0'/7'/2'" \
  --slot "@1.xpub=$X1" --slot "@1.fingerprint=bbbbbbbb" --slot "@1.path=m/48'/0'/8'/2'"
```

- `--json` reports `origin_paths: ["m/48'/0'/7'/2'","m/48'/0'/8'/2'"]` — what the operator asked for.
- The **card** records `[aaaaaaaa/48'/0'/0'/2']` and `[bbbbbbbb/48'/0'/0'/2']` — the *default-inferred*
  account 0, for keys that live at accounts 7 and 8.
- The branch fixes this (records `48'/0'/7'/2'` and `48'/0'/8'/2'`; address unchanged both ways).

So on the non-canonical path the old defect is not "the card is blank about the origin" but "the card
states a plausible, specific, **wrong** origin". An operator holding such a plate and following its
engraved origin derives a different key and concludes the plate is not theirs. A blank origin is
obviously suspicious; `[aaaaaaaa/48'/0'/0'/2']` is not.

This matters because that FOLLOWUP entry is what any operator advisory about already-engraved plates
will be written from, and because it shapes the remedy: the canonical fix ("the paths must be plumbed
into `PathDecl::Divergent` at emit") does not by itself tell you that the non-canonical arm was
overwriting supplied paths with inferred ones.

**Also uncovered:** the branch's new `cli_slot_path_reaches_the_card.rs` is canonical-only by
construction ("CANONICAL on purpose … the canonical arm is the one that dropped the paths"). The
non-canonical override-loop half of the fix — the one measured above — has no test asserting the
origin as stored. A cell mirroring the existing one with `wsh(and_v(v:pk(@0),pk(@1)))` would pin it.

---

# MINOR

## M1 — The default-path notice mislabels a canonical descriptor, and the verify side emits none at all

`bundle.rs:2499` hardcodes `"info: non-canonical descriptor; defaulting origin path for …"`. After the
ungating, `defaulted_indices` can be non-empty for a **canonical** descriptor (that is C1), so the
notice states a falsehood about the input it is describing. On the verify side the return value is
discarded (`let _defaulted = …`, `verify_bundle.rs:1514`), so verify fabricates origins with no notice
whatever. Whatever is decided about C1, the notice text should not assert canonicity it did not check,
and the two sides should not disagree about whether the operator is told.

## M2 — Three new md-codec refusals narrow the CLI accept-set with no CHANGELOG entry and an error message that names no remedy

`wsh(sortedmulti(2,@0,@1))` with two distinct xpubs sharing one fingerprint and no origin paths:
master exits 0 and emits a card; the branch exits 2 with `OriginKeyContradiction`. Same for
`wsh(and_v(v:pk(@0),pk(@1)))`. The refusal is defensible (the card *would* declare `[fp/m]` twice over
two keys), and there **is** an escape hatch — dropping `--slot @N.fingerprint` makes it encode again,
measured — but the error text says only *"at least one of these origins is wrong"* and offers no way
forward to an operator who was handed two xpubs and does not know their paths.

`CHANGELOG.md` is untouched by this branch, and `[Unreleased]` carries nothing about the accept-set
narrowing, the changed emitted bytes, or the changed restore serialisation. `changelog-check.yml`
fires only on a `mnemonic-toolkit-v*` tag and only checks that a *section* exists, so it cannot catch
a section that omits a breaking accept-set change.

## M3 — FOLLOWUPS entries land stale inside the same branch that falsifies them

- `bundle-descriptor-drops-per-slot-path` is `**Status:** OPEN` (FOLLOWUPS.md:5774 region) while
  commit `41c415fa` on this same branch ships its fix. `scripts/followup-reconcile.sh` prints
  *"Reminder: flip Status -> resolved IN THE SAME COMMIT that ships the deliverable."*
- The same entry says `audit_i10_same_xpub_two_paths_2of2_round_trips` *"stays RED on the
  `deps/md-codec-0.44.0` branch until this lands"*. It landed; the test is green.
- `adopt-md-codec-0-44-in-the-toolkit` cites tag `descriptor-mnemonic-md-cli-v0.16.0`; the shipped pin
  is `v0.16.2` (`Cargo.toml:52`, `Cargo.lock:688`).

## M4 — The `RelativeTimelockTruncated` arm is unreachable, and its message asserts a policy the toolkit does not have

md-codec's `validate_relative_timelocks` is explicitly opt-in and is **not** called from `encode.rs`
(only `validate_origin_key_consistency` and `validate_no_duplicate_key_slots` are, under
`Admission::Enforce`), so the new exit-2 mapping and friendly message can never fire from the toolkit.
Harmless as an exhaustive-match arm — but the message says *"the plate would assert a lock the chain
does not enforce"*, i.e. a refusal, while the toolkit's own policy for the same condition is the
**non-blocking advisory** in `src/timelock_advisory.rs`. If it ever does become reachable the two
surfaces will contradict each other.

## M5 — `template_form_accepts_tr_sortedmulti_a_since_the_render_gap_closed` asserts decode, not render

Its own doc comment says *"the whole point of the old refusal was that what got written could not be
read back"*, then asserts `md_codec::chunk::reassemble(…)` plus a `SortedMultiA` tag in the tree.
`reassemble` is decode; the gap was in `to_miniscript_descriptor`. The test is **not** vacuous — I
confirmed the master binary refuses the same invocation (`rc=2`, *"cannot template this descriptor
shape … the rust-miniscript v13 SortedMultiA gap"*) while the branch exits 0, so `bundle_ok` alone
carries the RED. But the assertion the comment advertises is not the assertion it makes; the render is
covered elsewhere (`template.rs`'s inverted pin, and
`non_nums_distinct_trunk_sortedmulti_a_restores_faithfully`).

---

# NIT

## N1 — `#[cfg(test)]` is sandwiched between two doc comments on `synthesize_multisig_full`

`synthesize.rs:602-645`: a "TEST-ONLY since 2026-09-19" block, then `#[cfg(test)]`, then a second
full doc comment starting "Synthesize a full-mode multisig bundle from ONE seed". Legal Rust, and both
render, but it reads as two competing descriptions of the same function with an attribute wedged in
the middle.

## N2 — Dead `by_index_subkeys` computation kept alive by `let _ = &subkeys;`

`bundle.rs:2349-2374`: the `by_index_subkeys` map is still built and the per-iteration `subkeys` set is
still cloned out of it, only to be discarded by `let _ = &subkeys;`. Also, the account guard in
`synthesize_multisig_full` uses `account.checked_add(i as u32)`, but the real ceiling for a hardened
index is `2^31 − 1`, not `u32::MAX`; `account = 0x7FFF_FFFF` with `n = 2` reaches
`DerivationPath::from_str` and fails with a `path parse` message rather than the intended "overflows
u32" one. Test-only function, no funds path.

---

# What I checked and found SOUND (so the next reviewer does not re-derive it)

1. **The three enumerated canonical cases in the `41c415fa` safety argument are genuine no-ops.**
   Emitted `md1`+`mk1` are byte-identical between master and branch for: canonical with both inline
   origins; canonical with both inline origins *and* matching `--slot @N.path=`; canonical with
   neither origins nor slot paths. Measured by sha256 over the emitted card lines.
2. **The §6.6 row-19 inline-vs-slot mismatch refusal is now REACHABLE and fires.** On master,
   `wsh(sortedmulti(2,[fp/48'/0'/0'/2']@0,[fp/48'/0'/1'/2']@1))` with a contradicting
   `--slot @0.path=m/48'/0'/9'/2'` was **silently accepted** and emitted the same card as the
   consistent invocation. The branch refuses it: `error: slot @0 path mismatch: --slot says
   48'/0'/9'/2', descriptor inline [.../48'/0'/0'/2'] disagrees`. This is a real hole closed.
3. **For watch-only (xpub) slots the ungating is label-only, as claimed** — the first receive address
   is unchanged in every xpub-slot case I measured, canonical and non-canonical.
4. **`tests/fixtures/export_wallet_allow/rcw_wsh_descriptor.txt` is a correct rewrite.** All 6 keys:
   chain code and public key **byte-identical** to the pre-diff value; new depth == origin component
   count (4) and new child == the origin's terminal component (`1'` → `2147483649`) for all 6;
   `parent_fingerprint` stays `00000000`; nothing but the xpub strings and the checksum changed. Both
   the old checksum `#aqeftg67` and the new `#3hghtfkg` recompute correctly under the BIP-380
   algorithm (independent Python implementation, not the tool's own).
5. **The `cli_check_pkk_canonical_golden.rs` relabel is correct, verified independently.** A
   from-scratch BIP-32/BIP-39 derivation in Python (own secp256k1, no toolkit code) over
   `abandon`×11 `about` gives master fingerprint `73c5da0a` and
   `m/48'/0'/1'/2'` → `xpub6DzhyrnFFYQ1HimDiM388xHnDiRPNdZJFBmmxge3Y1WWcHLtMJLfRuhRHqnQCPbTj3fGKTuKFLHzzwpJkp5Dtc3UtLKZKaVZe1yqMBXd6Vk`
   = `XPUB4_1` exactly. `k1()` really was mislabelled with `@0`'s account; `PATH4_1` is the true origin.
   Only the two two-key goldens moved, which is consistent with a per-key-origin change.
6. **The three `at_in_both_*` tests are NOT vacuous after the fixture change — mutation-proven.**
   Neutering `refuse_at_in_both` (`restore.rs:3297`, `if false && indices.iter().any(…)`) in a
   throwaway worktree turns all three RED:
   `test result: FAILED. 18 passed; 3 failed` in `cli_restore_taproot`, and the RED output is exactly
   the dangerous silent-wrong reconstruction the guard exists to prevent — exit 0 printing
   `tr([00bbccdd/…], sortedmulti_a(2,[01bbccdd/…],[02bbccdd/…]))` with the trunk key dropped from the
   leaf, at a different address. The guard fires for the right reason. Separately confirmed from the
   pinned md-codec source that `validate_no_duplicate_key_slots` is called only from
   `encode.rs:169`, so decode-side hostile cards still reach the toolkit and the guard remains
   load-bearing.
7. **The retired `tr(sortedmulti_a)` refusal genuinely expired.** Master refuses the exact
   `--template tr-sortedmulti-a --md1-form template` invocation with exit 2 citing *"the
   rust-miniscript v13 SortedMultiA gap"*; the branch exits 0. The `[patch.crates-io] miniscript`
   rev `ff4732e` was already on master (unchanged by this diff), so the gap closed on the md-codec
   side; `to_miniscript.rs:469-481` in the pinned checkout carries the conversion. `template_admissible`
   ends in `to_miniscript_descriptor(..).is_ok()` — renderability, not policy — so flipping the gate
   retired no safety refusal.
8. **`synthesize_multisig_full`'s per-slot wiring is consistent.** `slot_xpubs[i]`, `slot_origins[i]`
   and `slot_keys[i]` are all filled in the same `for i in 0..cosigner_count` loop from
   `account + i`; `pubkeys` is built by `enumerate()` over `slot_xpubs`; the mk1 card loop uses
   `slot_keys[i].0`/`.1`; the csi is still `derive_mk1_chunk_set_id_for_slot(&stub, i as u32)`.
   `#[cfg(test)]` on the function is proven by the fact that the crate compiles — any non-test caller
   would be a hard error.
9. **The F4 `Divergent`→`Shared` collapse is not a byte-change risk for canonical descriptors.**
   `wsh(sortedmulti(2,[aaaaaaaa/48'/0'/0'/2']@0,[bbbbbbbb/48'/0'/0'/2']@1))` emits byte-identical
   `md1` on master and branch, with `path_decl_shape: Shared` on both — `resolve_placeholders`
   already collapses identical inline origins.
10. **Emit and verify are symmetric on the branch.** Both call the same
    `bind_descriptor_mode_paths` with the same `is_non_canonical`; the only asymmetry is the
    Emit-only row-19 refusal. A branch-emitted card verifies `result: ok` on the branch for every
    shape I exercised, including the C1 shape. (The C1 harm is cross-version, not intra-version.)
11. **Full suite, re-run independently:** `cargo nextest run --locked` → **4037 passed, 0 failed,
    20 skipped**, 3.945s.

---

# Reproduction inventory

Everything above was produced by these, against the two binaries:

```sh
# canonical partial-origin, phrase slots — C1(a), different wallet + verify mismatch
mnemonic --allow-argv-secret bundle --network mainnet \
  --descriptor "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']@0,@1))" \
  --slot "@0.phrase=<abandon x23 art>" --slot "@1.phrase=<legal winner … title>" --json

# canonical partial-origin, xpub slots — C1(b), fabricated origin
mnemonic bundle --network mainnet \
  --descriptor "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']@0,@1))" \
  --slot "@0.xpub=<X0>" --slot "@1.xpub=<X1>" --slot "@1.fingerprint=aaaaaaaa"

# taproot header fix does not reach the wallet-policy arm — I1
mnemonic bundle --network mainnet --descriptor "tr(NUMS,multi_a(2,@0,@1))" <slots with paths> --json
mnemonic restore --network mainnet --md1 …            # depth 0 under a depth-4 origin

# non-canonical, xpub slots + slot paths — I2, master engraves the inferred account
mnemonic bundle --network mainnet --descriptor "wsh(and_v(v:pk(@0),pk(@1)))" \
  --slot "@0.xpub=<X0>" --slot "@0.fingerprint=aaaaaaaa" --slot "@0.path=m/48'/0'/7'/2'" \
  --slot "@1.xpub=<X1>" --slot "@1.fingerprint=bbbbbbbb" --slot "@1.path=m/48'/0'/8'/2'"

# row-19 now reachable — sound
mnemonic bundle --network mainnet \
  --descriptor "wsh(sortedmulti(2,[5436d724/48'/0'/0'/2']@0,[5436d724/48'/0'/1'/2']@1))" \
  --slot "@0.xpub=<X0>" --slot "@0.path=m/48'/0'/9'/2'" \
  --slot "@1.xpub=<X1>" --slot "@1.path=m/48'/0'/1'/2'"

# at_in_both mutation — sound
#   restore.rs:3297  ->  if false && indices.iter().any(|&idx| idx == *i) {
cargo test --test cli_restore_taproot        # 18 passed; 3 failed
```

`X0 = xpub6Bner3L3tdQW367NmmMsWKtMfP7hbu4JxdtbSGdWWjSzLkSUEnT7G9h5GFWUXtifeRhHiUXJuek1qeaTJqnXkveWpiHp8rmt53E8HTMshg9`
`X1 = xpub6Buxw9MmbkJr8dFGbbbjY46MzzbM8MCosN5AxgxVEstcQMYcAn7oV8DvwYouSbixK4zhej2oTUoMkFD6FaHu7tuZPLiGQ7VKcBcj8fmj4g9`

No secret-handling defects are reported here as blocking, per the standing operator ruling; none of
the findings above is of that class.
