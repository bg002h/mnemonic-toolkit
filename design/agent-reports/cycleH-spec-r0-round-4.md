# SPEC R0 review — F3 network fail-open (Cycle H) — round 4 (convergence)

**Reviewer:** Fable (SPEC R0 convergence round 4, read-only). SPEC + repo @ `713484c3`.
**Dispatched:** 2026-07-09 (Cycle H, SPEC R0 round 4). Persisted verbatim per CLAUDE.md.

## VERDICT: **GREEN — 0 Critical / 0 Important.** Implementation cleared.

(1 residual Minor at review time — SPEC line 11 tail "four laggards" → "five" — fixed in this fold-and-commit; the edges are exhaustively enumerated so zero implementation ambiguity.)

## Round-3 finding verification

**Important-D — RESOLVED.** All four sub-checks pass against live source:

- **(a) Helper exists, exact semantics.** `vendor/miniscript/src/descriptor/key.rs:1043-1049`, inside `impl DescriptorPublicKey` (key.rs:798): `pub fn xkey_network(&self) -> Option<NetworkKind>` with `Single(_) => None`, `XPub(xpub) => Some(xpub.xkey.network)`, `MultiXPub(multi_xpub) => Some(multi_xpub.xkey.network)`. `NetworkKind` = `bitcoin::NetworkKind` (key.rs:13). A vendored unit test (key.rs:1935-1947) even pins Main/Test round-trips.
- **(b) SPEC snippet uses it and compiles conceptually.** SPEC §1 E5 uses `k.xkey_network()` (with an explicit "do NOT hand-match `XPub` only" warning naming Important-D). The closure returns `bool`, captures the first mismatch, asserts after the walk. Types line up end-to-end: `parsed` is `MsDescriptor::<DescriptorPublicKey>` (bsms.rs:105), `inputs.network: CliNetwork` with `network_kind() -> NetworkKind` (network.rs:30), and `assert_network_agrees(decoded, asserted, &'static str)` (network.rs:94). Guard slot ("after `parsed`, before `derive_first_address`") matches the real code shape (bsms.rs:105-113).
- **(c) Cell E5e present.** §4: multipath `<0;1>/*` tpub + bsms defaults → `NetworkMismatch` exit 2, no address, plus the agreeing `--network testnet` control.
- **(d) Callable in this miniscript version — yes, zero new imports.** `pub trait ForEachKey` yields `&DescriptorPublicKey`; `xkey_network` is a pub inherent method. bsms.rs:40 already imports `ForEachKey`.

**Minor-E — RESOLVED except one residual word** (line 11 "four laggards", fixed here). Title, §0 head, §1 heading, §2 ("known set is E1-E5") all say five; no stale "E1-E4".

## Convergence confirmation

- **Edge set unchanged:** the round-3 fold widened E5's variant coverage (XPub-only → `xkey_network()`) at the same mint site; no 6th edge introduced. Export-emitter audit conclusion stands (descriptor/bitcoin-core/bip388 verbatim, 2-line no mint); §2 still instructs FLAG-don't-widen on any 6th find.
- **Precondition respected on all five:** E1/E2 guard only `Some(n)`; E3/E4 networks always-asserted; E5 skips `Single` via `None` (E5d pins the skip arm). Precondition doc (network.rs:88-93) matches the SPEC citation.
- **Prior folds intact:** E1-E4 unchanged; Minor-B (§2 + FOLLOWUP §3) and Minor-C (`slot.master_xpub`) present; `NetworkMismatch => 2` verified; MINOR v0.83.0, no schema_mirror; §4 cells non-tautological (accept/inference/signet/mk1 controls); §5 corpus-lockstep reminder present.

**R0 gate converged after 4 rounds. The loop caught three live-reproduced wrong-network mints (E4 export electrum/coldcard, E5 BSMS 4-line, the E5 MultiXPub/`<0;1>/*` variant) that a first-cut fix of the original three edges would have shipped. Implementation may begin per the GREEN SPEC.**
