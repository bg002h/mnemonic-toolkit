# SPEC — doc+test hygiene: fix the cell-27 shared-temp race + clarify the md1 "chunk_set_id" terminology

**Cycle:** toolkit NO-BUMP (test + docs only) · **Source SHA:** `be1a581` · **Recon:** ad-hoc (this cycle).
**Resolves:** `cell-27-verify-bundle-auto-fire-tty-flaky-on-macos` (Part A) + `anti-collision-16bit-invariant-false` (Part B). Two independent NO-BUMP items.

No binary/wire/CLI change → no `schema_mirror` / GUI / sibling / version bump / CHANGELOG (NO-BUMP). Part B touches `docs/technical-manual/` (fires `technical-manual.yml` symbol-ref-check — keep all `file.rs::symbol` anchors valid).

---

## PART A — cell-27 (and siblings) shared-temp-file race

### Problem (verified @ `be1a581`)
`write_temp_json` (`tests/cli_auto_repair.rs:412-419`) names the temp file `mnemonic_v0_22_1_bundle_{std::process::id()}.json` — **process-id only**. All tests in a test binary run as THREADS in ONE process, so the 5 callers (`cell_27`/`cell_28`/… at `:427,455,483,512,…`) ALL write the SAME path. Under parallel execution one test's `cargo_bin` subprocess reads the file while another test is mid-write/truncating it → "EOF while parsing a value at line 1 column 0" (empty file). This flaked the macos `test` job on the Cycle-A push (passed on rerun). Same class as the GUI shared-temp torn-content trap.

