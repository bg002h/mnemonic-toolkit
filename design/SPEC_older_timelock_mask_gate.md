# SPEC — build-descriptor `older()` BIP-68 mask gate (silently-weakened-timelock funds-safety fix)

**Cycle:** toolkit PATCH (v0.53.8 → **v0.53.9**) · **Source SHA:** `a7c1920` · **Recon:** deep-recon 2026-06-11 (this thread, agent `a563db3e`).
**Resolves:** `gate-after-older-upper-bound-deferred-to-step2` (reframed from cosmetic [obs] → **funds-safety fix-now**; the recon FALSIFIED the entry's "fail-closed" premise for `older()`).

## The bug (empirically reproduced @ `a7c1920`)
`build-descriptor`'s step-1 field gate (`descriptor_builder/gate.rs:244-256`) validates `older(N)` with only `*n == 0 || *n >= (1u32 << 31)`. This **ignores that BIP-68 consensus masks the CSV operand to `0x0040FFFF`** — only the low 16 bits are the value and bit 22 (`0x400000`) selects 512-second units; **bits 16–21 and 23–30 are silently dropped by consensus**, and a zero 16-bit value field is a zero-delay (no-op) timelock. So these all build **exit 0** into a checksummed, engraving-ready descriptor with a silently-weakened or nullified recovery timelock:

| Input | Consensus-effective | Harm |
|---|---|---|
| `older(105120)` ("2-year vault" in blocks) | `105120 & 0xFFFF` = 39584 blocks (~275 days) | shortened delay |
| `older(65536)` | masks to **0** | **recovery branch spendable immediately** |
| `older(4194304)` (`0x400000`, bit-22 only) | time-type, value **0** | **zero delay** |

This is the exact silently-weakened-spend-condition class the 4-step funds-safety gate exists to prevent, made concrete by the shipped archetype presets advertising "recovery after N **blocks**" (2 years = 105120 overflows the 16-bit field). `after(N)` is NOT affected — the recon empirically confirmed step-2 `from_str` rejects `after(N>0x7FFFFFFF)` with a clean node-localized error (BIP-65 absolute locktimes are not bit-masked), so the after-half is genuinely fail-closed/cosmetic.

## Correct `older(N)` authoring domain (the fix predicate)
Accept iff: only bit-22 (`0x400000`) and/or the low 16 bits may be set, AND the 16-bit value is non-zero.
**REJECT iff:** `(*n & !0x0040_FFFFu32) != 0 || (*n & 0x0000_FFFFu32) == 0`.

- Subsumes the current `n == 0` (value-zero) and `n >= 2^31` (bit-31 garbage) checks — strictly stronger.
- **The `(n & 0xFFFF) == 0` clause is load-bearing and was MISSING from the recon's proposed one-liner** (`n & !0x0040FFFF != 0` alone): it is what catches `older(0x400000)` (bit-22 set, value 0 → garbage_bits==0 → would otherwise pass). Verified on the edge-value table below.
- Accepts: `1..=65535` (blocks); `0x400001..=0x40FFFF` (512-second units, bit-22 + non-zero value).
- Rejects: 0, 65536, 105120, 0x400000, 0x80000000, 0xFFFFFFFF, 500000000, 0x7FFFFFFF.

Edge-value table (Python-verified @ recon time):
```
older(0)→REJECT  older(1)→accept  older(65535)→accept  older(52560)→accept  older(4032)→accept  older(144)→accept
older(65536)→REJECT  older(105120)→REJECT  older(0x400000)→REJECT  older(0x400001)→accept  older(0x40FFFF)→accept
older(0x80000000)→REJECT  older(0xFFFFFFFF)→REJECT  older(500000000)→REJECT  older(0x7FFFFFFF)→REJECT
```

**Zero false-reject risk (R0-r1 M4 — full enumeration):** ALL archetype preset descriptors (`mod.rs:56-88`: 65535, **1000, 2000**, 52560, 4032, 144) + all 6 fixtures + every `older(` in `tests/` are in the accepted domain (gate.rs tests: 1, 5, 7, 100, 144, 52560, 65535). The only >16-bit-value `older()` literals in the repo are `older(4194305)` (`cli_compare_cost.rs:889`) and `older(32768)` (BSMS intake fixtures) — both on *intake* paths (not the build-descriptor gate) AND both accepted by the new predicate anyway (`4194305 = 0x400001` = valid 512s-unit; `32768 < 65536`). `build-descriptor` is an *authoring* surface (JSON IR in), not a round-trip importer → no existing descriptor intake is affected; no golden breaks.

## PART A — tighten `older(N)` (the funds-safety fix)
In `descriptor_builder/gate.rs::validate_fields` `PolicyNode::Older(n)` arm, replace the condition with the reject predicate above. The message states the BIP-68 encoding plus what consensus would silently do with the input, so the footgun is concrete. **The "what consensus does" clause MUST branch on the bit-31 disable flag (R0-r1 I1):** per BIP-112, a CSV operand with bit-31 set is a **no-op (no timelock at all)**, NOT a masked value — printing "effective value of N blocks" for such an input is consensus-FALSE and unacceptable in a funds-safety diagnostic.
```
older(N) encodes a BIP-68 relative timelock: only the low 16 bits are the value, and bit 22 (0x400000) selects 512-second units. All other bits — including the bit-31 disable flag — must be clear, and the 16-bit value must be non-zero. got {n} (0x{n:08x}); <CONSEQUENCE>. Use 1..=65535 (blocks) or 0x400000|(1..=65535) (512-second units).
```
where `<CONSEQUENCE>` is:
- if `n & 0x8000_0000 != 0`: `the bit-31 disable flag is set, so consensus would treat this CHECKSEQUENCEVERIFY as a no-op — no relative timelock at all`
- else: `consensus would silently mask this to an effective value of {n & 0xFFFF}{ " (512-second units)" if (n & 0x400000) else " blocks" }, weakening or nullifying the timelock`

(Implementer: compute `<CONSEQUENCE>` from `n` — bit-31-clear arithmetic is consensus-accurate incl. unit selection by the original bit-22; keep it one diagnostic via `field_diag`. No overflow risk — plain u32 masking.)

## PART B — tighten `after(N)` upper bound (cosmetic; closes the slug as literally titled)
In the `PolicyNode::After(n)` arm, ADD `*n > 0x7FFF_FFFF` to the existing `*n == 0` rejection (keep the n==0 message; add an upper-bound message). This is **behavior-equivalent** — step-2 already rejects `after(N>0x7FFFFFFF)` — it only moves the error earlier with a field-localized message, for parity with the older() precision. BIP-65 absolute locktimes are not bit-masked, so 1..=0x7FFFFFFF is the full valid domain; the 500M height/time split and cross-branch mixing are out of scope (mixing already caught at step-3 `has_mixed_timelocks`).

## Tests (TDD, RED-first via scratch-revert)
**Test-construction discipline (R0-r1 M2):** a height `older()` and a time `older()` (or two `after()` on opposite sides of the 500M split) in ONE tree trip step-3 `HeightTimelockCombination`, contaminating a "no field diag" assertion. Put **one timelock value per tree** (reuse the `and_v(v:pk(A), older(N))` fixture pattern at `gate.rs:804`) or call `validate_fields` directly for the no-diag half. Reject cells assert **`kind == DiagnosticKind::SchemaField`** (the step-1 provenance), not just `message.contains("older")` — the step-1-vs-step-2 distinction is the whole point of PART B.

Add to `gate.rs` `#[cfg(test)]` (alongside `rejects_zero_timelock` at `:803`):
- **`rejects_masked_older_timelocks`** — older(65536), older(105120), older(0x400000), **older(0x80000090)** each produce a `DiagnosticKind::SchemaField` diag (mention "older"). **RED-proof asymmetry (R0-r2 M-A):** the first three produce NO field diag before the fix (their RED is diag-existence). `older(0x80000090)` is ALREADY rejected pre-fix by the current `n >= 2^31` check — its RED component is the **wording assertion only** (the pre-fix message uses the old `1 ≤ N < 2^31` text and lacks the bit-31 no-op wording). So: the `0x80000090` cell asserts the message contains the **bit-31 disable / "no-op"** wording (pins R0-r1 I1's branch, RED on the old text); a masked case (e.g. older(65536)) asserts the **"effective value"** wording — both branches exercised, and the scratch-revert RED check must read the `0x80000090` cell's RED as the wording mismatch, not diag-absence.
- **`accepts_valid_older_block_and_time`** — older(1), older(65535), older(52560), older(0x400001), older(0x40FFFF) — **each in its own single-timelock tree** — produce NO `SchemaField` diag; at least one builds to a descriptor end-to-end (exit-0 path intact).
- **`rejects_after_above_max`** — after(0x80000000), after(0xFFFFFFFF) produce a step-1 `SchemaField` diag (NOT a step-2 type_error); after(0) keeps its message; after(1), after(500000000), after(0x7FFFFFFF) accepted (own trees).
- Existing `rejects_zero_timelock` unchanged (asserts `SchemaField && contains("older"/"after")` — message reword safe).
- **CLI-level integration** (`tests/cli_build_descriptor*.rs`): (1) `build-descriptor --spec` with `older(65536)` in a recovery branch exits non-zero with the field diagnostic (gate fires on the real binary, mirroring the recon repro); (2) **preset-path cell (R0-r1 M3):** `--archetype kofn-recovery --older 105120` (the SPEC's own "2-year vault" headline) exits non-zero with a diagnostic carrying `--older` provenance (provenance attaches via the `node_path`/`None`-kind tables, `archetype.rs:128-137`).

## SemVer + lockstep
- **PATCH** v0.53.8 → **v0.53.9** (tightens input validation = bug fix; rejects previously-accepted-but-silently-wrong input; no flag/subcommand/wire/JSON-grammar change).
- **NO `schema_mirror` / GUI / sibling-codec lockstep** — no clap flag-NAME change; the `"older"`/`"after"` JSON grammar (schema.rs:31) is unchanged (still `uint`).
- **CHANGELOG.md** `[0.53.9]` entry (fix). **3 self-pin sites:** `README.md:13`, `crates/mnemonic-toolkit/README.md:9`, `scripts/install.sh:32`.
- **Manual prose accuracy:** update the `older()` domain line at `docs/manual/src/40-cli-reference/41-mnemonic.md:4009` (currently `1 ≤ N < 2³¹` — inaccurate post-fix) to the masked domain (low-16 value + bit-22 unit-flag; malformed BIP-68 encodings rejected). **The `--older` flag prose (`:3993`) sentence MUST be honest about the deferred M1 window (R0-r2 I-A): do NOT claim a blanket ">65535 rejected"** — that is false for the accepted `0x400001..=0x40FFFF` 512-second-unit window. Write instead: "interpreted as **blocks**; malformed BIP-68 encodings are rejected, but valid 512-second-unit encodings (`0x400001`–`0x40FFFF`) are currently accepted and reinterpreted as time units (tracked: `archetype-older-blocks-flag-accepts-time-units`)" — or drop the `--older`-row addition entirely and let the `:4009` domain line carry accuracy. NOT a flag-coverage gate trigger (no flag change) but a docs-accuracy fix; run the manual lint to be safe.

## Ritual / FOLLOWUPS
**Resolve** `gate-after-older-upper-bound-deferred-to-step2`. **File two new FOLLOWUPs (R0-r1 M1, M5):**
- `archetype-older-blocks-flag-accepts-time-units` (M1) — the preset `--older`/`--recovery-older` flags are documented "blocks" (`ParamKind::Blocks`) but post-fix still accept the narrow `0x400001..=0x40FFFF` window (valid BIP-68 512-second-unit values), silently reinterpreting an ~80-year-implausible "block count" as a ~days time-lock. Exposure near-nil. Fix would bound `ParamKind::Blocks` to `1..=65535` at the preset layer — but `validate_params` has a deliberate **"does not duplicate gate rules"** boundary (`archetype.rs:711` test), so this needs its own design call (preset-semantic vs gate-rule). Deferred, not folded, to keep this cycle within the gate boundary.
- `intake-surfaces-accept-masked-older-no-advisory` (M5) — intake/round-trip surfaces (`xpub-search`/`import-wallet`/`export-wallet --descriptor`/`compare-cost`) still accept masked `older()` via miniscript `from_str` (correct — must not block import of already-deployed wallets), but emit no advisory that the imported descriptor carries a consensus-weakened timelock. Future: an advisory (non-blocking) warning on verify/import surfaces.

Also fold in (same commit, recon dispositions already drafted in FOLLOWUPS working tree): `addresses-env-sentinel-overapplied` → WONTFIX (benign quirk), `two-miniscripts-patch-load-bearing-stale-error` → WONTFIX-in-toolkit (md-codec hygiene, already tracked). Stage paths explicitly. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Non-goals
- `after()` height/time 500M-boundary semantics or cross-branch mixing (already gated at step-3).
- Rejecting time-based `older()` entirely in spec-mode (bit-22 values are valid BIP-68; we accept the non-zero-value ones). The preset-flag blocks-only tightening is deferred to the M1 FOLLOWUP.
- Any change to descriptor *intake*/round-trip paths (this is the authoring/build surface only) — tracked as the M5 FOLLOWUP.
