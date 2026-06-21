# BRAINSTORM — cycle-11b: toolkit secret-hygiene / robustness cluster (L21 · L24 · L25)

**DESIGN ONLY — no code.** This brainstorm spec is decision-complete and R0-ready; it feeds the
mandatory opus-architect **R0 loop to 0C/0I** before any implementation begins (CLAUDE.md hard gate).

**Scope:** three toolkit-only findings from the constellation bug-hunt — **L21** (SECRET footgun:
silent empty-passphrase BIP-38 encrypt), **L24** (CLI-reachable OOB panic in `verify-bundle`
descriptor mode), **L25** (cosmetic x-only classifier mislabel in `import-wallet`). All three are
small / PATCH-level. The cycle ships toolkit **PATCH 0.65.1**, toolkit-only, **no registry publish**.

---

## 0. Source-SHA table (citations re-grepped against current `origin/master`)

| Item | Repo | `origin/master` SHA | Toolkit version | Citation status |
|---|---|---|---|---|
| all three | `mnemonic-toolkit` | **`4e8ad7923d03aea5569d4d73f22b6e99371037d8`** | `0.65.0` (`crates/mnemonic-toolkit/Cargo.toml`) | bug-hunt line numbers DRIFTED; **re-grepped live below** |

**Inputs folded:**
- Cycle-prep recon: `cycle-prep-recon-cycle11b-toolkit-hygiene.md` (authoritative; all 3 STILL-REPRODUCE).
- Bug-hunt report: `design/agent-reports/constellation-bughunt-2026-06-20.md` — L21 (§794-804), L24
  (§905-916), L25 (§918-926).
- Program plan: `design/PLAN_constellation_bughunt_fix_program.md` — L21 PATCH, L24 PATCH, L25 PATCH.

