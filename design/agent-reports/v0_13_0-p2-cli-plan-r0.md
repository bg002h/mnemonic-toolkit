# v0.13.0 P2 CLI plan — R0 architect-review

**Reviewer:** Opus (architect role) | **Date:** 2026-05-14 | **Plan under review:** `design/PLAN_v0_13_0_p2.md` | **HEAD at review:** `81488e3` (P1c-E.3 R1 LOCK clean)

**Persistence note:** the reviewer agent ran without Write/Bash tool access; full report content captured by the dispatching planner from the agent's final message and persisted here verbatim, with paragraph structure preserved.

## Verdict

**1 Critical / 5 Important / 4 Nit / 5 Notes — RECOMMENDATION: planner folds C1 + I1–I5 inline before P2.1 RED.** I1, I2, I5 affect tests and lint that get RED-pinned at P2.1; folding them late would force a RED rework. C1 is a correctness foot-gun the plan currently silently inherits. I3 reframes Q1 with new evidence (the SPEC G6 row enumeration is +4 because the SPEC-author actually intended to defer one row, not because it was an oversight — see below).

## Answers to §7 open questions

### Q1 (G6 row count: +4 vs +5) — Plan answer is CORRECT (+5/28), BUT the SPEC author's intent matters

Verified by reading SPEC §4 G6 (`SPEC_slip39_v0_13_0.md:267`) byte-by-byte: the SPEC enumerates exactly 4 rows: `slip39 split --from phrase=`, `slip39 split --from entropy=`, `slip39 combine --share`, `slip39 split --passphrase`. **`slip39 combine --passphrase` is omitted.** Independently verified by grep against `tests/lint_argv_secret_flags.rs` precedent: the canonical convention IS one row per (subcommand, flag) pair (e.g., `convert --passphrase` and `convert --bip38-passphrase` are 2 distinct rows; `bundle --passphrase`, `verify-bundle --passphrase`, `derive-child --passphrase` are 3 distinct rows for the same flag name). So the lint convention argues for +5.

But there's a subtlety the plan misses: per SPEC §2.2 combine table, `--passphrase <P>` AND `--passphrase-stdin` are BOTH on combine. So the row IS argv-leakage-bearing. The plan's +5 reading is correct. **Recommend: confirm +5/28; SPEC §4 G6 needs a paired patch ("count 23 → 28") landed at P2.2 GREEN, NOT deferred to PE** (deferral risks the lint shipping with stale narrative).

### Q2 (G4 SHA-pin determinism without `--identifier` override) — Plan option (c) is BRITTLE; recommend (b) with anchor-vector exception

The plan's option (c) — "SHA over normalized envelope MINUS identifier+shares" — defeats the G4 stability claim because the `shares` field IS the secret material whose schema-stability we want to pin. Removing it from the SHA reduces the pin to {`schema_version`, `operation`, `iteration_exponent`, `group_threshold`, `groups[].member_count`, `groups[].member_threshold`} — a SHA over five integers and two strings, which adds ~zero signal beyond field-shape tests.

**Better answer:** option (b) — schema-shape tests only on `--json-out` envelope. For deterministic SHA-pin, add a single hidden `--rng-seed-test-only` flag gated behind `#[cfg(any(test, feature = "test-determinism"))]` so it doesn't appear in production `--help` output (and therefore doesn't pollute the user surface OR the manual lint OR `gui-schema`). Precedent: none in this repo, but the cost is ~10 LOC and it cleanly closes G4. Document the wedge in the SPEC §B.2 stems table as "test-only".

