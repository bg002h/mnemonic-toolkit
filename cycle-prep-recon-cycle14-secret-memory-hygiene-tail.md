# Cycle-prep recon — secret-memory-hygiene tail (cycle-14 candidate)

**Recon only — no implementation.** Verifies the two remaining "secret-memory-hygiene-cycle-b tail"
candidates against current origin source.

- **Origin SHA verified against:** `origin/master = 7057b5cc` — *"release(cycle-13): toolkit v0.66.0 —
  fidelity tail (H11/H14/L8/L9/M1/M7/L18)"*.
- **Working tree** is on `feature/own-account-subset-search` (v0.60.0-era) — working-tree line numbers were
  NOT trusted; every citation below was re-grepped via `git show origin/master:<path>`.
- **Date:** 2026-06-21.

---

## TL;DR verdict

| Item | Status | Reproduces? |
|---|---|---|
| **Item 1 — L22** (`apply_slot_stdin` + `read_stdin_passphrase` bare `String`) | **OPEN** | **YES — both sites still bare `String`** |
| **Item 2 — FOLLOWUP `resolved-slot-derived-account-zeroizing-field`** | **ALREADY SHIPPED (v0.10.1, `ed5a1d9`)** | **NO — both fields already `Zeroizing`** |

The task framing called item 2 "the open FOLLOWUP." It is **not open** — origin/master FOLLOWUPS.md marks it
`resolved ed5a1d9` and both target fields are already migrated in source. **The tail is just L22 (+ its
sibling un-wrapped-`String` sites).** Recommend a single small toolkit-only PATCH cycle.

---

## Item 1 — L22: stdin secrets read into un-scrubbed `String`

### 1. WHAT
The `@N.<secret>=-` / `--*-stdin` sentinels keep secrets off argv, but the stdin readers materialize the
secret into a plain `String` (no `Zeroize`/`Drop`), so it lingers un-scrubbed (and, except where mlock-pinned,
swappable) in the heap until natural drop. Fix direction: make the readers return `Zeroizing<String>` (or wrap
`SlotInput.value` as `Zeroizing<String>` / a `SecretString`-style field).

### 2. Citations (re-grepped vs `7057b5cc`)

| Report citation | Live location | Tag |
|---|---|---|
| `slot_input.rs:203-232` (`apply_slot_stdin`) | `slot_input.rs:203-232` — fn spans `203`→`232` exactly | **ACCURATE** |
| `slot_input.rs:96-101` (`SlotInput`/`value`) | `slot_input.rs:97` `pub struct SlotInput`; `:100` `pub value: String` | **ACCURATE (off-by-1; struct at 97 not 96)** |
| `cmd/convert.rs:747` (`read_stdin_passphrase`) | `convert.rs:758` `pub(crate) fn read_stdin_passphrase` (body `758-765`) | **DRIFTED-by-11** (still reproduces) |

All citations still point at live, un-wrapped code. **L22 STILL REPRODUCES.**

### 3. Current types in source (the load-bearing facts)

- `slot_input.rs:97-101` — `pub struct SlotInput { pub index: u8, pub subkey: SlotSubkey, pub value: String }`.
  `value` is a **plain `String`**. `apply_slot_stdin` (`:215-225`) reads stdin into `let mut buf =
  String::new()` then `slots[i].value = buf;` — bare `String`, no scrub.
- `convert.rs:758-765` — `read_stdin_passphrase` returns bare **`String`** (`Ok(buf)`).
  Sibling `read_stdin_to_string` (`:745-752`) also returns bare **`String`**.

### Scope is broader than the two cited symbols (key finding)

`read_stdin_passphrase` / `read_stdin_to_string` have **~30 call sites** across 12 `cmd/*` modules. Many
**already wrap at the call site** in `Zeroizing::new(...)` (derive_child `:140`, electrum_decrypt `:115`,
import_wallet `:2300`, ms_shares `:267/374`, seed_xor `:172/302`, seedqr `:178/245`, silent_payment `:242`,
slip39 `:304/424/607`, final_word `:68`). But several **do NOT wrap**, leaving a bare `String` binding:

- `addresses.rs:150` (`passphrase`), `:160` (`from_value`).
- `convert.rs:854` / `:862` (`effective_passphrase` / `effective_bip38_passphrase`), `:869` (`primary_value`).
- `restore.rs:400/411`, `:830/840`, `:1292/1300`, `:3045/3053`.

