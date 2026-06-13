# SPEC — `restore --md1` reconstructs non-NUMS ("real key at the trunk") taproot

**Source SHA:** `29613f3` (origin/master == HEAD at write time; sync clean). All file:line
citations grep-verified against this tree; re-grep on any later rebase.
**Cycle:** lifts the `is_nums:false` taproot-restore refusal (`cmd/restore.rs:700`); resolves the
"`is_nums:false` (cosigner-internal) deferred" carve-out of `restore-multisig-taproot-reconstruction`
(v0.49.1; the deferral is a code-comment at `cmd/restore.rs:676-678`, not a `###` registry entry —
this cycle files a proper slug).
**SemVer:** PATCH (a previously-refused md1 shape becomes a faithful reconstruction; watch-only;
zero clap delta → no GUI `schema_mirror`).
**Scope:** toolkit-only. No md-codec / sibling / GUI code change.
**Predecessors:** `SPEC_restore_multisig_taproot.md` (v0.49.1, NUMS tr-multi-a/sortedmulti-a) and
v0.55.1 general single/2-leaf `tr(NUMS,…)` faithful restore — this extends both to a non-NUMS
internal key.
**Pre-R0 architect direction-consult folded** (this session): split routing (NOT a unified
route-around — md-codec hard-errors on `SortedMultiA`); `@-in-both` is a funds-safety trap the
fidelity guard cannot catch → refuse-and-defer. Verdict YELLOW→GREEN conditioned on the §4
structural guard. The mandatory **formal R0 gate runs on THIS spec** (0C/0I before any code).

---

## §1 Problem

A taproot wallet-policy md1 whose internal ("trunk") key is a real/cosigner key
(`Body::Tr { is_nums: false }`) is refused by `restore --md1` — only NUMS (`is_nums:true`,
script-path-only) taproot reconstructs. A non-NUMS tr means a **live key-path spend** exists (the
trunk-key holder can spend directly) alongside the script tree (see BIP-341 §1: the output key is
`Q = P + hash_TapTweak(P‖merkle_root)·G`; with a real `P` the key-path secret `p + t` is usable; with
NUMS nobody knows it). Such a card is a faithful backup but cannot be auto-restored to a usable
descriptor today.

**Reachability (verified):** `bundle --descriptor "tr(<real xpub>, multi_a(2,B,C))"` → exit 0,
emits a real `is_nums:false` md1. So this is a genuine engrave-yes / restore-no gap, not hypothetical.
(`bundle` from-seed always emits NUMS since v0.48.0; a non-NUMS card arises from `--descriptor`
intake or an external/legacy tool.)

## §2 Decision (user-approved scope)

**Support now** — faithful non-NUMS reconstruction for:
- **(a) General** single-leaf / depth-1 `tr(<real key>, <general miniscript>)` — via the
  **GeneralFaithful route-around**.
- **(b) Distinct-trunk multisig** `tr(cosigner_i, multi_a/sortedmulti_a(k, {the OTHER cosigners}))`
  where the trunk key is NOT one of the leaf keys — via the **Template path** (`Cosigner(idx)` mode).

**Refuse-and-defer** — the legacy **`@-in-both`** shape `tr(@i, multi_a/sortedmulti_a(k, {…@i…}))`
where the trunk key is ALSO a leaf key (§4). Depth-≥2 taptrees stay refused (separate upstream
FOLLOWUP, unchanged).

## §3 Architecture — lift one gate, read the trunk off the wire, **split routing**

1. **Lift the gate.** `classify_taproot_restore` (`cmd/restore.rs:692`) currently refuses
   `Body::Tr { is_nums: false, .. } => ModeViolation` (`:700`). Replace the blanket refusal with:
   capture `key_index` and route by leaf (below). NUMS path unchanged.
2. **Read the trunk key from the wire — no inference.** `Body::Tr.key_index` → the internal key.
   Map: `is_nums:true → TaprootInternalKey::Nums`; `is_nums:false → TaprootInternalKey::Cosigner(key_index)`.
