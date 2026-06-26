# Word-Card P6 — R0 review (round 1)

- **Phase:** P6 — toolkit integration of the finished `wc-codec` engine (adapter + `mnemonic word-card` CLI + `ToolkitError::WordCard` + fuzz + the engine `MAX_INTEGRITY_BITS 64→63` fix).
- **Branch:** `feat/wc-p6-toolkit` @ `606e0b1e` (parent `master@56a37bf3`, NOT merged).
- **Reviewer:** opus architect, adversarial, gate = 0 Critical / 0 Important.
- **Live worktree (built + exercised):** `.claude/worktrees/agent-a9286242d4b087000` (has the DEV `[patch]`).

---

## VERDICT: **GREEN — 0 Critical / 0 Important.**

P6 clears the per-phase R0 gate and may merge **as a toolkit-side-only branch**. The branch is correctly NOT-releasable as-is (the committed DEV `[patch.crates-io]` blocks publish); the release-seam lockstep (GUI schema mirror, manual, man-pages, version-sites, crates.io publish + dev-patch removal) is **explicitly the orchestrator's job per the commit body** and is out of scope for *this* gate. All Critical-class funds-safety properties verified independently at the CLI level. The engine fix is correct and does not regress the P4 never-wrong-payload floor. Findings below are 3 Minor/Nit (doc-comment staleness, one cosmetic error string) + 1 environmental note (pre-existing clippy red on master, NOT a P6 regression) — none gate-blocking.

---

## Critical
**None.**

## Important
**None.**

## Minor / Nit

- **[Nit, doc-only — adapter]** `crates/mnemonic-toolkit/src/word_card_adapter.rs:154` — the **function-level** doc-comment on `canonical_to_recovered` says the `Md1Descriptor` inverse re-emits via `md_codec::encode_md1_string` (deterministic). The **code** (line 176) correctly uses `md_codec::split`, and the *inline* comment at 173–175 even explains why `encode_md1_string` is wrong (it refuses a multi-chunk md1). So the line-154 summary contradicts its own body. **The code is correct and ships the plan-§2-mandated behavior** (deterministic multi-chunk via `split`); only the summary line is stale. Suggest: `→ md_codec::split (deterministic, multi-chunk)`.

- **[Nit, cosmetic — error string]** A `word-card --from ms1…` refusal prints `error: positional argument 'ms10entrsqqq…' does not begin with a recognized HRP prefix …` — the word "positional argument" is generic `UnknownHrp` Display text, but the input arrived via `--from`, not a positional. Functionally perfect (exit 2, refused, no wrong data); the wording is mildly misleading for the `--from` path. Pre-existing `UnknownHrp` Display, reused; cosmetic only.

- **[Minor, environmental — NOT a P6 regression]** `cargo clippy --all-targets -- -D warnings` (the exact CI gate, `rust.yml:200`) is **RED on master too**: 3 `doc_lazy_continuation` ("doc list item without indentation") errors in `crates/mnemonic-toolkit/tests/readme_dice_kat_parity.rs:14-16`, a file **untouched by P6** and byte-identical on master. This is newer-clippy toolchain drift, pre-dating P6. I verified P6's *own* new code (`wc-codec` all-targets, toolkit lib/bins, the `cli_word_card` test) is clippy-`-D`-clean — P6 adds **zero** new clippy warnings. Flagging so the orchestrator knows CI clippy needs a separate (pre-existing) fix; it does **not** block P6 and should not be charged to this branch.

---

## Suite results

| Gate | Result |
|---|---|
| `cargo test -p mnemonic-toolkit` | **3551 passed / 0 failed / 16 ignored** (matches implementer claim exactly) |
| `cargo test -p wc-codec` | **100 passed / 0 failed / 0 ignored** |
| `cargo clippy` — P6-new code (`wc-codec` all-targets; toolkit lib/bins; `cli_word_card` test), `-D warnings` | **clean** (only pre-existing md-codec deprecation warning from the dev-patched local md-codec) |
| `cargo clippy --all-targets -- -D warnings` (whole CI gate) | RED — but **identically RED on master** (`readme_dice_kat_parity.rs`, not a P6 file). See Minor above. |
| `cargo fmt -p mnemonic-toolkit --check` / `-p wc-codec --check` | clean (toolkit shows only the **mlock.rs** diff, which is the g6 exemption; CI fmt gate `rust.yml:68-77` filters it) |
| `cargo fmt --all --check` sweep | **only** `crates/mnemonic-toolkit/src/mlock.rs` flagged → confirms **NO `cargo fmt --all` was run**; no other file reformatted |
| **mlock.rs g6 exemption** | **byte-identical to master** — SHA256 `cfe41ed9…` on both branch and master; unchanged in the diff |
| `cargo metadata --locked` | exit 0 — Cargo.lock in sync (adds `wc-codec` dep; md/mk path-sourced per the dev patch) |
| fuzz `wc_roundtrip` (cargo-fuzz) | 13,666 runs / 41s / **0 crashes** (the harness that found the t=64 bug; now clean) |
| fuzz `wc_decode_never_panics` | 26,193 runs / 36s / **0 crashes** |

