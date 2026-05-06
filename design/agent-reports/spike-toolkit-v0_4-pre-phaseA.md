# Phase 2 SPIKE — toolkit v0.4 pre-Phase-A

**Date:** 2026-05-05
**Goal:** lock the two API surfaces that Phases A-G will consume, before Phase A starts. Per `feedback_spike_before_locking_wire_format`.
**Throwaway crate:** `.spike-v0.4/` (sibling to `.spike-v0.3/`). Two binaries: `spike1_taptree`, `spike2_slot_parser`.
**Locked deps:** `miniscript = "13.0.0"` + `[patch.crates-io] miniscript = git@95fdd1c5` (post-#910 + #915), `bitcoin = "0.32"`, `md-codec = git tag md-codec-v0.16.1`, `clap = "4"`.
**Status:** SPIKE complete. Both probes empirically PASS. Architect-review-pending.

---

## SPIKE-1 — multi-leaf TapTree round-trip via md-codec `Tag::TapTree`

**Probe matrix (all PASS, byte-equal topology after md-codec encode→decode):**

| Probe | Source descriptor (abbreviated) | Tap-tree depths (DFS) | Expected md-codec topology |
|---|---|---|---|
| 1-leaf control | `tr(K, pk(@1))` | `[0]` | leaf node (no `Tag::TapTree` wrapper) |
| 2-leaf | `tr(K, {pk(@1), pk(@2)})` | `[1, 1]` | `TapTree[leaf, leaf]` |
| 3-leaf asymmetric | `tr(K, {pk(@1), {pk(@2), pk(@3)}})` | `[1, 2, 2]` | `TapTree[leaf, TapTree[leaf, leaf]]` |
| 4-leaf balanced | `tr(K, {{pk(@1),pk(@2)},{pk(@3),pk(@4)}})` | `[2, 2, 2, 2]` | `TapTree[TapTree[l,l], TapTree[l,l]]` |
| 5-leaf left-heavy | `tr(K, {{{pk(@1),pk(@2)},pk(@3)},{pk(@4),pk(@5)}})` | `[3, 3, 2, 2, 2]` | `TapTree[TapTree[TapTree[l,l], l], TapTree[l,l]]` |
| 4-leaf right-spine (added r1) | `tr(K, {pk(@1), {pk(@2), {pk(@3), pk(@4)}}})` | `[1, 2, 3, 3]` | `TapTree[l, TapTree[l, TapTree[l, l]]]` |

For each probe the SPIKE: (a) builds the miniscript `Tr` via `TapTree::leaf` + `TapTree::combine`; (b) walks via the candidate `walk_tap_tree`; (c) builds a `md_codec::Descriptor` (n=key-count, BIP-86-shaped origin path, default UseSitePath, empty TLV) wrapping `Body::Tr { key_index: 0, tree: Some(walked) }`; (d) calls `md_codec::encode_md1_string(&md)` → md1 codex32 string; (e) decodes via `md_codec::decode_md1_string(&s)`; (f) extracts the decoded `Body::Tr.tree` and asserts byte-equal `Node` PartialEq against the walked tree.

### SPIKE-1 locked API surface (Phase F.1 reads this verbatim)

```rust
/// Walk a miniscript `TapTree<DescriptorPublicKey>` into a md-codec
/// `tree::Node`. Single-leaf descends directly to the leaf miniscript node
/// (no `Tag::TapTree` wrapper at root, matching v0.3 `walk_tap_tree_singleleaf`
/// behavior). Multi-leaf folds miniscript's flat DFS-preorder
/// `(depth, miniscript)` list into a binary tree of `Tag::TapTree` branches
/// using a depth-stack algorithm.
fn walk_tap_tree(
    tt: &miniscript::descriptor::TapTree<miniscript::descriptor::DescriptorPublicKey>,
    km: &BTreeMap<String, u8>,
) -> Result<md_codec::tree::Node, ToolkitError>
```

**Algorithm (verbatim — SPIKE-validated, Phase F.1 transcribes):**

```text
1. leaves = tt.leaves().map(|li| (li.depth(), li.miniscript())).collect()
2. if leaves.is_empty(): error (per SPEC §4.9.a invariant comment, this is
   unreachable from miniscript; SPIKE keeps as defensive Err for safety).
3. if leaves.len() == 1:
     assert leaves[0].depth == 0
     return walk_miniscript_node(leaves[0].miniscript, km, tap=true)
4. stack: Vec<(u8 depth, Node)> = []
   for (depth, ms) in leaves:
     stack.push((depth, walk_miniscript_node(ms, km, tap=true)))
     while stack.len() >= 2 && stack[-1].depth == stack[-2].depth && stack[-1].depth > 0:
       (d, right) = stack.pop()
       (_, left) = stack.pop()
       stack.push((d - 1, Node { tag: TapTree, body: Children([left, right]) }))
   if stack.len() != 1 || stack[0].depth != 0: error (malformed)
   return stack.pop().1
```

**Md-codec wire shape (confirmed):** `Tag::TapTree` is encoded by `tree.rs::write_node` as a 2-child node via `Body::Children(vec![l, r])`, decoded as the same shape (`tree.rs:142-146`). Validation enforces leaf-tag whitelist via `validate.rs::validate_tap_script_tree` recursively walking `Tag::TapTree` interior nodes (`validate.rs:120-138`). The walker output threads cleanly through both encode and decode.

**Md-codec checksum / chunk-set:** the SPIKE uses `encode_md1_string` (single-string codex32) for round-trip *as a topology-verification convenience only* — chunking via `md_codec::chunk::split` is orthogonal to the topology question and is the actual code path Phase A/D uses for bundle output. The SPIKE confirms `encode_md1_string` accepts the new multi-leaf shapes without overflow for the small probes (largest: 5-leaf, 46 chars). The relevant capacity bound is `chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT = 64 × 5 = 320 bits`, corresponding to the codex32 *regular*-form 80-char data-part limit (3 HRP + 1 separator + 64 data + 13 checksum); long-form codex32 was dropped in md-codec v0.12.0. Multi-leaf trees that exceed 320 payload bits go through `chunk::split` automatically — they are not single-string-encodable. Phase F.3 round-trip tests should cite `SINGLE_STRING_PAYLOAD_BIT_LIMIT`, not any "long-bracket" reference.

**Single-leaf preserved:** the walker's branch for `leaves.len() == 1` matches v0.3's `walk_tap_tree_singleleaf` exit behavior (no `Tag::TapTree` wrapper, leaf miniscript directly under `Body::Tr.tree`). The 1-leaf control probe round-trips byte-identically. Phase F.2 deletes `walk_tap_tree_singleleaf` after the new walker subsumes it.

**Empty-leaves invariant:** miniscript's `TapTree` constructors (`leaf` + `combine`) cannot produce an empty `depths_leaves` vector (`leaf` requires a Miniscript; `combine` chains two non-empty trees). Per SPEC §4.9.a the walker carries a one-line comment citing BIP-341 + the miniscript invariant rather than a defensive guard. The SPIKE walker's `is_empty()` Err arm is a SPIKE-only safety net; Phase F.1 may either keep it (defense in depth) or replace with a one-line invariant comment per SPEC §4.9.a — Phase F's discretion. **Decision deferred to Phase F.1 review** (impl plan §4.9.a says "no 0-leaf guard"; SPIKE takes that at face value).

---

## SPIKE-2 — clap `--slot @N.<subkey>=<value>` value-parser + removed-subcommand trap

### Locked SlotInput / SlotSubkey types (Phase B.1 transcribes verbatim)

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotSubkey {
    Phrase, Entropy, Xpub, Fingerprint, Path, Wif, Xprv,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotInput {
    pub index: u8,
    pub subkey: SlotSubkey,
    pub value: String,
}
```

**Note on `SlotValue` typing:** the impl plan B.1 sketches a stronger `SlotValue` enum with per-subkey type validation. The SPIKE locks the *string-typed* `value: String` shape at the value-parser boundary; per-subkey type validation (BIP-39 phrase parse, hex-entropy parse, xpub parse, etc.) belongs in Phase B's `validate_slot_set` + downstream `bind_slot_to_key` logic, not in the clap value-parser. Reasoning: the value-parser fires once per `--slot` flag and should NOT pull in BIP-39 wordlist parsing or secp256k1 validation; those live in Phase A's existing parsers. The B.1 impl-plan `SlotValue` type can still exist as a *post-validation* enum (constructed in `validate_slot_set`); the clap parser hands off `String` and `validate_slot_set` upgrades to typed `SlotValue`.

### Locked parse_slot_input signature (Phase B.2 transcribes verbatim)

```rust
pub fn parse_slot_input(s: &str) -> Result<SlotInput, ParseError>
```

`ParseError` is a `String`-wrapping error type implementing `std::error::Error + Display`. Clap accepts any `Fn(&str) -> Result<T, E>` where `E: Into<Box<dyn Error + Send + Sync + 'static>>`; SPIKE's `ParseError(String)` satisfies this.

**Grammar:** `@<u8>.<subkey>=<value>`.
**Subkey vocabulary:** `phrase | entropy | xpub | fingerprint | path | wif | xprv` (closed set; unknown → ParseError).
**Empty-value decision (LOCKED):** REJECTED at parse time. Reasoning: every supported subkey has a non-empty syntactic shape (BIP-39 phrase ≥ 1 word; hex-entropy ≥ 1 byte; xpub/xprv have HRP+payload; wif has alphabet+checksum; fingerprint is 8 hex; path may be `m` only — but the user MUST type at least `m`). Rejecting empty at the parser surfaces a clean clap error with the failing arg highlighted, rather than letting empty values flow into per-subkey type validators where the error message would be subkey-specific and harder to triage.

### Probe matrix (all PASS)

| Probe | Input | Outcome |
|---|---|---|
| happy: phrase | `@0.phrase=abandon×11 about` | Ok |
| happy: entropy | `@1.entropy=0102…0f10` | Ok |
| happy: xpub | `@2.xpub=xpub6BgB…` | Ok |
| happy: fingerprint | `@0.fingerprint=deadbeef` | Ok |
| happy: path | `@0.path=48'/0'/0'/2'` | Ok |
| happy: wif | `@0.wif=KwDiBf…` | Ok |
| happy: xprv | `@0.xprv=xprv9s21…` | Ok |
| index 255 | `@255.xpub=xpub-stub` | Ok |
| no @-prefix | `0.phrase=abc` | Err `"slot input must start with '@N.<subkey>=<value>'"` |
| missing index | `@.phrase=abc` | Err `"missing index after '@'"` |
| non-numeric index | `@xx.phrase=abc` | Err `"index must be a u8 (0..=255)"` |
| index overflow | `@256.xpub=xpub-stub` | Err `"index must be a u8 (0..=255)"` |
| missing dot | `@0phrase=abc` | Err `"missing '.<subkey>=' after '@N'"` |
| missing equals | `@0.phrase` | Err `"missing '=' between subkey and value"` |
| unknown subkey | `@0.unknown=abc` | Err `"unknown slot subkey \"unknown\"; expected one of: phrase, entropy, xpub, fingerprint, path, wif, xprv"` |
| empty subkey | `@0.=abc` | Err `"missing subkey between '.' and '='"` |
| **empty value (locked: REJECT)** | `@0.phrase=` | Err `"value is empty for subkey \"phrase\"; supply a non-empty value"` |

### Clap wiring (confirmed)

```rust
use clap::{Arg, ArgAction, Command};

Command::new("bundle")
    .arg(Arg::new("slot")
        .long("slot")
        .action(ArgAction::Append)
        .value_parser(parse_slot_input))
    .arg(Arg::new("template").long("template"))
    .arg(Arg::new("descriptor").long("descriptor"))
    .arg(Arg::new("threshold").long("threshold"))
```

Reading the matches: `bundle_m.get_many::<SlotInput>("slot").unwrap().collect::<Vec<&SlotInput>>()`. Confirmed: clap routes parser errors through `error: invalid value '@0.unknown=foo' for '--slot <slot>': <ParseError display>`. Phase B.3 wires this verbatim.

### Locked removed-subcommand trap mechanism (Phase C.1 transcribes verbatim)

**Mechanism:** **pre-clap argv inspection**.

```rust
const REMOVED_SUBCOMMAND_ERR: &str = "error: 'bundle multisig-full' / 'bundle multisig-watch-only' subcommands removed in v0.4. Use 'bundle' (mode auto-detected from --slot @N.<subkey>=<value> inputs).";

fn detect_removed_subcommand(argv: &[String]) -> Option<&'static str> {
    let mut iter = argv.iter().enumerate();
    while let Some((i, t)) = iter.next() {
        if t == "bundle" {
            if let Some(next) = argv.get(i + 1) {
                if next == "multisig-full" || next == "multisig-watch-only" {
                    return Some(REMOVED_SUBCOMMAND_ERR);
                }
            }
            break;
        }
    }
    None
}
```

`main.rs` calls `detect_removed_subcommand(&std::env::args().collect::<Vec<_>>())` BEFORE clap parses; if `Some(msg)`, prints `msg` to stderr and `std::process::exit(2)`.

**Rationale (alternatives considered + rejected):**

- **Custom positional handler under `bundle` subcommand:** clap's `Command::allow_external_subcommands(true)` would route the `multisig-full` token into a captured "external subcommand" — but capturing it requires reshaping the bundle command's matches structure, and we'd still need to special-case the byte-exact error text. Pre-clap inspection is simpler and bypasses clap's argument-validation entirely (no risk of a "missing required --template" error firing first if the user passed `bundle multisig-full` with no template).
- **`subcommand_negates_reqs` pattern:** designed for "this subcommand replaces all required parent args" — wrong shape; we want to REJECT, not REPLACE.
- **Adding `multisig-full` / `multisig-watch-only` as deprecated sub-subcommands of `bundle`:** would let the trap live inside clap, but adds AST baggage to the help output (`mnemonic bundle help` would list the removed subcommands as if they exist) and the error firing time is inconsistent (clap prints help-style errors, not the byte-exact §6.6 row 1 text). Pre-clap is cleaner.

**Bundle-scoped:** the trap fires ONLY when `bundle` is the immediate parent of `multisig-full` / `multisig-watch-only`. The SPIKE confirms `mnemonic verify-bundle multisig-full ...` does NOT fire the trap (verify-bundle has no such removed sub-subcommands; the user's intent there is unrelated).

**For comparison — clap's default behavior on `bundle multisig-full` (sans trap):**
```
error: unexpected argument 'multisig-full' found
```
…which would not match SPEC §6.6 row 1 byte-exact. Confirms the trap is required.

### Removed-subcommand trap probe matrix (all PASS)

| Argv | Trap fires? | Stderr |
|---|---|---|
| `mnemonic bundle multisig-full --phrase abc` | YES | byte-exact §6.6 row 1 |
| `mnemonic bundle multisig-watch-only` | YES | byte-exact §6.6 row 1 |
| `mnemonic bundle --template wpkh` | NO | — (clap proceeds normally) |
| `mnemonic verify-bundle multisig-full` | NO (out of scope) | — (verify-bundle handles it via its own argv) |

---

## Cross-SPIKE conclusions

1. **Both API surfaces locked.** Phase F.1 consumes the SPIKE-1 `walk_tap_tree` signature + algorithm; Phase B.1-B.3 + C.1 consume SPIKE-2's `SlotInput` / `SlotSubkey` / `parse_slot_input` / removed-subcommand trap.
2. **No SPEC drift.** SPIKE confirms the SPEC's §4.9.a (multi-leaf supported), §6.6 row 1 (removed-subcommand error text), §6.6.b (subkey vocabulary), §6.6 row 4 conflict semantics, and §5.7 verify-bundle assumptions are all consistent with what md-codec + clap actually deliver. No SPEC re-review required.
3. **Empty-value decision recorded** (LOCKED to REJECT at parse time). SPEC §6.6.b should be amended only if Phase B reviewers push back; SPIKE's recommendation is to LOCK as REJECT and add a sentence to §6.6.b: "Empty value (`@N.subkey=`) is rejected at the value-parser; users must supply non-empty values for every subkey."
4. **Walker empty-leaves arm decision deferred** to Phase F.1 review (SPIKE keeps a defensive `Err`; SPEC §4.9.a says "no defensive guard"; resolved at Phase F.1 review).
5. **L-tier FOLLOWUPS** (from r1 architect review):
   - L-2: `bundle multisig-full=value` token (positional with `=`) does not fire trap. Theoretical edge case (positional args don't take `=value` form in standard shells); routed to `design/FOLLOWUPS.md` at `v0.4-nice-to-have`.
   - L-3: `mnemonic bundle -- multisig-full` (post-`--` separator) bypasses trap; clap emits generic "unexpected argument" instead of byte-exact §6.6 row 1. Routed to `design/FOLLOWUPS.md` at `v0.4-nice-to-have`.

## r1 architect review changelog

- I-1: corrected the "127-char long-bracket max" claim to cite `chunk::SINGLE_STRING_PAYLOAD_BIT_LIMIT = 320 bits` (codex32 regular-form 80-char limit; long-form retired in md-codec v0.12.0). Phase F.3 doc-comments must use the correct constant.
- L-1: added the right-spine probe `{pk(@1), {pk(@2), {pk(@3), pk(@4)}}}` (depths [1,2,3,3]) to the SPIKE binary; topology round-trips byte-equal. Probe matrix above updated.
- L-2 + L-3: routed to `design/FOLLOWUPS.md` (v0.4-nice-to-have tier).
- L-4 (deliverable cross-reference imprecision): SPEC §4.9.a is the authoritative source; the Phase F.1 transcription cites SPEC, not the impl plan.

## SPIKE artifacts

- Throwaway crate: `.spike-v0.4/` (gitignored; not staged).
- Run output captured in this report's probe matrices (live runs, not stale).
- Per impl plan, SPIKE crate is not staged into the repo; this report is the durable record.

## Phase A green-light

SPIKE objectives met (both items conclusive, both API surfaces locked). Phase A may begin once architect review of this report returns 0C/0I.
