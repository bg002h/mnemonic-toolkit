# ms K-of-N — End-of-Cycle R0 Review

**Reviewer:** opus architect (mandatory end-of-cycle R0, CLAUDE.md hard gate — last gate before crates.io publish + git tags).
**Scope:** ms-codec/ms-cli `ms-v0.2-kofn` (tip `2743568`), toolkit `ms-v0.2-kofn-toolkit` (tip `fe55d75`), mnemonic-gui `ms-v0.2-kofn-gui` (tip `ec9f00b`).
**Ground truth re-read:** codex32 0.1.0 `lib.rs` (`interpolate_at` 217-309 verbatim), ms-codec `shares.rs`/`envelope.rs`/`error.rs`/`consts.rs`/`decode.rs`/`encode.rs`, ms-cli `combine.rs`/`split.rs`/`inspect.rs`/`error.rs`, toolkit `ms_shares.rs`/`friendly.rs`/`language.rs`/`secrets.rs`/`main.rs`, GUI `schema/ms.rs`/`schema/mnemonic.rs`/`tests/schema_mirror.rs`, all version/CHANGELOG/install.sh/README/MIGRATION/FOLLOWUP/manual sites, and the four prior persisted reviews (P1-R0, P2-R0, P3-R0, P3-R1).

**Verdict:** GREEN (0C / 0I)

---

## Critical (0)

None.

The four Critical-class crypto hazards are each disproven against source:

- **No secret leak / index-`s` never distributed** — `encode_shares` (`mnemonic-secret` `crates/ms-codec/src/shares.rs:148-155`) distributes only `defining[1..]` (the k−1 CSPRNG-filled defining shares) + `interpolate_at`-derived non-`s` indices; `defining[0]` (the secret-at-S) is never pushed to `distributed`. The id is `getrandom`-based (`shares.rs:40-55`), all N derived internally, no public `derive_share`. ZERO/n=1 path keeps `id = tag` (`shares.rs:115`), not random.
- **C1 index-`s` rejection ordering** — verified against codex32 `lib.rs:259-262`: the `if indices[i] == target { return Ok(shares[i].clone()) }` short-circuit fires in the *second* loop, bypassing the lazy `RepeatedIndex` (`lib.rs:283`) and ALL payload validation. `combine_shares` pre-rejects any index-`s` input (`shares.rs:207-209`) at step 2 — BEFORE the count check (step 3), distinct-index check (step 4), and `interpolate_at(Fe::S)` (step 5). Order is exactly: parse → reject-index-`s` → count≥k → distinct → interpolate. Test `combine_rejects_secret_share_index_s` (`shares.rs:467`) confirms.
- **Byte-identity invariant** — `payload_wire_bytes` (`envelope.rs:199-217`) is a verbatim extraction of `package`'s `[prefix]||payload` assembly; `package` (`envelope.rs:228-240`) calls it; `encode_shares` ZERO path reduces to the same `from_seed(HRP, 0, tag.as_str(), Fe::S, &bytes)`. Pinning tests `zero_share_is_byte_identical_to_encode_{entr,mnem}` (`shares.rs:307,314`) plus `encode_output_unchanged_after_split_refactor.rs` (full text+json for english/japanese/hex) gate it.
- **Threshold validation** — `Threshold::new` admits 2..=9 only (`shares.rs:72-78`); `ZERO` is a const (`:68`), not `new(0)`; `encode_shares` enforces `k ≤ n ≤ 31` (`:122`). `dispatch_payload` (`envelope.rs:167-188`) is header-gate-free, re-dispatching the recovered prefix byte post-interpolation; `combine_shares` hard-codes `Tag::ENTR` and discards the random id (`shares.rs:242`).

## Important (0)

None.

The two Phase-3 R0 Importants (I1 language advisory on `combine --to entropy`; I2 friendly prose for codex32 share errors) were folded in `6e6b97b` and verified GREEN in P3-R1. Independently re-confirmed at the tip:
- I1: `ms_shares.rs:420-428` emits `non_english_seed_advisory(recovered_lang, "raw entropy")` keyed off the RECOVERED mnem payload's wire language (`wire_code_to_cli`), not `args.language`. `--to phrase`/`--to ms1` correctly emit no advisory.
- I2: `friendly.rs:55-75` renders prose for `Codex32(ThresholdNotPassed/RepeatedIndex/MismatchedLength/MismatchedHrp/MismatchedThreshold/MismatchedId)`; the generic `Codex32(_)` Debug arm (`:76`) remains only for non-share codex32 errors. Tests at `friendly.rs:397-458` assert no `{`/`Fe(`/variant-name leakage.

