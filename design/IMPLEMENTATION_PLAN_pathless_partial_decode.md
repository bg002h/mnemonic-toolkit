# IMPLEMENTATION PLAN — pathless/dead-card partial-decode (md1) — P1

**Status:** plan-doc — **R0 LOOP CONVERGED GREEN (0C/0I)**. Round 1 = OPEN 1C/2I+4M (`design/agent-reports/pathless-partial-decode-plan-r0-round-1.md`) → ALL FOLDED (C-1 → the sh(wpkh) re-pin ships as a SEPARATE prerequisite cycle [P2.0; user decision 2026-07-11]; I-1 unconditional + fatal-in-partial empty-override reject; I-2 chunk-form fixtures + no-intake-change; M1-M4). Round 2 convergence = GREEN + 2 record-only Minors (`design/agent-reports/pathless-partial-decode-plan-r0-round-2-convergence.md`). **IMPLEMENTATION CLEARED — Track A (P0/P1) may start now; P2 gated on the separate sh(wpkh) re-pin cycle.** Source: `design/SPEC_pathless_partial_decode.md` (R0-GREEN). SHAs: descriptor-mnemonic `a39c9d9f`, mnemonic-toolkit `8e240d31` (vendors md-codec 0.40.0).
**Repos / order:** descriptor-mnemonic md-codec (P0) → md-cli (P1) → publish → mnemonic-toolkit (P2). Single implementer, per-phase TDD (tests RED first), per-phase R0 (full package suite), post-impl whole-diff before any tag/publish.
**Decisions carried:** per-call-site opt-in (render commands partial; verify-bundle explicit gate; derive/restore/address/repair STRICT). repair stays strict (partial-repair = follow-up). Guiding principle bound: never silently misrepresent.

---

## PHASE P0 — md-codec (descriptor-mnemonic `crates/md-codec`)

**P0.1 — `Descriptor::unresolved_origin_indices(&self) -> Vec<u8>` (on `validate` semantics).**
- Add a pure query mirroring `validate_explicit_origin_required` (`validate.rs:221-246`): return the ascending `@N` idxs where `canonical_origin(&tree).is_none()` AND the per-idx origin (override-or-`path_decl`) is empty. Returns `[]` when `canonical_origin` is `Some` OR every idx has a non-empty origin. Reuse the existing per-idx walk; do NOT call `expand_per_at_n`.
- TDD RED: unit tests — `[]` for canonical (`tr(@0)`, `wpkh(@0)`, `sh(wpkh(@0))`, `wsh(multi)`) + explicit-origin cards; `[0]`/`[0,1]` for dead shapes (`tr(@0,pk(@1))`, `sh(sortedmulti(2,@0,@1))`, bare `wsh`, raw miniscript).

**P0.2 — partial-allowing decode threaded through THREE layers.**
- Add an opts/param (e.g. `DecodeOpts{allow_unresolved_origin: bool}`, default false) to `decode_payload`, `decode_md1_string`, and `reassemble` — plumbed top-to-bottom. Default (false) = byte-identical current behavior (still rejects `MissingExplicitOrigin`). When true: skip ONLY the `validate_explicit_origin_required` reject (record nothing extra — the caller queries `unresolved_origin_indices()` on the returned `Descriptor`).
- **INVARIANT (assert in code + comment):** partial mode relaxes ONLY the origin gate. Per-chunk BCH, chunk-header consistency (`chunk.rs:346-361`, M-4), index-gap (`:363-372`), and the derived-chunk-set-id/content-id check (`chunk.rs:383-391`) stay enforced. `EmptyTlvEntry` empty-override reject (P0.3) stays FATAL even in partial mode.
- Keep the existing strict public entry points unchanged (existing callers pass default false); add the opt-in surface for md-cli/toolkit render + verify-bundle.
- TDD RED (funds-critical): (a) a MULTI-CHUNK dead card decodes under partial (returns Descriptor, `unresolved_origin_indices()` non-empty); (b) a multi-chunk dead card with a DOCTORED chunk-set-id still REJECTS under partial (content-id oracle intact — RED-proof: temporarily relax :383-391 → this test fails); (c) a non-chunked dead card decodes under partial; (d) strict default still rejects all of the above (the 12 committed `MissingExplicitOrigin` pins stay green).