Alternative if test-only flag is rejected: defer G4 SHA-pin to v0.14 cycle, file a FOLLOWUP, and implement schema-shape tests at P2 (still satisfies G4's "stability" intent, just via byte-shape pin not SHA-of-bytes pin). The seed-xor SHA-pin (`tests/cli_seed_xor_json.rs:194`) works because seed-xor split is deterministic via `--deterministic-from-master` — there's no equivalent in SLIP-39 because identifier randomness is part of the spec.

### Q3 (`MemberThresholdMismatch` SPEC drift) — Plan answer (add row 24) is CORRECT

Verified independently:
- `slip39/error.rs:64-66` defines `MemberThresholdMismatch` with the exact narrative "shares within a single group disagree on member_threshold".
- SPEC §2.5 row 15 (`SPEC_slip39_v0_13_0.md:226`) is `DuplicateMemberIndex` — a structurally distinct refusal class. Folding `MemberThresholdMismatch` into row 15 would mean two distinct lib variants render under the same SPEC stem, violating the SPEC §B.2.5 stems-table contract.
- The fold cited in `slip39/error.rs:14-19` doc-comment is rows 4 & 5 (both `BadGroupSpec`), NOT row 15.

**Confirm: add SPEC §2.5 row 24** with stem `slip39 combine: shares within a group disagree on member_threshold`. Row count goes 23 → 24. The SPEC patch lands at P2.2 GREEN paired with the §3.2 mapping table update.

### Q4 (Manual stub at P2.3 vs P3 canonical) — Plan answer CORRECT

Verified `lint.sh:84` — the flag-coverage step does `grep -oE '--[a-z][a-z0-9-]+'` on `--help` output, then asserts each flag appears anywhere in `41-mnemonic.md`. A 30-line stub that lists each flag string in a synopsis is sufficient. Confirm minimum-viable at P2.3; canonical chapter at P3.

### Q5 (extract `emit_world_readable_advisory` shared helper) — Plan answer CORRECT but with caveat

Three call sites is the right threshold. **Caveat:** the plan says "perhaps `crate::secret_advisory::warn_if_world_readable`" — `secret_advisory.rs` is currently 30 LOC and its sole exported fn is for argv-leakage. A path-permission helper is conceptually a different advisory class. Either (a) extract to `crate::secret_advisory::warn_if_world_readable` and add a doc-comment delineating the two classes, or (b) extract to a new `crate::path_advisory` module. (a) is fine; precedent `cmd/seed_xor.rs:425-445` is already an extracted private fn so the move-to-shared is mechanical.

### Q6 (per-share output O(1) pinning with `Vec<String>`) — Plan diagnosis CORRECT; recommend SPEC patch

Verified the foot-gun: a `Vec<String>` has its top-level array of `String` headers heap-allocated contiguously, but each `String`'s UTF-8 bytes live in a separate allocation. `mlock::pin_pages_for(&v[..])` pins the headers (non-secret), NOT the share bytes (secret). The SPEC §2.1 claim "O(1), not O(N)" is structurally unachievable with `Vec<Zeroizing<String>>`.

**Recommend SPEC patch at P2.2 GREEN:** rewrite SPEC §2.1 paragraph "Per-share output pin discipline" to say "per-share output pin (O(N), one pin per rendered share)" OR "single contiguous `Zeroizing<String>` with `\n`-separated shares" (the latter has the alleged O(1) property but breaks the stdout streaming model — every share is held in memory before any byte is written). The toolkit-precedent `cmd/seed_xor.rs:157-164` (cited in SPEC §5 line 12) uses `Vec<Zeroizing<String>>` which has the same O(N) property — so the seed-xor precedent itself is mis-cited for "O(1)".

The right answer for v0.13.0: O(N) per-share pinning is acceptable (max 256 shares × ~22 bytes/share value = ~5.6 KB pinned, well within `ulimit -l`). Update SPEC §2.1 + remove the "single pin" claim. The toolkit ships exactly what seed-xor ships.

### Q7 (gui-schema rendering of nested clap subcommand) — Plan diagnosis is correct AND this is an Important pre-RED probe

Verified `cmd/gui_schema.rs:87-102` (build_schema): `cmd.get_subcommands()` walks ONE level of subcommands, filtered to skip `gui-schema` and `help`. For `mnemonic`'s top-level walk it returns 7 subs (8 with slip39). Then `build_subcommand` (`gui_schema.rs:104-145`) calls `sub.get_arguments()` on each.

For `Slip39Args` which is `#[command(subcommand)]` with no top-level flags, `sub.get_arguments()` returns essentially nothing (just the auto `--help`, which is filtered at line 120). Result: `slip39` would render as `{name: "slip39", flags: [], positionals: []}`. The nested `split` and `combine` sub-subcommands are completely invisible in the schema.

**This is GUI-breaking** (cf. memory `feedback_r2_blocking_vs_cosmetic_gate`: "anything that prevents a test from RUNNING is Important"). Either (a) recurse `build_schema` into nested subcommand trees (preferred — schema becomes hierarchical, GUI gets full visibility), or (b) flatten nested subs into hyphenated names (`slip39-split`, `slip39-combine`) at schema-emit time only. (a) is a SPEC §7 schema extension and needs the GUI-side mirror; (b) is a 5-LOC patch but loses the nested structure.

**Pre-RED probe required:** the plan §6 risk 3 already calls for this. **Make it a P2.1 RED gate**, not "later in the cycle" — if the answer is (a), the schema version bumps from 1 to 2 and that's a GUI-breaking change requiring mnemonic-gui companion work that this v0.13.0 cycle does not budget for.

---

## Findings

### Critical

#### C1 — `--passphrase ""` (clap default) emits NO argv-leakage advisory but argv carries `""`; precedent matches but pattern is brittle for SLIP-39's TWO passphrase channels

**Location:** Plan §3.1 lines 61-66 + 102-107; §3.3 row 1.

**Issue:** The plan declares `#[arg(long = "passphrase", default_value = "")]` for both split and combine. With this clap-derive shape, when the user does NOT supply `--passphrase`, clap fills the field with `""`. The argv-leakage advisory at §3.3 row 1 fires "for inline `--from`, `--share`, `--passphrase`" (per-occurrence). The implementer needs to gate the advisory on "user supplied an INLINE non-empty value", not "field is non-empty". Otherwise either (a) every invocation emits a spurious `warning: secret material on argv (--passphrase)` for the empty default, or (b) the implementer guards with `if args.passphrase.is_empty()` which incorrectly silences legitimate empty-passphrase ARGV occurrences.

**Cross-check against precedent:** `cmd/derive_child.rs:61` uses `Option<String>` for `--passphrase` (NOT `String` with `""` default). `cmd/bundle.rs` uses `Option<String>`. The seed-xor handler has NO `--passphrase` at all. **No existing toolkit handler uses the `String` + `""` default pattern for passphrase.** The plan's pattern is unprecedented and the wrong precedent.

**Fix:** Change `pub passphrase: String` to `pub passphrase: Option<String>` for both split and combine args structs. Drop `default_value = ""`. Add `conflicts_with = "passphrase_stdin"` (matches `cmd/derive_child.rs:68`). The advisory then fires iff `args.passphrase.is_some()` (the user actually supplied the flag, regardless of value).

**Confidence: 90.**

---

### Important

#### I1 — Plan §3.5 `Vec<Zeroizing<String>>` single-pin claim is structurally false; SPEC §2.1 needs paired patch

**Location:** Plan §3.5 last paragraph; SPEC §2.1 "Per-share output pin discipline".

See Q6 answer above. The SPEC text + this plan both inherit a foot-gun that does not match what the code can do. The implementer at GREEN will either (a) ship correct O(N) pinning + the SPEC narrative becomes a lie, or (b) waste a day chasing the impossible "single pin" claim. Either is bad.

**Fix:** Update SPEC §2.1 paragraph to match what the code will actually do (O(N) per-share pinning). Remove `cmd/seed_xor.rs:157-164` from SPEC §5 cross-ref table line 12 as a "single pin" precedent — it is NOT a single-pin precedent, it's an O(N) precedent. Add explicit text to plan §3.5 saying "per-rendered-share `pin_pages_for` call inside the `for share in shares` loop in run_split, mirroring the loop at `cmd/seed_xor.rs:166-169`" — except that loop already does NOT pin per share (the only pin in seed-xor is on the parsed entropy + parsed master phrase, not on the rendered output). So the SLIP-39 O(N) loop pin is NEW pattern, not a precedent reuse. Be explicit about that.

**Confidence: 85.**

---

#### I2 — Plan §3.2 row 23 → SPEC §2.5 row 23 stem string drift; placeholder-to-field mapping unspecified for all 21 interpolated rows

**Location:** Plan §3.2 row 23 vs SPEC §2.5 row 23.

**Issue:** Plan §3.2 maps `GroupThresholdExceedsCount` to row 23 with stem `slip39 combine: share at position J: group_threshold T exceeds group_count N`. SPEC §2.5 row 23 (`SPEC_slip39_v0_13_0.md:234`) reads identically. **But:** the lib variant (`error.rs:120-124`) carries `share_idx: usize`, `threshold: u8`, `count: u8`. The stem MUST be a single rendered string with all three fields interpolated. The plan's "share at position J: group_threshold T exceeds group_count N" uses J/T/N as placeholders. The CLI handler MUST produce something like `slip39 combine: share at position 2: group_threshold 3 exceeds group_count 2`. The plan does not say J = `share_idx`, T = `threshold`, N = `count` — leaving the implementer to guess the field-to-placeholder mapping. The seed-xor precedent (`cmd/seed_xor.rs:340-349`) uses inline `format!` interpolation; the SLIP-39 mapping needs the same concreteness.

**Fix:** Plan §3.2 should render each row's stem as the actual `format!` template the implementer types. e.g., row 23: `format!("slip39 combine: share at position {share_idx}: group_threshold {threshold} exceeds group_count {count}", ...)`. Apply to all 21 rows that carry interpolated fields. This is what the SPEC §B.2.5 stems table is supposed to do — but the SPEC table also uses the I/J/N/T placeholders, so a paired SPEC clarification ("J = `share_idx`, T = `threshold`, N = `count`") is needed alongside.

**Confidence: 80.**

---

#### I3 — Plan §3.3 row 1 advisory description is incomplete; misses the `slip39 combine --passphrase` row that Q1 adds

**Location:** Plan §3.3 row 1.

**Issue:** Plan §3.3 row 1 says "Inline `--from`, `--share`, `--passphrase`". But Q1 establishes there are TWO passphrase channels (split's and combine's), and the plan's §4.3 lint table explicitly adds 5 rows including `slip39 combine --passphrase`. The §3.3 advisory section needs to enumerate `--passphrase` as a per-subcommand occurrence (split's `--passphrase` AND combine's `--passphrase`), NOT a single-row advisory. Per-occurrence-not-deduped is correct (matches `cmd/seed_xor.rs:237-241`), but the implementer needs to wire the advisory call in BOTH `run_split` AND `run_combine` independently.

**Fix:** Plan §3.3 row 1: split row 1 into 5 explicit rows: `split --from phrase=`, `split --from entropy=`, `split --passphrase`, `combine --share`, `combine --passphrase` (matches the §4.3 lint enumeration).

**Confidence: 85.**

---

#### I4 — Plan §3.1 `pub group: Vec<(u8, u8)>` semantically loses the order/index information needed for row 4 stem rendering

**Location:** Plan §3.1 lines 73-75; §3.2 row 4 stem.

**Issue:** The lib `BadGroupSpec` variant (`error.rs:46`) carries `group_idx: usize, n: u8, t: u8`. The CLI surfaces this via `format!("...; got group <idx>=N,T", ...)`. With `Vec<(u8, u8)>`, the `(u8, u8)` tuple's POSITION in the Vec IS the group_idx. The plan's `parse_group_spec` returns `(u8, u8)` (achievable per `parse_range -> (u32, u32)` precedent at `cmd/export_wallet.rs:128-138`), so the index emerges from `args.group.iter().enumerate()` at the lib-call boundary. So far so good — but the plan never spells this out, and the implementer might naively pass `Vec<(u8, u8)>` directly into a `Vec<GroupSpec>` adapter without preserving the relationship between the position in `args.group` and the `group_idx` carried back in the `BadGroupSpec` error.

**Fix:** Plan §3.1 add an explicit transform line: "`args.group.iter().enumerate().map(|(_, (n, t))| GroupSpec { member_count: *n, member_threshold: *t }).collect()` then forward — the lib's `BadGroupSpec.group_idx` is the index INTO this Vec, which equals the index in `args.group` because the transform is order-preserving."

**Confidence: 80.**

---

#### I5 — `cli_help_fixtures.rs` does not exist; plan §5 P2.1 RED commit references it

**Location:** Plan §5 P2.1 RED commit message ("`+ cli_slip39_help_fixtures + ...`") + §9 P2.1 LOCK criterion ("the cli_help_fixtures test must NOT exercise [unimplemented! panic]").

**Issue:** Verified via Glob: `tests/cli_*help*` returns zero files. The plan references `cli_slip39_help_fixtures` as if it's an established convention being extended. It is NOT — there is no `cli_help_fixtures.rs` in the repo to mirror. The plan needs to either (a) create a NEW `cli_slip39_help_fixtures.rs` test file with the convention defined inline, or (b) drop the reference and rely on `cli_gui_schema.rs::gui_schema_lists_all_eight_subcommands` as the sole P2.1 RED check. (b) is sufficient — the gui-schema test exercises the parsed clap surface end-to-end. (a) is overkill for P2.1.

**Fix:** Plan §5 P2.1 RED commit message: drop "`+ cli_slip39_help_fixtures`". §9 P2.1 LOCK criteria: drop the "cli_help_fixtures test" reference. The "stub returning a clean refusal message" pattern is still sound; the verification is via `cargo run -- slip39 split --help` manually + `cli_gui_schema` test. (Plan §9 P2.1 already lists the cargo run check.)

**Confidence: 95.** Verified via Glob.

---

### Nit

#### N1 — Plan §3.5 cites `cmd/seed_xor.rs:122,125-129,130,145,255-260,259,294` — line numbers shift

Verified ground-truth line ranges:
- `secret_in_argv_warning(stderr, "--from phrase=", "--from phrase=-")` is at `seed_xor.rs:122` ✓
- `Zeroizing<String>` master_phrase wrap is at `seed_xor.rs:125-129` ✓
- `mlock::pin_pages_for(master_phrase.as_bytes())` is at `seed_xor.rs:130` ✓
- entropy mlock is at `seed_xor.rs:145` ✓
- Per-share Zeroizing wrap (combine) is at `seed_xor.rs:244-261` (plan cites 255-260; close — actual range is wider)
- `mlock::pin_pages_for` for combine shares is at `seed_xor.rs:259` ✓
- `recovered` mlock pin is at `seed_xor.rs:294` ✓
- `map_seed_xor_error` is at `seed_xor.rs:338-350` ✓
- `SplitJson` + `CombineJson` at `seed_xor.rs:352-371` ✓
- `emit_world_readable_advisory` at `seed_xor.rs:425-445` ✓
- K-of-N TTY advisory: actual lines 192-198 (plan cite 184-198 includes the toolkit-only advisory above; precise range is 192-198)

#### N2 — Plan §3.3 row 4 cites `cmd/final_word.rs:178-197` for the world-readable helper; range verified ✓

`final_word.rs:178` starts `// SPEC §2.6 row 3 — world-readable permission-mode advisory.`, ends at `:197` with closing brace of `#[cfg(not(unix))]` arm. Plan cite is exact.

#### N3 — SPEC §5 cross-ref table cites `cmd/final_word.rs:175-200`; plan cites `:178-197`. Inconsistency between SPEC and plan, both within ±3 lines. Pick one.

#### N4 — Plan §3.4 `SplitGroupEntry` field order does not specify whether `shares` comes after `member_threshold` or in a different position

The seed-xor precedent (`SplitJson` at `seed_xor.rs:352-361`) puts `shares` LAST. Mirror that. Plan §3.4 declaration order is fine but should be explicit: "shares last; member_count, member_threshold first to match struct-init order".

---

### Notes

#### Note 1 — `read_stdin_to_string` trims trailing whitespace; SLIP-39 shares contain spaces

Verified `convert.rs:566-572`: `read_stdin_to_string` calls `.trim().to_string()` on the buffer. SLIP-39 share mnemonics are space-joined sequences of words. A `.trim()` removes leading/trailing whitespace (including the trailing newline from `echo`) but NOT internal spaces. Whitespace tolerance at parse-time is documented at `share.rs:194` ("any run of ASCII whitespace separates words"). Confirmed safe.

#### Note 2 — `read_stdin_passphrase` (cited at `convert.rs:579`) preserves NULL bytes; `read_stdin_to_string` does not

Per `convert.rs:574-590`, the passphrase reader strips ONLY a single trailing `\r?\n` to preserve user-supplied whitespace inside the passphrase. The SLIP-39 plan §3.1 currently routes `--passphrase-stdin` via... unspecified. The plan should explicitly call out `read_stdin_passphrase` (NULL-preserving) for `--passphrase-stdin` and `read_stdin_to_string` (trim-tail-whitespace) for `--from -` / `--share -`. Mismatching these is silently wrong.

#### Note 3 — `slip39-cli-extendable-flag` FOLLOWUP not yet filed

Verified via grep: `slip39-cli-extendable-flag` appears in 0 places in `design/FOLLOWUPS.md`. Plan §5 P2.1 RED commit message includes "filed slip39-cli-extendable-flag FOLLOWUP" — the FOLLOWUP entry needs to be drafted (companion to `slip39-shamir-secret-sharing` at `FOLLOWUPS.md:1039`). The R0 fold should specify the intended companion-line and tier.

#### Note 4 — `gui-schema` self-test (`tests/cli_gui_schema.rs:43`) test name is `gui_schema_lists_all_seven_subcommands`

Verified. Plan §4.2 says "Bump `..._seven_subcommands` → `..._eight_subcommands`" — confirmed exact rename at line 43. The list at lines 49-58 is alphabetically sorted; inserting `"slip39"` at the slot between `"seed-xor"` and `"verify-bundle"` is correct.

#### Note 5 — Per memory `feedback_default_cargo_test_runs_sibling_dependent_tests`, none of the proposed P2 tests should need `#[ignore]` gating

P2 tests are CLI-binary integration tests (`assert_cmd::Command::cargo_bin("mnemonic")`) that exercise only the toolkit binary; no sibling-codec dependency. Default `cargo test` includes them. Confirm no gating needed.

---

## Summary

**1C / 5I / 4N findings** (plus 5 Notes for the planner's awareness).

**Top 3:**
1. **C1** (§3.1): `--passphrase: String` + `default_value = ""` is unprecedented in this repo (`derive_child`/`bundle` both use `Option<String>`); will either spuriously emit argv-leakage advisory on default OR silence legitimate empty-passphrase argv. Switch to `Option<String>`.
2. **I5** (§5/§9): `cli_slip39_help_fixtures.rs` referenced as if convention exists — verified via Glob, no `tests/cli_*help*` file in the repo. Drop the reference; rely on `cli_gui_schema` for P2.1 RED.
3. **Q7** (§4.2/§6 risk 3): `cmd/gui_schema.rs:87-145` does NOT recurse into nested clap subcommands — `slip39` will render as `{name: "slip39", flags: [], positionals: []}` with `split`/`combine` invisible. This is GUI-breaking. Pre-RED probe required at P2.1.

**Recommended next action:** Planner folds C1 + I1–I5 + Q1/Q2/Q6/Q7 inline; runs the gui-schema probe; revises SPEC §2.1 (per Q6) and SPEC §4 G6 (per Q1) in lockstep at P2.2 GREEN. After fold, proceed to P2.1 RED.

**Open question for the user (NOT the planner):** Q2 — do you want a `--rng-seed-test-only` hidden flag for G4 SHA-pin determinism, or defer SHA-pin to v0.14 with schema-shape tests only at v0.13.0? This decision affects the v0.13.0 user-facing surface and is above the planner's pay grade.
