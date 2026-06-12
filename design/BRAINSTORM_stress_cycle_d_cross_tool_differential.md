# BRAINSTORM — stress Cycle D: cross-tool md1 differential (toolkit vs md-cli)

Status: R2 **GREEN (0C/0I)** — cleared for implementation. 2026-06-12.
Reviews: cycle-d-differential-r0-round1-review.md (YELLOW 0C/4I — folded
`[I1]`–`[I4]` + M1–M5) and …-round2-review.md (GREEN 0C/0I; 7 impl-detail
minors m1–m7, folded below). Program: Cycle D of the 6-cycle stress program
(A/B/C shipped). Repos: mnemonic-toolkit @ master, descriptor-mnemonic @
main (md-cli).

## IMPLEMENTATION NOTES (round-2 minors — load-bearing mechanics)

- **m1** `md inspect --json` keys are `wallet_policy_id.hex` and
  `wallet_descriptor_template_id.hex` (snake_case, nested under `.hex`) —
  NOT the hyphenated text-form labels.
- **m2** Pass each md1 chunk as a SEPARATE positional arg to `md inspect`
  (spread the toolkit's `.md1` array into argv); space-joining → "character
  '1' not in codex32 alphabet".
- **m3** md-cli `encode --json` → `.phrase` (single string) for the corpus,
  but `{chunk_set_id, chunks:[...]}` for large policies — handle both.
- **m4** `tr(NUMS, multi_a(...))` on md-cli needs the explicit-internal-key
  form `tr(<NUMS_H_point>, multi_a(...))` (or `--unspendable-key`).
- **m5** Corpus xpubs are FROZEN LITERALS, depth-matched (depth-3 wpkh/
  single-sig, depth-4 wsh/multisig — md-cli enforces depth). `mnemonic
  convert` cannot derive depth-4; ship abandon-mnemonic literals. Proven
  depth-4 key: `xpub6DkFAXWQ2dHxq…KFrf` mfp `73c5da0a`.
- **m6** The toolkit `tap_context` gate is at parse_descriptor.rs:**604**
  (not :603); deliberate test `walk_check_kept_in_non_tap_context`:2568.
- **m7** File the new FOLLOWUP `toolkit-check-pkk-non-tap-non-canonical` and
  cross-link descriptor-mnemonic FOLLOWUPS.md:562 (the existing related
  walker-Check note) to retarget the fix toolkit-side.

## Problem / charter

Two tools turn a descriptor STRING into an md1 card via near-identical
hand-written walkers, but they are NOT cross-checked, so they can silently
disagree — engraving DIFFERENT cards for the SAME wallet (an interop /
funds-safety hazard: a card made by `mnemonic` won't match one made by `md`
for the same descriptor, and a third party reconstructing may get a
different wallet_policy_id).

Recon (2026-06-12) CONFIRMED a real divergence:
- **toolkit** `parse_descriptor.rs:603` (`walk_miniscript_node(..,
  tap_context)`) GATES the `Terminal::Check(PkK|PkH) → bare Tag::PkK|PkH`
  collapse on `tap_context` — collapses ONLY inside tap leaves
  (parse_descriptor.rs:521 passes `tap=true`); in wsh/sh (`:431`/`:455`,
  `tap=false`) it emits `Tag::Check(Tag::PkK)`.
- **md-cli** `crates/md-cli/src/parse/template.rs:603` collapses
  `Check(PkK|PkH) → bare` UNCONDITIONALLY (its `walk_miniscript_node` has
  no `tap_context` param).
- ⇒ `wsh(pk(@0))`: toolkit → `wsh(Check(PkK))`, md-cli → `wsh(PkK)`.
  Different md1 wire bytes AND different `compute_wallet_policy_id`
  (different preimage). Same logical descriptor, two cards.

Note both md1 forms DECODE to the same miniscript (md-codec 0.35.1's
to_miniscript Check-idempotence arm accepts `Check(PkK)` and renders
`c:pk_k`, same as bare `PkK`) — so this is a WIRE-canonicity divergence,
not a decode error. But the cards/IDs differ, which is the hazard. R0
proved it end-to-end: `wsh(pk(@0/<0;1>/*))` @ a fully-origin-matched key →
toolkit wallet-policy-id `9ad78e4f…` vs md-cli `58d18033…` (template-ids
`ef980fcc` vs `9208f590`); both `md decode` → identical
`wsh(pk(@0/<0;1>/*))`.

**CANONICITY DIRECTION [I1] (round-1 correction — load-bearing):** md-codec
defines NO canonical wrapper for `c:pk_k` in wsh (`canonical_origin` treats
`wsh(pk)` as non-canonical in EITHER form; encode + `compute_wallet_policy_id`
faithfully hash whatever tree the walker emits, NOT normalizing
`Check(PkK)`↔bare`PkK`). The authority is **descriptor-mnemonic SPEC v0.30
§5.1**: "Walker emits bare `Tag::PkK`/`Tag::PkH`… regardless of context;
`Tag::Check` is never emitted wrapping a key leaf on the wire." So **md-cli
is CONFORMANT (bare PkK) and the TOOLKIT is the deviant** — its
`tap_context` gate keeps `Check(PkK)` in wsh/sh, a DELIBERATE + tested
choice (`walk_check_kept_in_non_tap_context`, parse_descriptor.rs:2568). The
FIX (deferred) is to make the TOOLKIT adopt SPEC §5.1 bare-key in non-tap
(drop the gate at parse_descriptor.rs:603, invert that test) — after a
fix-cycle R0 confirms the toolkit has no countervailing SPEC reason. NOT
"fix md-cli", NOT "define canonicity" (already defined).

## Goal

A differential harness that, for a corpus of descriptors, runs BOTH tools
and compares their md1 output (and/or wallet_policy_id), surfacing this
divergence and gating against new ones. Per the program's find→file
discipline: the harness is the deliverable (test-only, NO-BUMP); the
CANONICITY FIX (which collapse is correct — make the two tools agree) is a
separate follow-on filed as a FOLLOWUP.

## Constraints (recon-verified)

- **Both parse fns are bin-private** (toolkit `parse_descriptor` declared in
  main.rs not lib.rs; md-cli `parse_template` in the md-cli bin crate) → NO
  in-process call. md-codec has no miniscript→Descriptor parser (the
  walkers ARE the CLI-specific layer). ⇒ the differential must **shell out**
  to the two compiled binaries.
- The two tools link md-codec **0.35.x** (toolkit pins crates.io `"0.35"`;
  md-cli is in the descriptor-mnemonic workspace at path `0.35.1`). The
  resolved crates.io 0.35.x and the path 0.35.1 may differ by patch — R0 to
  confirm whether a version skew is itself a divergence source (and pin the
  comparison to the same md-codec or note the skew).
- Invocation surfaces: toolkit emits md1 via `bundle --descriptor` /
  `export-wallet --descriptor` (R0 to pick the leanest that yields a raw
  md1 for an arbitrary descriptor + concrete keys); md-cli via
  `md encode --template <t> --keys <k>` (NB: md-cli takes a TEMPLATE
  `@N`-form + keys, the toolkit takes a concrete descriptor — the harness
  must feed each tool its expected input form for the SAME logical wallet).
- No existing cross-tool parity test (`parity_smoke.rs` is BCH-only).

## Proposed design (round-1 folded)

### Home + shape [M2]

(a) an `#[ignore]`-by-default Rust integration test in the **toolkit repo**,
gated on `MNEMONIC_BIN` + `MD_BIN` env vars (a Rust test shells out cleanly,
asserts the Expect table, reuses the `MD_BIN` convention). CI: `mnemonic`
from the toolkit build; `md` from the EXISTING manual-lint install step
(`cargo install --git … --tag descriptor-mnemonic-md-cli-v0.6.2 md-cli
--features cli-compiler`, manual.yml:86) — reuse it, PIN the tag in the diff
job so a future md-cli tag can't silently move the walker. A dedicated
`cross-tool-differential.yml` job invokes the `#[ignore]`d test with both
env vars set.

### Invocation surface [M3] (R0-proven)

- **Toolkit md1:** `mnemonic bundle --descriptor "<concrete [fp/path]xpub
  desc>" --network mainnet --json` → stdout JSON `.md1` (a CHUNK ARRAY; the
  secret-on-argv advisory goes to STDERR — capture stdout only).
  `export-wallet` has no md1 format; md1 is `bundle`-only.
- **md-cli md1:** `md encode "<@N template>" --key "@0=<bare depth-N xpub>"
  --fingerprint "@0=<fp>" --path "<m/…>" --json` → stdout JSON `.phrase`
  (a SINGLE-STRING md1).
- For the oracle, feed each md1 to `md inspect --json` and read
  `wallet-policy-id` / `wallet-descriptor-template-id`.

### Input-form pairing [I2] (load-bearing — get this exact or expected-MATCH
rows spuriously diverge)

