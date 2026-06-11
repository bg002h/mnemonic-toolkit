# Impl Review — older() timelock mask gate — PRE-COMMIT (v0.53.9)

**Verdict: 🟢 — 0 Critical / 0 Important / 0 Minor.** Ready to commit.

All seven review items verified against the working tree:

**1–2. Predicate** (`descriptor_builder/gate.rs`, `PolicyNode::Older` arm): implemented condition is character-exact `(*n & !0x0040_FFFFu32) != 0 || (*n & 0x0000_FFFFu32) == 0` — correct mask constants, correct `||`. Bit-31 branch tests `*n & 0x8000_0000 != 0`; unit suffix selects on `*n & 0x0040_0000`. Traced edges: rejects 0 (clause 2), 65536/0x10000 (garbage bit-16), 105120→"39584 blocks", 0x400000 (bit-22 + zero value — the load-bearing clause-2 catch), 0x80000090 (bit-31 branch); accepts 1, 0x8000 (=32768, valid block count), 65535, 0x400001, 0x40FFFF. No value wrongly accepted (acceptance domain ≡ bits ⊆ 0x0040FFFF ∧ low16 ≠ 0, exactly valid BIP-68). Pure u32 masking — no overflow.

**3. Messages**: bit-31 branch says "no-op — no relative timelock at all" and contains no "effective value" (the test also asserts its absence); else branch prints `*n & 0x0000_FFFF` with " (512-second units)"/" blocks" keyed on the original bit-22 — consensus-accurate. All asserted substrings (`older`, `effective value`, `no-op`, `disable flag`) present. `after(0)` message preserved; new `after` arm states `1..=0x7fffffff`.

**4. Tests genuine**: pre-fix condition accepts 65536/105120/0x400000 → those cells are RED-by-diag-absence pre-fix; the 0x80000090 cell's no-op-wording assertion is RED against the old text (R0-r2 M-A asymmetry honored in a comment). `field_diags` filters `DiagnosticKind::SchemaField` and returns `[]` on `gate(...) → Ok` — no panic on success, unlike `errs`. Accept cells use one timelock per tree (M2) + one end-to-end `gate(...).is_ok()`. `rejects_after_above_max` is RED pre-fix because step-2 rejection is type_error, not SchemaField. CLI cells exercise both `--spec` (exit 2, stderr `schema_field`/`effective value`) and preset (`--older 105120`, JSON `flag == "--older"` provenance). `mixed_timelock_spec()` 4194304→4194305 keeps bit-22 (time) vs older(100) (height) — mix preserved.

**5. Regression sweep — complete**: `PolicyNode::Older/After` constructed only in `descriptor_builder/{archetype,ir,gate}.rs`. Full repo grep (incl. hex literals + `fixtures/*.json`): every gate-reachable `older` ∈ {1,5,7,100,144,1000,2000,4032,52560,65535,4194305} and `after` ∈ {100,500000,600000000} — all accepted (600000000 ≤ 0x7FFFFFFF). The `older(32768)`/`after(12000000)`/`after(1500000000)` hits are intake-path (parse_descriptor/wallet_import/compare-cost — miniscript `from_str`, never the gate) and accepted by the predicate anyway.

**6. after() bounds**: `else if *n > 0x7FFF_FFFF` — strict greater-than, so 0x7FFFFFFF and 500000000 pass step-1.

**7. Empirical**: 962/962 bin unit tests, 44/44 `cli_build_descriptor`, 51/51 `cli_compare_cost` green. Lockstep complete: Cargo.toml 0.53.9, both README markers, install.sh self-pin, CHANGELOG `[0.53.9]`, manual `:4006-4012` domain prose corrected (cspell clean; the `--older`-row addition dropped per the SPEC's allowed alternative, avoiding the R0-r2 I-A false-claim trap), FOLLOWUPS resolve + 2 carve-outs + 2 WONTFIX folds present.
