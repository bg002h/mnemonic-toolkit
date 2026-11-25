# FIX: a single mis-stamped stop-sign word makes an intact Word Card unreadable (bh-tk-I6)

Author: Claude Opus 5.5 (claude-opus-5-5), 2026-11-23. Full track (architect table row 36; ruling T39 "fix the codec: validate the stop-sign count against the RS solution"). Grade I (`grade-tk.md:30`). Not a funds-path (class B) row: today the defect fails closed (exit 2, never a wrong payload); the fix must keep "never a wrong payload".

Base: `origin/master` = `68ea05e0752ae5f5343a613376a46456fbdd49f7`. Sources: `design/agent-reports/bug-hunt-2026-11-19.md:43-47` (consultant I6), coordinator `agent-reports/bughunt-1119/triage-tk-t2.md:10` (CONFIRMED, RED log), `grade-tk.md:16,30,56,66`, architect T39.

## 0. Source files this FIX changes (for build ordering)

1. `crates/wc-codec/src/pipeline.rs` (decode: the alternate geometry and the selection rule, section 2; its unit tests, section 4.2)
2. `crates/wc-codec/tests/pipeline.rs` (new KAT section, section 4.1)
3. `crates/mnemonic-toolkit/tests/cli_word_card.rs` (one new CLI row, section 4.3)
4. `design/IMPLEMENTATION_PLAN_word_card_encoding.md` §4.4 (design text, section 3)

No other file. No flag, type, public API, wire format, printed text or exit-code table changes. `wc_codec::decode`'s signature and `Decoded` are unchanged. The toolkit manual is unchanged (section 5).

## 1. The defect, measured

The engraved stream is `[header][ledger 2U][interleave][parity m][stop-sign 2]`. The ledger and the stop-sign sit outside the RS codeword (plan §4.2 "P4 RESOLUTION"). Each carries an 11-bit word count. A ledger slot's 7-bit checksum covers its own marker and count (`pipeline.rs:491-497`). The stop-sign's 7-bit checksum covers only the words BEFORE it (`pipeline.rs:534-541`), so nothing authenticates the stop-sign's own count. Stop-sign word 1 (second to last) holds `marker(4)=0b1111` and the count's high 7 bits. Word 2 (last) holds the count's low 4 bits and the 7-bit checksum.

`decode` (`pipeline.rs:858-884`) bounds the parity region from the stop-sign count whenever the tail marker parses, and prefers it over the ledger (`creation_total = stop_count.or(ledger)`). So:

- **Last word mis-stamped** (marker intact): the count's low 4 bits change in 15 of 16 substitutes, `m` is mis-derived, interleave and parity are mis-sliced, and decode refuses (`Sync(MultiIndelBlock)` or `Uncorrectable`), exit 2.
- **Second-to-last word mis-stamped, marker broken** (15 of 16 substitutes): no stop-sign is seen, the two stop-sign words are read as two extra parity symbols (the parity is prefix-extensible, `append_only_upgrade_preserves_kprime_and_prefix`), and they count as 2 errors against that `m + 2` code's budget of `⌊m/2⌋ + 1`. The card recovers only if the rest of the card has at most `⌊m/2⌋ − 1` errors. With `m = 0` it refuses.
- **Second-to-last word mis-stamped, marker intact, checksum still valid**: the wrong count also raises the recorded length, and an intact card decodes with `truncated: true`.

Measured on `68ea05e0` with a scratch sweep (every one of the 2047 substitutes of each stop-sign word; log `/scratch/cache/fix-tk-stop-sign/red-measure2.log`):

| card (`m`, `U`, seed) + extra damage | second-to-last word | last word |
|---|---|---|
| m=4, U=3, seed 1 | 1925 ok, 118 ok with `truncated: true`, 4 refused | 127 ok, 1920 refused |
| m=8, U=3, seed 2 | 1925 ok, 118 truncated, 4 refused | 127 ok, 1920 refused |
| m=0, U=1, seed 4 | 18 ok, 119 truncated, 1910 refused | 1535 ok, 512 refused |
| m=16, U=1, seed 5 | 1924 ok, 118 truncated, 5 refused | 127 ok, 1920 refused |
| m=4, U=3, seed 6 + 2 parity words mis-stamped | 124 ok, 1923 refused | 127 ok, 1920 refused |
| m=8, U=3, seed 3 + 4 parity words mis-stamped | 124 ok, 1923 refused | 127 ok, 1920 refused |
| m=16, U=1, seed 7 + 8 parity words mis-stamped | 122 ok, 1925 refused | 127 ok, 1920 refused |
| m=2, U=7, seed 8 + 1 parity word mis-stamped | 123 ok, 1924 refused | 1023 ok, 1024 refused |

