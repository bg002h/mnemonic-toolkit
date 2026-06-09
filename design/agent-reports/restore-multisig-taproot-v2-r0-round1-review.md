# R0 Review — v2 SPEC round 1 — `SPEC_restore_multisig_taproot.md`

> Persisted verbatim from the opus-architect R0 dispatch on the v2 SPEC (taproot multisig restore). v2 supersedes the RED v1.

---

Important nuance. `derive_first_address` (`:23-26`) has a header comment: **"Reserved for non-taproot descriptors (BIP-386 is outside BIP-129 prerequisites); the caller must reject `tr(...)` before invoking."** This is a doc-comment contract caveat — but the underlying mechanism (`into_single_descriptors` + `derive_at_index(0)` + `.address(network)`) is miniscript-generic and DOES support taproot multipath at rev 95fdd1c (the v1 review M1 confirmed it renders `bc1p`). So the fix is: either use this helper (its "reject tr" caveat is a BIP-129/BSMS-context note, not a technical limit) or inline the same three-call miniscript sequence on the reconstructed descriptor. The mechanism is sound; the SPEC just needs to specify routing the taproot first-address through a descriptor-string path, not `d.derive_address`.

Now let me verify the `is_nums:false` third-party reconstructability point quickly — confirming a `multi_a` (not sorted) is_nums:false card with a real cosigner key IS renderable by md-codec (so the SPEC's "never renderable" rationale is factually wrong). The Tr arm at `to_miniscript.rs:161-165` does `lookup_key(keys, key_index)` when `!is_nums`, and the MultiA leaf renders. So yes, `tr(@cosigner, multi_a(...))` with is_nums:false renders fine. The SPEC's §2 rationale overclaims. This is a Minor (the refusal disposition is still safe; only the justification is wrong).

I have everything needed. Let me write the final R0 review.

---

# R0 Review (round 2, v2 SPEC) — `SPEC_restore_multisig_taproot.md`

## VERDICT: RED — 1 Critical / 2 Important / 3 Minor. Implementation MUST NOT begin.

The two v1 Criticals (C1 is_nums premise, C2 build_descriptor_string reproducibility) **are genuinely dissolved** — verified against source at `5a22552`, not papered over. But v2 introduces a **new, independent Critical**: the route-around fixes the descriptor *string* yet leaves the first-receive-address derivation routing back through the exact md-codec function (`to_miniscript_descriptor`) that the SPEC bypasses — which hard-errors on `sortedmulti_a`. As written, the headline `tr-sortedmulti-a` capability is non-functional (CLI hard-fails at address derivation). This blocks the gate.

---

## What I confirmed CLEAN (so it is not re-litigated)

- **C1 dissolved — accurate.** `template.rs:213` is `is_nums: true` for the shared `TrMultiA | TrSortedMultiA` arm (`:194`), so bundle emits `Body::Tr{is_nums:true}` for **both** leaf types (only `inner_tag` differs, `:195-199`). `synthesize.rs:398` feeds `template.wrapper_node(...)` straight into the bundle path. The old v1 `:446 assert!(!is_nums…)` is now `template.rs:450 assert!(is_nums, …)` — flipped, not removed; reinforced by `:540/:579/:613` wire-round-trip asserts. C1 is real.
- **C2 dissolved — accurate.** `pipeline.rs:18` signature matches; the `Nums` arm (`pipeline.rs:128-133`) emits `tr(NUMS_XONLY_HEX, {leaf_op}({k}, <all key_segs>))` — keeping **all** cosigners in the leaf, which byte-mirrors bundle's `is_nums:true` tree (leaf `indices:(0..n)`, `template.rs:219`). The round-trip oracle in §3 is non-tautological and exact. C2 is real.
- **md-codec types/mechanism reachable.** The toolkit compiles against **crates.io `md-codec 0.35.0`** (`Cargo.toml:27`, `Cargo.lock:655-658`) — NOT a git checkout. That source HAS `Body::Tr { is_nums: bool, key_index, tree }` (`tree.rs:49-54`), `pub tag`/`pub body` on `Node`, `Tag::{MultiA,SortedMultiA}`. `extract_multisig_threshold` recurses `Body::Tr{tree:Some(inner),..}` (`bundle.rs:1042`). `expand_per_at_n` at `canonicalize.rs:420`. `TaprootInternalKey{Nums,Cosigner(u8)}` at `wallet_export/mod.rs:86-94`. `CliTemplate::{TrMultiA,TrSortedMultiA}`.
- **Upstream-block reversal — accurate.** miniscript rev `95fdd1c` HAS `Terminal::SortedMultiA` (`decode.rs:161`, `astelem.rs:172` Display), confirmed in `Cargo.lock:673-675`. md-codec 0.35.0's `to_miniscript.rs:406-410` still hand-errors on `SortedMultiA` (no `Terminal::SortedMultiA` arm), so routing the descriptor *string* around md-codec is the right call.

---

## CRITICAL

### C1(new) — `tr-sortedmulti-a` restore hard-errors at first-address derivation; the route-around does not cover the address path. (`restore.rs:840-842`)
The SPEC §1 step 7 asserts the existing emission "already renders `tr(...)`/`bc1p` … unchanged." It does not, for `sortedmulti_a`. The chain:
- `restore.rs:840-842` derives `first_recv` via `d.derive_address(0, i, network)`.
- `md-codec 0.35.0 derive.rs:120` — `derive_address` internally calls `to_miniscript::to_miniscript_descriptor(self, chain)`.
- `to_miniscript.rs:406-410` — `Tag::SortedMultiA` returns `Err` unconditionally (`tree_to_taptree:277` → `node_to_miniscript::<Tap>` → the SortedMultiA error arm; no tap-leaf-root special case exists).

So for a `tr-sortedmulti-a` md1, `restore --md1` aborts at `:842` (`bad("first receive address @{i}: …")`) **after** the descriptor string was correctly reconstructed at `:835`. `first_recv` is consumed in both the text path (`:1097/:1130`) and JSON (`:1075`), so it is unavoidable. The route-around is exactly the recon's "Phase B" reason-for-being (`tr-sortedmulti-a`), and as specified that capability is dead on the standard output path.

- **`tr-multi-a` is fine** end-to-end: `Tag::MultiA` renders (`to_miniscript.rs:394-398`), so `d.derive_address` succeeds. The defect is **`sortedmulti-a`-only**.
- **Not silent-wrong:** it is a hard error, no xpriv leak, no wrong wallet emitted. But it defeats the headline scope as written → Critical.
- **Internal inconsistency (corroboration):** the §5 test "first address is `bc1p…`" is *unsatisfiable as specified* for sortedmulti_a — the full-CLI test goes RED at address derivation, contradicting the SPEC's claim the suite is green. The author did not run the sortedmulti_a CLI path end-to-end.

**Concrete fix (in-hand, local):** in the new taproot branch, derive the first receive address from the **reconstructed descriptor string** via the toolkit's pinned miniscript — i.e. `MsDescriptor::from_str(&descriptor)` then `into_single_descriptors()` + `derive_at_index(0)` + `.address(network)` (the exact sequence `derive_address.rs:34-66 derive_first_address` already implements; miniscript `95fdd1c` renders `bc1p` for tr multipath, per v1-review M1). Do NOT use `d.derive_address`. Note `derive_first_address`'s header (`derive_address.rs:24-25`) says "the caller must reject `tr(...)`" — that is a BIP-129/BSMS-context caveat, not a technical limit; either relax that contract deliberately or inline the identical three-call sequence. The SPEC must add this as an explicit step and add an address-rendering test that actually exercises the sortedmulti_a CLI to `bc1p`.

---

## IMPORTANT

### I1 — "cf. the bip86 single-sig taproot restore that already ships" (§1 step 7) is a false analogy and is the source of C1(new).
Single-sig restore (`run`, `restore.rs:362`) renders via `render_address_from_xpub` (xpub-based), not `d.derive_address`; and the descriptor-string helper `derive_first_address` (`derive_address.rs:26`) uses miniscript directly. bip86 single-sig has no `sortedmulti_a` leaf, so its success proves nothing about the multisig md1 path, which uses a *different* address mechanism (`d.derive_address`). Remove/correct this justification — it is what masked the gap. (Maps to the dependency-order spirit of v1's verdict: the "existing emission works" claim was never validated for the sortedmulti_a leaf through `run_multisig`.)

### I2 — `--format` `taproot_internal_key:None` citation is `:662`, not `:696`; and the thread-through is under-specified.
The hardcoded `taproot_internal_key: None` for the multisig `--format` payload is at `restore.rs:662` (inside `build_multisig_import_payload`); the call site is `:1034`. There is a *second* `None` at `:606` (single-sig `build_import_payload` — leave it). The SPEC §4 (and v1 review, and recon) all cite `:696`, which at `5a22552` is `xpub_from_65_bytes` — a stale/wrong line. The doc-comment cite `:636` is accurate. Fix: thread `Some(internal_key)` by adding a `taproot_internal_key` parameter to `build_multisig_import_payload` (it currently takes 7 args and hardcodes `None` internally) and passing it from the `:1034` call site. The SPEC says "thread the reconstructed `Some(internal_key)` through" but cites the wrong line and omits that the function signature must change. Note: the `--format` payload path (`emit_payload`) does NOT call `d.derive_address`, so `--format` itself is not hit by C1(new) — but it still needs the correct `internal_key` to emit a correct taproot descriptor in the payload.

---

## MINOR

### M1 — §2 / §0 refusal *rationale* for `is_nums:false` is factually wrong (the disposition is still safe).
The SPEC claims `is_nums:false` taproot md1s "were never renderable" / "never functional." False for a third-party `tr-multi-a` card carrying a genuine cosigner internal key: md-codec's Tr arm (`to_miniscript.rs:161-165`) does `lookup_key(keys, key_index)` when `!is_nums` and the `MultiA` leaf renders (`:394-398`) — so `tr(@cosigner, multi_a(k, …))` renders fine, and `build_descriptor_string`'s `Cosigner(idx)` arm (`pipeline.rs:135-155`) reproduces it. It is a **reconstructable** shape being refused, not a non-existent one. Refusing it for v2 is the correct, safe scope call (avoids a leaf-membership analysis), but state it as "deferred/filed reconstructable shape," not "never renderable." (`sortedmulti_a` + is_nums:false would additionally hit C1(new)'s md-codec error, but that's a separate axis.)

### M2 — Stale citations beyond I2.
`extract_multisig_threshold` is `bundle.rs:1036` (recurse at `:1042`), not `:1021` (SPEC §4 / recon). `template.rs:213 is_nums:true` ✓, `pipeline.rs:18/128` ✓, `wallet_export/mod.rs:86` ✓, `restore.rs:777` ✓, `:742` ✓ are all accurate. Fix the two line numbers (`:696→:662`, `:1021→:1036`) in the SPEC and any descendant plan-doc.

### M3 — Disposition mechanics are otherwise sound.
PATCH is correct (`restore --md1` already exists; widening accepted input — no new flag/subcommand → no GUI `schema_mirror`; matches the `restore --md1 --format` precedent). The release-gate checklist (§7: dual README markers + `scripts/install.sh` self-pin + `readme_version_current.rs` + `install-pin-check.yml`, re-run suite after bump) is complete. The descriptor-mnemonic companion filing (§7.6) and FOLLOWUP-body collapse are correctly carried from recon. `lint_argv_secret_flags` N/A ✓.

---

## Disposition

Not a re-scope like v1 — the premise is now correct and the architecture (route the descriptor string around md-codec) is sound. The blocker is a **single localized hole**: the first-address derivation must also route around md-codec for `sortedmulti_a`. Fix C1(new) + I1 + I2 in the SPEC (add the descriptor-string address-derivation step; correct the false bip86 analogy; correct the `--format` thread-through line/signature), downgrade the §2 rationale (M1), fix the two stale line numbers (M2), then this is a clean GREEN. Re-R0 the revised SPEC before implementation.
