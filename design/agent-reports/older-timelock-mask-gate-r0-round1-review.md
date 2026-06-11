# R0 Review — older() timelock mask gate — ROUND 1

**Source SHA:** `a7c1920` (verified == local HEAD) · **SPEC:** `design/SPEC_older_timelock_mask_gate.md`
**Verdict:** 🟡 — **0 Critical / 1 Important / 5 Minor** (one fold round needed; the Important is a one-line message-branch fix)

## Predicate + fail-closed INDEPENDENTLY VERIFIED

I did not take the SPEC's word on any load-bearing claim:

- **`older()` is NOT fail-closed downstream — confirmed from pinned miniscript source AND a fresh empirical repro.** The pinned checkout (`~/.cargo/git/checkouts/rust-miniscript-ce5fa57e8900265e/95fdd1c`) parses `older` via `expression/mod.rs:363-372 verify_older` → `RelLockTime::from_consensus` → `TryFrom<Sequence>` (`src/primitives/relative_locktime.rs:71-79`), which accepts **iff `seq.is_relative_lock_time() && seq != ZERO`** — i.e. *any nonzero value with bit-31 clear*, including all masked-garbage values. Steps 3/4 cannot catch them (the five `ext_check` arms are sigless/malleable/resource/repeated/mixed only). I rebuilt the binary at `a7c1920` and reproduced: `older(65536)`, `older(105120)`, `older(4194304)` each emit a **checksummed engraving-ready descriptor, exit 0**. The funds-safety framing holds.
- **`after()` IS fail-closed at step-2 — confirmed.** `verify_after` → `AbsLockTime::from_consensus` (`src/primitives/absolute_locktime.rs:10,51-56`) bounds `[1, 0x7FFF_FFFF]` (`MAX_ABSOLUTE_LOCKTIME = 0x7FFF_FFFF`). Empirically `after(2147483648)` exits 2 with a node-localized `type_error` at `root.or_d[1].and_v[1]`. PART B is genuinely cosmetic/behavior-equivalent.
- **Predicate complete — verified by independent bit arithmetic (python3).** All 15 SPEC table values reproduce exactly. The second clause is load-bearing exactly as claimed: `0x400000 & !0x0040FFFF == 0` (first clause alone does NOT reject `older(0x400000)`). The predicate strictly subsumes the current `n == 0 || n >= 2^31` (`gate.rs:245`) — a scan found no value the old check rejects that the new one accepts. `!0x0040_FFFFu32 = 0xFFBF_0000`; no overflow/panic risk; `n & 0xFFFF` in the message is plain u32 masking.
- **Accepting 512-second units (`0x400001..=0x40FFFF`) is the right call.** They are exactly the canonical BIP-68 encodings; miniscript renders and round-trips them; rejecting all bit-22 values would false-refuse legitimate time-based vaults and is a capability removal beyond a PATCH. The residual risk is the narrow reinterpretation window (M1 below).
- **Zero false-reject — verified by exhaustive grep, with corrections (M4).** All preset descriptors (`mod.rs:56-88`: 65535, 1000, 2000, 52560, 4032, 144), all 6 fixtures, and every `older(` in `tests/` are in the accepted domain. The only >16-bit test value in the repo is `older(4194305)` (`cli_compare_cost.rs:889`) — an *intake* path AND accepted by the new predicate anyway. No golden breaks.
- Citation spot-checks: Older/After arms at `gate.rs:244-256` ✓, `field_diag` :609 ✓, `rejects_zero_timelock` :803 (reword-safe — asserts only `contains("older")`/`contains("after")`) ✓, `schema.rs` grammar `"older": uint` unchanged ✓, manual prose `older 1 ≤ N < 2³¹` at `docs/manual/src/40-cli-reference/41-mnemonic.md:4009` ✓, self-pin sites `README.md:13` / `crates/mnemonic-toolkit/README.md:9` / `scripts/install.sh:32` all `0.53.8` and **complete** (full-repo grep found no 4th prose pin besides `Cargo.toml`) ✓, all three FOLLOWUP slugs exist at base SHA (FOLLOWUPS.md:25/:46/:68) ✓, `tests/cli_build_descriptor.rs` exists ✓.

## Critical

None.

## Important

