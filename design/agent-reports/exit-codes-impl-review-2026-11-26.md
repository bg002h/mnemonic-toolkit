# Independent review: constellation exit codes in mnemonic-toolkit (2026-11-26)

**Scope.** I reviewed `e51607b8` (the closed-stdout fix), `71dc68ac` (the code), `f8b8d19c` (the user manual) and `a32c3bd2` (the GUI manual) on branch `fx/mnemonic-toolkit`. I checked them against `SPEC_constellation_exit_codes.md` rev 5 (§2-§5, §7f, §7i), the implementer's report `impl-toolkit.md`, and the sibling reports (`impl-md/mk/ms/mt/me/gui.md`).

**Method.** I read the diff. I also ran a freshly built `target/debug/mnemonic` (HEAD `a32c3bd2`) side by side with the sibling builds in `/home/user/fx/*/target/debug`:
- mk-codec's negative vectors N6 to N21 (`crates/mk-codec/src/test_vectors/v0.1.json`);
- a BIP-93 string with an unknown tag, built the same way as ms-cli's `unknown_tag()`;
- the 15-chunk md1 card from the toolkit tests;
- the 2-chunk mk1 card `CHUNK0`/`CHUNK1`;
- usage, input and stdout cases.

I ran no test suites.

**Verdict: NOT GREEN. 1 Critical, 6 Important, 11 Minor, 5 Nit.**

## Critical

### C1. `mnemonic repair --mk1` exits 0 ("clean") on strings that are not a card (decision 6)

- **Where:**
  - `crates/mnemonic-toolkit/src/repair.rs:1159-1163`, in `verify_mk1_set`: "Untouched group — no aliasing risk, skip re-verify entirely."
  - The per-chunk loop at `:1249-1266`. It never decodes a chunk that needed no correction.
- **Failure, measured.** Each of the 16 mk-codec negative vectors N6, N7 and N9 to N21 was given with a clean checksum: `mnemonic repair --mk1 …` → **0**, with the strings echoed.
  - `mk repair` gives **2** on the same strings, and `mk decode` gives 2.
  - Examples: N12 (unsupported version), N19 (all-zero public key), N21 (trailing bytes).
  - Decision 6 says every `repair` decodes "each corrected *or clean* string … and adds 2 for any that fails". §8 lists exactly this as a defect of `mk`, and `mk` has since fixed it.
- **Why it is Critical.**
  - 0 means "every check passed". An operator who checks a plate set with `mnemonic repair` is told a plate that cannot be restored is clean.
  - The manual also tells them that the two tools agree (I3).
  - The toolkit is not even consistent with itself: a clean ms1 that is not a card gives 2 (measured), because `repair_via_ms_codec` decodes it.
- **Fix.**
  - Decode every group whether or not it was touched: `mk_codec::decode` for a complete group, and a header parse for each chunk of an incomplete group.
  - A clean group that fails is 2, with nothing printed, as `mk` does it.
  - Add a table test over N6 to N21 that asserts the same code from `mk repair` and `mnemonic repair --mk1`.

## Important

### I1. Spec question 2: `repair --md1` still refuses an incomplete set or a lone chunk (2). `md` gives 0 or 16. This is a defect.

- **Where:** `repair.rs:1361-1373` → `repair_via_md_codec` (the full decode in `md_codec::decode_with_correction`).
- **Measured, with one chunk of the 15-chunk md1 card:**

  | input | `md repair` | `mnemonic repair --md1` |
  |---|---|---|
  | the clean chunk | 0 | 2 ("chunk set incomplete: got 1 chunks, expected 15") |
  | the same chunk with one substitution | 16 | 2 |

- **Why it is a defect.**
  - Decision 7 says "*Recommended: 16 everywhere* … **The toolkit changes.** `md` and `mr` … correct and print each chunk of an incomplete set (16), and a clean one becomes 0."
  - It also says "A lone chunk of a multi-chunk set counts as an incomplete set (16)."
  - This is not a reading of an ambiguous rule. It is the mixed state that §1 exists to end, on the funds-recovery path ("a partly damaged set repairable before every plate is found").
  - The API is already there: `md_codec::correct_chunks` is called at `repair.rs:2030`.
