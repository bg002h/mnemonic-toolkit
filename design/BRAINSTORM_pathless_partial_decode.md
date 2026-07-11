# BRAINSTORM (draft, pre-R0) — pathless/dead-card partial-decode (md1) — P1

**Status:** brainstorm design, NOT yet R0-gated. Feeds the mandatory R0 SPEC gate. Cross-repo: descriptor-mnemonic (md-codec + md-cli) leads, mnemonic-toolkit ripples.
**Source recon:** md-codec decode data-model map (descriptor-mnemonic HEAD `a39c9d9f`, 2026-07-11) — all file:line cites below are from that.
**FOLLOWUP:** `pathless-wallet-backup-partial-decode` (toolkit `design/FOLLOWUPS.md`, companion in descriptor-mnemonic). Origin: BIP-alignment cycle A1b (dropped hard-refusal → this redesign).

## Guiding principle (standing, user 2026-07-11)
**Maximally expressive on OUTPUT, permissive on INPUT** — BOUNDED by **never silently misrepresent** (permissive ≠ silently-wrong-wallet). Decode → partial-decode-with-placeholders over hard-reject.

## Problem (the "dead card")
`md encode` exits 0 and mints an md1 card for every `canonical_origin=None` shape when no `--path` is supplied (tr+tree, `sh(sortedmulti)`, bare `wsh`, raw miniscript); `md decode` on that card then hard-rejects `MissingExplicitOrigin` (exit 1). An un-readable engraved card = funds trap. Unlike `sh(wpkh)→49'` (one canonical path, fixed by a table entry), these shapes have no single canonical origin.

## Reframe (from recon — changes the mental model)
1. **A dead card and a decodable template are byte-identical except the tree *shape*.** Both carry the same elided origin (`PathDeclPaths::Shared(OriginPath{components:[]})`, i.e. `path_decl:"m"`, depth 0). `tr(@0)` decodes (canonical_origin=`Some(m/86'/0'/0')`), `tr(@0,pk(@1))` rejects (`None`). Nothing is missing *on the card* vs a good one — the gate is purely a function of wrapper topology (`canonical_origin.rs:46-85`).
2. **At the reject point everything is already decoded** — full tree, all `@N`, use-site path. The reject is the LAST step: `validate_explicit_origin_required` (`validate.rs:221-246`) called from `decode_payload` (`decode.rs:75`), after header/path_decl/use_site/tree/tlv + placeholder/multipath validation all pass. The template renderer `descriptor_to_template` (`render.rs:52-62`) would render the dead card perfectly.
3. **The RESOLVED (canonical-filled) origin is never shown** — a successful `tr(@0)` never displays its internal `m/86'/0'/0'` (used only by `compute_wallet_policy_id` via `expand_per_at_n`). **[architect I2 correction]** BUT the RAW elided origin IS emitted in `--json` as `path_decl` (`format_origin_path`, `json.rs:222-263` → `"path_decl":{"data":"m","tag":"Shared"}`); the original reframe's "never anywhere" was wrong for `--json`. Text-mode `descriptor_to_template`/`render_key` (`render.rs:543-572`) emit only `@{idx}` + use-site path — no origin.