**I1 — The new diagnostic's "consensus-effective value" claim is consensus-FALSE for the bit-31-set subdomain.** SPEC §PART A message (SPEC:38) unconditionally states "consensus would silently mask this to an effective value of `{n & 0xFFFF}` …". Per BIP-112, when the **script operand's** bit-31 disable flag is set, `CHECKSEQUENCEVERIFY` behaves as a **NOP** — no timelock at all, not a masked value. The new arm fires for these inputs (first clause rejects all `n ≥ 2^31`), so e.g. `older(0x80000090)` would print "effective value of 144 blocks" when the truth is **no lock whatsoever**. A factually wrong consensus claim inside the very diagnostic this fix exists to make precise is not acceptable in a funds-safety tool, even on a rejected input. **Fix direction:** branch the message: if `n & 0x8000_0000 != 0` → "the bit-31 disable flag is set; consensus would treat this CHECKSEQUENCEVERIFY as a no-op (no relative timelock at all)"; else the masked-value sentence as drafted. (For bit-31-clear values the drafted arithmetic is consensus-accurate, including unit selection by the original bit-22 — verified against BIP-68/112 mask semantics.) Add `older(0x80000090)` (or similar bit-31+value input) to the reject cell asserting the disable-flag wording.

## Minor

**M1 — Residual silent-unit-flip window on the preset surface.** `--older` is documented "relative timelock (**blocks**)" (`41-mnemonic.md:3993`; `ParamKind::Blocks`, `archetype.rs:121`) yet post-fix `--older 4200000` (=`0x401640`) is accepted and reinterpreted as **5696 × 512s ≈ 33.8 days** (verified). Values in `4194305..=4259839` are implausible as intended block counts (~80 years), so funds exposure is near-nil, but it is the same silent-unit class one notch narrower. Direction: bound `ParamKind::Blocks` params to `1..=65535` at `archetype::validate_params` (preset-only; doesn't restrict spec-mode JSON) — either fold in or file a FOLLOWUP; at minimum note the window in the manual prose update.

**M2 — Test-construction trap in the accept cells.** Putting older(1) (height) and older(0x400001) (time) — or after(100) and after(500000000) — in ONE tree trips step-3 `HeightTimelockCombination` and contaminates the "no field diag" assertion. Use one sane tree per value (the existing `and_v(v:pk(A), older(N))` fixture pattern at `gate.rs:804`) or call `validate_fields` directly for the no-field-diag half. Also have the reject cells assert `kind == DiagnosticKind::SchemaField` (not just message `contains("older")`), pinning step-1 vs step-2 provenance — that distinction is the whole point of PART B's RED proof.

**M3 — Consider one preset-path cell** (`--archetype kofn-recovery --older 105120` → non-zero exit, diagnostic carries `--older` provenance). The provenance tables map by `node_path` with `None` kind filter (`archetype.rs:128-137` etc.), so attribution attaches automatically — a cheap cell that pins the SPEC's own "2-year vault" headline narrative on the user-facing path.

**M4 — SPEC enumeration inaccuracies (fix prose so future greps don't read it as exhaustive).** SPEC:33 omits preset values `older(1000)`/`older(2000)` (`mod.rs:64`, decaying-multisig) and gate.rs test values 100/52560; and the repo's tests DO contain `older(4194305)` (`cli_compare_cost.rs:889`, intake + accepted) and `older(32768)` (BSMS intake fixtures) — conclusion (zero false-reject) unchanged, but say so explicitly.

**M5 — Scope confirmation, no change requested:** intake/round-trip surfaces (`xpub-search`/`import-wallet`/`export-wallet --descriptor`, `compare-cost`) still accept masked `older()` via miniscript `from_str` (`wallet_import/*`, `parse_descriptor.rs`, `descriptor_intake.rs`). Leaving them open is correct — blocking import of an already-deployed wallet would be wrong — but consider a future advisory-warning FOLLOWUP so verify/import surfaces flag pre-existing weakened descriptors.

## Scope

PATCH v0.53.9 correct (validation-tightening fix; precedent holds), **no** schema_mirror/GUI/sibling lockstep (no flag-name change, `schema.rs` grammar unchanged; GUI Validate shells out to the binary so it inherits the gate), manual prose line `41-mnemonic.md:4009` + CHANGELOG + 3 verified self-pin sites — coherent as one cycle.
