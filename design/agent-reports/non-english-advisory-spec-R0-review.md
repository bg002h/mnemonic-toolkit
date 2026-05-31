# R0 Review — SPEC_non_english_seed_advisory.md

Opus architect, mandatory pre-impl R0. Branch `non-english-seed-advisory`, base `master`
`9f11a31`. Verified vs live source. Persisted by controller. (RED 2C/1I/3M.)

## Headline confirmations
- **`CliLanguage` already `#[derive(… PartialEq …)]`** (`language.rs:8`) — the "add if absent" is moot. `human_name()` lowercase/kebab.
- **Bundle single chokepoint REAL** — `emit_unified(args, &bundle, …)` (`bundle.rs:698`); all 3 dispatch branches converge once (slot/template `:388`, descriptor `:1442`, from-import-json `:1674`); `run` dispatches to exactly one (`:223-286`). `Bundle::any_secret_bearing()` (`synthesize.rs:35`) in scope. The highest-risk assumption holds.
- **No bip39 auto-detect** — every parse is explicit `Mnemonic::parse_in(language, …)`; `--language non-English` reliably signals a non-English phrase.
- **bip48 stderr-only precedent real** (`bundle.rs:349-353`, `export_wallet.rs:340-344`, no manual lockstep). `--json` is stdout-only (advisory stderr-safe). export-wallet is watch-only (no ms1).

## CRITICAL
- **C1 — `convert --to seedqr` DOES NOT EXIST** (refused at parse, `convert.rs:867-871` + `Seedqr => unreachable!` `:1207`). The SPEC's convert→SeedQR trigger is dead, and §2's "standalone seedqr is covered by `convert --to seedqr`" is FALSE. Net real-world: the toolkit can emit a non-English SeedQR by NO path (seedqr encode English-only + convert --to seedqr refused) → the SeedQR footgun genuinely does not exist. Fix: drop `Seedqr` from the convert trigger + the `"a SeedQR"` form; rewrite §2/§3.2 to say no non-English SeedQR is producible at all.
- **C2 — phantom type `enum ConvertTarget`** (§2 cited `:31`). Live: `convert.rs:31` is `enum NodeType`; `--to` is `pub to: Vec<String>` (`:226`) → `targets: Vec<NodeType>` (`:850`), **multi-target** (`--to xpub,entropy`). Trigger must be `targets.contains(&NodeType::Entropy)` evaluated ONCE (`:874`), not a single-`ConvertTarget` match / per-target loop (double-fire risk).

## IMPORTANT
- **I1 — missed sibling sites: `slip39` (fix-the-class).** `slip39` split (`--from phrase=` + `--language` `:132`) → SLIP-39 shares (lose BIP-39 language); combine (`--language` `:171` + `--to Slip39ToShape::Entropy` `:185`) → raw entropy. Both are language-losing emits with `--language` in hand. Cover them (consistent with the user's "wide" choice) or defer-with-FOLLOWUP. `seed-xor` shares are BIP-39 phrases (carry language) — not a miss.

## MINOR
- **M1 — raw-`entropy`-slot-without-`--language`** (`bundle.rs:595-632`): a `--slot @0.entropy=<hex>` with no `--language` emits ms1 but has no language signal → silent by necessity. Document as a known limitation (the advisory keys off declared `--language`).
- **M2 — citation drift:** seedqr "English only" doc is `seedqr.rs:11` (not `:4`); `NodeType` not `ConvertTarget`.
- **M3 — test a kebab-name language** (e.g. `simplified-chinese`, `language.rs:29`) in addition to `french`, to lock the message format.

## VERDICT: RED (2C / 1I / 3M)
Bundle half sound (chokepoint real, no auto-detect bypass, PartialEq present). Convert half built on wrong facts (C1 dead --to seedqr; C2 phantom ConvertTarget / Vec multi-target). I1 slip39 siblings. Fold → re-dispatch.