### Design
Make each `write_temp_json` call produce a UNIQUE path: add a process-global atomic counter to the filename.
```rust
use std::sync::atomic::{AtomicU64, Ordering};
static TMP_SEQ: AtomicU64 = AtomicU64::new(0);
fn write_temp_json(body: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "mnemonic_v0_22_1_bundle_{}_{}.json",
        std::process::id(),
        // Relaxed: we need distinct values, not cross-thread ordering (m1).
        TMP_SEQ.fetch_add(1, Ordering::Relaxed),
    ));
    std::fs::write(&path, body).unwrap();
    path
}
```
(`tempfile = "3"` IS a dev-dep; `NamedTempFile` is an alternative but changes the return type + every caller's path handling — the counter is the minimal, no-API-change fix. Pre-existing non-cleanup of the temp files is unchanged + out of scope.)

### Part A test/verification
The fix is the deliverable; the 5 cells already exercise `write_temp_json`. **Verify:** run `cargo test -p mnemonic-toolkit --test cli_auto_repair` (all green); the uniqueness is structural (distinct counter per call). A targeted stress (run the test binary a few times) confirms no collision. RED is the FLAKE itself (intermittent), so this is a structural fix — note that the pre-fix shared path is the demonstrated race.

---

## PART B — clarify the md1 `chunk_set_id` terminology (the "invariant" is TRUE for engraved ids)

### Problem (verified @ `be1a581`)
The FOLLOWUP framed the manual's "leading 16 bits of `chunk_set_id` agree across all three cards" as FALSE. **Ground truth (verified):** the ENGRAVED card identifiers the toolkit PRINTS are BOTH policy-id-derived — `bundle.rs::build_unified_card`: md1 = `compute_wallet_policy_id(d).as_bytes()[0..2]` (16 bits, `:1071-1078`); mk1/ms1 = `derive_mk1_chunk_set_id_for_slot(policy_id[0..4], slot)` (20 bits, `:1092-1103`). So the engraved leading-16-bits genuinely AGREE — the human-facing binding claim is CORRECT. The imprecision is TERMINOLOGY: md1's *wire* `chunk_set_id` (the field inside the md1 wire string, md-codec) is a DIFFERENT value, derived from `Md1EncodingId` = SHA-256 of the canonical bytecode (per the manual's own `21-md1-wire-format.md:50` + `glossary.md:221`), and does NOT share 16 bits with mk1's wire csi. The manual overloads "chunk_set_id (md1)" for BOTH the engraved policy-id binding AND the md1 wire field — so a reader who extracts md1's wire chunk_set_id and compares would (correctly) find disagreement, contradicting the stated invariant.

### Design — clarify, don't reverse (R0: verify the ground truth before editing)
Correct the prose at the conflation sites to distinguish:
- the **engraved bundle-binding identifiers** (policy-id-derived, leading-16-agree — `bundle.rs::build_unified_card`) — the cross-card binding the invariant is about, AND
- the **md1 wire `chunk_set_id`** (`Md1EncodingId`/bytecode-derived, md-codec-internal chunk-grouping) — a SEPARATE value that is NOT the cross-card binding and does NOT agree with mk1's wire csi.

**R0-r1 confirmed the ground truth** (engraved md1 = `compute_wallet_policy_id[0..2]`; mk1/ms1 = `derive_mk1_chunk_set_id_for_slot(policy_id[0..4], slot)`; both policy-id-derived → engraved leading-16 AGREE; md1 WIRE csi = `Md1EncodingId`/bytecode-derived = a SEPARATE value; `chunk_set_id_extract` is used for MK1 only, per `41:138`). So this is a TERMINOLOGY clarification, NOT an invariant reversal.

Sites (re-grep at write time; keep every `file.rs::symbol` anchor resolvable for `technical-manual.yml` symbol-ref-check):
- `docs/technical-manual/src/40-bundle-formation/42-anti-collision-invariants.md:19` (R0-r1 I1) — REMOVE the dangling "...see the `anti-collision-16bit-invariant` note in `design/FOLLOWUPS.md`" forward-ref (the slug is resolved by this cycle) and REPLACE with the definitive statement: *"The md1 **engraved** identifier (4 hex = `policy_id[0..2]`, printed on the card) shares the same leading 16 bits. The md1 **wire** `chunk_set_id` (20 bits from `Md1EncodingId` = SHA-256 of the canonical bytecode, in the chunked-string headers) is a DIFFERENT value and does NOT necessarily agree with mk1's wire csi — expected: the wire field serves md1 chunk-grouping at reassembly, not cross-card binding. Cross-card binding uses only the engraved display identifiers."* Also tighten `:17`'s "leading 16 bits unchanged" wording to say "engraved" where it means the displayed id.
- `docs/technical-manual/src/40-bundle-formation/41-bundle-anatomy.md:138,193` — TRUE as written; add the "**engraved** identifier" qualifier so a reader doesn't conflate it with the md1 wire `chunk_set_id`. (m2: `:193` cites `bundle.rs::build_unified_card` twice — collapse to one anchor "(for both)".)
- `docs/technical-manual/src/60-back-matter/61-glossary.md:89` — the `chunk_set_id (cross-card binding)` entry: clarify it is the ENGRAVED policy-id identifier, distinct from the md1 wire `chunk_set_id` (glossary:221). (m3: `glossary.md:57` "`chunk_set_id` cross-prefix agreement" is also unqualified — add "engraved" / "(engraved display ids)" there too.)
- `CHANGELOG.md:2009` (R0-r1 I2 — the SPEC's `:1899` was WRONG, unrelated BIP-86 line) — the §IV.2 release-history line ("leading 16 bits agree across all three cards from one bundle") is TRUE for engraved ids; add a short parenthetical `(engraved display identifiers; md1's wire chunk_set_id is Md1EncodingId-derived, a separate value)` rather than rewording — it's release history.

### Part B verification
`make -C docs/technical-manual lint` (or the symbol-ref-check the CI runs) passes post-edit; the corrected prose is internally consistent with `21-md1-wire-format.md:50` (md1 wire csi = Md1EncodingId) + `bundle.rs::build_unified_card` (engraved = policy_id).

---

## Ritual
NO version bump / CHANGELOG (NO-BUMP test+docs). FOLLOWUPS resolve both slugs. Stage paths explicitly. Mandatory R0 gate to 0C/0I; persist reviews to `design/agent-reports/`.

## Non-goals
The zeroize-row audit (separate cycle, in progress); md1 WIRE-csi changes (it's a md-codec value, out of scope); temp-file cleanup in cli_auto_repair (pre-existing).
