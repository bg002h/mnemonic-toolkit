# v0.9.0 Cycle B Phase 1 R0 design pass (bip85 heap-promote)

**Reviewer:** Opus 4.7 (1M context), invoked as design-review on Cycle B Phase 1 (bip85 heap-promote) before any code lands.
**Date:** 2026-05-13.
**SPEC:** `design/SPEC_secret_memory_hygiene_v0_9_B.md` (commit `f84d998`, master).
**Plan:** `~/.claude/plans/v0_9_B-secret-memory-hygiene-cycle-b.md` (R0 draft, 2026-05-13).
**Predecessor cycle plan:** `~/.claude/plans/v0_9_0-secret-memory-hygiene.md` (Cycle A, shipped 2026-05-13).
**Scope of review:** Phase 1 design (return-type lock, callee enumeration, test impact, P3a Site 4 forward guidance). No code reviewed (none exists yet).
**Verdict:** **LOCK — 0 Critical / 0 Important** at confidence ≥ 80. Return type locked as `Zeroizing<Vec<u8>>`. Plan/SPEC inaccuracies folded inline as Important findings (all addressable in P1.T2 prose without re-spinning the design).

---

## Summary

Total findings at confidence ≥ 80: **0 Critical / 4 Important (folded inline) / 2 Nit**.

The "Critical" gate is reserved for issues that block the design lock. None exist — every plan/SPEC inaccuracy surfaced below is a correction to descriptive text, not a design-shape problem. The recommended return type holds against the alternative on every axis examined. The callee and test enumeration is complete.

The 4 Important findings are factual reconciliations against the plan/SPEC narrative (callee count, signature shape, visibility, internal-vs-public framing). They are folded into this same report so the implementer (P1.T2) and R1 reviewer pick up the corrected facts without needing a SPEC/plan re-spin.

---

## §1. Locked return type — `Zeroizing<Vec<u8>>`

