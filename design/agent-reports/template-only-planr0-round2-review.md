# Plan-doc R0 round 2 — bundle-md1-template-only-option (opus architect, verbatim)

> Reviewer: opus architect (read+bash; pins md-codec 0.36.0 == descriptor-mnemonic `c85cd49`, mnemonic-key `913febc`, toolkit local `4e21d94`). **Verdict: GREEN — 0 Critical, 0 Important.** Two new compile-completeness Minors + three citation-precision Minors, all non-blocking / compiler-or-grep-caught. UNCOMMITTED (freeze). Plan-doc R0 converged. Both folded below.

---

**Verdict: GREEN — 0 Critical, 0 Important.**

## I1
**CLOSED.** The fold (plan §P1.2 + the I1 paragraph) correctly diagnoses and resolves the round-1 finding, verified against source:
- `synthesize_descriptor(descriptor: &Descriptor, ...)` is by SHARED ref at `synthesize.rs:258-263` — cannot mutate in place. Confirmed.
- `synthesize_unified:776` builds the keyed descriptor at `synthesize.rs:835-850` (`fingerprints: Some(...)` `:845`, `pubkeys: Some(...)` `:846`, populated `path_decl` `:837-840`) and delegates at `:859`. Confirmed.
- The 4 mutation points all exist and are reachable on a `descriptor.clone()`: stub at `:272-274`, csi at `:290`, the `debug_assert!(descriptor.is_wallet_policy())` at `:346`, and the elide target `path_decl.paths = PathDeclPaths::Shared(OriginPath{components: vec![]})` (`OriginPath.components` field at `md-codec/origin_path.rs:49`; for n==1 the path is always `Shared` per `synthesize.rs:816`). The encoder writes `path_decl` verbatim at `md-codec/encode.rs:85` and empty origin round-trips (`OriginPath::write` depth-0 at `:61`, `read` at `:70`). All four read the mutated clone since they all consume the same `descriptor` binding/param. Confirmed implementable as written.
- The three descriptor-mode callers the fold names — `bundle.rs:1616` (verified: `synthesize_descriptor` in descriptor-mode `run`), `:1726`, `:1969` — all exist and call `synthesize_descriptor`. Confirmed.

Caveat (see Drift check / Minors): the fold's threading directive understates the blast radius of the *signature* change — but that is a compile-completeness Minor, not a reopening of I1.

## I2
**CLOSED.** All three sub-points (a)/(b)/(c) verified correct and sufficient against `restore.rs`:
- **(a) tree→type:** Today restore iterates `ALL_SINGLE_SIG` at `restore.rs:328-331` (`None => &ALL_SINGLE_SIG`) over the `:339` loop, deriving all four. The fold correctly requires mapping the md1's single encoded `d.tree → CliTemplate` to derive ONE type. Sound. (Citation imprecision noted in Minors: it is NOT literally the inverse of `script_type_from_template:402` — that maps `CliTemplate → Option<ScriptType>`, not tree↔template — but the implementer intent is unambiguous.)
- **(b) `--from`-required hole:** GENUINELY CLOSED in plan text. Verified the hole is real: `--from` carries `#[arg(long, required_unless_present = "md1")]` at `restore.rs:60-61`, so `restore --md1 <template>` with no `--from` is clap-valid and the `:177-179` dispatch routes it to `run_multisig` (watch-only) — the silent mis-route. The fold's explicit "template-completion arm MUST REJECT a missing `--from`" closes it. Confirmed.
- **(c) restore-side typed Descriptor:** Verified restore today builds a STRING via `build_descriptor_string` (`restore.rs:387`, imported `:39`). The fold correctly names the typed builder `build_descriptor:131` (`synthesize.rs:131-159`), which produces the fully-keyed (`pubkeys: Some` `:154`, `fingerprints: Some` `:153`), explicit-origin (`template.md_origin_path(network, account)` `:140`), presence-`0b11`, `n:1` `md_codec::Descriptor` — byte-identical preimage to the bundle side. The I4 same-preimage requirement is independently corroborated by `compute_wallet_policy_id`'s INVARIANT doc (`md-codec/identity.rs:172-185`): it does NOT consult `canonical_origin`, so the elided template would hash differently — the explicit-origin typed builder is mandatory on both sides. Confirmed correct and sufficient.

