# SPEC R0 round 2 — bundle-md1-template-multisig (#28 phase 2) (opus architect, verbatim)

> Reviewer: opus architect (read+bash; toolkit `e97e8470`/src `cbdadbb7`; md-codec 0.37.0 registry copy; mk-codec **0.4.0** registry copy = the actual lib dep; rust-miniscript 13.1.0). **Verdict: RED — 0 Critical, 1 Important.** I-A + I-B CLOSED in the normative body; 3 Minors folded; C1 invariant preserved + stronger. One NEW Important: §9 line 154 retained pre-fold "reuse the rebuild as origin source" language contradicting §4.2a + line 155. One-line fix → re-dispatch.

---

**Verdict: RED — 0 Critical, 1 Important.**

## I-A — CLOSED (normative body)
- `compute_default_origin_path` hardcodes BIP-48 (`bundle.rs:2144-2168`, purpose=48 fixed `:2151-2154`) — SPEC's "NOT this" correct.
- Cited rebuild is overwrite-sink not origin source: `verify_bundle.rs:1080-1148` — `default_path=compute_default_origin_path(network,account)`, ALL slots that one path (`(0..n).map(|_| default.clone())` `:1087`), overlays only `--slot @N.path=` (`:1106-1135`), binding loop reads rebuilt `path_decl` not mk1 (`:1175-1182`). Literal reuse → every slot BIP-48 → degrade2 never matches. §4.2a:78 now labels it write-SITE-only. Correct.
- Respecified build correct + sufficient: own from `--account`/`--origin` (actual purpose) + cosigner mk1 `origin_path` → fresh `path_decl` (`Divergent`) → `compute_wallet_policy_id` on it (`identity.rs:172`→`expand_per_at_n:185`→hashes `e.origin_path:193`; `canonicalize.rs:436-444` reads `path_decl` when no overrides — `OriginPathOverrides` not needed, M2 correct). degrade2 now completable (own purpose 84' not forced BIP-48). §7 adds the load-bearing pin (compute_default_origin_path build must FAIL) + multi-account own resolve-all-4.

## I-B — CLOSED
- `KeyCard` exposes `origin_fingerprint: Option<Fingerprint>` (`key_card.rs:36`), `origin_path: DerivationPath` (`:42`), `xpub` (`:53`); `mk_codec::decode` returns it. Extraction feasible.
- Today's `--cosigner @N=` (`restore.rs:1952-1990`) requires `=` (`:1953`, no unassigned parse), decodes to `supplied65` for the cross-check only (`:1978-1986`), never reads origin. Both the unassigned parse + origin extraction genuinely net-new. §4.2a:78 + §9:155 enumerate + flag net-new. Correct.

## Folded Minors
- M1 CLOSED (§2:28 reworded: exists both, multisig wiring new). M2 CLOSED (`path_decl` write sink, no `OriginPathOverrides`-as-target in the body). M3 CLOSED (§6.2:111 labels `astelem.rs` rust-miniscript; md-codec wire-order is `tree.rs:115-139`).

## Drift/regression
- **C1 invariant PRESERVED + STRONGER:** §4.2a builds fresh, "does NOT consume the template's carried `path_decl`" → carried origin NEVER reaches `compute_wallet_policy_id`. Never-loaded > overwritten-before-gate. No path loads carried template origins into the completion descriptor.
- Round-4 GREEN intact: §3.2 conditional decode (`validate.rs:222/242`; `canonical_origin.rs:57-77`), I_new ladder, distinct-keys, sortedmulti (id never sorts `identity.rs:172-229`, address sorts `to_miniscript.rs:354-409`), emit guards (`synthesize.rs:987/1005/1013`).
- §3.2-carries-origins vs §4.2a-builds-fresh CONSISTENT (decode/display-only vs build-fresh split, explicit §3.2:54 + §4.2a:73).

## Important
### I-NEW — §9 line 154 retained pre-fold "reuse the rebuild as origin source" language; contradicts §4.2a + line 155.
§9:154 said "supplied-key origins win at completion via the `verify_bundle.rs:~1080-1150` rebuild). Plan-time: confirm the rebuild reuse + … the carried-origin-overwrite invariant test." This is the brainstorm-era framing I-A repudiated (reuse-the-rebuild → BIP-48 → degrade2 never matches). The next bullet §9:155 (correct, citing I-A/I-B) says "BUILD per-slot `path_decl` … NOT `compute_default_origin_path`." The two bullets are mutually exclusive. §9 is the plan-lift checklist → a stale bullet naming the killed primitive as a confirmation target is a real implementer/plan-author stall (could re-introduce the dead-on-arrival path). Also "carried-origin-overwrite" is now imprecise (§4.2a: never-LOADED, the stronger invariant). Important, not Critical (normative body correct; C1 intact; no silent-wrong-wallet/decode-break/regression). **Fix:** reword §9:154 to match §4.2a (general→carry for DECODE-VALIDITY only; completion builds fresh `path_decl` from supplied keys, rebuild SITE reused NOT origin source; plan-time = Guard-C relaxation + carried-origin-NEVER-LOADED test). Or merge 154/155.

## Minors (non-blocking)
1. Pin label: SPEC says "mk-cli v0.10.0", but the toolkit's linked lib dep is `mk-codec = "0.4.0"` (`Cargo.toml:35`, `Cargo.lock` 0.4.0 registry). I-B feasibility holds (KeyCard origin fields present in 0.4.0). Label the LIB crate (mk-codec 0.4.0) at the pin line, not just the CLI binary.
2. Once 154 reworded, keep the SITE cite, drop the "win via / origin source" verbs.
3. §6.2 ladder matches round-4 — no action.

## Verdict
**RED — 0 Critical, 1 Important.** I-A/I-B closed + every source fact verified; 3 Minors folded; C1 invariant stronger. Single blocker: §9:154 fold-completeness gap (stale reuse-the-rebuild language). Bounded one-line fix; re-dispatch.