**Decision:** `Zeroizing<Vec<u8>>` (plan's recommendation), NOT `Box<Zeroizing<[u8; 64]>>`.

**Rationale — examined against the four axes in the prompt:**

### 1.1 Phase 3 ergonomic alignment (Site 4 wrapping)

SPEC §2 row 1: `MlockedZeroizing<T: Zeroize>` is parameterized on `T: Zeroize` and owns `Box<T>` directly. SPEC §4 P3a line 102 + plan §"Phase 3a" Site 4 line 370: the Phase 3a code is literally `MlockedZeroizing::new(... Vec<u8> ...)`. Both `Vec<u8>` and `[u8; 64]` are `Zeroize`, so either could theoretically be the wrapped `T`, but:

- `Zeroizing<Vec<u8>>` → unwrap to `Vec<u8>` (one step). Wrap as `MlockedZeroizing::<Vec<u8>>::new(vec)`. **Single type-parameter substitution.** This is what the plan §"Phase 3a" T3 step 3 already writes.
- `Box<Zeroizing<[u8; 64]>>` → unwrap to `Zeroizing<[u8; 64]>` via `*box_val`, then unwrap to `[u8; 64]`. Wrap as `MlockedZeroizing::<[u8; 64]>::new(arr)`. **Two unwraps + an extra moved-stack-array copy through `MlockedZeroizing::new`'s `ptr::write` path.** The `Box`'s heap allocation is then immediately freed, replaced by `MlockedZeroizing`'s page-aligned heap allocation. The intermediate `Box` is dead weight.

**Verdict on axis 1.1: `Zeroizing<Vec<u8>>` wins** — clean Vec-typed handoff to Site 4; matches the wrapper-site type pattern at Sites 2 and 3 (`Option<MlockedZeroizing<Vec<u8>>>` and `MlockedZeroizing<Vec<u8>>` per SPEC §4 P3a lines 100-101). Three of the four mlock sites that wrap an OWNED entropy buffer use `Vec<u8>` as the inner T; uniformity reduces cognitive load.

### 1.2 Callee slice-pattern compatibility

All 7 callees consume the result via slice indexing or borrow-as-slice (full enumeration in §3 below). Under either return type:

- `Zeroizing<Vec<u8>>` derefs (via `zeroize`'s `Deref for Zeroizing<T: Zeroize>`) → `Vec<u8>` → derefs again → `&[u8]`. Index expressions `&e[..n]` are slice operations on the deref'd slice; compile clean.
- `Zeroizing<[u8; 64]>` derefs → `[u8; 64]` → derefs → `&[u8]`. Index expressions `&e[..n]` compile clean today; this is the current shape.

**One subtle point** (verified against zeroize docs): `Zeroizing<Z>` impls `AsRef<T> where Z: AsRef<T>` via a blanket impl, so `hex::encode(e)` (line 357 test, which consumes by value and requires `AsRef<[u8]>`) works under BOTH return types because both `Vec<u8>: AsRef<[u8]>` and `[u8; 64]: AsRef<[u8]>`. No callee or test breaks under one but not the other.

**Verdict on axis 1.2: tie** — no callee breaks under either; all `&entropy[..n]` index forms continue to compile.

### 1.3 Length invariant (always 64 bytes)

- `Zeroizing<Vec<u8>>`: invariant must be enforced at runtime. The plan's `debug_assert_eq!(result.len(), 64, ...)` immediately before return is the right discipline. The current implementation (line 47-48) already writes 64 bytes via `copy_from_slice(mac.as_byte_array())` on a freshly-allocated `Vec`; if we `Vec::with_capacity(64)` + `extend_from_slice` (or `vec![0u8; 64]` + `copy_from_slice`), len=64 is structurally guaranteed at the construction site. The `debug_assert` is belt-and-braces and self-documenting.
- `Box<Zeroizing<[u8; 64]>>`: invariant is compile-time, no debug_assert needed.

This is the one axis where the alternative has a marginal edge. But the loss is small: HMAC-SHA512's output is exactly 64 bytes (a domain-level invariant well outside the toolkit's control surface); the `debug_assert!` is a one-line cost that doubles as documentation. The compile-time guarantee of `[u8; 64]` is nice but not load-bearing for the call sites (none of them index past 64).

**Verdict on axis 1.3: marginal edge to alternative; not decisive.**

### 1.4 Drop semantics (heap-resident, Zeroize-on-drop)

- `Zeroizing<Vec<u8>>`: `Vec<u8>` owns a heap allocation; `Zeroizing<Vec<u8>>`'s Drop calls `Vec::zeroize()` on the buffer in place (zeroize crate has `impl Zeroize for Vec<u8>` that writes zeros across the entire `len()` extent) before `Vec`'s Drop deallocates. Heap-resident throughout. Page-pinnable (mlock works on heap pages).
- `Box<Zeroizing<[u8; 64]>>`: Box's heap allocation holds the `Zeroizing<[u8; 64]>` (which is `#[repr(transparent)]` over `[u8; 64]`, so the heap memory IS the 64-byte array). When `Box` drops, it first runs `Drop` on its contents (the `Zeroizing` zeroizes the array in place), then deallocates. Heap-resident throughout. Page-pinnable.

Both deliver the heap-resident + Zeroize-on-drop property Phase 3 needs.

**Verdict on axis 1.4: tie.**

### 1.5 Synthesis

Axis 1.1 (Phase 3 ergonomic alignment) is the load-bearing tiebreaker. The other three either tie or give the alternative a marginal cosmetic edge that does not outweigh axis 1.1's site-uniformity win. **Decision: `Zeroizing<Vec<u8>>` locked.** Confidence: 90.

---

## §2. Plan/SPEC reconciliation — Important findings (folded inline)

The plan and SPEC contain four narrative inaccuracies relative to the current `bip85.rs` source. None invalidate the design shape; all need correction in the implementer's working brief (P1.T2/T3) and in the R1 reviewer's checklist. Captured here so they enter the audit trail:

### I-R0-1 — Signature shape mis-stated as `pub fn derive_entropy(index: u32) -> [u8; 64]` (Confidence: 95)

**Plan §"Phase 1" line 41 + SPEC §2 row 5 line 37 + SPEC §4 P1 line 75** all describe the current signature as:

```rust
pub fn derive_entropy(index: u32) -> [u8; 64]
```

The actual signature at `crates/mnemonic-toolkit/src/bip85.rs:22-27` is:

```rust
pub(crate) fn derive_entropy(
    master: &Xpriv,
    app_code: u32,
    app_params: &[u32],
    index: u32,
) -> Result<Zeroizing<[u8; 64]>, ToolkitError>
```

Three differences material to Phase 1:
1. **Visibility is `pub(crate)`, not `pub`.** `bip85` is declared as `mod bip85;` in `main.rs:3` (private module in the binary crate). There is no `lib.rs`. So this function is internal — it has no public-API surface to preserve under "no public-API removal." The plan's discipline note "No new public API surface beyond the return-type change" (plan line 17) is technically vacuous for `derive_entropy` since the symbol is unreachable from any external consumer; what the discipline actually protects is the binary's CLI surface (which is unchanged) plus the `pub mod mlock` to be added in Phase 2.
2. **Four parameters, not one.** The function takes `master`, `app_code`, `app_params`, `index`. The plan's RED test sketch (P1.T2 step 1: `derive_entropy(0)`) won't compile against the real signature.
3. **Return is `Result<Zeroizing<[u8; 64]>, ToolkitError>`, not `[u8; 64]`.** The Cycle A R1 I-4 fold already wrapped the buffer in `Zeroizing` and put it inside `Result` for hardened-index propagation. So "heap-promote" is more precisely: **swap the inner `[u8; 64]` to `Vec<u8>` within the existing `Result<Zeroizing<...>, ToolkitError>` envelope.** The `Result` shape stays; the `Zeroizing` wrapper stays; only the innermost type changes.

**Fold (corrected scope statement for P1.T2/T3):**

```rust
// BEFORE (current, post-Cycle-A):
pub(crate) fn derive_entropy(
    master: &Xpriv,
    app_code: u32,
    app_params: &[u32],
    index: u32,
) -> Result<Zeroizing<[u8; 64]>, ToolkitError>

// AFTER (Phase 1):
pub(crate) fn derive_entropy(
    master: &Xpriv,
    app_code: u32,
    app_params: &[u32],
    index: u32,
) -> Result<Zeroizing<Vec<u8>>, ToolkitError>
```

The P1.T2 RED test must use the real four-parameter call shape (e.g., `derive_entropy(&master(), 39, &[0, 12], 0)?` as the existing `bip39_12_words_entropy_matches_spec` test does).

### I-R0-2 — Plan claims "6 callees" in `format_*` functions; actual count is **7** (Confidence: 100)

**Plan §"Phase 1" lines 55, 90, 102 + SPEC §2 row 5 line 37 + SPEC §4 P1 line 76** state "6 callees in `format_*` functions." Reading `crates/mnemonic-toolkit/src/bip85.rs`:

1. `format_bip39_phrase` (line 67) — calls `derive_entropy` at line 74.
2. `format_hd_seed_wif` (line 94) — line 99.
3. `format_xprv_child` (line 121) — line 126.
4. `format_hex_bytes` (line 152) — line 157.
5. `format_password_base64` (line 169) — line 174.
6. `format_password_base85` (line 183) — line 188.
7. **`format_dice_rolls`** (line 208) — line 225. **MISSED by plan/SPEC.**

The DICE app (BIP-85 v1.3.0 §"DICE", app code `89101'`) was added in v0.8 (per `design/agent-reports/`); SPEC §2 row 5 + plan §"Phase 1" both still carry the pre-v0.8 "6 in-scope apps" framing. SPEC §1 of the predecessor `SPEC_derive_child_v0_7.md` is canon for the original 6; the DICE addition was carried in via the v0.8 SPEC extensions.

**Fold (corrected scope statement):** Phase 1 updates **7 callees**, not 6. `format_dice_rolls` consumes the entropy via `&entropy[..]` at line 229 (`shake.update(&entropy[..])`) — this is a Deref-and-borrow that works identically under `Zeroizing<Vec<u8>>` and `Zeroizing<[u8; 64]>`. No additional design surface beyond the rest of the callees.

### I-R0-3 — Plan §"Phase 1" line 17 + line 120 "No new public-API surface" is correct in spirit but imprecise in framing (Confidence: 82)

`derive_entropy` is `pub(crate)`, not `pub`. The function is internal to the binary crate. The plan's "no new public API surface beyond the return-type change" discipline note is true (Phase 1 adds no `pub` items at all), but the framing "return-type change" implies an external observer who could see the change — there is none. The real discipline at stake in Phase 1 is **byte-determinism** (the encoded output of every `format_*` function — the strings the CLI prints — must remain byte-identical pre/post swap, since G7 SHA pins cover these outputs through the v0.1/v0.2 fixture corpora).

**Fold:** the plan §"Phase 1" R1 checklist line 117 already names "Byte-determinism unaltered" — good. R1 reviewer should weight this above the public-API framing. The "no public-API removal" Cycle B discipline (plan line 17) actually applies to the `pub mod mlock;` addition in Phase 2; Phase 1 is internal.

### I-R0-4 — Plan §"Phase 1" T3 step 4 "callees... currently consume `[u8; 64]`; update to consume `&[u8]`" is misleading (Confidence: 85)

**Plan line 90:** *"each currently consumes `[u8; 64]`; update to consume `&[u8]` (deref through Zeroizing)"*.

This is misleading because:
1. The callees do NOT consume `[u8; 64]` by-value today — they consume `Zeroizing<[u8; 64]>` returned by `derive_entropy`, holding it in a local `let entropy = ...` binding, then **borrowing as `&entropy[..n]`** (which already goes through Zeroizing's Deref to a slice).
2. **No callee signature change is needed.** Each callee's `let entropy = derive_entropy(...)?` line is unchanged textually; the binding's type changes from `Zeroizing<[u8; 64]>` to `Zeroizing<Vec<u8>>`, but the `&entropy[..n]` borrow expressions on every subsequent line continue to type-check unchanged (both wrappers Deref to `&[u8]` for index expressions).

**Fold (corrected T3 step 4):** *"7 callees' `let entropy = derive_entropy(...)?` bindings change inferred type from `Zeroizing<[u8; 64]>` to `Zeroizing<Vec<u8>>`. The subsequent `&entropy[..n]` / `&entropy[..]` / `&entropy[..32]` / `&entropy[32..]` / `hex::encode(entropy)` borrow-and-consume expressions are transparent to the swap (both Zeroizing flavors expose `&[u8]` via Deref and `AsRef<[u8]>` via the blanket impl)."*

This is the GREEN-step rewrite the implementer should follow.

---

## §3. Callee enumeration (the 7 `format_*` functions)

| # | Function | Defined at | `derive_entropy` call | Consumption pattern (line) | Transparent? |
|---|---|---|---|---|---|
| 1 | `format_bip39_phrase` | `bip85.rs:67` | `bip85.rs:74` | `&entropy[..bytes]` at line 80 (passed to `Mnemonic::from_entropy_in`) where `bytes = words * 4 / 3` (16/20/24/28/32) | Yes |
| 2 | `format_hd_seed_wif` | `bip85.rs:94` | `bip85.rs:99` | `&entropy[..32]` at line 104 (passed to `SecretKey::from_slice`) | Yes |
| 3 | `format_xprv_child` | `bip85.rs:121` | `bip85.rs:126` | `&entropy[..32]` at line 127 (chain code, via `<[u8; 32]>::try_from`) **and** `&entropy[32..]` at line 133 (privkey scalar, via `SecretKey::from_slice`) | Yes |
| 4 | `format_hex_bytes` | `bip85.rs:152` | `bip85.rs:157` | `&entropy[..num_bytes as usize]` at line 158 (passed to `hex::encode`) | Yes |
| 5 | `format_password_base64` | `bip85.rs:169` | `bip85.rs:174` | `&entropy[..]` at line 175 (passed to `base64_standard`) | Yes |
| 6 | `format_password_base85` | `bip85.rs:183` | `bip85.rs:188` | `&entropy[..]` at line 189 (passed to `base85_btc`) | Yes |
| 7 | `format_dice_rolls` | `bip85.rs:208` | `bip85.rs:225` | `&entropy[..]` at line 229 (passed to `shake.update`) | Yes |

All 7 callees use slice-borrow expressions. **Every borrow expression continues to type-check unchanged under the swap from `Zeroizing<[u8; 64]>` to `Zeroizing<Vec<u8>>`.** No callee signature changes; no callee body edits beyond the inferred-type change at the `let entropy = ...` binding (which is implicit — no source edit needed).

**Implication:** The "update 7 callees" task in P1.T3 is **a no-op in source-text terms** for the callee bodies. The only source edits are:
1. `derive_entropy`'s signature line (line 27): `[u8; 64]` → `Vec<u8>`.
2. `derive_entropy`'s body lines 47-48: construct `Vec<u8>` of length 64 instead of `[u8; 64]`.
3. `bip85.rs` test module: 2 tests' `let e = derive_entropy(...)` inferred-type changes (transparent; tests' bodies don't change either — `&e[..16]` and `hex::encode(e)` both continue to work).
4. **Out-of-`bip85.rs`:** `tests/lint_zeroize_discipline.rs` evidence anchors (see §4 below).

This is a much narrower scope than the plan's "update 7 [or 6] callees" prose suggests. The implementer should expect a ~10-line diff inside `bip85.rs` and a ~3-line diff inside `lint_zeroize_discipline.rs`.

---

## §4. Test impact list

### 4.1 Direct callers of `derive_entropy` (in `bip85.rs` test module)

| # | Test | Location | Assertion | Post-swap status |
|---|---|---|---|---|
| 1 | `bip39_12_words_entropy_matches_spec` | `bip85.rs:346-349` | `assert_eq!(hex::encode(&e[..16]), "6250b68daf746d12a24d58b4787a714b")` | **Byte-equality.** Passes unchanged. `&e[..16]` works on both shapes. |
| 2 | `hex_64_bytes_entropy_matches_spec` | `bip85.rs:354-360` | `assert_eq!(hex::encode(e), "492db4...")` | **Byte-equality.** Passes unchanged. `hex::encode(e)` consumes by value; both `Zeroizing<Vec<u8>>` and `Zeroizing<[u8; 64]>` implement `AsRef<[u8]>` via zeroize's blanket impl (since `Vec<u8>: AsRef<[u8]>` and `[u8; 64]: AsRef<[u8]>`). Verified against zeroize crate docs. |

### 4.2 Indirect callers (via `format_*` wrappers in same test module)

| # | Test | Location | Assertion | Post-swap status |
|---|---|---|---|---|
| 3 | `pwd_base64_matches_spec` | `bip85.rs:364-367` | byte-equality on String output | Transparent |
| 4 | `pwd_base85_matches_spec` | `bip85.rs:370-373` | byte-equality on String output | Transparent |
| 5 | `dice_d6_10_rolls_matches_spec` | `bip85.rs:378-382` | byte-equality on String output | Transparent |
| 6 | `dice_d2_rolls_in_range` | `bip85.rs:385-392` | range-check on parsed u32 | Transparent |
| 7 | `dice_d256_rolls_in_range` | `bip85.rs:395-402` | range-check on parsed u32 | Transparent |
| 8 | `dice_sides_too_small_refused` | `bip85.rs:405-409` | error-shape match | Transparent |

### 4.3 Integration tests (via `mnemonic derive-child` CLI subcommand)

`crates/mnemonic-toolkit/tests/cli_derive_child.rs` — multiple `#[test]` functions invoking `derive-child` through `assert_cmd::Command`. Each asserts byte-equality of stdout (BIP-39 phrase / WIF / xprv / hex / base64 / base85 / dice rolls) and optionally stderr advisory presence. All **byte-equality on the CLI's stdout** — transparent to the internal type swap. Reproduces G7 wire-format-pin coverage at the CLI surface.

Other test files that invoke `derive-child` indirectly (verified via grep):
- `tests/cli_argv_leakage.rs` — asserts stderr advisory shape, not entropy bytes. Transparent.
- `tests/cli_secret_in_argv_warning.rs` — same. Transparent.
- `tests/cli_gui_schema.rs` — JSON/schema output. Transparent.
- `tests/lint_argv_secret_flags.rs` — lint discipline, doesn't run the binary against entropy. Transparent.

**All assertion shapes are byte-equality or error-shape, not type-shape.** Zero tests need body edits.

### 4.4 Lint test that DOES require updating (in lockstep with the swap)

`crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs:88-97` has TWO evidence anchors on `bip85.rs`:

```rust
ZeroizeRow {
    label: "bip85::derive_entropy returns Zeroizing<[u8; 64]>",
    source_file: "src/bip85.rs",
    evidence: &["-> Result<Zeroizing<[u8; 64]>"],   // <-- WILL RED post-swap
},
ZeroizeRow {
    label: "bip85 entropy locals scrub via derive_entropy's Zeroizing return",
    source_file: "src/bip85.rs",
    evidence: &["let mut out = Zeroizing::new([0u8; 64])"],   // <-- WILL RED post-swap
},
```

Both anchor substrings will disappear post-swap. **Both anchor strings must be updated in lockstep with the P1.T3 source change**, e.g.:

```rust
evidence: &["-> Result<Zeroizing<Vec<u8>>"],
// and
evidence: &["Zeroizing::new(vec![0u8; 64])"],   // or whatever the chosen Vec-construction shape is
```

Also: update both `label` strings to read `Vec<u8>` instead of `[u8; 64]`.

**This is the ONE outside-`bip85.rs` source-text edit Phase 1 must make.** The plan's file inventory at line 704 (`crates/mnemonic-toolkit/src/bip85.rs (P1: return-type swap + 6 callees)`) does NOT list `tests/lint_zeroize_discipline.rs` as a Phase 1 modification. Adding it now closes the gap before P1.T3 lands.

**Fold (corrected Phase 1 file inventory):**
- Modify: `crates/mnemonic-toolkit/src/bip85.rs` (signature + body construction; 7 callee bodies are transparent).
- Modify: `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` (2 evidence-anchor updates at lines 89-97).

---

## §5. P3a Site 4 forward sketch (read of `cmd/derive_child.rs`)

**Question to answer:** do any `format_*` callers retain the bip85 entropy beyond the immediate `format_*` call, or do all consume it immediately and emit a `String`?

**Read of `crates/mnemonic-toolkit/src/cmd/derive_child.rs:174-252`:**

Every `match args.application.as_str()` arm calls a single `bip85::format_*(...)` with `?` propagation and binds the **result `String`** to `output`:

```rust
let output = match args.application.as_str() {
    "bip39" => { ... bip85::format_bip39_phrase(...) ? }
    "hd-seed" => { ... bip85::format_hd_seed_wif(...) ? }
    "xprv" => { ... bip85::format_xprv_child(...) ? }
    "hex" => { ... bip85::format_hex_bytes(...) ? }
    "password-base64" => { ... bip85::format_password_base64(...) ? }
    "password-base85" => { ... bip85::format_password_base85(...) ? }
    "dice" => { ... bip85::format_dice_rolls(...) ? }
    ...
};
writeln!(stdout, "{output}").ok();
```

**`cmd/derive_child.rs` does NOT see the 64-byte entropy at all.** The `Zeroizing<Vec<u8>>` is created inside `bip85::format_*`, lives for the duration of that function (line 74 → line 81 in `format_bip39_phrase`, ~7 lines of lifetime), is borrowed once for the `from_entropy_in`/`from_slice`/encoder call, and drops at the closing brace of the `format_*` function.

**Forward guidance for Phase 3a Site 4:**

The entropy's lifetime is **strictly inside each `format_*` function** — there is no caller-side scope where the buffer escapes. The Phase 3a Site 4 wrapping point is therefore **inside `bip85.rs`**, at the `let entropy = derive_entropy(...)?;` site within each of the 7 `format_*` functions. The plan §"Phase 3a" T3 step 3 line 424 sketch (`let entropy = MlockedZeroizing::new(derive_entropy(index).into_inner());`) is the right shape — but it lives at the head of each `format_*` body, not at any callsite in `cmd/derive_child.rs`.

**Practical concern for Phase 3a (do not lock here):** 7 sites means 7 `MlockedZeroizing::new` calls, each allocating one page (typically 4 KiB on Linux/macOS) per `format_*` invocation. For a single-invocation CLI this is trivial; if the toolkit ever batches BIP-85 derivations in a tight loop (no current use case) the page-allocation cost would surface. Note for the Phase 3a R0 reviewer to weigh. Not actionable in Phase 1.

**Conclusion for Phase 1:** the heap-promotion design does not need to "leave room" for caller-side wrapping at `cmd/derive_child.rs`, because there is no caller-side buffer to wrap. The return-type lock (§1) is sufficient.

---

## §6. Risks and open questions

### 6.1 (Nit) Vec construction shape

The body change at `bip85.rs:47-48` from:

```rust
let mut out = Zeroizing::new([0u8; 64]);
out.copy_from_slice(mac.as_byte_array());
```

to a `Zeroizing<Vec<u8>>` equivalent has two reasonable forms:

```rust
// Option A: pre-sized vec
let mut out = Zeroizing::new(vec![0u8; 64]);
out.copy_from_slice(mac.as_byte_array());

// Option B: capacity + extend
let mut out: Zeroizing<Vec<u8>> = Zeroizing::new(Vec::with_capacity(64));
out.extend_from_slice(mac.as_byte_array());
```

Option A preserves the `copy_from_slice` discipline already used (lint anchor friendly) and gives the new evidence anchor a clean substring match. Option B uses `extend_from_slice` which is a fresh idiom. **Recommendation: Option A** for diff-minimalism. Implementer's choice; note for R1.

Confidence: 65 — this is preference, not correctness. Both are correct.

### 6.2 (Nit) `debug_assert!` placement

Plan T3 step 3 line 89 says "Add `debug_assert_eq!(result.len(), 64, ...)` immediately before return." Under Option A above (`vec![0u8; 64]`), `out.len()` is structurally 64 from line 1; under Option B (`Vec::with_capacity(64) + extend_from_slice(&[u8; 64])`) it's 64 after the extend. Either way the `debug_assert!` is a tautology on the happy path — but it self-documents the invariant and catches a future refactor that swaps `mac.as_byte_array()` for a non-64-byte source. **Keep the debug_assert as the plan specifies.** Confidence: 70.

### 6.3 (Open question for implementer, NOT a finding) Should P1.T2's RED test verify the new evidence anchors?

The `lint_zeroize_discipline.rs` test runs as part of the workspace test suite; updating the anchors in lockstep with the source change means the lint stays GREEN through P1.T3. But under strict TDD, P1.T2's RED commit could deliberately update the anchors first (making the lint RED until the source catches up), giving the implementer two RED gates to pass:

1. The new `derive_entropy_returns_zeroizing_vec_of_64_bytes` type-shape test (plan P1.T2 step 1).
2. The lint anchor RED.

**Recommendation:** the plan's P1.T2 should add a step ("Update `lint_zeroize_discipline.rs` evidence anchors to expect `Zeroizing<Vec<u8>>`") in the RED commit. This is consistent with TDD-first discipline and ensures the implementer doesn't forget the lint update.

Not a finding (the plan is correctable inline by the implementer reading this report); flagged as forward guidance.

### 6.4 (Open question) Does `into_inner()` exist on `Zeroizing<T>`?

The plan §"Phase 3a" T3 step 3 line 424 sketches `MlockedZeroizing::new(derive_entropy(index).into_inner())`. Verification: the `zeroize` crate's `Zeroizing<T>` does NOT expose a stable `into_inner()` method in recent versions (the type is `#[derive(Default)]` + `Deref`/`DerefMut` + Drop; the inner is private). The Phase 3a wrap pattern will need to be:

```rust
// Either: consume Zeroizing via DerefMut + std::mem::take (for Vec specifically)
let mut z = derive_entropy(...)?;
let vec = std::mem::take(&mut *z);   // leaves an empty Vec in z; z drops harmlessly
let entropy = MlockedZeroizing::new(vec);

// Or: clone-then-zeroize (cheap for 64 bytes but wasteful)
let z = derive_entropy(...)?;
let entropy = MlockedZeroizing::new(z.to_vec());
```

This is a **Phase 3a Site 4 implementation concern, not a Phase 1 concern.** Phase 1 only locks the return type; the unwrap pattern is the Phase 3a R0 reviewer's call. Flagged here because the plan's `into_inner()` sketch is misleading. Not a Phase 1 finding.

### 6.5 (Open question for the implementer) `extra_capacity_on_Vec`

`Vec<u8>` with `len == capacity == 64` has no excess capacity. If the constructor accidentally produces a Vec with `capacity > 64` (e.g., via `Vec::with_capacity(128)` + `extend`), the extra trailing bytes (uninitialized, never touched) are still inside the heap allocation that Drop zeroizes via `Vec::zeroize` (which zeros only `0..len`, not `0..capacity`). So uninitialized trailing capacity isn't zeroed.

For HMAC-SHA512 output (exactly 64 bytes, deterministic), this is moot — `len == capacity == 64` regardless. But the implementer should NOT pre-allocate excess capacity. Recommendation: Option A above (`vec![0u8; 64]`) eliminates the concern by construction. Confidence: 60 — defense-in-depth nit.

### 6.6 (Closed) Does `cargo build --tests 2>&1 | grep -i error` (plan P1.T2 step "Verification") still produce useful RED output?

Plan P1.T2 step "Verification" expects type-mismatch errors against `[u8; 64]`. Given the corrected understanding from I-R0-1 (signature is `Result<Zeroizing<[u8; 64]>, ToolkitError>`, callers borrow via Deref to slice), the RED commit MUST include some test that fails type-shape, e.g.:

```rust
#[test]
fn derive_entropy_returns_zeroizing_vec_of_64_bytes() {
    let m = master();
    let e: Zeroizing<Vec<u8>> = derive_entropy(&m, 39, &[0, 12], 0).unwrap();
    assert_eq!(e.len(), 64);
}
```

The `let e: Zeroizing<Vec<u8>>` type-ascription will produce a compile error against the current `Zeroizing<[u8; 64]>` return type. This is the RED-shape the plan's P1.T2 step 1 wanted; with the corrected signature, the cast in the test must include the four parameters.

---

## §7. Verdict

**LOCK — proceed to P1.T2.**

- **Return type locked:** `Result<Zeroizing<Vec<u8>>, ToolkitError>`.
- **Callee count corrected:** 7 (not 6); all use slice-borrow expressions transparent to the swap.
- **Test impact corrected:** 0 test bodies need editing; 2 lint evidence anchors in `tests/lint_zeroize_discipline.rs` need updating in lockstep with the source change.
- **File inventory corrected:** add `crates/mnemonic-toolkit/tests/lint_zeroize_discipline.rs` to Phase 1's modified-files list.
- **P3a Site 4 forward guidance:** entropy is fully internal to each `bip85::format_*` function; no caller-side wrap site exists in `cmd/derive_child.rs`. Site 4 wrapping happens inside `bip85.rs`, one wrap per format_* function.

No findings block the design lock. The 4 Important findings are narrative corrections folded into this report; the implementer of P1.T2/T3 reads §2 + §4 + §5 here to get the corrected picture.

R1 architect-review (P1.T4) checklist additions:
- Verify `tests/lint_zeroize_discipline.rs` evidence anchors updated in lockstep.
- Verify the actual diff is ~10 lines inside `bip85.rs` (no callee body edits) + ~3 lines in `lint_zeroize_discipline.rs`. If the diff is materially larger, the implementer has done something unexpected and the R1 reviewer should investigate.
- Verify byte-determinism (the plan's checklist line 117 already covers this).
- Verify the 7 callees are all transparent post-swap (no `as_ref` / `into` / `try_into` workarounds were introduced).