---

## CLI round-trip + safety verification (run against the built binary)

**1. End-to-end round-trip (load-bearing) — PASS.**
- **mk1:** `word-card --from <mk1> --parity-words 8` → `--decode` recovered the EXACT original xpub `xpub6CatW…PC7PW6V`. Re-emitted mk1 differs by `chunk_set_id` (orig `mk1qprsqhpq…` → recovered `mk1qp3qsqpq…`) yet decodes back to the same xpub — confirms the plan-§7-P6 "assert on recovered payload/xpub, NOT the literal string" contract.
- **md1:** round-trip re-emitted the **byte-identical** chunk set (string-deterministic via `md_codec::split`).

**2. Funds-safety at CLI level (Critical class) — PASS.**
- Within budget (1 substitution, m=8 corrects 4): recovers the exact xpub, exit 0.
- Beyond budget (40 words wrecked): **exit 2**, `stderr: "word-card: uncorrectable (beyond RS budget) — refuse"`, **empty stdout**, xpub NOT leaked.
- **Independent CLI fuzz — 400 corrupted decode calls** (1..30 random word substitutions, straddling the budget): **recovered_correct=13, refused=387 (all exit 2), other_exit=0, WRONG_PAYLOAD=0, 0 panics.** The integrity tag is the net — every successful decode yielded the original xpub; every failure refused. **No wrong payload ever emitted.**

**3. Engine fix `MAX_INTEGRITY_BITS 64→63` — VERIFIED CORRECT.**
- The GEOM `t` field is read/written as **6 bits** (`pipeline.rs:421` `gr.read(6)`; `build_geom` packs `t` in 6 bits → max 63). Pre-fix `t=64` would store `64 & 0x3F = 0`, then the decode parser's range check (`pipeline.rs:430`, `!(33..=63).contains(&t)`) would reject `t=0` as `HeaderCrcMismatch`: an encode-accepted-but-NEVER-decodable card (silent unrecoverability — funds-safety hole).
- Now encode **refuses** `t > 63` at `encode_inner` (`pipeline.rs:668`, `WcError::InvalidParams`). Verified via CLI: `t=63` encodes + round-trips to the correct xpub; `t=64` → **exit 2** (`invalid encode/decode parameter`). Default `t=44` unaffected (round-trips throughout). `t=63` is the correct decodable ceiling.
- **P4 floor NOT regressed:** re-ran the wc-codec funds-safety suite — `beyond_budget_refuses_never_wrong`, `miscorrection_caught_by_tag_substitutions`, `miscorrection_forced_toward_wrong_codeword_refused`, `prop_single_corruption_never_wrong_payload`, `prop_recover_or_refuse_never_wrong`, plus the new `integrity_bits_ceiling_is_field_capacity_63` all PASS.

**4. Adapter correctness — PASS.** mk1 via `mk_codec::decode`→`canonical_payload_bytes` (byte-aligned, `8·len`); md1 via `md_codec::reassemble`→`canonical_payload_bytes` carrying exact `total_bits`; inverse md1 via `md_codec::split` (deterministic, multi-chunk) — NOT `encode_md1_string` (the code is correct; only the line-154 doc summary is stale, see Nit). Probed: 2-chunk mk1 + 3-chunk md1 round-trips through the full adapter→wc→adapter path. **ms1 (secret entropy) REFUSED** via `UnknownHrp`, exit 2.

**4b. RAID via CLI — PASS.** `--raid 1` emits `[data,data,data,recovery-a]`; dropping data[0] reconstructs (`reconstructed=[0]`, all 3 xpubs match originals in order). `--raid 2` emits +`recovery-b`; dropping data[0]+data[1] reconstructs (`reconstructed=[0,1]`, xpubs match). Underdetermined drop (r=1, 2 data plates lost) → **exit 2**, "underdetermined" refusal.

