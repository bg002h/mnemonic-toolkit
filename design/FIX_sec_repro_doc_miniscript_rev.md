# FIX: repro doc keys its `[source]` stanzas on Cargo.lock's revs (r11 F-371)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Base: origin/master
`68ea05e0`. Unit `repro-doc-miniscript-rev` (security-sweep routing, grade I).
Track: small-fix. Docs plus one test; no flag, type, command or API.

## 0. The finding, re-checked at 68ea05e0

`docs/verify-reproducibility.md` prints two offline rebuild commands (section 4,
x86_64 docker, lines 113-115; section 9.3, aarch64 cross, lines 400-402). Both
key the miniscript `[source]` stanzas on `95fdd1c5773bd918c574d2225787973f63e16a66`.
`Cargo.lock:705` pins `ff4732e5f75aa555682343cb180fa72ee3e8e9d5`. Prose lines
143 and 151 cite the old rev too, and line 152 cites `Cargo.toml:28-29` for the
`[patch.crates-io]` entry, which is now at `Cargo.toml:38-39`.

New fact found while writing this FIX: `Cargo.lock:688` also pins a second git
source, md-codec at `https://github.com/bg002h/descriptor-mnemonic` rev
`cf35d61af0f058029df841f4882945e53f86cd4b` (F-642). The doc prints no stanza
for it. Re-keying the miniscript rev alone does NOT make the printed command
resolve (row R2b below, run). See section 1.

## 1. OPEN POINT (STOP): does this unit also add the md-codec stanzas?

The routed scope is "re-key the three stanzas (8 sites)". With only that, the
section 9.3 command still fails offline, now on md-codec:

    error: failed to load source for dependency `md-codec`
    ...
      can't checkout from 'https://github.com/bg002h/descriptor-mnemonic': you are in the offline mode (--offline)

- **Option A (recommended).** Re-key miniscript AND add the three md-codec
  stanzas to both commands, and update the "three-block" prose (section 3.2).
  The test (section 4) checks every git source Cargo.lock pins. This is the
  only option after which the printed command works as printed, which is what
  F-371's I grade is about. Cost: three prose sentences more than routed.
- **Option B.** Re-key miniscript only (section 3.1); the test checks only the
  rust-miniscript source; file the md-codec gap as a new FOLLOWUPS line (its
  own I unit). R2 stays RED after this FIX.

The prose wording in 3.2 is a proposal and is part of this decision.

## 2. Holder-visible output, refusals, exit codes, manual lockstep

None at runtime. No `mnemonic`/`md` CLI text or flag changes, so no toolkit
manual lockstep and no CLI WORDING_REVIEW row. The doc prose in 3.2 (option A)
is the only new text; it is proposed as WORDING_REVIEW rows in section 7.

## 3. Spec (doc) delta: exact old/new

### 3.1 Both options

S1. Lines 113, 114, 115, 400, 401, 402 and 151: every occurrence of
`95fdd1c5773bd918c574d2225787973f63e16a66` becomes
`ff4732e5f75aa555682343cb180fa72ee3e8e9d5` (9 occurrences on these 7 lines, counted by `grep -o`;
114 and 401 carry two each). Nothing else on those lines changes.

S2. Line 143: `` `source."git+…rev=95fdd1c5…"` `` becomes
`` `source."git+…rev=ff4732e5…"` ``.

S3. Line 152: `` `Cargo.toml:28-29` `` becomes `` `Cargo.toml:38-39` ``.

After S1-S3, `grep -c 95fdd1c docs/verify-reproducibility.md` prints `0`.

### 3.2 Option A only

A1. Section 4, after line 115 (the miniscript `.replace-with` line), insert
three lines, same 6-space indent and `'"'"'` quoting as line 115:

    --config '"'"'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".git="https://github.com/bg002h/descriptor-mnemonic"'"'"' \
    --config '"'"'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".rev="cf35d61af0f058029df841f4882945e53f86cd4b"'"'"' \
    --config '"'"'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".replace-with="vendored-sources"'"'"' \

A2. Section 9.3, after line 402, insert the same three stanzas with 2-space
indent and plain `'...'` quoting, as line 402:

    --config 'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".git="https://github.com/bg002h/descriptor-mnemonic"' \
    --config 'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".rev="cf35d61af0f058029df841f4882945e53f86cd4b"' \
    --config 'source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61af0f058029df841f4882945e53f86cd4b".replace-with="vendored-sources"' \

A3. Line 136: `**The three-block job-scoped `[source]` activation is mandatory.**`
becomes `**The four-block job-scoped `[source]` activation is mandatory.**`

A4. Line 141-142: `**All three` / `` `cargo vendor`-emitted blocks are required** ``
becomes `**All four` / `` `cargo vendor`-emitted blocks are required** ``; and
after the sentence ending `` not served by `source.crates-io`. `` (line 146)
insert: `` The md-codec git dependency (`source."git+…descriptor-mnemonic?rev=cf35d61a…"`) needs its own block for the same reason. ``

