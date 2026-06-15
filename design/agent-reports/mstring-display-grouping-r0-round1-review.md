# R0 round 1 — architect review (verbatim) — SPEC_mstring_display_grouping.md

> Persisted verbatim per CLAUDE.md BEFORE the fold. Dispatched via Agent
> (feature-dev:code-architect, Opus 4.8). **Verdict: NOT GREEN — 3 Critical /
> 9 Important / 8 Minor.** Spec SHA at review: toolkit `107f19d`.

---

## R0 Architecture Review — SPEC_mstring_display_grouping.md

### Critical Findings

**C1. The `ms split` emit structure is two-part, not the documented "bare strings" only — the SPEC's round-trip claim for `ms split | ms combine` is broken by its own design.**

Source evidence: `mnemonic-secret/crates/ms-cli/src/cmd/split.rs:147-163`. `emit_text` emits: (1) each bare share on its own line, then (2) a blank line, then (3) `"share N of M:\n<chunked(share)>\n"` blocks. When `ms split ... | ms combine -` pipes, `ms combine`'s `read_input` calls `strip_whitespace` (the doubling-heuristic path). After this feature: `render_grouped` replaces `chunked`, and `strip_display_separators` replaces `strip_whitespace`. But the split text-output is still two-part: bare canonical shares + grouped header blocks. `ms combine`'s intake (`read_input → strip_display_separators`) will now receive ALL of those lines in one stdin blob, strip the separators from the grouped blocks, and produce doubled content (canonical share + its stripped-group copy concatenated) without any dedup heuristic. The doubling-detection heuristic in `strip_whitespace` was exactly what absorbed this. Removing it without changing `ms split`'s emit structure breaks the pipe. The SPEC §10 says the heuristic is "removed because the trigger becomes unreachable" but does not address `ms split`'s two-part emit (canonical + grouped); §6 says "single form, print-once" only for the standalone-encode case. The SPEC must either (a) change `ms split` to also print-once (one grouped form only, removing the bare-first block) or (b) keep some intake guard that handles the double-on-pipe. This is a semantic correctness bug. File: `mnemonic-secret/crates/ms-cli/src/cmd/split.rs:144-163`.

**C2 (revised).** [Reviewer note: the original C2 claimed a `format.rs` citation error, then RETRACTED it after confirming lines 10/32/37 are correct. The standing C2 is restated below.] The `ms split` text-form emit is structurally two-part, but the intake stripping cited in §9.2 ("ms-cli `parse.rs:97` `strip_whitespace`") is wrong about what it strips. `strip_whitespace` strips ALL Unicode whitespace. `strip_display_separators` strips ONLY `{space, -, ,}`. The `ms split` bare-share block uses bare newlines (not display separators) between share strings. After the heuristic is removed, `ms decode -` receiving `ms split` output (which interleaves bare shares and grouped-header blocks labeled `"share N of M:\n..."`) will strip separators out of the grouped copies, then feed the duplicate stripped strings to the codec. The codec will return a "repeated share index" error. Concretely: `ms split | ms decode -` will break post-implementation even though §10 asserts `back_typed_chunked_form_decodes` still passes. That test exercises `ms decode` (single ms1 with spaces), not `ms split | ms decode`; the integration path across commands is distinct. The SPEC's §15 question 1 punts to R0; the answer is this IS a real consumer that depends on the two-part structure and the heuristic. Critical because it ships a broken pipe.

**C3.** Missing intake strip at `ms combine`'s positional share intake (`mnemonic-secret/crates/ms-cli/src/cmd/combine.rs:38-39`). The `shares: Vec<String>` positionals are read raw by clap and passed directly to `ms_codec::combine_shares`. When a user copies a grouped ms1 share (with spaces or hyphens) from engraved steel and supplies it as a positional arg to `ms combine`, the share string is not stripped. After this feature, if `ms split --separator hyphen` is used and a user reads back the grouped form, `ms combine <grouped-share>` will fail with a BCH decode error. The SPEC §9.2 lists `parse.rs:97 strip_whitespace` as the only intake site for ms-cli but does not list `ms combine`'s positional share intake. Incomplete inventory in a safety-bearing context (backup recovery tool).

### Important Findings

**I1.** `render_codex32_grouped` in md-codec is referenced in the technical manual at `docs/technical-manual/src/50-rust-api/51-md-codec-api.md:194` and `54-mnemonic-toolkit-api.md:51` as part of the public API surface. §8 says "GENERALIZE" it. Renaming/signature-changing it breaks the documented path + the `manual-lint` CI gate. The SPEC does not mention updating the technical manual. Missed lockstep.