## Minor (3)

### M1 — `cli_language_to_wire_code` carries a now-stale `#[allow(dead_code)]`
- **Where:** toolkit `crates/mnemonic-toolkit/src/language.rs:54`.
- **What:** The attribute predates this cycle; the fn is now genuinely used at `ms_shares.rs:269` (split's non-English → `Payload::Mnem` route). The allow is superfluous, not harmful — clippy `-D warnings` does not flag a redundant `#[allow]` by default, and both P3 reviews ran clippy clean. Cosmetic; safe to leave or drop in a future sweep.

### M2 — `ms combine --to entropy` (ms-cli) does not emit a non-English-seed advisory, unlike the toolkit's `mnemonic ms-shares combine --to entropy`
- **Where:** ms-cli `crates/ms-cli/src/cmd/combine.rs::emit_entropy` (`:124-142`).
- **What:** The toolkit deliberately added this advisory (P3-R0 I1) because it mirrors `slip39.rs`. ms-cli has **no** `non_english_seed_advisory` infrastructure anywhere (confirmed: `ms encode` also doesn't emit it — `encode.rs` only prints an informational "language: X" line). The non-English-seed advisory was a toolkit-only feature (v0.37.11). So ms-cli's silence is internally consistent and NOT a regression; SPEC §3 does not require it. The mnem language is preserved on `--to phrase`/`--to ms1`. Flagged only as a cross-tool consistency observation for a possible future ms-cli advisory cycle. Does not gate ship.

### M3 — toolkit `SplitJson`/`CombineJson` wire-shapes are leaner than ms-cli's siblings
- **Where:** toolkit `ms_shares.rs:489-505` (`SplitJson` emits `{schema_version, operation, threshold, shares}` — no `n`/`id`/`kind`/`language`; ms-cli's `SplitJson` carries all of those).
- **What:** Explicitly ungated by SPEC §6 / the filed FOLLOWUP `ms-kofn-json-wire-shape-ungated` (companion entries in both repos). The schema_mirror gate is flag-NAME parity only (confirmed: `tests/schema_mirror.rs:52-54` compares only `f.name` sets). Paired-PR self-update discipline applies. Not a ship blocker.

---

## Compliance matrix

- **§1 threshold-field dispatch** → `envelope.rs:110-127` (discriminate routes `'0'`→proceed / `b'2'..=b'9'`→`IsShareNotSingleString` / else→`ThresholdNotZero`); `decode.rs:42-64` union-length gate routes shares before kind-bind. ✓
- **§1 bounds 2≤k≤n≤31** → `shares.rs:72-78,122`. ✓ (`non_s_index_pool` `shares.rs:28-34` = 31 indices; `n=32` rejected.)
- **§1 0x01 unallocated / orthogonal axes** → `MIGRATION.md:11-23`; `consts.rs:60-72`. ✓
- **§2 `Threshold`** (`ZERO` const + `new(2..=9)` + `get`) → `shares.rs:62-84`. ✓
- **§2 `encode_shares(tag, Threshold, n, &Payload)`** (derive-all-N, getrandom, ZERO byte-identical) → `shares.rs:101-159`. ✓
- **§2 `combine_shares` C1+I2+I3 pre-validation** → `shares.rs:180-243`. ✓
- **§2 `payload_wire_bytes` extraction** → `envelope.rs:199-217`, reused by `package` (`:229`) + `encode_shares` (`:108`). ✓
- **§2 `dispatch_payload` header-gate-free** → `envelope.rs:167-188`, reused by `discriminate` tail (`:150`) + `combine_shares` (`:241`). ✓
- **§2 errors alphabetical-among-themselves + Display arms only** → `error.rs:90-112` (InvalidShareCount, InvalidThreshold, IsShareNotSingleString, SecretShareSuppliedToCombine) + matching Display `:166-186`; no `exit_code`/`kind` on `ms_codec::Error`. ✓
- **§2 `RESERVED_ID_BLOCKLIST` retains `mnem`; distinct from `RESERVED_NOT_EMITTED_V01`** → `consts.rs:62,71-72`. ✓
- **§3 `ms split`/`ms combine`/`ms inspect`-of-share** → `split.rs`, `combine.rs`, `inspect.rs:36-48,60-99`. ✓
- **§3 ms-cli exit mapping** (share errors→FormatViolation/exit 2; Invalid*→BadInput/exit 1; wildcard fronted) → `ms-cli/src/error.rs:199-247`. ✓
- **§4 `mnemonic ms-shares split|combine`** → `ms_shares.rs`; enum+dispatch `main.rs:121,176`. ✓
- **§4 toolkit friendly + exit-code arms** → `friendly.rs:106-127`; `ms_codec_exit_code` arms (P3-R1 confirmed). ✓
- **§4 combine `--to entropy` recovered-language advisory; `--to phrase|ms1` preservation** → `ms_shares.rs:392-447`. ✓
- **§4 `Vec<Zeroizing<String>>` for shares** → `ms_shares.rs:342,364-368` + transient `Zeroizing<Vec<String>>` view `:383-384` (M1 of P3-R0 resolved). ✓
- **§5 MIGRATION + SPEC_ms_v0_1 §4/§5/§8 amendment** → `MIGRATION.md`, confirmed in P1-R0. ✓
- **§6 GUI schema mirror** → `schema/ms.rs` (`split`/`combine` + `--to` enum) + `schema/mnemonic.rs:1176-1293,3344-3357` (`ms-shares-split`/`ms-shares-combine`); `cli_gui_schema.rs:76-110` pins 28 flattened subcommands; `--share` secret in both `secrets.rs:62` and GUI mirror. Flag-name + dropdown-value parity verified. ✓
- **§6 manual lockstep** → `43-ms.md`, `41-mnemonic.md`, `cli-subcommands.list:22-23,55-56`. ✓
- **§8 tests** → byte-identity, K-of-N round-trip (entr+mnem, all 5 lengths, k∈2..9), C1, bounds, decode/inspect routing, toolkit round-trip + composition — all present and GREEN in the persisted per-phase suites (ms-codec 127/0, ms-cli 0-fail, toolkit 2626/0/12 at P3-R1). ✓

## Ship-step reminders (NOT findings)

- **Remove the toolkit `[patch.crates-io] ms-codec = { path = ... }`** at `Cargo.toml:18-20` AFTER publishing ms-codec 0.4.0 + ms-cli 0.7.0 to crates.io; then `cargo build` to relock and assert `Cargo.lock` ms-codec carries `source = "registry+..."` + a checksum, and `cargo metadata --locked` passes.
- **Publish order:** ms-codec 0.4.0 → ms-cli 0.7.0 → push `ms-codec-v0.4.0`/`ms-cli-v0.7.0` → remove toolkit override + relock → commit toolkit re-pin → tag `mnemonic-toolkit-v0.40.0` → merge default branches → flip SPEC §8 / FOLLOWUP statuses.
- **GUI:** the `schema_mirror` gate is a lagging indicator — it fires on the next mnemonic-gui pin bump to v0.40.0. The `ms.rs` mirror is also gated by `ms_schema_flag_names_match_help_text` against the installed `ms` v0.7.0 binary; ensure GUI CI `pinned-upstream.toml` bumps `ms-cli`/toolkit tags in lockstep when the GUI PR lands.
- **Clean working tree** before the checkout→ff→tag→push sequence (`git status --porcelain` empty).

**Bottom line:** The most crypto-sensitive code (encode_shares all-N derivation, combine_shares C1 ordering, byte-identity, Tag-from-id avoidance, threshold-field dispatch) is correct against codex32 0.1.0 source. The error taxonomy is alphabetical, exit-code-consistent across ms-cli and toolkit, with both wildcards correctly fronted. Versions (ms-codec 0.4.0 / ms-cli 0.7.0 / toolkit 0.40.0), CHANGELOGs (2026-06-03), both README markers (0.40.0), install.sh self-pin + ms-cli pin, MIGRATION amendment, manual, and GUI mirror are all in lockstep. **0 Critical / 0 Important — GREEN to ship** once the path-override removal + relock ship steps execute.
