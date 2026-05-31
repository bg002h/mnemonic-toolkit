# R1 Review — SPEC_non_english_seed_advisory.md

Opus architect, continuing from R0 (RED 2C/1I/3M). Verified the fold vs live source @ `9f11a31`. Persisted by controller.

## Fold verification
- **C1 RESOLVED** — §2/§3.2 state the SeedQR footgun doesn't exist (`convert --to seedqr` refused `convert.rs:866-871`/`:1207`; `seedqr encode` English-only `seedqr.rs:11/102/120/142/179`); convert trigger is `Entropy` only. (Residual stale `"a SeedQR"` example in the §3.1 helper doc → M-new-1, folded post-review.)
- **C2 RESOLVED** — `enum NodeType` (`:31`), `to: Vec<String>` (`:226`) → `targets: Vec<NodeType>` (`:850`); trigger `targets.contains(&NodeType::Entropy)` evaluated ONCE. No `ConvertTarget` string remains.
- **I1 RESOLVED + gates verified** — slip39 split (`cmd/slip39.rs:132` `--language`, `--from phrase=/entropy=`) always-fires (phrase→entropy→shares, `:436/485/497`, language genuinely lost); combine (`:171` `--language`, `--to Slip39ToShape::Entropy` `:185`/`Phrase` `:188`) fires only on `--to Entropy` (`run_combine:647-657`). Single chokepoint per subcommand confirmed.
- **M1/M2/M3 RESOLVED** — raw-entropy-without-language limitation documented; citations fixed (`seedqr.rs:11`, `NodeType`); kebab-name (`SimplifiedChinese`→`"simplified-chinese"`, `language.rs:29`) in §5.

## Additional checks
- `CliLanguage` derives `PartialEq` (`language.rs:8`) → helper compiles; `human_name()` (`:26`) kebab.
- bundle single emit `emit_unified` (`bundle.rs:698`), all 3 branches converge once; `any_secret_bearing()` suppresses watch-only.
- **No missed site** — export-wallet (watch-only, secrets refused), verify-bundle (consume-to-verify), derive-child (key-target class), seed-xor (BIP-39-phrase shares carry language), final-word (phrase), xpub-search — all correctly excluded.
- §2 ↔ §3.2 ↔ §5 ↔ §6 consistent on 4 sites + SeedQR-impossible.

## CRITICAL — None.  ## IMPORTANT — None.
## MINOR
- M-new-1 stale `"a SeedQR"` helper-doc example (folded post-review).
- M-new-2 qualify `slip39.rs` → `cmd/slip39.rs` (same-named `src/slip39/` sibling) — plan-doc.
- M-new-3 convert trigger line refs `:850/874` bracket the parse loop; insertion is post-loop+empty-guard (~`:880`) — pin in plan-doc (prose already correct).

## VERDICT: GREEN (0C/0I/3M) — clear to plan-doc.
The two highest-value checks pass (slip39 gates correct; split genuinely loses the language); 4-site scope complete; no fold drift. The Minors are doc/citation precision for the plan-doc.