**P0.3 — `EmptyTlvEntry`-class empty-override reject (I-1: UNCONDITIONAL + fatal-in-partial).**
- An `OriginPathOverrides[idx]` entry that is PRESENT but zero-component must be REJECTED, converging `validate.rs` and `expand_per_at_n` (`canonicalize.rs:465` currently skips its reject via `is_none()` when an empty override is present → silently returns origin `[]`).
- **PLACEMENT (I-1a):** the validate-side reject MUST run UNCONDITIONALLY — BEFORE the `canonical_origin(&tree).is_some()` early-return (`validate.rs:222-224`) OR as its own separate validator — so a CANONICAL-shape wire (`wpkh(@0)`) carrying an empty override is ALSO rejected (else decode passes but expand rejects = the decode/expand divergence SPEC I1 kills).
- **MECHANISM (I-1b, fatal-in-partial):** the empty-override reject must be a DISTINCT error variant from `MissingExplicitOrigin` so partial mode swallows ONLY `MissingExplicitOrigin`. Implement as a SEPARATE validator (runs always) OR have the partial path match-and-swallow specifically `MissingExplicitOrigin`. Do NOT gate all of `validate_explicit_origin_required` behind `if !allow` (that skips the empty-override reject too).
- TDD RED: (a) non-canonical `sh(sortedmulti(2,@0,@1))` + empty `OriginPathOverrides[0]` → decode rejects (was: OK via path_decl fallback); policy-id no longer computes from empty `@0` (was `e54fed29…`); (b) CANONICAL-shape `wpkh(@0)` + empty override → decode rejects (I-1a); (c) empty-override under PARTIAL mode → STILL rejects (I-1b fatal-in-partial). No committed vector carries a legit empty override (all `origin_path_overrides: null`).

**P0.4 — `identity.rs` comment (`:206-214` text, code at `:215`, M-4).** Update the "structurally precluded upstream / unreachable-but-safe" comment (partial `Descriptor`s now exist in-process; policy-id stays fail-closed via `expand_per_at_n`).

**P0 gate:** `cargo test -p md-codec` green; clippy `-D warnings`; per-phase R0. Version bump md-codec (MINOR) staged for lockstep publish with P1 (do NOT publish until P1 code + R0 done).

---

## PHASE P1 — md-cli (descriptor-mnemonic `crates/md-cli`)

**P1.1 — decode/inspect partial render.**
- `md decode` (`cmd/decode.rs`) + `md inspect` (`cmd/inspect.rs`): decode via the partial-allowing entry (`allow_unresolved_origin: true`); then `let unres = d.unresolved_origin_indices()`. If empty → today's behavior EXACTLY (exit 0, byte-identical). If non-empty → render the template as usual + a text line `origin: «unspecified — supply on restore»` (partial only) + stderr note + **exit 4**.
- **`md inspect` — gate the COMPUTATION, not just the output (M-2):** `compute_wallet_policy_id` is called UNCONDITIONALLY with `?` at `inspect.rs:20` → under partial it errors (`expand` raises `MissingExplicitOrigin`) BEFORE any render. Branch on `unres` BEFORE computing the policy-id. On partial, OMIT all THREE policy-id-derived outputs: the `wallet-policy-id` line, the `wallet-policy-id-fingerprint` line (`inspect.rs:62-65`), and the `wallet_policy_id` JSON key. Keep `md1-encoding-id` + `wallet-descriptor-template-id` (no origin dep).
- JSON (hand-built `serde_json::Map`, `decode.rs:19-31`/`inspect.rs:22-49`): conditionally insert `"partial": {"reason":"missing_explicit_origin","unresolved_indices":[…]}` only on partial; keep raw `path_decl:"m"` (do NOT double-represent); `SCHEMA` stays `"md-cli/1"`.
- TDD RED: goldens for each dead shape (text partial render + exit 4 + JSON `partial`); BOUNDARY (canonical shapes byte-identical + exit 0 — any drift fails); chunked dead card partial + exit 4; cross-binary parity deferred to P2 (needs the toolkit).

**P1.2 — `md encode` advisory.** `cmd/encode.rs`: **gate on the FINAL descriptor's actual resolvability (post-impl-R0 I-1): fire iff `!descriptor.unresolved_origin_indices().is_empty()` on the post-`--path`-applied descriptor — NOT the crude `canonical_origin==None && no --path` heuristic** (which FALSE-fires on a valid inline-per-`@N`-origin card that full-decodes, and is BYPASSED by `--path m` on a dead shape). Print a loud stderr advisory (mirror the F-A4 footgun tone) nudging an explicit origin for a fully-decodable backup. Still emits, exit 0. TDD RED: advisory fires on a truly-unresolvable (dead) card, ABSENT on canonical / explicit-origin / inline-per-@N-origin cards; `--path m` on a dead shape → advisory PRESENT (not suppressed); card bytes unchanged.

**P1.3 — repair UNCHANGED** (stays strict). Add a regression test pinning: untouched dead card → `md repair` exit 2; a correction resolving to a dead card → pruned/exit 2 (never Ok/5).