**5. `ToolkitError::WordCard` — PASS.** Placed **alphabetically** (between `VerifyMessage` and `XpubSearchNoMatch`) in all four blocks: enum decl, `exit_code` (=2), `kind` (="WordCard"), `Display` (surfaces `WcError`'s own `word-card: …` message). `From<wc_codec::WcError>` impl present. **No panics** on malformed CLI input: empty decode (exit 1), garbage words (exit 2), 5000-word huge list (exit 2), missing `--from` (exit 1), raid-with-1-card (exit 1) — all refuse cleanly.

**6. In-repo lockstep lints — GREEN.** `cli_gui_schema` updated 31→32 subcommands (`word-card` added; assertion text + count bumped). `lint_argv_secret_flags` adds the `word-card --from` route as a **non-secret** `-`-stdin route (correct: mk1/md1 = PUBLIC xpub/descriptor; the engine refuses ms1). Both pass in the 3551 suite.

**7. Dev `[patch]` — VERIFIED.** Committed, clearly marked `# DEV-ONLY (P6) — remove + bump to mk-codec 0.4.1 / md-codec 0.39.1 at release`. **ONLY** `md-codec` + `mk-codec` path entries were ADDED (all `+` lines); the pre-existing `miniscript` patch is **untouched** (no `±` on its line). Cargo.lock reflects the dev state (md/mk path-sourced, wc-codec added) and is `--locked`-consistent. This whole block + the lock's md/mk path-sourcing is DEV state the orchestrator reverts at the release seam.

---

## Lockstep-checklist accuracy (for the orchestrator)

The commit body's scope statement is **accurate**: *"Toolkit-side only; the orchestrator does the release-seam lockstep."* The branch deliberately defers the release-seam items, and the committed dev-patch makes the branch un-releasable until they're done — so deferral is safe, not silent drift. For the orchestrator's release-seam checklist, the following are **OPEN and required** before tag (all confirmed absent on this branch — correctly, per scope):

1. **GUI schema mirror** (`mnemonic-gui/src/schema/mnemonic.rs`) — add the `word-card` subcommand + its flag-NAME set. **The actual shipped flags differ from plan §6.2's names** — mirror the REAL set, not §6.2: `--from`, `--decode`, `--decode-plate`, `--parity-words`, `--parity-pct`, `--raid`, `--integrity-bits`, `--json` (plus the global `--no-auto-repair`, which clap renders on every subcommand). The `schema_mirror` gate is a **lagging** indicator (fires only on the next GUI pin bump) — author the paired GUI PR now per the leading paired-PR rule.
2. **Manual** — `docs/manual/src/40-cli-reference/41-mnemonic.md` has **no** `word-card` chapter, and `docs/manual/tests/cli-subcommands.list` does **not** list `mnemonic word-card`. Note: the manual flag-coverage lint is driven by that **hand-maintained list**, so it will NOT auto-catch the gap (same lagging pattern as the GUI). Add both in lockstep.
3. **gen-man** — `word-card` will auto-appear in `mnemonic gen-man` (clap-derived) once shipped; no source work, but the man-pages tag asset must be regenerated.
4. **Version-sites (§8):** `CHANGELOG.md` (tag-gated `changelog-check`), **both** READMEs, `fuzz/Cargo.lock`, `scripts/install.sh` sibling pins. Remove the dev `[patch]` + bump pins to published **mk-codec 0.4.1 / md-codec 0.39.1** (publish the P0 accessors first). Re-run full suite + fuzz before tag.
5. **Plan-vs-impl verb/flag deviations to record** (the orchestrator should note these so the manual/GUI mirror the *shipped* surface, and ideally back-annotate the plan): plan §6.2 specified a `mnemonic recover` (extend) verb + `--parity-tier <words|pct>` + `--group-size`/`--separator` + `--from <…|@slot>`. The implementation instead ships **`word-card --decode`** (no separate `recover` extension), **`--parity-words`/`--parity-pct`** (split, not a single `--parity-tier`), and **omits** `--group-size`/`--separator` and the `@slot` source. These are defensible UX refinements (and not funds-relevant), but they are **undocumented deviations from the plan's lockstep-surface §6.2** — the commit body does not call them out. Not a P6 gate-blocker (the gate authority is correctness + funds-safety, both GREEN), but the orchestrator MUST mirror the ACTUAL flags and should record the deviation.

Also intact: `ToolkitError` alphabetical ordering, the `wc-codec` fuzz target (both targets committed + run clean), and the cross-repo P0-accessor companions remain a release-seam item.

---

## Diff hygiene
Clean. No `dbg!`/`println!`/`TODO`/`FIXME` in non-test code; all `.unwrap()` hits are in `#[cfg(test)]`. No stray fuzz corpus/artifacts committed (only `Cargo.toml`/`Cargo.lock`/`rust-toolchain.toml`/2 fuzz_targets); the runtime `corpus/`+`target/` are gitignored (new `.gitignore` block for `crates/wc-codec/fuzz/`). Worktree left **pristine** (`git status --porcelain` empty); HEAD unchanged at `606e0b1e`.

---

### Bottom line
**GREEN (0C/0I).** Merge P6 as the toolkit-side-only branch. The funds-safety net (integrity tag refuses, never emits a wrong payload), the engine `t≤63` fix, the adapter payload-not-string round-trip, RAID, and refuse-on-malformed are all independently verified at the CLI level. The only follow-through is the orchestrator's release-seam lockstep (items 1–5 above) + the pre-existing master clippy red (item Minor) — neither charged to this gate.