The toolkit consumes ONE concrete descriptor with a mandatory `[fp/path]xpub`
bracket. md-cli consumes a `@N` TEMPLATE + THREE separate inputs:
`--key @i=<BARE xpub, NO bracket>` (a bracket → base58 decode error),
`--fingerprint @i=<HEX>`, `--path <m/…>`. **The xpub depth must match the
context** (md-cli rejects depth≠4 for wsh/MultiSig, depth≠3 for wpkh/
SingleSig). Per corpus entry the harness stores the matched pair: the
toolkit's `[fp/path]xpub` concrete descriptor AND md-cli's
(template, bare-depth-N-xpub, fingerprint, path) that reconstruct the SAME
origin. (R0 proof: wpkh with md-cli `--fingerprint` but no `--path` →
policy-id `e25d7948` ≠ toolkit `1c0170fe`; adding `--path m/84'/0'/0'` →
`1c0170fe` == `1c0170fe`.) Use depth-matched abandon-mnemonic account xpubs.

### Corpus [M4] (curated)

Generated is impractical (each entry needs the paired forms + depth-matched
xpubs per context). Curated, with EXPECTED-MATCH CONTROLS for anti-vacuity:
- **Expect::Match controls:** `wpkh`, `pkh`, `tr(<key>)` keypath,
  `wsh(multi(2,@0,@1))`, `wsh(sortedmulti(...))`, `sh(wsh(sortedmulti))`,
  `tr(NUMS, multi_a(...))`, and `tr(<key>, pk(@1)-leaf)` (the toolkit
  collapses Check in TAP context too → matches). These prove the harness
  isn't vacuously green/red.