**mlock ≠ zeroize.** In `convert.rs:880-886` (and `bundle.rs:237-240`) those bare `String`s are mlock-**pinned**
(Cycle-B Site 1 — no-swap), but the pin does NOT scrub the heap buffer on drop. So the residue L22 names is
real even at the pinned sites. The function itself also holds a transient un-scrubbed `buf` regardless of what
the caller does with the return.

**Cleanest fix:** change the two readers' return type to `Zeroizing<String>` once. Every already-wrapping call
site collapses to a no-op rewrap (or drops the explicit wrap); the non-wrapping sites get scrubbing for free via
the new return type's `Deref<Target=String>`. Then wrap `SlotInput.value` (the one persistent field) as
`Zeroizing<String>` and adjust `apply_slot_stdin`.

### 4/5/6 — see "Cross-cutting / scope / SemVer" below.

---

## Item 2 — FOLLOWUP `resolved-slot-derived-account-zeroizing-field`

### 1. WHAT
Migrate the two Cycle-A entropy fields `ResolvedSlot.entropy: Option<Vec<u8>>` and
`DerivedAccount.entropy: Vec<u8>` → `Zeroizing<Vec<u8>>`, delete `impl Drop for DerivedAccount`, adjust
`into_parts`, relabel/add zeroize-lint rows, CHANGELOG.

### 2. Citations + status — **THIS IS ALREADY SHIPPED.**

origin/master `design/FOLLOWUPS.md`:
- Line **1179** (the `resolved-slot-derived-account-zeroizing-field` Status): `resolved ed5a1d9 —
  mnemonic-toolkit-v0.10.1 tag pushed 2026-05-13`. 12 ctor sites wrapped, `impl Drop` deleted, companion
  `pub-struct-drop-semver-risk-monitor` also resolved.
- The superseded predecessor `resolved-slot-entropy-zeroizing-field` (line **1093**) is marked
  `superseded by resolved-slot-derived-account-zeroizing-field`.

### 3. Current types in source confirm the FOLLOWUP — both fields already `Zeroizing`:

- `synthesize.rs:907` — `pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>` ✔ (was `Option<Vec<u8>>`).
- `derive.rs:24` — `pub entropy: zeroize::Zeroizing<Vec<u8>>` ✔ (was `Vec<u8>`).
- `derive.rs:15-19` — doc comment records *"v0.10.1: `entropy` is `Zeroizing<Vec<u8>>` … The previous
  `impl Drop for DerivedAccount` (Cycle A v0.9.0) is deleted"*. **No `impl Drop for DerivedAccount` in source.**
- `derive.rs:47` — `into_parts(mut self)` migrated (`mem::take(&mut *self.entropy)` deref-through-Zeroizing,
  outward `-> Vec<u8>` preserved).
- `tests/lint_zeroize_discipline.rs` — rows already relabeled: `:55` *"DerivedAccount entropy field is
  Zeroizing<Vec<u8>>"*, `:120` *"ResolvedSlot entropy field is Option<Zeroizing<Vec<u8>>>"*. The deferred-
  FOLLOWUP comment block is gone.

**Item 2 has nothing open. It DOES NOT reproduce.** Tag each citation: every "Where" line in the FOLLOWUP body
is a **pre-ship snapshot** that is now STRUCTURALLY-WRONG against live source (the fields it says to migrate are
already migrated) — i.e. accurately-resolved.

---

## Canonical 5-site list — resolved vs open

Per `cycle-b-pre-spec-questions` Q1 (resolved): the canonical SPEC §3 list is
`{clap-args, ResolvedSlot.entropy, DerivedAccount.entropy, bip85 [u8;64], ms-cli stdin String}`.

| # | Site | Cycle-B mlock pin | Zeroizing field-type | Status |
|---|---|---|---|---|
| 1 | clap-args (passphrase / per-slot bindings) | ✔ pinned (Site 1, v0.10.0) | clap `args.phrase` scrubbed via `mem::take`+`Zeroizing` (Cycle A); per-slot bare `String` (= L22) | **mlock done; L22 = the residual scrub gap** |
| 2 | `ResolvedSlot.entropy` | ✔ `_entropy_pin` (Site 2, v0.10.0) | ✔ `Option<Zeroizing<Vec<u8>>>` (v0.10.1) | **RESOLVED** |
| 3 | `DerivedAccount.entropy` | ✔ `_entropy_pin` (Site 3, v0.10.0) | ✔ `Zeroizing<Vec<u8>>` (v0.10.1) | **RESOLVED** |
| 4 | bip85 `[u8;64]` | ✔ heap-promoted+pinned (Site 4, P1, v0.10.0) | ✔ wrapped (`bip85.rs` uses `Zeroizing`) | **RESOLVED** |
| 5 | ms-cli stdin `String` | ✔ ms-cli Site 5 (`ms-cli-v0.3.0`, Cycle-B PE) | ✔ `Zeroizing<String>` (ms-cli Cycle A, `parse::read_stdin`) | **RESOLVED (in ms-cli, not toolkit)** |

