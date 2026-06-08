# SPEC — Cycle B: friendly-mapper unit-test coverage + chunk_mk1 alias swap

**Cycle:** toolkit test-hygiene + trivial cleanup (FOLLOWUPs `friendly-mapper-unit-test-gaps` + `bundle-emit-bypasses-chunk-mk1-alias`).
**Date:** 2026-06-08.
**Source SHA:** `origin/master` == local `HEAD` == `8665d91`.
**Disposition:** toolkit, **no version bump, no tag** (test-only + byte-identical alias swap; binary unchanged).
**Recon:** `cycle-prep-recon-trmultia-friendly-canonicity-chunkmk1.md`.
**Locksteps:** none (no clap-flag / CLI / wire change).

---

## Item 1 — `bundle-emit-bypasses-chunk-mk1-alias` (trivial)

`crates/mnemonic-toolkit/src/cmd/bundle.rs` — inside the **`emit_unified`** fn (`:778`; the FOLLOWUP + first SPEC draft mis-cited `bundle.rs::emit`, R0-r1 M2) the mk1 text-card emit calls `chunk_5char(s)` directly at **two** sites:
- `MkField::Single` branch (`:962`)
- `MkField::Multi` per-cosigner branch (`:974`)

Switch **both** to `chunk_mk1(s)` (add `chunk_mk1` to the `use crate::format::{...}` import at `:7`; keep `chunk_5char` — still used at the ms1 site `:951`). `chunk_mk1` (`format.rs:33`) is `{ chunk_5char(s) }` → **byte-identical output**. This makes the future mk-codec grouping-helper swap (`mk-codec-chunked-visual-grouping-helper`) a single-edit at `format.rs:33`. Leave the ms1 site (`:951`) on `chunk_5char` (ms1 is not mk1).

Two source-hygiene riders (R0-r1 M3/M4), now that `chunk_mk1` becomes live:
- **Remove the vestigial `#[allow(dead_code)]` at `format.rs:32`** (the allow would otherwise mask a future real dead-code regression; build-warning-clean is the RED-equivalent — mirrors the `CosignerKeyInfo` allow-removal hygiene cycle).
- **Reword the `chunk_mk1` doc comment (`format.rs:28-31`)** — it currently says "Reserved: mk1 currently uses `chunk_5char` directly; mk-specific helper retained" (false post-swap). Reword: mk1 now routes through `chunk_mk1`; the single future swap point stays the body (`:33`).

**No-bump (binary byte-identical).**

---

## Item 2 — `friendly-mapper-unit-test-gaps`

`crates/mnemonic-toolkit/src/friendly.rs::tests`. Current state (re-counted at `8665d91`, the as-filed "covers 3" is stale): **12 tests** covering ~15 of **~94 match arms** across the 5 mappers. The dominant gap is **`friendly_md_codec` (~44 arms, 0 tests)**. **Exhaustiveness taxonomy (R0-r1 I2 + R0-r2 I-new-1, primary-source-corrected — the distinction is MAPPER-level, NOT enum-level).** What matters is whether the *mapper* has a bare `_ => "unhandled … {:?}"` arm (the fallthrough trap), not whether the wrapped enum is `#[non_exhaustive]`:
- **Mappers with NO bare `_` (all arms testable; a new variant breaks toolkit compilation) — THREE:** `friendly_md_codec` (44, closed `md_codec::Error`), `friendly_bip39` (5, closed `bip39::Error`), **`friendly_bitcoin` (3, closed toolkit-local `BitcoinErrorKind`)**.
- **Mappers WITH a bare `_` "unhandled" arm (a future variant silently falls through) — TWO:** `friendly_ms_codec` (`:129`), `friendly_mk_codec` (`:181`).

Note `friendly_bitcoin` matches the **toolkit-local closed `BitcoinErrorKind`** (`Bip32`/`XpubParse`/`FingerprintParse`); the `#[non_exhaustive]` `bitcoin::bip32::Error` is only Display-forwarded inside the `Bip32(b)` arm, so the mapper has no wildcard despite the wrapped enum being non_exhaustive. The `friendly.rs` module-doc at `:4-6` wrongly lists **both `bip39::Error` and `bitcoin::bip32::Error`** as non_exhaustive-with-wildcard — the implementer fixes that stale source comment in the same edit (only ms_codec + mk_codec carry the wildcard).

**Approach — table-driven "every constructible variant maps to a friendly message".** For each mapper, add a `#[test]` iterating a `[(variant, needle)]` table; for each row assert the rendered string:
1. **contains the codec tag** (`md1`/`mk1`/`ms1`/`BIP-39`/`BIP-32`-family) + a distinctive `needle`;
2. **does NOT contain `"unhandled"`** (the wildcard-fallthrough trap that the **2 wildcard mappers** `friendly_ms_codec`/`friendly_mk_codec` risk — vacuous for the 3 closed mappers, see the M5 note below);
3. **does NOT leak the raw Debug variant name** (e.g. `"TagOutOfRange"`, `"PathTooDeep"`) — the friendly message must be prose, not a `{:?}` dump (mirrors the existing `ms_codec_*_renders_prose` assertions).