⇒ "partial-decode-with-placeholders" is really: there are NO missing keys (it's a template — keys ARE `@N`); the template always renders; the only thing "missing" is the RESOLVED derivation ORIGIN, which the text output doesn't display anyway. So the "placeholder" is a NOTE + a newly-surfaced origin line on the partial case (text), not a literal token — and the additive `--json` `partial` field must be COHERENT with the already-emitted raw `path_decl:"m"` (don't double-represent), with `wallet-policy-id` omitted on partial.

## Decisions (locked with user)
- **Partial outcome, not reject:** decode/inspect render the (always-renderable) template + mark origin unspecified; **exit 4 (VERIFY-ME)** (reuses the existing repair-demote/BSMS-lenient idiom). Full decodes stay **exit 0, byte-identical**.
- **Surface origin on PARTIAL ONLY:** partial prints `origin: «unspecified — supply on restore»`. Successful output unchanged (no shared-renderer churn, no verify-examples transcript rewrite, no manual rewrite). (Surfacing origin on ALL decodes = a separate, higher-churn enhancement, deferred.)
- **Encode advisory:** when `md encode` emits a `canonical_origin=None` card with no `--path`, loud stderr advisory (mirror F-A4 footgun) nudging `--path` for a fully-decodable backup; still emits, exit 0.
- **JSON:** additive `partial: {reason, unresolved_indices}` field (existing `Option`/`skip_serializing_if` convention allows additive without a tag bump; `SCHEMA="md-cli/1"` `json.rs:6`). Lean: additive-optional, bump to `md-cli/2` only if an existing field also changes. On partial, `wallet-policy-id` unavailable; `md1-encoding-id` + `wallet-descriptor-template-id` (`identity.rs:71-96`, no origin dep) still emitted.
- **P2 reframed / P3 wontfix:** P2 (fixed single-chain `/0/*`, ubiquitous per BIP-389 motivation) → separate sub-project reframed as "accept per-chain descriptors, combine to `/<0;1>/*`" (permissive input, likely no wire change). P3 (non-ranged single key, niche) → keep the v0.79 reject, steer to a watch-only descriptor file (consistent with the keyless-concrete ruling). Neither built in P1.

## Architecture — how "partial" threads through the shared decoder (ARCHITECT-FOLDED)
**Verdict: Approach A with a MANDATORY Approach-C fallback on `verify-bundle`** (architect OPEN 1C/3I → folded; `design/agent-reports/pathless-partial-decode-brainstorm-architect-r0.md`). The naive "relax the shared gate, rely on `expand_per_at_n` everywhere" is UNSAFE: `verify-bundle` never calls `expand_per_at_n`/derive/policy-id on the supplied card, and the two origin checks are NOT the same predicate.

**Do NOT globally relax `decode_payload`.** A PARTIAL-ALLOWING decode is opted into per-call-site by the render-only commands; every origin-consuming/verifying command keeps strict behavior:

- **Read/render paths — partial-allowing (A applies):** `md decode`, `md inspect` (text template), toolkit `mnemonic inspect`. Render `descriptor_to_template`, never consume a resolved origin → permissive partial + marker + exit 4 (policy-id line omitted on partial).
- **Derive/policy paths — already fail-closed via `expand_per_at_n` (VERIFIED COVERED):** `restore --md1` (`restore.rs:3181`), `md address`, `md inspect` policy-id, toolkit `address`/`addresses`/`export-wallet` faithful path. Safe for the realistic dead-card shape (`Shared(empty)`, no override).
- **`verify-bundle` — EXPLICIT partial gate (Approach C, MANDATORY — architect C1):** verdict compares EXCLUDE origin by design (`verify_bundle.rs:2903-2923` multisig, `:3077` singlesig; template via origin-invariant `compute_wallet_descriptor_template_id`) and it NEVER calls `expand_per_at_n` on the supplied card → relaxing decode would emit `result:ok`/exit 0 on a keyed empty-origin card that `restore` REFUSES (verify↔restore funds divergence, same class as v0.76/v0.79). Fix: after decoding the supplied card, query `unresolved_origin_indices()`; if non-empty → `result: partial`/**exit 4**, NEVER a pass, regardless of structural match. CANNOT be delegated to `expand`.
- **`repair` — partial correction is NOT Blessed (architect I3):** repair uses decode-success as the BCH-correction validity oracle (`repair.rs:118`/`:1491`/`:1641`) → relaxing decode enlarges the accepted set. A correction resolving to a PARTIAL (unresolved-origin) card MUST demote to Unverified/exit-4, never `Blessed`/exit-5 (consistent with the v0.86.0 non-chunked demote).

**The `unresolved_origin_indices()` query (architect I1):** build it on `validate_explicit_origin_required` SEMANTICS (`validate.rs:221-246`), NOT `expand_per_at_n`'s — the two DIVERGE on an empty `OriginPathOverrides[idx]` entry (`expand` `canonicalize.rs:465` skips the reject when an empty override is present → silently returns origin `[]`; probe-proven policy-id divergence `e54fed29…` vs `eb6d5e10…`). Additionally add an `EmptyTlvEntry`-class reject for empty origin overrides so the decode-gate and `expand` semantics converge (closes a latent bug independent of this feature).

**Approach B (rejected):** opt-in `--partial`/`--allow-pathless` flag — leaves reject-by-default, the opposite of the principle; a naive `md decode` still dead-ends.

## Components
1. **md-codec:** relax `decode.rs:75` gate to non-fatal; expose `Descriptor::unresolved_origin_indices() -> Vec<u32>` (the `validate.rs:221` predicate as a query).
2. **md-cli `decode`/`inspect`:** on unresolved → template + `origin: «unspecified»` line + stderr note + exit 4; `--json` `partial` field; `wallet-policy-id` shown unavailable on partial.
3. **md-cli `encode`:** F-A4-style advisory on `canonical_origin=None` + no `--path`.
4. **mnemonic-toolkit:** `mnemonic inspect` inherits the partial render via the shared byte-identical renderer (render-only, safe). `restore` already fail-closes via `expand_per_at_n` (`restore.rs:3181`) — no change. **`verify-bundle` needs an EXPLICIT partial gate (architect C1):** its compares exclude origin and it never calls `expand` on the supplied card, so it MUST, after decoding the supplied card, query `unresolved_origin_indices()` and force `result: partial`/exit 4 when non-empty — never exit-0-verified. `bundle` mints (self-check default-fills the empty non-canonical origin, `bundle.rs:2272-2297`) — unaffected.
5. **`repair` (md-cli + toolkit) — partial-correction disposition (architect I3):** a BCH correction that resolves to a PARTIAL (unresolved-origin) card must NOT be `Blessed`/exit-5; demote to Unverified/exit-4 (or exclude from the accepted set), reusing `unresolved_origin_indices()`.

## Data flow
encode (no --path, non-canonical) → card + stderr advisory (exit 0). decode/inspect → template + origin-unspecified + partial marker (exit 4). address/policy-id → refuse (unchanged, via expand_per_at_n). verify-bundle on a partial → partial/exit-4, never verified-pass.

## Testing
- Golden before/after for each `canonical_origin=None` shape (tr+tree, `sh(sortedmulti)`, `sh(multi)`, bare `wsh`, raw miniscript body).
- BOUNDARY: `tr(@0)`/`wpkh(@0)`/`sh(wpkh(@0))` (canonical) stay exit 0, byte-identical (RED-proof: any drift in successful output fails).
- `address` on a partial still refuses (fail-closed preserved).
- verify-bundle on a partial → exit 4, NOT exit-0-verified (funds-critical negative test).
- Round-trip: partial-decode → re-encode byte-identical (the card didn't change).
- Cross-binary parity: `md decode` template == `mnemonic inspect` template (shared renderer).
- JSON: `partial` field present on partial, absent on full; `md1-encoding-id`/`wallet-descriptor-template-id` present on partial, `wallet-policy-id` absent.

## Scope / SemVer / BIP
- md-codec + md-cli MINOR (new decode behavior + JSON field + encode advisory). Toolkit MINOR (exit-code behavior change on the affected call-sites). Manual lockstep (decode/inspect exit-code tables + a partial-decode subsection). GUI schema: exit-code/behavior, not a flag → informational only.
- md1 BIP: fill in the "pathless/non-canonical backup handling under separate design" placeholder with this partial-decode contract.

## Architect verdict (FOLDED — Fable, OPEN 1C/3I → all folded above)
Full report: `design/agent-reports/pathless-partial-decode-brainstorm-architect-r0.md` (traced real code + runtime probes).
- **C1 (Critical, folded):** `verify-bundle` false-pass under a global relax → EXPLICIT partial gate (Approach C for verify-bundle only), never delegate to `expand`. [→ Architecture + Component 4]
- **I1 (folded):** the decode-gate and `expand` origin checks DIVERGE on an empty override → build `unresolved_origin_indices()` on `validate` semantics + add an `EmptyTlvEntry` reject. [→ Architecture]
- **I2 (folded):** reframe #3 corrected — the raw `path_decl` IS emitted in `--json`; only the RESOLVED canonical origin is hidden. [→ Reframe #3]
- **I3 (folded):** repair partial-correction → non-`Blessed`/exit-4. [→ Component 5]
- **M1 (noted):** round-trip byte-identical (re-encode canonicalizes indices only, not origin) — keep the RED-proof; the hazard doesn't exist on current code.
- **M2 (noted):** exit-4 no collision on decode/inspect; document in the manual exit-code tables.
- **P2-reframe caveat:** the per-chain combine MUST fold `/0/*`+`/1/*` into `/<0;1>/*`, NOT accept a bare fixed step — else it re-opens the v0.76 `descriptor-use-site-collapse` class. P3 wontfix sound.

**Post-fold status:** sound basis for the formal SPEC (architect concurs). Next: user approval on this revised design → write `design/SPEC_*` → **mandatory SPEC R0 gate** (per CLAUDE.md) before any code.