- **Fix.**
  - For an md1 group that is incomplete (each chunk once, every index inside the set), correct each chunk with `correct_chunks` and print it with an advisory: 16, or 0 if clean.
  - A set with a repeated, stray or extra chunk stays 2, as in `md`.
  - Add `md`-parity tests.

### I2. Spec question 1: a corrected single string that is not a card stays atomic 2 with nothing printed. Siblings print it at 22 (or 18). This deviates from decision 6.

- **Measured.**
  - ms1 (one substitution in an unknown-tag string, given on stdin): `ms repair` 22 with the string printed; `mnemonic repair --ms1` **2**, with nothing printed.
  - mk1 N6 corrected: `mk repair` **18**, with the string printed; the toolkit gives **2**.
- **Why it deviates.** Decision 6 says "every `repair` … adds 2 for any that fails. **It still prints the corrected strings with its warning** … no oracle, as for a single string, … 22."
  - R1 lists `repair` under decision 6 among the verbs that print despite a failure.
  - Only "a *complete* set that fails reassembly" is atomic. A single string is not a set.
  - The toolkit applies 22 only to the F-642 unreadable-version md1 (`cmd/repair.rs:186-192`).
- **On the safety argument.** The toolkit's "repair never wrong" stance is not a reason here: a printed string that does not decode cannot be mistaken for a card. The code 2 is truthful, but bit 16 is lost, along with the output a person repairing by hand needs.
- **Fix.** Either:
  - print it at 22 (ms1, non-chunked md1) or 18 (mk1 with an unparseable header); or
  - obtain Brian's ruling that amends decision 6 for the toolkit, and record it in the spec.

  Until one is done, do not claim parity (I3).

### I3. The manuals claim the toolkit and the siblings give the same codes. C1, I1 and I2 make the claims false.

- `docs/manual/src/40-cli-reference/44-mk-cli.md:270-271`: "`mk repair` and `mnemonic repair --mk1` give the same code on the same strings."
- `docs/manual-gui/src/70-mk/79-repair.md:67-68`: the same sentence.
- `docs/manual/src/40-cli-reference/41-mnemonic.md:3684-3687`: "the two agree on every md1 shape".
- `42-md.md` (the replaced D26 paragraph): "`md repair` and `mnemonic repair --md1` now agree on every md1 shape".

**Failure scenario.** A reader scripts against one tool on the strength of the other's documentation, and gets 0 or 2 where 2 or 16 was promised.

**Fix.** Fix C1, I1 and I2, or state the differences.

### I4. `41-mnemonic.md` still says an incomplete mk1 group is a 20 VERIFY-ME candidate

- **Where:** `41-mnemonic.md:3553-3562`: "An INCOMPLETE group is supplied … `mnemonic repair` reports it as an exit-`20` VERIFY-ME candidate."
- **The contradiction:** the chapter's own exit table (`:3491`), the binary (`SetVerify::IncompleteSet` → 16) and decision 7 all say 16.
- **Fix:**
  - Rewrite the paragraph: 16, with the advisory that the rest of the set verifies the correction on reassembly.
  - Drop "the same precedence tier as an ambiguous `--max-indel` recovery".
  - The "reject dominates" sentence stays.

### I5. `--expect-fingerprint zz` exits 1, and exits 0 under `--allow-mismatch`. Decision 2 names this case: it is 2.

- **Where:** `src/cmd/restore.rs:630-635`. The expected value is lower-cased and compared as a string. No validation runs first.
- **Measured** (with the abandon…about phrase on stdin, `--template bip84 --network mainnet`):
  - `--expect-fingerprint zz` → 1 ("✗ MISMATCH");
  - `--expect-fingerprint ZZZZZZZZ` → 1;
  - `--expect-xpub garbage` → 1;
  - adding `--allow-mismatch` turns a typo into exit 0.