A5. Line 410: `The same three-block `[source]` activation` becomes
`The same four-block `[source]` activation`.

## 4. Tests that change

- NEW `crates/mnemonic-toolkit/tests/repro_doc_pins.rs`, one test
  `repro_doc_source_stanzas_match_cargo_lock`. It reads the doc and
  `Cargo.lock` from `CARGO_MANIFEST_DIR/../..` (precedent:
  `tests/readme_version_current.rs`) and asserts: (a) every
  `git+<url>?rev=<40 hex>` key in the doc is a (url, rev) pair Cargo.lock pins;
  (b) every `rev=` followed by 7+ hex digits in the doc starts a Cargo.lock rev
  (catches the prose sites 143 and 151); (c) for every Cargo.lock git source,
  each of its three stanzas (`.git=`, `.rev=`, `.replace-with="vendored-sources"`)
  appears exactly 2 times (sections 4 and 9.3). Option B: (a) and (c) are
  limited to `https://github.com/rust-bitcoin/rust-miniscript`.
- `ci/docs-only-paths.txt`: add, with the other `-` rules,
  `# read by tests/repro_doc_pins.rs` then `- ^docs/verify-reproducibility\.md$`
  (without it `ci/code-docs-check.sh` flags the new test, and a doc-only edit
  would skip the test that reads the doc).
- COMMITTED here: `design/fix-sec-repro-doc-miniscript-rev-red.sh` (probe R2).

## 5. RED rows (run on unmodified origin/master 68ea05e0)

R1 `repro_doc_source_stanzas_match_cargo_lock`. Run by the author as a
standalone `rustc --test` of the draft test (same body) against the base tree:

    thread 'repro_doc_source_stanzas_match_cargo_lock' panicked at repro_doc_pins.rs:48:9:
    docs/verify-reproducibility.md keys a [source] stanza on https://github.com/rust-bitcoin/rust-miniscript at rev 95fdd1c5773bd918c574d2225787973f63e16a66, which Cargo.lock does not pin (Cargo.lock git sources: {("https://github.com/bg002h/descriptor-mnemonic", "cf35d61af0f058029df841f4882945e53f86cd4b"), ("https://github.com/rust-bitcoin/rust-miniscript", "ff4732e5f75aa555682343cb180fa72ee3e8e9d5")}); re-key the doc to Cargo.lock
    test result: FAILED. 0 passed; 1 failed

Builder command: `cargo nextest run --locked -p mnemonic-toolkit --test repro_doc_pins`.
Expected after the fix: 1 passed. Trial with S1-S3 only (option-A test): fails
with `prints 0 copies of `source."git+https://github.com/bg002h/descriptor-mnemonic?rev=cf35d61a...".git="https://github.com/bg002h/descriptor-mnemonic"``;
with S1-S3 + A1-A5: `test result: ok. 1 passed`.

R2 (the command that proves the new rev resolves):
`bash design/fix-sec-repro-doc-miniscript-rev-red.sh`. At base, last lines:

    error: failed to load source for dependency `miniscript`
    Caused by:
      Unable to update https://github.com/rust-bitcoin/rust-miniscript?rev=ff4732e5f75aa555682343cb180fa72ee3e8e9d5#ff4732e5
    Caused by:
      can't checkout from 'https://github.com/rust-bitcoin/rust-miniscript': you are in the offline mode (--offline)
    exit=101

R2b, trial with S1-S3 only: fails on `md-codec` (`can't checkout from
'https://github.com/bg002h/descriptor-mnemonic'`), `exit=101`.
Expected after option A: last line `exit=0` (trial run: `exit=0`).
After option B: R2b's failure, stays RED (owned by the new FOLLOWUPS line).

## 6. Gate (builder)

`cargo nextest run --locked -p mnemonic-toolkit --test repro_doc_pins`;
`bash ci/code-docs-check.sh`; `bash ci/docs-only.test.sh`;
`bash ci/code-docs-check.test.sh`; R2; then full CI on the head (RULES.md).
Not covered by any of these: the section 4 docker command is not executed
(needs docker and the pinned image; its stanzas are checked as text only), and
nothing compiles or byte-compares the rebuild.

## 7. WORDING_REVIEW row proposals (option A only)

- `docs/verify-reproducibility.md` A3/A5: "three-block" -> "four-block".
- A4: "All four `cargo vendor`-emitted blocks are required" and "The md-codec
  git dependency (`source."git+…descriptor-mnemonic?rev=cf35d61a…"`) needs its
  own block for the same reason."

## 8. Out of scope (noted)

`.github/workflows/man-pages.yml:223` hard-codes `MINISCRIPT_REV: ff4732e5…`
(correct today; md-codec comes from its `pin` job). The `fuzz/` pin and the
`ci/repro` defaults are FOLLOWUP `fuzz-and-repro-miniscript-pin-realign`.
Code comments naming `95fdd1c` (restore.rs, tests) are history, not commands.