**I2.** The conformance-vector TSV separator-encoding problem is real and unresolved. A literal-space separator value stored as a TSV field is indistinguishable from a column separator. The SPEC does not specify how a literal-space separator, the unbroken (group_size=0) row's separator field, or empty-string inputs are encoded. Every parser needs a documented escape convention.

**I3.** §9.1 inventory misses `ms split`'s second emit block (`split.rs:157-163`, `"share N of M:\n<chunked(share)>"`). The cite `:159` is only the grouped line; the bare-share emit is at lines 152-154. Incomplete.

**I4.** `ms decode` intake is not listed in §9.2 but is real: `cmd/decode.rs:42` `read_input(...)` → `strip_whitespace`. If `strip_whitespace` is replaced by a `{space,-,,}`-only stripper, decode NARROWS from stripping all whitespace (tabs, CRLF) to three chars — copy-paste artifacts (`\t`, `\r\n`) would no longer be stripped → decode failures. The SPEC does not address this behavioral narrowing.

**I5.** `mk-cli read_mk1_strings` does `.trim()` per line only (`mk-cli/src/cmd/mod.rs:91-94`), not interior strip. §9.2 "add strip" is correct as a change but does not document the narrowing/interior-strip intent. Should be explicit.

**I6.** §12 phase ordering: toolkit Phase 3 consumes the MINOR-bumped md-codec. If md-codec renames/removes `render_codex32_grouped` in Phase 1 without an alias, the toolkit's `format.rs:38` call won't compile until Phase 3 completes the `chunk_md1` deletion — a multi-step atomic dependency. The SPEC doesn't describe the rename/removal/alias strategy.

**I7.** The `--separator` value parser ("NOT clap ValueEnum") is underspecified re: literal-space `" "` surviving shell + clap. The GUI dropdown should send keyword values (`space`) not literals; this constraint is not stated as a design requirement (schema_mirror gates flag-NAME only, not value-shape).

**I8.** CLAUDE.md alphabetical-variant rule: a new `ToolkitError` variant will likely be needed for `--separator`/`--group-size` parse failures; the SPEC names none and states no alphabetical placement. Concurrent-PR conflict risk.

**I9.** §13 "golden regen" omits the differential-harness / fuzz-corpus reference artifacts (the stress-testing cycle-B harness on disk uses golden output as reference); they'll false-positive after the default format changes. Should be updated in Phase 3.

### Minor Findings

**m1.** §1 cites ms-cli `cmd/encode.rs:201` (the chunked line); the bare-ms1 line is 199 and `emit_text` is ~198. Cite both/the fn.

**m2.** §1 cites `cmd/split.rs:159` (grouped line); bare-share loop is 152-154. Cite the containing `emit_text` (147).

**m3.** §10 references a test `back_typed_chunked_form_decodes` that does not exist; the real tests are `strip_whitespace_dedupes_doubled_content` and `strip_whitespace_handles_all_three_workflows` (`parse.rs:138`). Fix the name or state it's new.

**m4.** §3.3 "MAY use bytes" is fine for ASCII; conformance vectors should make the bytes≡chars equivalence explicit so a future non-ASCII HRP doesn't silently break it.

**m5.** §4 charset claim confirmed correct: alphabet `qpzry9x8gf2tvdw0s3jn54khce6mua7l` (`md-cli/src/cmd/repair.rs:34`) contains none of `{space,-,,}` nor `{ms,mk,md,1}`. No bug.

**m6.** §8 copy-with-checksum has no automated link from a canonical change to sibling CI failure until a sibling copies the file — a lagging indicator like paired-PR. Note the gap or propose a cross-repo check.

**m7.** §7 SemVer table should explicitly note `ms split`'s stdout format change (removing the bare-strings-first block) as a downstream-script breakage risk (still MINOR).

**m8.** §15 Q3 recommendation: `repair` corrected output should ALWAYS emit unbroken (canonical) — repair is a recovery precision tool; grouping would add separators that confuse re-inspection; also simpler (repair need not thread the flags). User can group after a successful repair.

### Verdict
NOT GREEN — 3 Critical / 9 Important / 8 Minor. Highest-severity: C1/C2 (`ms split` two-part emit vs heuristic removal breaks `ms split | ms combine`) and C3 (`ms combine` positional intake missing — grouped shares from steel can't be decoded, the exact recovery scenario the feature improves). I1–I9 are blocking-important. Before implementation: resolve C1 (concrete `ms split` emit decision), add C3 to §9.2, and address all Important findings.