- **What the spec and the code say.**
  - Decision 2: "Examples: … `--expect-fingerprint zz`." It also says "a fingerprint typo is 2 in every tool".
  - The comment on `INPUT_VALUE_ARGS` (`cli.rs:55-60`) claims this behaviour, but clap never sees the value as malformed, because the argument is a plain `String`.
- **Fix:** parse `--expect-fingerprint` as 8 hex digits, and `--expect-xpub` as an extended key, before the comparison and before R4's override. A failure is `BadInput` (2). Add a test.

### I6. R2: settings checks run after the input is read, so the code depends on the input

- **Measured:**
  - `echo garbage | mnemonic seed-xor split --from phrase=- --shares 1` → **2** (word count). With a valid phrase it is 64. The check is `TooFewShares` → `Usage`, at `src/cmd/seed_xor.rs:410`, after the split runs.
  - `echo garbage | mnemonic convert --from phrase=- --to xpub` (no `--template`/`--path`) → 2. With a valid phrase it is 64.
  - `--to wif` with no path: the same.
  - `restore --format` without `--template` is correctly 64 before the read.
- **Why it matters.**
  - R2: "A usage error exits 64 before any input is read or any output is written."
  - The siblings hoisted these checks (impl-mk's "flag-only checks now run before any input is read (R2)", and md's `--force-long-code`).
  - It also means a secret is read on a run that was going to fail on the command line.
- **Fix:**
  - Move every check that is judged from argv alone ahead of the first stdin or file read in `seed_xor::run_split` and `convert::run`.
  - Audit every verb whose `Usage` errors (`convert.rs:497-547`) are raised after `parse_from_input` consumes stdin.
  - Add one bad-input + bad-setting test per verb.

## Minor

**M1. `BundleMismatch` is NO (1), but it is raised only by `bundle --self-check`, so it is a bug (101).**
- **Where:** `error.rs:640`.
- All 16 construction sites are `self-check[...]` in `src/cmd/bundle.rs:2739-2935`. They fire after `emit_unified` has printed the bundle, when the toolkit's own freshly encoded cards fail to round-trip.
- §3 test 7 makes that 101 ("report it"), not "read the verdict". The implementer's report lists it among the "mismatches that were 4".
- **Fix:** `INTERNAL`, and update the table row.

**M2. The R13 table is thin where the classification actually varies.**
- **Where:** `error.rs:1157-1530`.
- `MsCodec` is sampled once, on the USAGE arm. None of the 15 INVALID variants of `ms_codec_exit_code` is sampled, nor either `_ => INTERNAL` fallback. `MkCodec` is sampled once, and `CompareCost` twice of its six.
- `BadInput`, which now covers hundreds of reclassified sites, is one row.
- The completeness guard is weak. If someone adds a variant, gives it arm `=> 63` in `variant_index`, and neither adds a table row nor bumps `VARIANT_COUNT`, the test still passes, because `missing` only scans `0..63`.
- **Fix:**
  - Iterate every named ms-codec and mk-codec variant and every `CompareCostError` variant.
  - Derive the count from `variant_index`, for example with a test that the largest index + 1 == `VARIANT_COUNT`.

**M3. R6 adds 16 to failures, and the auto-repair JSON's `exit_code` can differ from the process status (R1, R14).**
- **Where:** `cli.rs:335-346` ORs in 16 for any code below 64, so a verb that fails atomically after decoding a corrected mk1 exits 18 or 24.
- **R1:** "A verb that fails atomically reports only its failure bits."
- **R14:** the auto-repair JSON hardcodes `exit_code: 2` (`repair.rs:2309`), but the process can then exit 18 (R6) or 34 (R8).
- mk raised the same R6 question (impl-mk Q11).
- **Fix:** get a constellation-wide ruling. Until then, apply it the same way as md, mk and ms, and state it.