Each "+ N parity words" control (the same parity damage, stop-sign intact) decodes exactly. No row ever returned a wrong payload. So the manual's promise (`41-mnemonic.md:5113` "corrects `⌊m/2⌋` substitutions", `:5132` "corrects up to 4 mis-stamped words") is unmet for both stop-sign words.

## 2. Spec delta: decode geometry (wc-codec `pipeline.rs::decode`)

Notation, all at decode time: `n` = words present; `interleave_start` = header words + 2U; `interleave_len` = K + checkpoints (both from the CRC-checked GEOM header, as today); `L` = the largest count over the VALID ledger slots only (the value of `recorded_max` after the ledger loop at `pipeline.rs:819-832`, before the stop-sign block can raise it), absent when no slot is valid; `c` = the tail stop-sign's count when the tail marker parses (`stop_count`), absent otherwise.

**Rule 1 (primary attempt, unchanged).** The primary attempt P is today's computation, byte for byte: the same `parity_end`, `parity_start`, `truncated` and the same call to `decode_short_interleave` or `finish_decode`. Its result (Ok or the specific error) is kept, not returned early.

**Rule 2 (when an alternate exists).** An alternate attempt A exists iff `L` is present and one of:
- **2a** the tail marker parses and `c ≠ L`; or
- **2b** the tail marker does not parse and `n == L`.

Otherwise there is no alternate and decode returns P's result exactly as today.

**Rule 3 (the alternate geometry).** A's geometry treats the last two words as the stop-sign and takes the creation total from the ledger: `parity_end_A = n − 2`; `m_A = L − (interleave_start + interleave_len + 2)`; `parity_start_A = parity_end_A − m_A`. If `m_A` underflows, or `parity_end_A − m_A` underflows, or `parity_start_A < interleave_start`, there is no alternate (decode returns P's result). Otherwise A runs `sync::sync_classify` over `[interleave_start, parity_start_A)` and `finish_decode` with parity `[parity_start_A, parity_end_A)`, exactly as the primary non-short path does. A never calls `decode_short_interleave`. A's truncation flag is `n < L` (the stop-sign count is not used: A exists only because the stop-sign disagrees with the ledger).

**Rule 4 (selection; only a unique verified payload is returned).** "Same payload" means equal `kind`, `payload` and `payload_bits`.