**Live citations verified at `4e8ad792`** (bug-hunt's snapshot line numbers in parentheses):

| Item | File | Live line(s) | What's there | (bug-hunt cited) |
|---|---|---|---|---|
| L21 outer match arm | `src/cmd/convert.rs` | `1231` | `Seedqr \| Phrase \| Entropy => {` — the composite BIP-39-source arm; the inner `Bip38 =>` sub-arm is reached for **all three** of `from ∈ {Seedqr, Phrase, Entropy}` | (—) |
| L21 guard | `src/cmd/convert.rs` | `932` | `if bip38_edge && effective_passphrase.is_none() && effective_bip38_passphrase.is_none()` → `refusal_bip38_no_passphrase()` | (`:932`) |
| L21 empty fallback | `src/cmd/convert.rs` | `1376` | `let scrypt_pp = bip38_passphrase.unwrap_or("");` — composite `(Seedqr\|Phrase\|Entropy, Bip38)` arm (`Bip38 =>` sub-block at `1350-1379`, inside the `:1231` outer arm) | (`:1366`) |
| L21 `(Seedqr, Bip38)` permitted | `src/cmd/convert.rs` | `637`, `77` | `\| (Seedqr, Bip38)` whitelisted in `classify_edge` (`:637`); `"seedqr" => Self::Seedqr` argv parse (`:77`) — so `convert --from seedqr=… --to bip38` is argv-reachable and hits the same `:1376` empty-encrypt | (—) |
| L21 direct edges | `src/cmd/convert.rs` | `1523`, `1543` | `let scrypt_pp = bip38_passphrase.unwrap_or(pbkdf2_passphrase);` — direct `(wif↔bip38)`, fall back to `--passphrase` | (`:1502,1522`) |
| L21 warning-suppress | `src/cmd/convert.rs` | `954` | `let edge_uses_passphrase = edge_uses_pbkdf2 \|\| bip38_edge;` — suppresses the "ignored" warning on BIP-38 edges | — |
| L21 refusal helper | `src/cmd/convert.rs` | `508-513` | `fn refusal_bip38_no_passphrase() -> ToolkitError { ToolkitError::ConvertRefusal(...) }` | — |
| L24 `let n` | `src/cmd/verify_bundle.rs` | `1349` | `let n = descriptor_resolved.n as usize;` | (`:1349`) |
| L24 `validate_slot_set` call | `src/cmd/verify_bundle.rs` | `1351` | `crate::slot_input::validate_slot_set(&args.slot)?;` (contiguity only, NOT range vs n) | — |
| L24 `is_non_canonical` block | `src/cmd/verify_bundle.rs` | `1371` | `if is_non_canonical {` — entry to the path-override region | — |
| L24 `new_paths` built len n | `src/cmd/verify_bundle.rs` | `1384-1404` | every arm yields `(0..n).map(...)` / maps over `Divergent(v)` (len n) | (`:1374-1393`) |
| L24 **OOB write, no guard** | `src/cmd/verify_bundle.rs` | `1435` | `new_paths[*idx as usize] = crate::cmd::bundle::derivation_path_to_origin(&user_path);` | (`:1425`) |
| L24 range-checked loop AFTER | `src/cmd/verify_bundle.rs` | `1466` | `for idx in 0..(n as u8) {` | (`:1456`) |
| L24 reference gate (the mirror omits) | `src/cmd/bundle.rs` | `1373-1388` | `if slots.iter().map(\|s\| s.index as usize + 1).max().unwrap_or(0) != n { return Err(ToolkitError::DescriptorParse(...)); }` | (`:1373-1388`) |
| L25 `has_any_key_token` | `src/wallet_import/pipeline.rs` | `53-61` (regex `:56`) | `r"[xtyzuvYZUV]pub[A-HJ-NP-Za-km-z1-9]+\|\b0[23][0-9a-fA-F]{64}\b"` — matches xpub-family + `02/03`-prefixed 66-hex, **NOT bare 64-hex x-only** | (`:53-60`) |
| L25 `(false,false)` arm | `src/wallet_import/pipeline.rs` | `185-208` | `if has_any_key_token(input) { Err("...must carry a key origin...") } else { Err("...no keys to engrave...keyless script...") }` — both arms `Err` | (`:160-180`) |

**Error variants confirmed present** (`src/error.rs`, no new variant needed): `ConvertRefusal(String)`
(`:89`, exit_code `2` `:562`), `DescriptorParse(String)` (`:123`, exit_code `2` `:569`). Both already
alphabetically placed; CLAUDE.md's alphabetical-insertion discipline does **not** bite this cycle.

---

## 1. Finding summary (all REPRODUCE at `4e8ad792`)

| Finding | Class | Reproduce | Fix family | New variant | New flag | Behaviour delta |
|---|---|---|---|---|---|---|
| **L21** | D-secret-leak (SECRET, SILENT) | **YES, silent empty-passphrase BIP-38 encrypt** | REFUSE the composite arm (reuse `ConvertRefusal`) | no | **no** | previously-accepted invocation now refused (exit ≠ 0) |
| **L24** | E-panic-dos (CLI-reachable, operator-misuse) | **YES, OOB panic** | bounds gate mirroring `bundle.rs` (reuse `DescriptorParse`) | no | no | panic → clean typed error (exit ≠ 0) |
| **L25** | other (cosmetic err-msg) | **YES, wrong "keyless" message** | position-aware x-only detection (reuse `DescriptorParse`) | no | no | error-message text only; both arms still `Err` |

---

## 2. Per-finding fix design

### 2.1 L21 — composite `(seedqr\|phrase\|entropy) → bip38` silently encrypts with an EMPTY passphrase

**Source set (CRITICAL — all three, not two).** The composite BIP-39-source arm is the outer match
`Seedqr | Phrase | Entropy => {` at `convert.rs:1231`. Its inner `Bip38 =>` sub-arm (the empty-encrypt
site, `:1350-1379`) is therefore reached for **`from ∈ {Seedqr, Phrase, Entropy}`** — Seedqr decodes its
digit-string to a BIP-39 phrase (`:1240`) and folds into the **identical** entropy-projection path, so
`(Seedqr, Bip38)` travels the same `:1376` `unwrap_or("")` as `(Phrase, Bip38)` / `(Entropy, Bip38)`.
`(Seedqr, Bip38)` is an explicitly-permitted, **argv-reachable** edge (`classify_edge` whitelist `:637`;
`"seedqr" => Self::Seedqr` argv parse `:77`). **Seedqr MUST NOT be omitted from the refusal** — omitting it
leaves the footgun fully live for the seedqr source (and the test suite would falsely certify it fixed).

**Bug.** On the composite `(Seedqr|Phrase|Entropy → Bip38)` edge:
- The line-`932` guard `bip38_edge && effective_passphrase.is_none() && effective_bip38_passphrase.is_none()`
  is satisfied by `--passphrase` **alone** (the `&&` means any passphrase passes), so a user who supplies
  `--passphrase X` (the v0.7 dual-purpose habit) sails past it.
- `--passphrase` is consumed by BIP-39 PBKDF2 only (`pbkdf2_passphrase`, `:979`).
- The BIP-38 Scrypt layer reads `bip38_passphrase`, which is `None` → `unwrap_or("")` at `:1376` → an
  **empty-passphrase BIP-38 ciphertext**.
- The "ignored passphrase" warning is **suppressed on BIP-38 edges** (`edge_uses_passphrase = … || bip38_edge`,
  `:954`) → **no warning** → **silent**.
- This asymmetry is intentional-by-design (documented v0.8 BREAKING, doc-comment `:1351-1357`: composite arm
  `--passphrase` feeds BIP-39 only; if `--bip38-passphrase` unset, BIP-38 uses `""`), but the **footgun is
  live**: an engravable steel card a user *thinks* is BIP-38-protected is encrypted with the empty string.

**DECISION — REFUSE (fail-closed); no new flag.** On the composite `(Seedqr|Phrase|Entropy → Bip38)` edge,
when `effective_bip38_passphrase.is_none()`, return a clean `ToolkitError::ConvertRefusal` BEFORE the
empty encrypt. This is the smaller, fail-closed change, matches the project's funds-safety discipline,
and reuses the existing `ConvertRefusal` variant → **no new error variant, no new clap flag, no
GUI-schema / manual-flag-table lockstep** (only manual PROSE, §4.4).

**Refusal condition — POSITION-BASED (preferred framing; no `from`-set test).** Enforce the refusal **at
the head of the composite `Bip38 =>` sub-arm** (`convert.rs:1350`, ending `:1379`), which sits **inside**
the outer `Seedqr | Phrase | Entropy =>` arm at `:1231`. Because that arm *structurally IS* the full
composite source set, reaching the `Bip38 =>` sub-arm already proves `from ∈ {Seedqr, Phrase, Entropy}` —
**no `from` test is needed, and none should be written.** The only runtime condition to check at the arm
head is:

> `effective_bip38_passphrase.is_none()` → return `ToolkitError::ConvertRefusal(…)` before the `unwrap_or("")` at `:1376`.

This is deliberately position-based, NOT a `from`-set enumeration: a future reader cannot drop `Seedqr` from
a written `{Phrase, Entropy}` list (the exact gap C1 flagged) because there is no list — the arm's *position*
inside `:1231` is the membership proof. The site is adjacent to the empty-encrypt it prevents, and the direct
`(wif↔bip38)` edges have their own separate arms (`:1523/:1543`), so they are structurally untouched.

**R0 alternative (equivalent):** the `:932` guard region is the conceptual funds-safety gate. If the
implementer prefers a single gate there, the predicate MUST be the **full** composite source set
`from ∈ {Seedqr, Phrase, Entropy}` (matching the `:1231` outer arm) ∧ target `Bip38` ∧
`effective_bip38_passphrase.is_none()` — Seedqr included. The position-based arm-head site is preferred
precisely because it avoids re-deriving (and risking under-specifying) that source set.

**Refusal message (`ConvertRefusal` string).**
> `composite (seedqr|phrase|entropy)→bip38 requires --bip38-passphrase (or --bip38-passphrase-stdin); on`
> `this edge --passphrase feeds only BIP-39 PBKDF2 and the BIP-38 layer would be encrypted with the empty`
> `passphrase. To deliberately use an empty BIP-38 passphrase, pass --bip38-passphrase "" explicitly.`

This (a) names the missing flag, (b) explains *why* `--passphrase` doesn't cover it, and (c) points at the
explicit empty-passphrase escape hatch. (Reuse the existing `refusal_bip38_no_passphrase()` helper pattern;
add a sibling helper, e.g. `refusal_composite_bip38_no_bip38_passphrase()`, or inline the `ConvertRefusal`.)

**`--bip38-passphrase ""` STILL WORKS (explicit empty path).** With `--bip38-passphrase ""` supplied,
`effective_bip38_passphrase` is `Some("")` (an explicitly-typed empty string), so the
`is_none()` arm-head condition is **false** → the refusal does **not** fire → the arm proceeds →
`bip38_passphrase.unwrap_or("")` yields `""` → BIP-38
encrypts with the empty passphrase. The deliberate empty path is preserved; only the *silent default* is
closed. (R0 note: confirm `effective_bip38_passphrase` distinguishes "flag absent" `None` from
"flag = empty string" `Some("")` — the clap `Option<String>` does, and `--bip38-passphrase-stdin` reads a
raw byte string that can be empty; the RED test below pins this.)

**`--allow-empty-passphrase` opt-in — REJECTED.** An empty-passphrase BIP-38 card is never a sane
production artifact for an engravable steel card; an opt-in flag would add clap + GUI-schema + manual
flag-table lockstep surface for a near-zero-value escape hatch already served by `--bip38-passphrase ""`.
**Decision: do not add a flag.**

**Direct `(wif↔bip38)` fallback — LEFT AS-IS (decided).** The direct edges at `:1523/:1543`
(`unwrap_or(pbkdf2_passphrase)`) intentionally fall back to `--passphrase`; this is the documented v0.8
behaviour (manual `56-bip39-vs-bip38-pass.md` edge table) and is **not silent** in the same way (the user
supplied a passphrase that is then used). **We do NOT harmonize the direct edges** in cycle-11b — touching
them changes a deliberate, documented, non-footgun behaviour and would expand scope/SemVer. The asymmetry
(composite refuses; direct falls back) is preserved and is internally consistent with the manual's edge
table. (Rationale: the footgun is specifically the *silent empty* on the composite arm; the direct edges
have a meaningful fallback and no silent-empty hazard.)

### 2.2 L24 — `verify-bundle` descriptor-mode `--slot @N.path` OOB-indexes `new_paths[idx]` → panic

**Bug.** In `verify_bundle.rs`, the non-canonical descriptor block builds `new_paths` with exactly `n`
entries (`:1384-1404`) and then, in the per-slot path-override loop (`:1422-1436`), writes
`new_paths[*idx as usize]` (`:1435`) with **no `idx < n` bound check**. `validate_slot_set` (`:1351`,
`slot_input.rs:249-267`) checks only **contiguity** (`0..=max_idx`, no gaps) — it does **not** range-check
against `n` (`n` is unknown at that layer). So a contiguous slot set whose max index exceeds the
descriptor's placeholder count passes validation, reaches the override loop, and **panics on the OOB write**.

The sibling in `bundle.rs` (`:1373-1388`) HAS an unconditional guard — `max(idx+1) != n` →
`ToolkitError::DescriptorParse("descriptor has n=… placeholders but --slot vec covers … slots")` — fired
BEFORE its own override loop. The `verify_bundle.rs` mirror **omits it** (guard-drift between hand-copied
descriptor-mode bindings).

**Trigger (CLI-reachable, operator-misuse, NO funds/secret impact).**
```
verify-bundle --descriptor "<non-canonical 2-key descriptor>" \
  --slot @0.phrase=<seed> --slot @1.phrase=<seed> --slot @2.path=m/…
```
Indices 0,1,2 are contiguous (passes `validate_slot_set`); the descriptor is non-canonical (enters the
`is_non_canonical` block at `:1371`); `@2` is a path-override on a phrase-bearing slot
(`subkeys.contains(Phrase|Seedqr|Ms1)` at `:1430-1436`); `new_paths` has length n=2 → `new_paths[2]`
**panics (index out of bounds)**.

**Fix — add the `bundle.rs`-style bounds gate, mirrored exactly.** Insert, immediately after the
`validate_slot_set` call (`:1351`) and before the `is_non_canonical` block (`:1371`), the same
exact-coverage gate `bundle.rs:1373-1388` uses:
```
if args.slot.iter().map(|s| s.index as usize + 1).max().unwrap_or(0) != n {
    return Err(ToolkitError::DescriptorParse(format!(
        "descriptor has n={n} placeholders but --slot vec covers {} slots", …)));
}
```
Reuses the existing `DescriptorParse(String)` variant (exit_code `2`, non-zero). This catches the
over-`n` case (panic → clean error) AND, as a bonus, the under-`n` case (matching `bundle.rs` parity).
Mirroring `bundle.rs`'s exact `!= n` predicate (rather than a looser `idx < n` per-write check) is
deliberate: it keeps the two descriptor-mode bindings symmetric and reuses the identical error text, so
future readers see one gate, not two divergent ones. (`validate_slot_set` already guarantees contiguity
from `@0`, so `max(idx+1) == n` ⟺ exactly the slots `@0..@n-1` are present — the bundle.rs invariant.)

**SEQUENCING NOTE (carry into impl).** The program's S-VERIFY thesis (`PLAN §292`) is to **deduplicate the
`bundle.rs ↔ verify_bundle.rs` descriptor-mode binding into one shared function** so this exact guard-drift
class cannot recur. That dedup is **NOT scheduled for cycle-11b**. **Decision: land the standalone gate
now** (self-contained, mechanical). **Annotate the inserted gate with a comment** noting it duplicates
`bundle.rs:1373-1388` and **should fold into the shared function when the S-VERIFY dedup lands** (cite the
FOLLOWUP slug, §5). When the dedup runs, this gate moves inside the shared fn (one gate, both callers).

### 2.3 L25 — `import-wallet` keyed/keyless classifier blind to raw x-only taproot keys

**Bug.** `has_any_key_token` (`pipeline.rs:53-61`) matches xpub-family and `02/03`-prefixed 66-hex
compressed pubkeys but **NOT** bare 32-byte (64-hex) x-only (BIP-340/341) keys. A
`tr(<xonly>, pk(<xonly2>))` with no `[fp/path]` origin and no `@N` falls into `classify_descriptor_form`'s
`(false,false)` arm (`:185`); `has_any_key_token` returns `false` → routes to the **"this descriptor has no
keys to engrave … keyless script (hashlock/timelock only) … export-wallet"** message (`:198-205`) — but the
descriptor **does** carry taproot keys, so the message is **misleading**.

**Benign.** BOTH `(false,false)` sub-arms return `Err` (the descriptor is rejected either way for lacking
origins) → this affects **only the error text**. No funds / secret / routing impact. The *correct* message
for a key-bearing-but-origin-less descriptor is the other arm's **"…concrete descriptors must carry a key
origin, e.g. [<fp>/84h/0h/0h]xpub…"** (`:189-196`).

**Fix — position-aware x-only detection, NOT a regex widen.** The 64-hex token is genuinely ambiguous: an
x-only taproot pubkey vs a `sha256()`/`hash256()` hash literal (the doc-comment at `:49-52` says so). A
naive regex widen to `\b[0-9a-fA-F]{64}\b` would mis-flag a keyless `sha256(<64-hex>)` hashlock descriptor
as keyed → a NEW wrong message (a different regression). **Decision: detect a bare 64-hex token only when it
appears in a taproot KEY position**, not anywhere in the string. Two acceptable implementations (R0 to pick):
- **(a) Position-aware token scan (minimal):** extend `has_any_key_token` (or a new helper it calls) to
  recognize a bare 64-hex token that sits in a taproot key position — i.e. as the internal key directly
  after `tr(`, or as the argument of `pk(`/`pk_k(`/`pk_h(` — while continuing to *exclude* 64-hex that is
  the argument of `sha256(`/`hash256(`/`ripemd160(`/`hash160(` (hash literals). This can be a targeted set
  of context-anchored matches (e.g. `tr(<64hex>`, `pk(<64hex>`) rather than a bare-token widen.
- **(b) Structural parse (the bug-hunt's preferred):** parse the descriptor tree and check for x-only keys
  in key positions. More robust but heavier; only worth it if a parser is already on the path here.

**Recommend (a)** for a PATCH-scoped cosmetic fix (smaller, no new parse dependency), with the explicit
constraint that **sha256/hash256/ripemd160/hash160 64-hex arguments MUST NOT be classified as keys** (the
existing keyless-hashlock behaviour at `:198-205` is preserved). R0 picks (a) vs (b); both must satisfy the
"hash literal stays keyless" invariant pinned by the existing test `has_any_key_token_distinguishes_keys_
from_hashes` (`:557`) — which the fix MUST keep GREEN.

**M3 — option (a)'s `pk(` anchor MUST NOT regress the existing 66-hex assertions.** `has_any_key_token`'s
current regex (`:56`) already matches `02/03`-prefixed 66-hex compressed keys via the `\b0[23][0-9a-fA-F]{64}\b`
alternation, and `has_any_key_token_distinguishes_keys_from_hashes` (`:557`) asserts those **compressed-key**
tokens classify as keys (GREEN today). When option (a) adds a `pk(<64hex>` / `tr(<64hex>` context anchor for
**x-only** 64-hex, it MUST be **additive** — the new anchor must still accept a `pk(02…{64})` /
`pk(03…{64})` compressed key (66 hex after the `pk(`), not only a bare-64-hex x-only one. Implementations
that naively anchor exactly 64 hex after `pk(` could fail to match the 66-hex compressed argument and flip
an existing GREEN assertion to RED. The fix keeps **both** the 66-hex compressed-key assertions and the
hash-literal-stays-keyless assertions in `:557` GREEN, and only ADDS the x-only-in-key-position case.

**No behaviour change beyond the message.** Both `(false,false)` arms still `Err`; the descriptor is still
rejected for lacking origins. The fix only re-routes the x-only-bearing case from the "keyless" message to
the correct "must carry a key origin" message. Confirm in the test (§3.3).

---

## 3. Per-finding tests (TDD — RED before impl)

### 3.1 L21
- **RED-1 (silent-empty now REFUSED — phrase):** `convert --from phrase="<seed>" --to bip38 --path m/0'/0
  --passphrase X` (NO `--bip38-passphrase`) → exit ≠ 0 (specifically the `ConvertRefusal` exit code `2`),
  stderr carries the refusal message (assert it mentions `--bip38-passphrase` and the `""` escape hatch).
  Today this **silently** emits a `6P…` empty-passphrase ciphertext, exit 0 — so this test is RED today.
- **RED-2 (silent-empty now REFUSED — entropy):** `convert --from entropy=<hex> --to bip38 --passphrase X`
  (NO `--bip38-passphrase`) → same refusal (exit 2). RED today.
- **RED-3 (silent-empty now REFUSED — SEEDQR — C1):** `convert --from seedqr=<valid-seedqr-digits>
  --to bip38 --passphrase X` (NO `--bip38-passphrase`) → same refusal (exit 2). **This is the C1 source the
  pre-fold spec omitted.** `(Seedqr, Bip38)` is an argv-permitted edge (`:637`) that decodes to a BIP-39
  phrase and hits the **same** `:1376` empty-encrypt; the position-based arm-head refusal (inside the
  `:1231` outer `Seedqr | Phrase | Entropy =>` arm) covers it structurally. RED today (silently emits a
  `6P…` empty-passphrase ciphertext, exit 0). **This test MUST be present — without it the seedqr footgun
  is uncovered and the suite would falsely certify L21 fixed.**
- **GREEN-1 (explicit empty STILL encrypts — phrase):** `convert --from phrase="<seed>" --to bip38
  --path m/0'/0 --passphrase X --bip38-passphrase ""` → exit 0, emits a valid `6P…` BIP-38 string;
  round-trip `convert --from bip38=<that> --to wif --bip38-passphrase ""` recovers the WIF. Pins that
  `Some("")` is distinguished from `None`.
- **GREEN-1b (explicit empty STILL encrypts — SEEDQR):** `convert --from seedqr=<valid-seedqr-digits>
  --to bip38 --passphrase X --bip38-passphrase ""` → exit 0, emits a valid `6P…` BIP-38 string. Pins that
  the position-based refusal does NOT over-fire on the seedqr source when the explicit empty path is taken.
- **GREEN-2 (real passphrase STILL encrypts):** `… --bip38-passphrase secret` → exit 0, valid `6P…`,
  round-trips with `--bip38-passphrase secret`. (Regression guard: the refusal doesn't over-fire.)
- **REGRESSION (direct edge untouched):** `convert --from wif=<wif> --to bip38 --passphrase X` → exit 0
  (direct edge still falls back to `--passphrase` per `:1523`; the refusal does NOT fire on direct edges).

### 3.2 L24
- **RED (over-n panic → typed reject):** `verify-bundle --descriptor "<non-canonical 2-key descriptor>"
  --slot @0.phrase=<seed> --slot @1.phrase=<seed> --slot @2.path=m/84'/0'/0'` → today **panics** (OOB on
  `new_paths[2]`); after fix → `ToolkitError::DescriptorParse` (exit `2`), message names the n/slot mismatch
  ("n=2 … covers 3 slots").
  **Fixture preconditions (M2 — pin in the test/fixture comment so the override loop actually reaches the
  `:1435` OOB write):**
  1. The descriptor MUST be genuinely **non-canonical** — assert `canonical_origin(<descriptor>).is_none()`
     so control enters the `is_non_canonical` block at `verify_bundle.rs:1371` (a canonical descriptor would
     skip the override region entirely and the test would pass vacuously, never exercising the gate).
  2. `@2` MUST carry a **subkey** in `{Phrase, Seedqr, Ms1}` (here `@2.path=…` riding a phrase-bearing slot)
     so it (i) lands in the `by_index_subkeys` map built at `:1417-1421` and (ii) clears the
     `subkeys.contains(Phrase|Seedqr|Ms1)`-else-`continue` filter at `:1427-1432`, so the override loop
     reaches the `new_paths[*idx as usize] =` write at `:1435`. A path-only override on a
     non-phrase-bearing slot hits the `continue` at `:1431` before the OOB write and the RED would not
     reproduce.
- **REGRESSION (exact-coverage still works):** the canonical 2-key path-override flow with `@0`/`@1`
  path-overrides only (max idx+1 == n) still verifies as before (gate doesn't over-fire). If the suite has
  an existing verify-bundle descriptor-override green-path test, assert it stays GREEN.

### 3.3 L25
- **RED (x-only routes to RIGHT message):** `import-wallet --descriptor "tr(<64-hex-xonly>,pk(<64-hex-
  xonly2>))"` (no origin, no `@N`) → today surfaces the **"no keys to engrave / keyless script"** message;
  after fix → the **"…must carry a key origin…"** message. Assert exit ≠ 0 in BOTH (the descriptor is
  rejected either way — message-only change), and assert the post-fix message contains "must carry a key
  origin" and does NOT contain "keyless script".
- **REGRESSION (hash literal stays keyless):** a genuinely keyless `wsh(and_v(v:pk(...nope use no key...),
  sha256(<64-hex>)))` — or simplest, the existing `classify_keyless_routes_to_export_wallet` /
  `has_any_key_token_distinguishes_keys_from_hashes` tests (`:529,:557`) — MUST stay GREEN: a
  `sha256(<64-hex>)` hashlock with no real key still routes to the "keyless / export-wallet" message. This
  is the guard against the regex-widen regression.
- **REGRESSION (M3 — 66-hex compressed key still keyed):** the existing
  `has_any_key_token_distinguishes_keys_from_hashes` (`:557`) assertions that a `0[23]…{64}` 66-hex
  compressed pubkey classifies as a KEY MUST stay GREEN — option (a)'s new `pk(`/`tr(` x-only anchor must be
  additive and not flip the compressed-key case to RED. Assert both the compressed-key-is-keyed AND the
  hash-literal-is-keyless sub-assertions remain GREEN after the fix.

---

## 4. SemVer / lockstep / sequencing

### 4.1 SemVer
- L21 → **PATCH** (REFUSE; no flag; reuses `ConvertRefusal`; funds-safety hardening). FORMAL per the
  program's rule-clause-1 (adds a refusal of a previously-accepted invocation), but behaviourally a
  bug-fix with no new surface → **PATCH is correct** (matches `PLAN §210`).
- L24 → **PATCH** (panic → clean `DescriptorParse`; no surface change; `PLAN §213`).
- L25 → **PATCH** (cosmetic message; no surface change; `PLAN §214`).
- **Composite: toolkit PATCH → `0.65.1`.** (If a concurrent MINOR-bearing cycle lands on the integration
  branch first, renumber accordingly — the own-account cycle would renumber this to `0.65.x+1`. Whoever
  lands second renumbers `Cargo.toml`.)

### 4.2 No new error variant
**No new `ToolkitError` variant** for any of the three — L21 = `ConvertRefusal`, L24 + L25 = `DescriptorParse`
(all already present and alphabetically placed). The alphabetical-insertion discipline (CLAUDE.md) does
**not** bite cycle-11b.

### 4.3 No clap / `--json` / GUI schema_mirror change
**No new clap flag, subcommand, dropdown value, or `--json` wire-shape change** → **NO
`mnemonic-gui/src/schema/mnemonic.rs` schema_mirror update required**, and **NO
`docs/manual/src/40-cli-reference/41-mnemonic.md` flag-table change**. (This is precisely why the
`--allow-empty-passphrase` opt-in for L21 was rejected — it would have forced GUI-schema + manual-flag-table
lockstep for near-zero value.) L24 and L25 are pure internal robustness/text — no surface at all.

### 4.4 L21 manual PROSE lockstep (decided: SAME COMMIT)
L21's new refusal is **prose-only** (no flag table). Update, **in the same commit as the L21 code change**
(not a paired-PR — this is a single-repo PROSE edit, gate-free, and same-commit avoids any drift window):
- **`docs/manual/src/50-comparing/56-bip39-vs-bip38-pass.md`** — the composite-edge table (lines 49-54 on
  `origin/master @ 4e8ad792`) currently has `(phrase, bip38) composite` (`:53`, "BIP-38 (independent; no
  fallback)") and `(entropy, bip38) composite` (`:54`, "BIP-38 (defaults to `""` if unset; BREAKING)") but
  **has NO `(seedqr, bip38)` row** — even though `(Seedqr, Bip38)` is a permitted composite edge with the
  identical empty-encrypt behaviour. **Required edits (C1):**
  1. **Amend** the existing `(phrase, bip38)` and `(entropy, bip38)` composite rows so an **unset**
     `--bip38-passphrase` reads **REFUSED** (not silently `""`); the `(entropy, bip38)` row's stale
     "defaults to '' if unset" becomes "REFUSED if unset; `--bip38-passphrase ""` for an explicit empty
     passphrase".
  2. **ADD a `(seedqr, bip38)` composite row** ("REFUSED if unset; `--bip38-passphrase ""` for explicit
     empty") — OR equivalently **generalize** the three composite rows into one
     `(phrase|entropy|seedqr, bip38) composite` row. Pick one; either way the seedqr source MUST appear in
     the table so a reader sees it is refused, not silently `""`. (Leaving seedqr out of the table is the
     prose mirror of the C1 predicate gap.)
- **`docs/manual/src/40-cli-reference/41-mnemonic.md`** — the `convert` reference; the `--bip38-passphrase`
  row (`:802`) and surrounding `convert` prose. Add a one-line note: "on a composite
  `(seedqr|phrase|entropy)→bip38` edge, `--bip38-passphrase` is **required**; an unset value is refused (it
  would otherwise encrypt with the empty passphrase). Pass `--bip38-passphrase ""` to deliberately use an
  empty BIP-38 passphrase." **No flag-table row added or removed** — this is a clarification of the existing
  row.

**This prose update is NOT gate-enforced** (the manual lint checks flag-NAME coverage, not prose) — so it
is a **discipline** item: it MUST ride the same commit. (Manual lint stays GREEN regardless; the value is
correctness for the reader.)

### 4.5 Version sites (PATCH 0.65.0 → 0.65.1)
**CORRECTION (I1 — verified against `origin/master @ 4e8ad792`):** ALL FIVE version sites are currently at
**`0.65.0`; NONE has drifted.** (An earlier draft claimed four sites had "silently drifted to 0.60.0" — that
was **false**, an artifact of reading the **dirty local own-account worktree** (branched off v0.60.0), not
origin/master. Re-verified live: every site below reads `0.65.0`.)

| # | Site | `origin/master @ 4e8ad792` value | Bump to |
|---|---|---|---|
| 1 | `crates/mnemonic-toolkit/Cargo.toml` | `version = "0.65.0"` (`:3`) | `0.65.1` |
| 2 | `README.md` | `<!-- toolkit-version: 0.65.0 -->` (`:13`) | `0.65.1` |
| 3 | `crates/mnemonic-toolkit/README.md` | `<!-- toolkit-version: 0.65.0 -->` (`:9`) | `0.65.1` |
| 4 | `scripts/install.sh` self-pin | `mnemonic-toolkit-v0.65.0` (`:32`) | `mnemonic-toolkit-v0.65.1` |
| 5 | `fuzz/Cargo.lock` | `mnemonic-toolkit … version = "0.65.0"` (`:575`) | `0.65.1` |

**Decision (MANDATORY — no escape hatch).** The release ritual MUST bump **all five sites + a `CHANGELOG.md`
entry** (new `## mnemonic-toolkit [0.65.1]` section documenting L21 refusal / L24 gate / L25 message) **to
`0.65.1` in lockstep** — this is **mandatory, not optional cleanup**. Because all five start at `0.65.0`,
bumping only `Cargo.toml` (the version-of-record) while leaving the other four at `0.65.0` would *introduce*
exactly the drift the earlier draft wrongly believed already existed. None of sites 2-5 are gate-enforced
(per `project_toolkit_release_ritual_version_sites`), so the lockstep is **discipline**, not a CI gate —
which makes it easy to miss; the implementer MUST treat all five as one atomic bump. No tag/publish beyond
the local toolkit tag `mnemonic-toolkit-v0.65.1` per the normal release flow.

**PROCESS (I1).** The implementer MUST `git worktree add` a fresh worktree off **`origin/master @
4e8ad792`** and build cycle-11b there. They MUST **NOT** build on the local own-account worktree (which is
on `feature/own-account-subset-search` off v0.60.0); doing so reintroduces the stale-0.60.0 version values
and any other v0.60.0-era drift, and corrupts the version-site bump.

### 4.6 Shared-file sequencing (carry into impl)
All three files are in active multi-cycle zones (recon §Cross-cutting):
- `cmd/convert.rs` (L21) ↔ **S-NET** (`PLAN §616/§687`) — serialize after S-NET if cycle-13 carries
  convert.rs hunks; L21's edit is localized (`:932` guard region + `:1376` arm) → mechanical rebase.
- `cmd/verify_bundle.rs` (L24) ↔ **S-VERIFY** dedup (`PLAN §292`) — the one genuinely
  collision-sensitive item; land the standalone gate now, fold into the shared fn when S-VERIFY dedups
  (§2.2 note). The S-VERIFY dedup is **not scheduled now** → standalone gate is correct.
- `wallet_import/pipeline.rs` (L25) ↔ **S-NET / H15** (`PLAN §426/§619`) — rebase onto any cycle-13
  S-NET pipeline.rs hunks.

**Net:** cycle-11b's three fixes are individually small and self-contained. If cycle-13 carries
S-NET/S-VERIFY work, cycle-11b should rebase onto (or serialize after) those landings rather than run
concurrently in the same file zones. cycle-10's md-codec pin-bump is orthogonal (Cargo metadata only;
meet only at `Cargo.toml`/`Cargo.lock`, mechanical).

---

## 5. FOLLOWUP slugs

| Slug | Repo | Status | What |
|---|---|---|---|
| `convert-composite-bip38-empty-passphrase-refusal` | toolkit | **CLOSE in the L21 shipping commit** | The L21 refuse-decision record; flip to done when 0.65.1 ships. |
| `verify-bundle-bundle-rs-descriptor-mode-dedup` | toolkit | **OPEN (carries the L24 gate)** | S-VERIFY dedup of `bundle.rs ↔ verify_bundle.rs`; when it lands, fold L24's standalone gate (and the H1-class checks) into the shared fn. The L24 commit MUST cite this slug in the gate comment. |
| `import-classify-xonly-position-aware` | toolkit | **CLOSE in the L25 shipping commit** | Position-aware x-only key detection; flip to done when 0.65.1 ships. If R0 picks structural-parse (b) and defers it, leave a residual slug for the structural upgrade. |

(Verify "open" status at decision time per the FOLLOWUP-status-discipline memory note; flip in the shipping
commit. The S-VERIFY dedup slug stays OPEN — cycle-11b lands only the standalone gate.)

---

## 6. Resolved decisions (NO open questions)

| # | Decision | Resolution |
|---|---|---|
| D1 | L21 fix family | **REFUSE** the composite `(seedqr\|phrase\|entropy)→bip38` arm when `--bip38-passphrase` unset (reuse `ConvertRefusal`). NOT a warning, NOT an opt-in flag. |
| D2 | L21 refusal predicate | **POSITION-BASED** — refuse iff `effective_bip38_passphrase.is_none()` at the head of the `Bip38 =>` sub-arm that sits inside the outer `Seedqr \| Phrase \| Entropy =>` arm (`convert.rs:1231`). The arm position structurally proves the full composite source set `{Seedqr, Phrase, Entropy}` — **no `from`-set test is written** (a written list would risk dropping `Seedqr`, the C1 gap). If R0 instead enforces at the `:932` guard, the predicate is the **full** `from ∈ {Seedqr, Phrase, Entropy}` ∧ target `Bip38` ∧ `effective_bip38_passphrase.is_none()` — **Seedqr included**. |
| D3 | L21 enforcement site | Head of the composite `Bip38 =>` sub-arm (`convert.rs:1350`, inside the `:1231` outer arm), before `:1376`'s `unwrap_or("")` — **preferred** (position is the membership proof). R0 may relocate to the `:932` guard region; if so, predicate D2's full-source-set form applies. |
| D4 | L21 explicit empty path | `--bip38-passphrase ""` (→ `Some("")`) STILL encrypts (refusal checks `is_none()`, not emptiness). Pinned by GREEN-1. |
| D5 | `--allow-empty-passphrase` opt-in | **REJECTED** — adds clap/schema/manual lockstep for near-zero value; the explicit path is `--bip38-passphrase ""`. |
| D6 | L21 direct `(wif↔bip38)` edges | **LEFT AS-IS** — documented v0.8 fallback to `--passphrase`; not the silent-empty footgun; not harmonized this cycle. |
| D7 | L24 fix | Add `bundle.rs:1373-1388`-style `max(idx+1) != n` gate at `verify_bundle.rs` after `:1351`, before `:1371`; reuse `DescriptorParse`. |
| D8 | L24 sequencing | Land the **standalone gate now**; annotate to fold into the S-VERIFY shared fn when that dedup lands (not scheduled this cycle). |
| D9 | L25 fix | **Position-aware** x-only detection (a bare 64-hex in a taproot key position), NOT a regex widen; `sha256`/`hash256` 64-hex args MUST stay keyless. R0 picks impl (a) targeted-context-match vs (b) structural parse; (a) recommended for PATCH. **M3:** option (a)'s `pk(`/`tr(` anchor must be ADDITIVE — keep the existing `0[23]…{64}` 66-hex compressed-key assertions in `has_any_key_token_distinguishes_keys_from_hashes` (`:557`) GREEN, not just the hash-literal ones. |
| D10 | L25 behaviour | Message-only — both `(false,false)` arms still `Err`; x-only case re-routes from "keyless" to "must carry a key origin". |
| D11 | SemVer | All PATCH → toolkit **0.65.1**; no registry publish; tag `mnemonic-toolkit-v0.65.1`. |
| D12 | New variant | **None** — `ConvertRefusal` (L21) + `DescriptorParse` (L24/L25) already present; alphabetical discipline moot. |
| D13 | clap / schema_mirror / manual flag table | **No change** — no new flag/subcommand/dropdown/`--json` shape → no GUI schema_mirror update, no manual flag-table edit. |
| D14 | L21 manual PROSE | **Same commit** — amend `56-bip39-vs-bip38-pass.md` composite-edge table (amend `(phrase,bip38)`/`(entropy,bip38)` rows to "REFUSED if unset" **AND add/generalize a `(seedqr,bip38)` row** — the seedqr source MUST appear) + `41-mnemonic.md` `--bip38-passphrase` prose note (`(seedqr\|phrase\|entropy)→bip38`). Not gate-enforced; discipline. |
| D15 | Version sites | **All five sites are at 0.65.0 on `origin/master @ 4e8ad792` — NONE drifted** (the earlier "drifted to 0.60.0" claim was a dirty-worktree artifact, corrected). **MANDATORY lockstep bump to 0.65.1**: `Cargo.toml` + `README.md` + `crates/mnemonic-toolkit/README.md` + `scripts/install.sh` + `fuzz/Cargo.lock` + `CHANGELOG.md`. **No "skip already-drifted sites" escape hatch** (it would *introduce* drift). Implementer MUST `git worktree add` off `origin/master @ 4e8ad792`, NOT the local v0.60.0 own-account worktree. |

---

## 7. Mandatory R0 gate (CLAUDE.md hard gate)

**NO code before R0 GREEN (0C/0I).** This brainstorm spec MUST pass the opus-architect **R0 review loop to
0 Critical / 0 Important** BEFORE any implementation (writing code, dispatching implementer subagents)
begins — and the reviewer loop continues after every fold (re-dispatch the architect; folds can introduce
drift). Persist each review verbatim to `design/agent-reports/cycle11b-<round>-review.md` BEFORE applying
folds. R0 reviews run the **full `cargo test -p mnemonic-toolkit` suite**, not targeted `--test` targets
(a CLI/convert/import phase ripples into argv/schema/version lints outside any one phase's targets). Only
after this brainstorm is R0-GREEN does the cycle proceed to the SPEC / plan-doc (each with its own R0 loop),
then single-subagent-per-phase TDD with per-phase R0, then the mandatory whole-diff adversarial execution
review.