3. **`TaprootRestore` carries the internal key per arm** (`cmd/restore.rs:661-668`, currently
   `Template(CliTemplate)` / `GeneralFaithful` with no key):
   ```rust
   enum TaprootRestore {
       Template(CliTemplate, TaprootInternalKey),
       GeneralFaithful(TaprootInternalKey),
   }
   ```
4. **Route by leaf tag** (in `classify_taproot_restore`):
   - `Tag::MultiA` / `Tag::SortedMultiA` (`:719-720`) → **Template path**. For `is_nums:false`, FIRST
     apply the §4 `@-in-both` structural guard; if it passes, `Template(t, Cosigner(key_index))`.
     (Template path = `build_tr_multi_a_descriptor`, `wallet_export/pipeline.rs:113-156`, which writes
     the descriptor STRING directly — dodging md-codec's `SortedMultiA` gap, `to_miniscript.rs:423-425`.)
   - general leaf (`:730`) → **route-around** `GeneralFaithful(Cosigner(key_index))`. The route-around
     (`faithful_multisig_descriptor` → `md_codec::to_miniscript`) already emits a real internal key:
     `to_miniscript.rs:161-164` `is_nums:false → lookup_key(keys, *key_index)`, and renders
     `Terminal::MultiA` (`:411-415`) fine. (It only hard-errors on a `SortedMultiA` leaf — which never
     reaches this arm; those route to the Template path above.)
5. **Thread the internal key at the call site** (`cmd/restore.rs:1207-1208`, currently hard-codes
   `Some(TaprootInternalKey::Nums)` for both arms) — use the `TaprootInternalKey` carried in the
   `TaprootRestore` variant.
6. **Keep the Display-fidelity guard** (`cmd/restore.rs:~1287`, parse→print before any address
   derivation). It is the real net for the route-around arm; for the Template path it is a no-cost net.

## §4 The `@-in-both` guard — the funds-safety crux (architect's YELLOW→GREEN condition)

The Template path's `Cosigner(idx)` mode reconstructs the leaf as **`{all cosigners EXCEPT idx}`**
(`pipeline.rs:143-148`). For the `@-in-both` shape (`tr(@0, multi_a(k, @0, @1, @2))`, leaf indices
`{0,1,2}` including the trunk index 0), that shortcut would emit `multi_a(k, @1, @2)` — **a different
multisig, a different address, a silently-wrong wallet.**

**Critically, the Display-fidelity guard does NOT catch this:** the Template path builds the descriptor
by `MsDescriptor::from_str(rendered).to_string()` (`pipeline.rs:28-31`), so its output IS its own
re-print — a wrong-but-self-consistent leaf passes the parse→print check. Therefore the protection
**must be a STRUCTURAL precondition at classify time, not a post-reconstruction comparison.**

**Guard:** when routing a `Tag::MultiA`/`Tag::SortedMultiA` leaf with `is_nums:false`, read the leaf
`Body::MultiKeys { indices, .. }` (cf. `restore.rs:1079`) and check whether the trunk `key_index ∈
indices`. If present → **refuse loudly**: `ModeViolation` (exit 2), message stating the card is a
faithful backup but its trunk key is also a leaf key, citing the deferred FOLLOWUP slug
`restore-non-nums-tr-internal-key-also-in-leaf`. Never run the Cosigner shortcut on it.

(General-arm leaves cannot hit this: a general miniscript leaf reconstructs via the route-around,
which reads the ACTUAL tree and would render any internal-key-also-in-a-sub-fragment faithfully, with
the Display-fidelity guard as backstop. The trap is specific to the Template/Cosigner "leaf=all-others"
computation.)

## §5 Components / files
- `cmd/restore.rs` — `TaprootRestore` enum (add `TaprootInternalKey` to both variants);
  `classify_taproot_restore` (lift the `:700` gate, thread `key_index`, add the §4 guard); the call
  site `:1207-1208` (thread the internal key). Display-fidelity guard unchanged.
- `wallet_export/pipeline.rs` — `build_tr_multi_a_descriptor` `Cosigner(idx)` arm already exists
  (`:113-156`); reached now for `is_nums:false` distinct-trunk multisig. No change expected.
- **No md-codec change** (route-around uses the existing `is_nums:false` branch). No clap change.

## §6 Error handling
- `@-in-both` → `ModeViolation` exit 2, slug-cited (§4).
- Depth-≥2 → unchanged refusal (`upstream-miniscript-taptree-depth2-display-asymmetry`).
- Any reconstruction whose descriptor fails parse→print → `bad()` (the fidelity guard).
- `--format green` for a non-NUMS general-tr → keep the existing P2tr singlesig refusal behavior
  (R0 to confirm a key-path-bearing tr classifies sanely under `script_type_from_descriptor`).

## §7 Testing (all via the bundle→restore round-trip)
- Golden: non-NUMS **general** single-leaf `tr(D, and_v(v:pk(B),older(N)))` → reconstructs the
  descriptor (real trunk key D) + a receive address; cosigner fingerprints/origins preserved.
- Golden: non-NUMS **distinct-trunk multisig** `tr(D, multi_a(2,B,C))` AND `tr(D, sortedmulti_a(2,B,C))`
  → reconstruct (trunk D not in the leaf; leaf = {B,C}).
- **Refusal, RED-proven:** `@-in-both` `tr(<leaf-key-i>, multi_a(2,<key-i>,<key-j>))` → `ModeViolation`
  + slug. RED-proof: with the §4 guard removed, the reconstruction silently drops the trunk key from
  the leaf AND passes the fidelity guard (demonstrating the guard's necessity).
- Depth-≥2 non-NUMS → still refused.
- NUMS regression: existing v0.49.1 / v0.55.1 NUMS goldens stay byte-identical (the NUMS path is
  unchanged — `Nums` still threads through).

## §8 SemVer & locksteps
- **PATCH** — watch-only; lifts a refusal into a faithful reconstruction; zero clap delta → **no GUI
  `schema_mirror`, no paired-PR**. (`schema_mirror` is flag-NAME parity only.)
- **Manual:** `docs/manual/src/40-cli-reference/41-mnemonic.md`, restore taproot section — note
  non-NUMS key-path taproot support + the `@-in-both` refusal. Run the FULL manual lint.
- **FOLLOWUPS:** (a) file this cycle's slug `restore-non-nums-taproot-internal-key` and mark RESOLVED
  on ship; (b) file the deferred `restore-non-nums-tr-internal-key-also-in-leaf` (the `@-in-both`
  shape; route-around-for-multi_a is the eventual mechanism, blocked-adjacent to the md-codec
  SortedMultiA gap).
- No md-codec / sibling companions.

## §9 R0 open items / non-goals
R0 (mandatory, 0C/0I before code) must confirm:
1. The §4 `@-in-both` guard is a **structural classify-time** check (the fidelity guard cannot catch
   it) — this is the architect's explicit YELLOW→GREEN condition.
2. The route-around renders general non-NUMS faithfully — md-codec `is_nums:false → lookup_key`
   (`to_miniscript.rs:161-164`) + the `ReconstructTranslator` handling the real trunk key as an XPub
   (not tripping the NUMS-only `Single`-guard at `restore.rs:963-970`).
3. Address derivation works for a key-path-bearing tr (`derive_address::derive_receive_addresses` on
   the non-NUMS descriptor string).
4. `--format green`/`bip388`/other formats classify a non-NUMS general-tr sanely (no wrong-payload).

**Non-goals:** the `@-in-both` shape (deferred, §4); depth-≥2 taptrees (upstream-blocked); any
md-codec wire change; from-seed `bundle` emitting non-NUMS (it intentionally emits NUMS).