| P | A | decode returns |
|---|---|---|
| any | none | P's result |
| Ok | Ok, same payload | A's `Decoded` (its `truncated` and `repair`) |
| Ok | Ok, different payload | `Err(WcError::IntegrityMismatch)` |
| Err | Ok | A's `Decoded` |
| Ok | Err | P's `Decoded` |
| Err(e) | Err | `Err(e)` (P's error, so every existing refusal text is kept) |

The builder implements rule 4 as a private function in `pipeline.rs`, `select_geometry(primary: Result<Decoded, WcError>, alternate: Option<Result<Decoded, WcError>>) -> Result<Decoded, WcError>`, so its rows are unit-tested directly (4.2); the "different payload" row needs two tag-valid payloads, which no card can be made to produce on purpose.

**Why this is safe.** A adds no new acceptance path: its payload passes the same RS decode and the same SHA-256 integrity tag (`rs_decode_and_check`), so a wrong payload still needs a `≤ 2⁻ᵗ` tag collision. When both geometries verify, they must agree or decode refuses. When there is no disagreement (`c == L`, or no ledger, or a lost tail with `n ≠ L` and no marker), A does not exist and decode is today's.

**Edge cases decided.**
- Lost tail whose last two surviving words happen to carry the `0b1111` marker (about 1 in 16): `c` is garbage, so A exists (2a). A's interleave region comes out `L − n` words short, and its `truncated` is `n < L` = true. So the flag is kept whichever attempt wins.
- Lost tail with no marker and `n < L`: no alternate (2b needs `n == L`); today's behaviour.
- Single upstream indel with an intact stop-sign: `c == L`, no alternate; today's behaviour. An indel together with a stop-sign mis-stamp is two conditions; it is not covered (rules 2b and 3 assume `n == L` and an unshifted tail), and P is still tried.
- A mis-stamped ledger slot that keeps a valid checksum (`L` wrong, `c` right): A exists (2a), A fails or verifies the same payload, and P is returned (or A, when both verify the same payload). Its `truncated` flag can still be wrong, as today; section 6 files this.
- Several valid ledger slots (upgrades, plan §4.2): `L` is their maximum, as `recorded_max` is today.
- RAID plates: `raid_reconstruct` decodes each plate with `decode` (`raid.rs:405`), so each plate gets this rule. No RAID-specific change.
- No new error variant and no new printed text. A stop-sign mis-stamp recovered through A prints nothing extra (product point Q1, section 7).

**Code comments the builder updates** (not holder-visible): `pipeline.rs:853-857` and `:877-879` (they say the stop-sign count is preferred and "exact even under upstream corruption"; after this fix the ledger-anchored alternate is also tried when they disagree), and the module doc at `pipeline.rs:39-40` ("mitigated by their own checksums plus the stop-sign cross-check"), which gains a clause naming the ledger-anchored alternate of rule 2.

## 3. Design-doc delta (`design/IMPLEMENTATION_PLAN_word_card_encoding.md` §4.4)

After the bullet that ends "(spec §6.3 / §8 step 1)." (line 277), add this bullet verbatim:

> - **Stop-sign vs ledger disagreement (bh-tk-I6, 2026-11-23).** The stop-sign checksum covers only the preceding words, so its own count is unauthenticated. When a valid ledger record `L` exists and either the tail stop-sign's count differs from `L`, or the tail carries no stop-sign marker and exactly `L` words are present, decode also tries the ledger geometry (parity ends 2 words before the end; `m = L − interleave_start − interleave_len − 2`; truncated iff `words-present < L`). Only a unique tag-verified payload is returned: both geometries verifying different payloads ⇒ `IntegrityMismatch`; both failing ⇒ the stop-sign geometry's error. A single mis-stamped stop-sign word therefore costs no RS budget.

## 4. Tests (RED rows; each function name is its TRACE row name)

### 4.1 Codec KATs: new section "KAT 11 — stop-sign mis-stamp recovery (bh-tk-I6)" at the end of `crates/wc-codec/tests/pipeline.rs`

Payloads are `det_bytes(73, seed)`, `bits = 584`, `SourceKind::Mk1Xpub`, `integrity_bits = 44`, encoded with the file's `enc` helper. "Every substitute" means each of the 2047 symbols `s ≠` the original at that index, word `symbol_to_word(s)`. "Recovers" means `dec` returns Ok with `payload == det_bytes(73, seed)`, `payload_bits == 584`, `kind == Mk1Xpub` and `truncated == false`; the assertion message names `m`, the index and the substitute word.

| row (test fn) | input | expected after the fix | measured on `68ea05e0` (RED) |
|---|---|---|---|
| `stop_sign_last_word_every_substitute_recovers` | cards (m, U, seed) = (4,3,1), (8,3,2), (0,1,4), (16,1,5); every substitute of word `n−1` | all 4 × 2047 recover | FAIL: m=4 1920 refused (1792 `Sync(MultiIndelBlock)`, 128 `Uncorrectable`) |
| `stop_sign_second_to_last_word_every_substitute_recovers` | same cards; every substitute of word `n−2` | all recover, `truncated == false` | FAIL: m=4 118 with `truncated == true`, 4 `Sync(MultiIndelBlock)`; m=0 1910 refused |
| `stop_sign_misstamp_keeps_full_parity_budget` | cards (m, U, seed) = (4,3,6), (8,3,3), (16,1,7), (2,7,8); first `⌊m/2⌋` parity words mis-stamped (index `n−3−j`, `j = 0..⌊m/2⌋`, symbol `(s + 1 + j) mod 2048`); control: that card alone recovers; then every substitute of word `n−2`, and separately of word `n−1`, on top | control and all substitutes recover | FAIL: m=8 second-to-last 1923 refused, last 1920 refused |
| `lost_tail_still_flagged_after_stop_sign_fix` (guard; passes on RED and GREEN) | card (16,3,9); for each `k` in 1..=10, drop the last `k` words | each Ok has `truncated == true` and the exact payload; each Err is `Truncated`, `Uncorrectable`, `IntegrityMismatch` or `Sync(_)` | PASS on both (scratch equivalent: every `k` gives Ok, exact payload, `truncated == true`, on `68ea05e0` and on the reference fix; `guard-red.log`, `guard-green.log`) |

Every existing `wc-codec` test stays unchanged and passes (checked: section 8).

### 4.2 Unit rows for `select_geometry` (in `pipeline.rs`'s `#[cfg(test)] mod tests`, new; the file has none today)

Each builds `Decoded` values directly (`kind: Mk1Xpub`, `payload_bits: 8`, `raid: None`, `repair.erasures_filled` as stated).

| row (test fn) | primary | alternate | expected |
|---|---|---|---|
| `select_geometry_no_alternate_returns_primary` | `Err(Uncorrectable)` | `None` | `Err(Uncorrectable)` |
| `select_geometry_same_payload_returns_alternate` | Ok payload `[1]`, truncated true, erasures 0 | Ok payload `[1]`, truncated false, erasures 2 | the alternate (truncated false, erasures 2) |
| `select_geometry_different_payloads_refused` | Ok payload `[1]` | Ok payload `[2]` | `Err(IntegrityMismatch)` |
| `select_geometry_alternate_only_verifies` | `Err(Sync(MultiIndelBlock))` | Ok payload `[1]` | the alternate |
| `select_geometry_primary_only_verifies` | Ok payload `[1]` | `Err(Uncorrectable)` | the primary |
| `select_geometry_both_fail_keeps_primary_error` | `Err(Sync(MultiIndelBlock))` | `Err(Uncorrectable)` | `Err(Sync(MultiIndelBlock))` |

RED: the module does not compile on `68ea05e0` (`select_geometry` does not exist).

### 4.3 CLI row in `crates/mnemonic-toolkit/tests/cli_word_card.rs`

`word_card_decode_stop_sign_misstamp_recovers_xpub`: encode `MK1` with `--parity-words 4` (92 words; words 87, 88, 91, 92 are `funny life vague shed`), then decode in text mode (`word-card --decode -`, words on stdin) each of the five cases below. Each must exit 0 with stdout containing `truncated: false` and `xpub: ` + `MK1_XPUB`.

RED rows run on `68ea05e0` (`target/debug/mnemonic`), by `/scratch/cache/fix-tk-stop-sign/red-cli.sh <binary>` (sha256 `ca806619836e6295a1e6cf1bcc991915306dc7384eabf3ed906a7868167b5a68`; capture `red-cli.out`). Its card is `WORDS=$(mnemonic word-card --from "$MK1" --parity-words 4 | sed -n '2p')`, and each row is `echo "<word list>" | mnemonic word-card --decode -`. Controls C0 (intact) and C1 (words 87, 88 = `abandon ability` only) exit 0 with `truncated: false` on both builds.

| row | change to the card | today (`68ea05e0`) | after |
|---|---|---|---|
| R1 | word 92 `shed` → `stem` | `error: word-card: uncorrectable (beyond RS budget) — refuse`, exit 2 | exit 0, `truncated: false`, the xpub |
| R2 | word 92 `shed` → `torch` | `error: word-card: sync: >=2 indels in a block could not be localized`, exit 2 | same as R1 |
| R3 | word 91 `vague` → `zoo` | exit 0 but `truncated: true` | exit 0, `truncated: false` |
| R4 | R1 plus words 87, 88 → `abandon ability` | `error: word-card: regroup: trailing pad bits are non-zero (must be zero)`, exit 2 | exit 0, `truncated: false`, the xpub |
| R5 | word 91 → `abandon` plus words 87, 88 → `abandon ability` | `error: word-card: uncorrectable (beyond RS budget) — refuse`, exit 2 | exit 0, `truncated: false`, the xpub |

### 4.4 Tests that change

None. No existing assertion changes. `prop_single_corruption_never_wrong_payload` (it accepts Err) keeps its body; it now also sees Ok at the stop-sign positions.

## 5. Manual and help (lockstep)

No change. The manual never describes the stop-sign. Its repair promise (`41-mnemonic.md:5113`, `:5132`) becomes true for the stop-sign words, and its exit-code table (`:5220`) already covers the only refusal this FIX can add (`IntegrityMismatch`, exit 2). No `--help`, `gui-schema`, `--json` field or GUI manual change. The CHANGELOG entry and version bump go in the release commit after merge, as for earlier tk bug-hunt fixes.

## 6. FOLLOWUPS

Closes nothing in this repo's `design/FOLLOWUPS.md` (bh-tk-I6 has no line there). The report proposes one Minor line for the ledger-side residual of section 2 (a checksum-valid ledger mis-stamp can still set a false `truncated: true`; payload correct, needs a 1-in-128 checksum pass), to be direct-pushed after merge. It is not appended in this PR.

## 7. Open product point (architect confirms; the FIX is written for the recommendation)

**Q1. Should a recovered stop-sign mis-stamp be reported?** After this fix a mis-stamped stop-sign word decodes with no notice. On a detection-only card (`m = 0`) this departs from the manual's note "a corrupted word is flagged but not auto-fixed" (`41-mnemonic.md`, Notes, `--parity-words` vs `--parity-pct`): the stop-sign word is neither flagged nor fixed, but ignored.
- (a) **No notice (recommended; this document).** The stop-sign carries no payload, the payload is tag-verified, and the consultant's fix asks only for "a unique verified payload". No new text.
- (b) Add a stderr notice when A wins and P failed. PROPOSED (architect confirms) text: `word-card: NOTE — the stop-sign (last 2 words) did not match the length ledger; decoded using the ledger length. Re-check the last 2 engraved words.` This needs a new `Decoded` field (a public API change), a `--json` field and manual text. That is out of scope for this row.
- (c) Keep (a) and qualify the manual note. PROPOSED (architect confirms) text, appended to that Notes bullet: `The two stop-sign words at the end carry no payload: a mis-stamp there is ignored when the length ledger confirms the card's length.`

## 8. How this document was checked

- RED: the codec sweep (scratch test file `/scratch/cache/fix-tk-stop-sign/stop_sign_recovery_scratch.rs`, run as `crates/wc-codec/tests/stop_sign_recovery.rs`, then removed) and the CLI script above, run on unmodified `68ea05e0` (logs `/scratch/cache/fix-tk-stop-sign/red-measure2.log`, `red-cli.out`).
- GREEN (scratch, reverted, not committed): a reference implementation of rules 1-4 (`/scratch/cache/fix-tk-stop-sign/proto-fix.diff`). With it, every sweep row in section 1 recovered all 2047 × 2 substitutes with `truncated == false` (`proto-measure2.log`); `red-cli.sh` rows C0, C1 and R1-R5 all exited 0 with `truncated: false` and the xpub (`green-cli.out`); `cargo nextest run --locked -p wc-codec` gave `112 tests run: 112 passed, 1 skipped` (`proto-codec.log`, which includes the scratch sweep); `cargo nextest run --locked -p mnemonic-toolkit -E 'binary(cli_word_card) | test(/word_card/)'` gave `41 tests run: 41 passed` (`proto-cli-tests.log`).
- Not checked here: the full workspace gate, clippy and fmt (the builder runs them; section 9), and the 4.1/4.2/4.3 test bodies as written (only their scratch equivalents ran).

## 9. Gate

The builder's gate is the repo's full CI on the branch head, captured once to a file and grepped. Minimum local line:

`PATH=$HOME/.cargo/bin:$PATH cargo nextest run --locked --workspace --no-fail-fast > gate.log 2>&1; cargo clippy --locked --all-targets --workspace -- -D warnings; cargo fmt --all --check`

The new rows of 4.1-4.3 are seen RED (fail or do not compile) on `68ea05e0` before the `pipeline.rs` change, except the guard row `lost_tail_still_flagged_after_stop_sign_fix`, which passes on both.