**P1.4 — md-cli manual exit-code table** (in `docs/manual/src/40-cli-reference/42-md.md` in the TOOLKIT repo — the md-cli chapter is hosted there): add exit 4 for decode/inspect partial. (md-cli returns 4 nowhere today.)

**P1.5 — BIP text (M-5).** `bip/bip-mnemonic-descriptor.mediawiki:~218-227`: replace the "pathless/non-canonical backup handling under separate design" placeholder with the partial-decode contract (decode renders template + marks origin unspecified; verify/derive fail-closed). Ship in THIS leg so the tags don't carry stale text.

**P1 gate:** `cargo test -p md-cli` green; clippy; per-phase R0. **Post-impl whole-diff R0 over P0+P1**, THEN release: bump md-codec + md-cli (MINOR lockstep), tag + publish both to crates.io (per the codec release ritual). Note the P1.4 manual line lands in the toolkit repo (P2 leg) — but flag it here as a cross-repo lockstep item.

---

## PHASE P2 — mnemonic-toolkit

**P2.0 — PREREQUISITE (C-1, user decision 2026-07-11): the `toolkit-repin-sh-wpkh-canonical-flip` cycle ships FIRST as its own R0-gated cycle.** It bumps the toolkit off the frozen md-codec 0.40.0 pin onto 0.41.x, activating + testing + documenting the F-A1 `sh(wpkh)→m/49'/0'/0'` flip: 4 flip tests (`bundle` default-origin flip `m/48'/0'/0'/1'`→`m/49'/0'/0'`; `--account≠0` succeed→refuse; `--slot @N.path=` succeed→refuse; pre-bump elided-sh(wpkh) bundle verify-fail) + the companion `canonical-origin-sh-wpkh-toolkit-mirror-divergence` comment updates (6 sites incl. `error.rs:343-349`) + a `gui_schema.rs:1317-1320` `--classify-descriptor` verdict-flip (`non-canonical`→`canonical`) test cell + CHANGELOG/manual migration note + resolve BOTH slugs. NOT part of this plan — its own brainstorm→SPEC→plan→R0→impl. Do NOT "fix" the `synthesize.rs:359-367` single-sig-template mirror (correctly un-flipped). **This plan's P2 ASSUMES the toolkit is already on md-codec 0.41.x (post-repin).**
**P2.1 — pin-bump md-codec + re-vendor.** Bump `md-codec` dep **0.41.x → 0.42.0** (the partial-decode release) in Cargo.toml + root `Cargo.lock` + `fuzz/Cargo.lock` (via `cargo check`, not sed); **re-vendor** `vendor/md-codec` (vendor-freshness gate). Clean bump — NO sh(wpkh) delta (already landed in P2.0). NO install.sh sibling-pin change (see P2.5).
**P2.1b — `EmptyOriginOverride` exhaustive-match arm (COMPILE-BLOCKING at 0.42.0; from the Track-B SPEC-R0 cross-track finding).** The partial-decode md-codec 0.42.0 adds `Error::EmptyOriginOverride{idx:u8}` (P0.3). The toolkit's two exhaustive `md_codec::Error` matches — `md_codec_exit_code()` (`error.rs:505-569`) + `friendly_md_codec()` (`friendly.rs:216-405`) — WILL NOT COMPILE without an arm. Add `EmptyOriginOverride{idx} => 2` (decode-reject class) + a friendly message + update the curated `md_codec_all_arms_render_prose` array (`friendly.rs:608`) + a routing-pin test. (Track B's 0.41.0 re-pin handles the OTHER new variant `MalformedPayloadPadding`; THIS one is the 0.42.0 delta P2 owns.)

**P2.2 — verify-bundle explicit partial gate (funds-critical).** At the supplied-card decode sites `verify_bundle.rs:2450` (policy flow) + `:3045` (descriptor flow): decode via the partial-allowing entry, then query `d.unresolved_origin_indices()`. **Verdict selection (M-1 — NOT an early-return):** run the existing origin-excluding structural compares (`:2903-2923`/`:3077`), then choose `mismatch > partial > ok` — a non-empty `unresolved_origin_indices()` downgrades an otherwise-`ok` verdict to `result:"partial"`/exit 4, but any FAILED structural check → `"mismatch"` (its existing exit) WINS. The partial gate guards the OK verdict ONLY. The `:388` template route + `restore.rs:316` stay STRICT (dead template card → fail-closed `--template is required`/mismatch, exit≠0 — do NOT opt into partial). `:3193`/`:3403` stay strict-and-bail.
- **Test-author note (R0):** at the `:2450` policy flow, `partial` is reachable ONLY with a `canonical_origin==None` EXPECTED tree → use `tr-multi-a`/`tr-sortedmulti-a` (`template.rs:37-41`), NOT wsh-sortedmulti (which is canonical). Chunk-form supplied fixtures (I-2).
- TDD RED (funds-critical): keyed empty-origin `canonical_origin==None` card via BOTH `--descriptor` mode AND the `:2450` taproot-multisig policy flow → `result:"partial"`/exit 4, NEVER ok/exit 0 (RED-proof: without the gate → false-pass); `mismatch` beats `partial`; verify↔restore parity (same card `restore --md1` refuses); dead keyless-template via `:388` → fail-closed exit≠0, not partial.