**Coverage targets (variants are pub-constructible — the existing tests already build `bip39::Error::UnknownWord(5)`, `mk_codec::Error::PathTooDeep(11)`, `ms_codec::Error::WrongHrp{..}`):**
- **`friendly_md_codec`: ALL ~44 arms** (the biggest gap; exhaustive match → a future codec variant already breaks compilation, but the test pins each arm's *message* quality). Highest priority.
- **`friendly_mk_codec`: the ~20 untested arms** (only `PathTooDeep` + `XpubOriginPathMismatch` covered today).
- **`friendly_ms_codec`: the ~7 untested structural arms** (`ThresholdNotZero`, `ShareIndexNotSecret`, `TagInvalidAlphabet`, `UnknownTag`, `ReservedPrefixViolation`, `UnexpectedStringLength`, `PayloadLengthMismatch`) — the share/Codex32 arms are already covered.
- **`friendly_bip39`: the 3 constructible untested arms** (`BadEntropyBitCount`, `BadWordCount`, `InvalidChecksum`). **NOT `AmbiguousLanguages`** (R0-r1 M1): its payload `AmbiguousLanguages([bool; MAX_NB_LANGUAGES])` is a tuple struct with a private field + no public constructor → not buildable from the test crate; drop that row.
- **`friendly_bitcoin`: all 3 arms** (`Bip32`, `XpubParse`, `FingerprintParse`).

**Out of scope / documented limit:** the bare `_` "unhandled" arms of the **two** wildcard mappers (`friendly_ms_codec`/`friendly_mk_codec`) cannot be exercised by enumerating unknown future variants — the wildcard IS the safety net and stays untested-by-construction. Note this in a test-module comment. (`friendly_md_codec`, `friendly_bip39`, AND `friendly_bitcoin` have no `_` arm → no untestable arm.)

**Assertion meaningfulness (R0-r1 M5 + R0-r2 I-new-1) — note in the table comment so it's not cargo-culted:** the `!contains("unhandled")` guard is load-bearing ONLY for the **2** wildcard mappers (`friendly_ms_codec`/`friendly_mk_codec`, where a future variant silently falls to the `_` "unhandled"/Debug-dump arm — that's the real regression the test catches). For the **3** closed mappers (`friendly_md_codec`, `friendly_bip39`, `friendly_bitcoin`) it is vacuous (no `_` arm exists; a new variant is caught by the compiler, not the test) — there the substantive assertions are **no-Debug-variant-name-leak** + **codec-tag-present** (message-quality, not fallthrough-catching).

**Feasibility check (implementation must confirm, not assume):** a few variants carry fields that may need specific construction (e.g. `bitcoin::bip32::Error` for the `Bip32` arm; `md_codec::Error` variants with struct fields). If any variant is NOT publicly constructible from the toolkit test crate, drop that row and record it in the comment (do not fake it). The existing tests prove the common ones are constructible.

---

## 3. Verification
1. **`cargo test -p mnemonic-toolkit --bin mnemonic friendly`** (R0-r1 I1 — `friendly` is declared in the **binary** crate at `src/main.rs:16`, NOT in `lib.rs`; a `--lib` run finds no `friendly` module and runs ZERO tests, a false-green on this cycle's whole deliverable) → all new tests green. (`format.rs` is likewise bin-only; its byte-identical test runs under the same bin target.)
2. `cargo build -p mnemonic-toolkit` warning-clean (the `chunk_mk1` import now used → no dead-code warning on `chunk_mk1`).
3. **Byte-identical proof for #4:** the bundle text-card output is unchanged (`chunk_mk1` ≡ `chunk_5char`) — a characterization test or a before/after diff of a bundle emit confirms no output drift.
4. `cargo clippy --all-targets` clean.
5. No CLI/wire/flag change → no schema_mirror, no manual mirror.

## 4. Ship plan
1. Apply Item 1 (2-line swap + import) + Item 2 (table-driven tests).
2. Verify §3.
3. `design/FOLLOWUPS.md`: flip both `friendly-mapper-unit-test-gaps` + `bundle-emit-bypasses-chunk-mk1-alias` → resolved (record the re-counted coverage + the wildcard-arm limit). **Also fix the stale `bundle.rs::emit`→`bundle.rs::emit_unified` citation in the `bundle-emit-bypasses-chunk-mk1-alias` entry** (R0-r1 M2; the FOLLOWUP body carries the same mis-citation).
4. Stage explicitly; commit (`git commit -F -`, Co-Authored-By); push to `master`. **No bump, no tag.**
5. Memory.

### Out of scope
- `mk-codec-chunked-visual-grouping-helper` (the actual mk grouping helper — `chunk_mk1` stays an alias; this just routes the call site through it).
- The `#[non_exhaustive]` wildcard arms (untestable by construction).
- Cycles A (trmultia) + C (canonicity, GUI repo).