**L22's two sites map onto Site 1's residual scrub gap** — the mlock pin landed but the heap buffer is never
zeroized. They are the only un-closed part of the 5-site list **in the toolkit**, and they're a *Zeroizing*
gap, not an *mlock* gap.

**ms-cli stdin (Site 5) is NOT the toolkit's concern** — already `Zeroizing<String>` in ms-cli since Cycle A,
mlock-pinned since ms-cli-v0.3.0. **No cross-repo coordination required** for this tail.

---

## 4. Coherence — one cycle or two?

**One cycle — and it collapses to just Item 1.** Item 2 is fully shipped, so there is no "continuation" to
fold in. The remaining work is L22 alone: a single coherent `Zeroizing<String>` wrapping of the slot/convert
stdin-secret reads + `SlotInput.value`. It is the natural sibling of the v0.10.1 `Vec<u8>→Zeroizing<Vec<u8>>`
field migration (same discipline, `String` instead of `Vec<u8>`), and of the v0.33.3 `read_blob →
Zeroizing<Vec<u8>>` precedent (FOLLOWUPS `:3612`) — both cited the same playbook. Recommend modeling the PR on
v0.33.3's "change the reader return type once; deref-coercion absorbs read sites" approach.

---

## 5. SemVer + gates

### Public-API surface
- `slot_input` is **`pub mod`** (`lib.rs:178`); `SlotInput` is a **`pub struct`** with **`pub value: String`**.
  Changing `value: String` → `value: Zeroizing<String>` (or a `SecretString` newtype) **changes a public field
  type → MINOR bump** under the pre-1.0 convention this repo uses (matches the v0.10.1 precedent that took a
  MINOR for the same class of change).
- `read_stdin_passphrase` / `read_stdin_to_string` are **`pub(crate)`** — return-type change is internal, not
  semver-visible.
- `DerivedAccount::into_parts` outward `-> Vec<u8>` is unchanged and already shipped; no interaction.
- **Recommendation:** **MINOR** (toolkit `v0.67.0` off current `v0.66.0`), driven solely by the `SlotInput.value`
  public-field-type change. If the cycle instead leaves `SlotInput.value` a `String` and only fixes the two
  reader return types + the un-wrapping call sites (no public field change), it could ship **PATCH** — but that
  leaves the persistent secret-bearing field un-scrubbed, which is the more important leg of L22. Prefer MINOR +
  fix the field.

### Zeroize-completeness lint gate
- `tests/lint_zeroize_discipline.rs` is an **evidence-anchor lint**: `ZEROIZE_ROWS` lists canonical owned-secret
  sites; each row asserts an evidence substring exists in `source_file`. Adding a new owned-secret allocation
  REQUIRES adding a row + the wrap. **This cycle MUST add rows** for `SlotInput.value` (e.g. *"SlotInput value
  field is Zeroizing<String>"* against `src/slot_input.rs`) and for the reader return types (e.g.
  *"read_stdin_passphrase returns Zeroizing<String>"* against `src/cmd/convert.rs`). This is the "lint anchor
  relabel + new row" mechanic the report/FOLLOWUP references — same pattern v0.10.1 used.
- Sibling lints to keep green: `lint_safety_first_party_mlock.rs` (the mlock pins on these `String`s must stay),
  `lint_argv_secret_flags.rs` (per MEMORY: CLI/flag phases ripple into argv lints — this cycle touches no flags,
  but run the **full** `cargo test -p mnemonic-toolkit` suite in R0 per the standing lesson).

### Cross-repo / Drop interactions
- **No ms-cli/ms-codec coordination** — Site 5 already done in ms-cli; no flag surface changes, so no
  manual-mirror or `schema_mirror` (GUI) lockstep is triggered (`SlotInput` is not a clap-flag or dropdown).
  Confirm `bundle`/`verify-bundle` `--slot` parsing is unaffected (it is — `apply_slot_stdin` keeps its
  signature; only the buffer type inside changes).