**M4. verify-bundle classes a row by its name suffix.**
- **Where:** `verify_bundle.rs:4145-4159`.
- A failed `md1_wallet_policy` row ("descriptor is template-only (no pubkeys TLV)", `:3361`, `:3513`) becomes NO (1). It is an input insufficient for the verb (test 6, 2).
- The suffix rule will also silently misclass any future row.
- **Fix:** give `VerifyCheck` an explicit class field.

**M5. Spec question 3 is inconsistent.**
- A wrong BIP-38 passphrase or Electrum password is 2.
- A BSMS MAC mismatch, which is the same kind of failed secret-against-tag check, is 1 (decision 11).
- **Fix:** ask Brian for one ruling and apply it to all three.

**M6. `44-mk-cli.md` has stale codes.**
- `:39` (`--from-md1-set`) and `:40` (`--keys`): a malformed line, a CR-only file or a misplaced BOM is "refused (exit 64)". mk measures **2** (BOM in `--keys` and `--from-md1-set` → 2, CR-only `--keys` → 2).
- `:266` says a string `mk decode` rejects is 2 with "Nothing is printed". mk prints a string that is not a chunk (N6, N9) with an error line, and a corrected N6 is **18**, which is in no table row.

**M7. `41-mnemonic.md:4692` (compare-cost `--descriptor`) still says a multi-leaf or keypath-only `tr` is "refused with exit `3`".** The chapter's own table at `:4831` and the binary say 2.

**M8. Appendix J (`60-appendices/6A-exit-codes.md`) against spec §3.**
- The "Per binary" row for `md` omits 22 (a corrected single string that does not decode) and 101. The row for `mk` omits 22 and 101; impl-mk has both.
- The 32 row drops "a lock, a RAM folder" from §3's list. These are `mr-sitting`'s members.
- The NO row drops "an input the node proves spent". That is `mt`'s member, and §3 places it explicitly.
- The appendix renumbers the tests 1 to 8, where the spec has 0 to 7. Harmless, but state that the order is the spec's.

**M9. `43-ms.md` does not state decision 8.** The `ms inspect` section says nothing about a string that would NOT decode now exiting 2. It covers only the would-not-combine share. Add one sentence.

**M10. The GUI manual never states mnemonic-gui's own exit codes (R12).** impl-gui changed them: 64 for a bad command line, 32 for "cannot start", 101, and 130 kept. They belong in `80-troubleshooting/82-binary-and-launch.md`.

**M11. `45-mr.md:55-56` has a stray table header.** An empty `| Code | Meaning |` table sits above the intro paragraph and renders as an empty table. Delete the first two lines.

## Nit

- **N1. Stale comments in the source:**
  - `restore.rs:650`: "Hard fail (exit 4)".
  - `cost/mod.rs:41-55`: "Exit 3", on the `CompareCostError` variants, which are now 2 or 8.
- **N2. `exit_code.rs` claims to mirror mnemonic-io-lib 0.1.4, but the two differ.**
  - The library's `HELP_LEGEND` is one line, built with `\` continuations, so it has no embedded newlines. The toolkit's has newlines.
  - The library has no `with_io_failure`.
  - When the toolkit switches to the shared crate, `cli.rs:374` (`ends_with(HELP_LEGEND)`) fails, because `ROOT_AFTER_HELP` duplicates the wrapped literal.
  - **Fix:** compare with normalised whitespace, or build `after_help` from the constant.
- **N3. `42-md.md:792`:** "…(0 becomes 32, 4 becomes 36, 20 becomes 52); A verb…" has a capital after a semicolon.
- **N4. `82-binary-and-launch.md` (new paragraph):** "a secret typed on the command line is 8, not 2". The 2 was `mnemonic`'s alone; ms gave 1, mk 64 and me 3.
- **N5. A spec gap, not a toolkit defect.** Two mk1 chunks with different `chunk_set_id`s that together cover every index (N8) exit 0 in both tools (impl-mk Q10). Worth a spec line.

## Spec questions (impl-toolkit.md)