- **Expect::Diverge (the known finding):** `wsh(pk(@0))`, `wsh(pkh(@0))`,
  and bare-key-in-combinator shapes `wsh(and_v(v:pk(@0),pk(@1)))`,
  `wsh(or_d(pk(@0),pk(@1)))` (each contains a `Check(PkK)` the toolkit keeps
  in non-tap) — pinned with a comment citing the FOLLOWUP.

### Oracle [I3][I4]

Run both tools per entry, then:
- **Gate first [I3]:** assign a verdict ONLY when BOTH tools exit 0 AND emit
  a parseable md1. Otherwise the entry's actual state is
  `BothError` / `ToolError(which)` — NEVER silently `Match`. The Expect
  enum is **`Match | Diverge | BothError | ToolError`** (four arms).
- **O2 (PRIMARY) [I4] — wallet-policy-id equality:** decode both md1
  (`md inspect` → `wallet-policy-id`) and compare. Robust to the chunking
  difference (toolkit 3-chunk vs md-cli single-string) that makes raw md1
  strings always differ.
- **O2' — wallet-descriptor-template-id equality:** the key-independent tree
  hash — the cleanest WALKER-ISOLATING signal (strips key/origin confounds).
  Compare this too; a template-id divergence is unambiguously a walker
  difference.