## Folded Minors
- **M1 — CONFIRMED.** §3/§P2.1 now say **mnemonic-key**. `derive_stub_from_md1` is at `mnemonic-key/crates/mk-cli/src/cmd/mod.rs:63` (body `:63-70`); repo confirmed `mnemonic-key` (workspace members `mk-codec`/`mk-cli`). It calls `compute_wallet_policy_id` unconditionally with NO keyless branch; stale doc present at `mod.rs:55-62` ("top 4 bytes of the policy's **WalletPolicyId**"). Needs the `!is_wallet_policy()` branch. Correct.
- **M2 — CONFIRMED.** §P1.3 cites ms1 live encode at `synthesize.rs:339` — verified live (`ms_codec::encode` inside `synthesize_descriptor`). `:172` is dead (`ms_codec::encode` inside `#[allow(dead_code)] synthesize_full`, fn at `:163-164`). Correct.
- **M5 — CONFIRMED.** `bundle.rs:1616` is a REAL second emission path (descriptor-mode `synthesize_descriptor`, reached via `--descriptor`), distinct from the `--template` path through `synthesize_unified:421`. Plan P1.1 correctly applies the gate to both. Correct.
- **M6 — CONFIRMED but INCOMPLETE (see Minors).** The exit-code block (`error.rs:501`, `SlotInputViolation=>2` at `:550`) and the name/kind block (`:562`, `:615`) are both exhaustive (no catch-all) over `ToolkitError` — a new variant without arms fails to compile. So the two cited sites are correct. **However** there is a THIRD exhaustive block — `message():628` (`SlotInputViolation { message, .. } => message.clone()` at `:804`, no catch-all) — that the plan does NOT cite and that ALSO fails to compile without an arm. (`details():847` has `_ => None` so is exempt; `Display::fmt:875` delegates to `message()`.)

## Drift check
No new Critical or Important.

One **NEW Minor** the I1 fold introduced (flagged below, non-blocking): the fold directs threading an `Md1Form`/`bool` param through the SIGNATURES of `synthesize_descriptor:258` and `synthesize_unified:776`, but enumerates only the descriptor-mode trio (`bundle.rs:1616/1726/1969`). A signature change forces updates at all callers. Additional **production** callers not enumerated: `verify_bundle.rs:427/525/630` (`synthesize_unified`) + `:1104` (`synthesize_descriptor`), and `import_wallet.rs:1455` (`synthesize_descriptor`); plus `bundle.rs:421` (`synthesize_unified`, the `--template` path — mentioned in prose but not the fold list) and ~8 test callers in `synthesize.rs`/`parse_descriptor.rs`. All pass `Md1Form::Policy` (today's behavior) — mechanical, compiler-caught, zero funds-safety/behavioral impact. Same class as M6 ("else it won't compile").

The C1 emit-gate and C2 restore-ingest predicates remain internally consistent and source-faithful (`canonical_origin.rs:48-79` Some-shapes incl. canonical multisig `:58-70`; `n==1` load-bearing; bare-wsh→None `:234`). I4/D7 same-preimage corroborated by the `identity.rs:172-185` invariant doc. Undisturbed.

## Minors (non-blocking)
- **M6-extension (new):** add a `message()` arm at `error.rs:628` (exhaustive, no catch-all) alongside the cited exit_code/name arms — else it won't compile. Plan P1.1's "else it won't compile" enumeration is incomplete.
- **I1 signature-threading blast radius (new):** fold the non-enumerated production callers (`verify_bundle.rs:427/525/630/1104`, `import_wallet.rs:1455`, `bundle.rs:421`) + test callers into P1.2, each passing the `Policy` default; or note "re-grep all callers at execution and pass the default" so the implementer doesn't stall.
- **I2(a) citation imprecision:** "inverse of `script_type_from_template:350`" is loose — `script_type_from_template` (`convert.rs:402`) is `CliTemplate → Option<ScriptType>`, not a tree↔template map; the needed map is `d.tree → CliTemplate` (a tag-match mirroring `canonical_origin`). Reword for the implementer; intent is clear.
- **path_decl mutation phrasing:** P1.2 says elide `path_decl` origin to `OriginPath{components:vec![]}`; the exact field write is `descriptor.path_decl.paths = PathDeclPaths::Shared(OriginPath{components:vec![]})` (n==1 ⇒ always `Shared`). Tighten phrasing.
- Round-1 M3/M4 line-number nits (`expected_stub` at `:2162` not `:2158`; reassemble `:1229`, refusal `:1232-1238`) are correctly reflected in the plan-doc body (§P1.4 says `:2158/2187/2236`; §8 names `self_check_bundle`). No action.
- Pre-existing §4.3-style "four re-root sites" wording in the SPEC vs the live `:272/:290` production-only sites is already corrected in the plan (P1.3). No action.

## Verdict
**GREEN — 0 Critical, 0 Important.** Both round-1 Important findings (I1 form-threading + by-ref mutation; I2 restore tree→type / `--from`-required / typed D7 preimage) are CLOSED with source-verified evidence; M1/M2/M5/M6 folds confirmed. Two new compile-completeness Minors (the third exhaustive `message():628` match arm; the wider signature-threading caller set) and three citation-precision Minors are non-blocking and compiler/grep-caught — fold them into P1.1/P1.2 at execution per the standing re-grep discipline. Plan-doc is cleared for phase-1 implementation once the SeedHammer freeze lifts and the user gives an explicit go (per-phase R0 still gates each phase).