1. **Atomic 2 for a corrected single string that is not a card.** This is a deviation from decision 6, which says every `repair` prints and adds 2. See I2: fix it, or get a recorded ruling.
2. **An incomplete md1 set refused (2).** This is a defect against decision 7 ("16 everywhere … the toolkit changes"). See I1.
3. **BIP-38 and Electrum password mismatch at 2.** These need one ruling together with the BSMS MAC (M5).
4. **The one-way `convert` refusals at 2.** Consistent: §3's worked table places "`convert` from xpub to xprv" at 2.
5. **verify-bundle sums its rows** (an undecodable card plus a mismatch row is 3). Consistent with R1.
6. **electrum/taproot and bip388 NUMS or repeated key at 2.** Consistent (test 6, the template is input).
7. **Envelope self-disagreement at 2.** Consistent (test 6).
8. **New `kind` vocabulary.** Acceptable. `kind()` is dead code in the CLI, and the GUI does not parse it.
9. **The GUI relies on the legend being last in `--help`.** This is pinned by `cli_help_fixtures::root_help_ends_with_the_exit_code_legend` for `--help`, `-h` and `help`. Verified: the binary ends with it.

## Verified correct (no finding)

**`e51607b8`, the closed-stdout probe.**
- `.init_array` / `__mod_init_func` checks `F_GETFD` + `EBADF` for fds 0 to 2. It plugs each closed fd with `/dev/null` at the lowest free number, so a later `--output` open cannot become stdout. The flag is read by `fd1_read_only`.
- Measured at HEAD: `gui-schema >&-`, `--help >&-` and `--version >&-` → 32, with one stderr line. A usage error with `>&-` → 64.
- R26 holds: `export-wallet --output FILE` with fd 1 closed → 0, and the file is byte-exact.
- The probe is in `main.rs` only, so test binaries are unaffected.

**R7.** A damaged md1 to `inspect` → 2 with `MNEMONIC_FORCE_TTY=0` and with `=1`. The report is printed only on the "terminal".

**R6.**
- `inspect --mk1`, `word-card --from` and `convert --from "mk1=c0 c1"` on a damaged chunk → 16, with the note. The clean card → 0 with no note.
- Every user-facing mk1 decode goes through `decode_mk1`. The remaining direct `mk_codec::decode` calls are in the bundle self-check, the repair engine and the tests.

**R8.** `>/dev/full` → 32. `| head -c1` → 0 (EPIPE keeps the verb's code). `with_io_failure` keeps 64, 101 and 130.

**R11.**
- The legend text is byte-equal to the spec once whitespace is normalised, and it is the last text of root `--help`, `-h` and `help`.
- Per-verb help texts were spot-checked: repair 20; restore 1 and 64; compare-cost 2; build-descriptor 2; verify-bundle 1; import-wallet 1.

**The classes themselves, measured:**
- empty stdin → 2; no verb arguments → 64;
- `@env:` unset → 32; `--template bogus` → 2;
- a malformed `--from` under `--allow-argv-secret` → 2, and without the flag → 8 (the argv guard);
- `--import-json-index 5` out of range → 2;
- `restore` mismatch → 1, `--allow-mismatch` → 0, no reference → 0.

**Repair.** A complete mk1 set that is corrected but does not reassemble → 2, with nothing printed (as the spec says). `indel_exit_code` gives 0, 16 or 20.

**The alphabetical-variant convention.** The five new variants are in alphabetical place. The pre-existing order of `Bip39` before `Bip388*` is not in C order, but that predates this change.

## Gaps in 45-mr and 46-mr-sitting (written from the spec; not counted as defects)

- **45-mr.**
  - The per-refusal passages, the placeholder paragraph ("a usage error, exit 2; … exit 1") and the `--shape` row ("exit 2") keep the old numbers. The chapter's note says so.
  - The Streams row claims that `>&-` adds 32. This is unverifiable until mr lands.
- **46-mr-sitting.** The 2 row ends "Always before any secret is read". It now also holds "end of input at a prompt", which happens mid-session, so the sentence is no longer true.