- **O1 (secondary/informational) — md1 string:** only after normalizing both
  to the unchunked single-string form (re-encode); NOT the primary (raw
  chunked vs single-string always differ).
- **Expect table (anti-vacuity):** each corpus entry declares its expected
  arm; the test fails if an entry's ACTUAL arm differs from EXPECTED (a
  known-Diverge starting to Match = the fix landed → flip to Match; a
  known-Match diverging = a regression; any entry hitting BothError/ToolError
  unexpectedly = an invocation/corpus bug). Mirrors Cycle-C held-out +
  the GUI canonicity-table pattern.

### Version skew [M1] (controlled, wire-neutral)

Toolkit + descriptor-mnemonic both resolve md-codec 0.35.1 (published src
byte-identical to path). The CI manual-lint `md` (tag v0.6.2) links md-codec
0.35.0, but the ONLY 0.35.0→0.35.1 src delta is `to_miniscript.rs` (+17, the
Check-double-wrap RENDERER tolerance — md1→miniscript direction), which is
WIRE-NEUTRAL (does not affect md1 bytes or wallet-policy-id). So the skew
does NOT confound the walker-divergence signal. Document this; pin the `md`
tag.

### FOLLOWUP filed this cycle [I1]

`toolkit-check-pkk-non-tap-non-canonical` (both repos): the TOOLKIT keeps
`Tag::Check(Tag::PkK/PkH)` in wsh/sh (gated on `tap_context` at
parse_descriptor.rs:603) whereas descriptor-mnemonic SPEC v0.30 §5.1
mandates bare `PkK`/`PkH` "regardless of context", so the toolkit emits a
NON-CONFORMANT md1 for `wsh(pk)`-shaped descriptors — a different
wallet-policy-id than md-cli for the same wallet (interop hazard;
wire-canonicity, not funds-loss — both decode to the same descriptor).
Records: the two sites (toolkit parse_descriptor.rs:603 gate + the
deliberate test `walk_check_kept_in_non_tap_context`:2568; md-cli
template.rs:603 conformant), the proven diverging example
(`wsh(pk)` → policy-ids `9ad78e4f` vs `58d18033`), the SPEC §5.1 citation,
and the FIX direction (toolkit drops the gate → bare key in non-tap; invert
the test). Fix is R0-gated, separate cycle.

## Resolved decisions (round-1 R0 answers, adopted)

1. Home/shape: (a) `#[ignore]`-gated Rust test on `MNEMONIC_BIN`+`MD_BIN`;
   CI `md` from the manual-lint tag install (reuse, pin the tag). [M2]
2. Invocation surface proven: toolkit `bundle --descriptor … --json`→`.md1`;
   md-cli `md encode <@N> --key --fingerprint --path --json`→`.phrase`;
   oracle via `md inspect`. [M3] Input-form pairing per entry with
   depth-matched xpubs + the md-cli `--fingerprint`/`--path` triple. [I2]
3. md-codec skew controlled + wire-neutral (to_miniscript-only delta);
   pin the `md` tag. [M1]
4. Canonicity DEFINED (SPEC §5.1 = bare PkK); md-cli conformant, toolkit
   deviant; fix = toolkit, deferred. [I1]
5. Curated corpus + expected-MATCH controls (wpkh/tr-pk-leaf/multi). [M4]
6. Oracle: primary = wallet-policy-id (O2) + template-id (O2'); four-arm
   Expect enum (Match|Diverge|BothError|ToolError); both-exit-0-and-parseable
   gate. [I3][I4]
7. Scope: surface + pin + file, NO-BUMP; canonicity fix deferred to its own
   R0-gated cycle. [M5]