- **No `impl Drop` deletion this cycle** — `Zeroizing<String>` carries its own `Drop`; `SlotInput` has no
  hand-rolled `Drop` to delete (unlike v0.10.1's `DerivedAccount`). Adding a `Zeroizing` field to a `pub struct`
  with no `impl Drop` does NOT introduce the E0509 move-out hazard (that hazard came from `impl Drop`, not from
  the field type). `SlotInput` derives `Clone, PartialEq, Eq, Debug` — `Zeroizing<String>` is `Clone` and
  `PartialEq`, but **NOT `Eq`** and its `Debug` redacts. Either drop the `Eq`/`Debug` derives on `SlotInput`
  (check the ~6 consumer modules + tests that compare `SlotInput`), or wrap in a `SecretString`-style newtype
  that re-implements `Eq`/`Debug` (the crate already has `secret_string::SecretString` at
  `src/secret_string.rs` — but it is `Eq`-less and length-only-`Debug`, so reusing it still forces dropping
  `#[derive(Eq, Debug)]` on `SlotInput` or adding manual impls). **This is the one real implementation snag —
  flag it for the SPEC.**

---

## 6. Effort / LOC + recommended structure

**Small.** One toolkit-only MINOR (`v0.67.0`). No cross-repo, no GUI, no manual/schema mirror.

LOC estimate ~60-120:
- 2 reader return-type changes (`convert.rs`) + ~30 call-site touch-ups (most are no-op deref or drop-the-rewrap;
  a handful at the non-wrapping sites in `addresses.rs`/`convert.rs`/`restore.rs` newly benefit). ~30-40 LOC.
- `SlotInput.value: String → Zeroizing<String>` + `apply_slot_stdin` buffer wrap + derive-adjustment
  (`Eq`/`Debug`) + the ~6 consumer modules' `s.value.as_str()` reads (these all already go through `Deref`,
  so most compile unchanged). ~20-40 LOC.
- 2-3 new `lint_zeroize_discipline.rs` rows + CHANGELOG. ~15 LOC.

**Recommended phase structure (single-subagent-per-phase TDD, per CLAUDE.md):**
- **Cycle-prep / brainstorm + SPEC** — settle the `SlotInput.value` type decision (raw `Zeroizing<String>` vs a
  newtype) and the `Eq`/`Debug` derive question; pick MINOR. R0 to GREEN.
- **Plan-doc** — enumerate the exact call sites (the ~30 reader sites + 6 `SlotInput` consumers) with current
  line numbers re-grepped at write time; R0 to GREEN.
- **P1** — reader return types → `Zeroizing<String>` + non-wrapping call-site fixes + lint rows (RED→GREEN).
- **P2** — `SlotInput.value` field migration + `apply_slot_stdin` + consumer adjustments + lint row + CHANGELOG.
- **PE** — whole-diff adversarial review; full `cargo test -p mnemonic-toolkit` + clippy + (miri if cheap);
  bump to `v0.67.0`, update BOTH READMEs + fuzz/Cargo.lock self-pins (release-ritual sites), flip L22's report
  checkbox and the FOLLOWUP tail note in the shipping commit.

**Note for the SPEC author:** flip Item 2's framing — do NOT re-open `resolved-slot-derived-account-zeroizing-field`;
cite it as the *precedent* (v0.10.1) the L22 fix mirrors, exactly as v0.33.3 (`read_blob`) did.

---

## Cross-cutting observations

1. **The "tail" is one item, not two.** Item 2 shipped at v0.10.1 (`ed5a1d9`); the recon's main job was to
   prove that against source — done (both fields are `Zeroizing` in origin/master). The prompt's "open
   FOLLOWUP" premise is stale tracking, not a live gap. (Matches MEMORY `feedback_followup_status_discipline`:
   verify "open" at decision time — here it was already resolved.)
2. **L22 is a `Zeroizing` gap layered on an mlock-`done` site.** Cycle-B's Site 1 pinned these `String`s (no
   swap) but never scrubbed them. The fix is the *Zeroizing* leg, completing Site 1 to the same standard as
   Sites 2-5. Do not re-do the mlock work.
3. **Reader-return-type-once beats per-site wrapping.** ~30 call sites; flipping the two `pub(crate)` readers to
   `Zeroizing<String>` is the minimal-surface, deref-coercion-friendly move (v0.33.3 `read_blob` precedent).
4. **The only real snag is `SlotInput`'s `#[derive(Eq, Debug)]`** vs `Zeroizing<String>` (no `Eq`, redacting
   `Debug`). Resolve in SPEC before coding.
