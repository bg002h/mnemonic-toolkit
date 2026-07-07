# SPEC R0 review — concrete-nonranged-xpub-implied-wildcard — round 1

**Verdict: GREEN (0 Critical / 0 Important)** — 5 Minor (non-blocking).
**Reviewer:** adversarial opus architect (read-only). Source verified @ toolkit `91e584dd` (== `e092f679` for all cited files) / md-codec `ef1f3e71` (origin/main).
**Dispatched:** 2026-07-07 (Cycle D, SPEC R0 loop round 1). Persisted verbatim per CLAUDE.md.

The core mechanism — reject a `key_regex` match whose immediately-following char is not `/` — is CORRECT and COMPLETE. The funds property (verify-bundle false-pass closed at re-parse before card comparison) is genuinely achieved. Implementation may proceed after folding (or waiving) the Minors; none gate R0 exit.

## Citation verification (all §4 anchors) — ALL ACCURATE
`concrete_keys_to_placeholders` @330-400 (m @341, m.end() @383, ImportWalletParse "import-wallet: bsms: parse error:"); `descriptor_concrete_to_resolved_slots` @406-418 (remap `.replace("import-wallet: bsms: parse error: ","")`→DescriptorParse @413-414); `key_regex` @37-43 (base58 class excludes `/`, so m.end() sits at first structural char); `make_use_site_path` @325-338 (wildcard_hardened bool, no wild-present field); bare-`@N` accept test @1905-1914 (names this slug); md-codec `wildcard_for` @133-139 (two wildcard arms, no None) + `UseSitePath` @48-54 (no no-wildcard field); CHANGELOG deferred-residual @48 (inside [0.76.0]). No DRIFTED/WRONG. `error.rs`: both `DescriptorParse` + `ImportWalletParse` → exit_code 2 (@597,610) → remap exit-neutral.

## Design-attack results — every vector SAFE
- **False-REJECT of ranged forms — SAFE.** `/` is the only valid non-terminator continuation after an xpub (base58 greedy, `/`,`)`,`,`,`}`,`#`,`<`,`*` all outside base58 class). `xpub/*`, `xpub/<0;1>/*`, nested `sh(wsh(…/*))`/`tr(…/*)`, `xpub/**` (also pre-expanded at verify_bundle:1354) all start `/` → pass. No false-reject possible.
- **MISS non-ranged (false-accept) — SAFE.** `!starts_with('/')` ⟺ "no derivation suffix" exactly. Accepted (`/`-prefixed) keys are provably ranged-and-representable (downstream residue floor + multipath/wild validators). The sole previously-false-accepting case (bare, immediate terminator) is now the rejected case. Complete.
- **Bare `@N` template unaffected — SAFE (structurally unreachable).** `key_regex` requires `[fp/path]xpub`, never matches `@N`; the check is inside the match loop → never runs for a template. Pinned by test @1905-1914.
- **Cycle-A floor interaction (`xpub/0/*`) — SAFE.** New check sees `/` → passes through; floor rejects `/0`. Orthogonal, no overlap/regression.
- **Multibyte/byte-boundary — SAFE.** m.end() at end of all-ASCII base58 → valid char boundary; `&descriptor[m.end()..]` cannot panic; empty tail → reject (correct).
- **Single-choke-point completeness — CONFIRMED (stronger than recon).** Every md1-card-producing/comparing surface routes through `concrete_keys_to_placeholders` (bundle @1849,2104; verify-bundle concrete fork @1368, reject via `?` BEFORE `verify_emit_from_expected` @1373; all 8 import parsers). The two OTHER `--descriptor` commands do NOT bypass it: `export-wallet --descriptor` parses via rust-miniscript `from_str` (@803, faithfully preserves a non-ranged key — no md1 card), `compare-cost` is key-agnostic (@16). `concrete_keys_to_placeholders` is the ONLY toolkit path from a concrete xpub to a `UseSitePath`. Complete.
- **Error type/exit — SAFE.** bundle/verify-bundle → DescriptorParse exit 2; import-wallet → each parser `.replacen("import-wallet: bsms:","import-wallet: <fmt>:",1)` + ImportWalletParse exit 2. No prefix leak PROVIDED the new message uses the byte-exact prefix (Minor 4).
- **§3 REJECT-vs-handle — CORRECT.** UseSitePath has no no-wildcard field; wildcard_for can't return None; the no-fixed-step ruling is prior-R0-blessed (SPEC_cycleA:226). Fail-closed reject is the only funds-safe option short of an out-of-scope wire-format change.

## Minor findings (non-blocking)
- **M1** — §6: add a taproot key-position cell (`REJECT tr([fp]xpub)` exit 2 names @0; `ACCEPT tr([fp]xpub/<0;1>/*)`). Mechanism is script-agnostic (keyed on key_regex) so already handled; exercise it explicitly. (`tr(xpub)` no-origin never matches key_regex → handled by origin-required path, no coverage needed.)
- **M2** — §7 CHANGELOG: primary action is a NEW `## [0.79.0]` entry (funds-framed, "[funds]…" style); LEAVE the shipped [0.76.0] "Deferred residual" line intact (optionally append "(RESOLVED in 0.79.0)"). Don't rewrite shipped history.
- **M3** — §7 omits the FOLLOWUPS.md status flip (OPEN→RESOLVED v0.79.0 in the shipping commit; [[feedback_followup_status_discipline]]). Add to the release ritual.
- **M4** — §5: elevate "message MUST begin byte-exactly with `import-wallet: bsms: parse error: `" to an explicit impl/TDD acceptance line (so the ImportWalletParse→DescriptorParse remap + importer replacen don't leak "bsms:").
- **M5** — §5: (a) state the completeness argument in-SPEC (concrete_keys_to_placeholders is the ONLY toolkit concrete-xpub→UseSitePath path; export-wallet=rust-miniscript, compare-cost=key-agnostic) so it's self-contained. (b) Note the placement side-effect: a key both malformed-base58 AND non-ranged reports the no-derivation reject (check is before xpub decode) rather than "xpub decode failed" — harmless (both exit 2), one-line note so a test doesn't assert the decode message on such input.

## Structural correctness — confirmed
§2 provenance framing CORRECT (lost signal = real-key-vs-hand-typed, not wildcard text). §6.2 anti-C1 anchor GENUINE (replays recon 6-step; reject at verify_bundle.rs:1368 via `?` before comparison @1373; non-tautological end-to-end). §6 matrix otherwise complete (only gap = taproot cell M1). §7 SemVer MINOR CORRECT; codecs NO-BUMP; no manual/GUI `schema_mirror` (private-helper fix, zero CLI-surface delta) — CONFIRMED.

**R0 exit: GREEN (0C/0I).** Fold the 5 Minors (recommended) then re-dispatch a scoped convergence check per reviewer-loop discipline, or proceed to single-implementer TDD (the Minors do not gate).