**P2.3 — `mnemonic inspect` inherits partial** via the shared byte-identical renderer; assert exit 4 + JSON `partial` directly. **I-2: use CHUNK-FORM fixtures (`--force-chunked`/chunked-of-1, mirror `tests/cli_inspect.rs:246` rechunk) for ALL toolkit-inspect + verify-bundle partial tests** — toolkit `mnemonic inspect --md1` cannot decode a NON-chunked single-string md1 today (`inspect.rs:207` is `reassemble`-only; a plain single-string → `unsupported version 2`/exit 3, a PRE-EXISTING intake gap). **P2.3 does NOT change the md1 intake dispatch** (switching `:207` to a dispatching entry is unreviewed scope). File FOLLOWUP `toolkit-inspect-nonchunked-md1-intake-gap`. Auto-repair triggers (inspect `inspect.rs:123-136`; verify-bundle `:2451-2461`/`:3119-3127`) no longer fire on an intact dead card (Ok-partial ≠ decode-err) — assert clean fall-through on BOTH (M-3b).

**P2.4 — repair UNCHANGED** (strict; `repair_via_md_codec` v0.86.0 demote stays Ok-arm-only, unreachable for dead cards). Regression test: untouched/pruned dead card → exit 2.

**P2.5 — manual + release ritual.** Manual lockstep: exit-code tables on decode/inspect/verify-bundle (`41-mnemonic.md`) + a partial-decode subsection; the P1.4 md-cli `42-md.md` exit-4 line. Version bump (MINOR): both README markers, root + `fuzz/Cargo.lock`, **install.sh SELF-pin ONLY — md/ms/mk sibling pins UNCONDITIONALLY FROZEN incl. md-cli despite the md-cli publish (v0.75.0 incident)**; `.examples-build` gen.sh pins + regen (encode advisory may drift `md encode` transcripts — regen with the REAL binary, confirm no unexpected drift). CHANGELOG. Tag on CI-green.

**P2 gate:** `cargo test -p mnemonic-toolkit` green; clippy; `make -C docs/manual lint` + `verify-examples`; vendor-freshness; per-phase R0; **post-impl whole-diff R0** before tag.

---

## FOLLOWUPs
- RESOLVE `pathless-wallet-backup-partial-decode` in BOTH toolkit + descriptor-mnemonic (companion) in the shipping commits.
- FILE `repair-corrupted-pathless-card-partial` (P1 follow-up — opt repair into partial `decode_with_correction` + demote composing with v0.86.0 + indel oracle).
- FILE `verify-bundle-json-partial-result` (GUI informational — `result:"partial"` wire-shape; confirm GUI only displays verify-bundle output, no `result`-string parse, per paired-PR rule).
- FILE `toolkit-inspect-nonchunked-md1-intake-gap` (I-2 — `mnemonic inspect --md1` can't decode a non-chunked single-string md1; `inspect.rs:207` reassemble-only).
- NOTE: `toolkit-repin-sh-wpkh-canonical-flip` + `canonical-origin-sh-wpkh-toolkit-mirror-divergence` are RESOLVED in the SEPARATE P2.0 prerequisite cycle (C-1), NOT here.
- RECORD P2 (per-chain combine) + P3 (single-key wontfix-as-card) dispositions.

## Cross-cutting RED-proofs (must fail before impl, pass after)
1. Chunked dead card + doctored content-id REJECTS under partial (oracle intact) — the funds-load-bearing test.
2. verify-bundle false-pass negative (no gate → ok/exit-0; with gate → partial/exit-4).
3. Boundary: canonical shapes byte-identical/exit-0.
4. Empty-override wire rejected consistently (decode + expand), incl. canonical-shape + fatal-in-partial (I-1a/b).
5. Round-trip: partial-decode → re-encode byte-identical.
6. Cross-binary parity (M-3a): `md decode` template == `mnemonic inspect` template on the same (chunk-form) partial card, both exit 4 (P2).

## Acceptance = SPEC §Acceptance 1-9. Gate at every phase: 0C/0I R0 before advancing.
