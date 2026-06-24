## Opus ARCHITECT R0 — `reproducible-builds-musl` brainstorm spec (round 1)

**Verdict: RED — 2 Critical / 4 Important.** The spec's *strategy* is sound (digest-pinned Docker, musl-only, stay-on-1.85.0-with-remap, POC-first, CI self-rebuild gate, NO-BUMP) and the fork resolutions are well-reasoned. But the **load-bearing fix recipe is transcribed incorrectly in two independent ways**, both of which would silently ship the path leak (and the $HOME privacy leak) while *appearing* fixed, and the stale-FOLLOWUP premise is itself now stale. These must converge to GREEN before implementation.

Reviewed against recon `cycle-prep-recon-reproducible-builds-musl.md` @ `43298598` (HEAD matches) and live source.

### What's verified CORRECT in the spec
- Toolkit `.cargo/config.toml` ABSENT; descriptor-mnemonic has ONLY `codegen-units=1`+`strip="symbols"`, no remap (§1.3 current-state claim — TRUE). Path-leak open in all 5 repos confirmed.
- CI `tar czf` is bare, no determinism flags (§3.13 is load-bearing — confirmed).
- `--locked` absent from all release builds; no RUSTFLAGS/profile overrides (§3.3 — confirmed).
- 5-near-identical musl jobs, no parity gate; `cross` for aarch64 bundling its own musl C toolchain (F6/§5 premise — confirmed via mk/ms workflows).
- trim-paths nightly-gated on 1.85.0 → remap is the right form (F4 — sound).
- NO-BUMP justification (no clap-flag/wire-shape change → no manual-mirror / schema_mirror trip) — correct.
- The secp256k1-sys/cc-under-musl per-arch double-build as the FIRST gate (§5) — correct instinct and the right showstopper to foreground.

### Critical (must fix before GREEN)
1. **`-Cremap-path-prefix` is wrong** — it's a top-level `--remap-path-prefix` flag, NOT a `-C` codegen option (verified: `rustc -C help` on 1.85.0 does not list it). The recon used the correct form; the spec mutated the proven recipe into one that won't compile, and it propagates into the published rebuilder doc.
2. **`$PWD`/`$CARGO_HOME` in a committed `.cargo/config.toml` do not expand** — Cargo passes `rustflags` array elements verbatim, no shell interpolation. The proof worked because it used the `RUSTFLAGS` ENV channel in a shell. The 'commit it so local installs inherit the fix' design is internally contradictory (and collides with the spec's own override note) and ships the leak unfixed.

### Important
3. The md `reproducible-builds` slug was **already corrected and committed** (`a759c79`, 2026-06-24) — §1.3/§7 'falsely reads RESOLVED, must correct' is stale. Only 'add the actual remap' + 'flip to RESOLVED on ship' remain.
4. **`cross` env-passthrough unspecified** — SOURCE_DATE_EPOCH/CFLAGS/LC_ALL/TZ don't auto-reach the aarch64 container; needs explicit `[build.env] passthrough` in Cross.toml or the cc mitigations silently no-op on half the targets.
5. **Same-day/same-machine A/B gate can't detect the `__DATE__` class** — §5 needs an epoch-is-honored probe (build without/with a different SOURCE_DATE_EPOCH and confirm the .o changes), not just two-paths-same-output.

### Minor
6. F6 cross-repo `workflow_call` adds a supply-chain trust edge (toolkit CI → 4 pipelines) + a new SHA-pin drift surface — weigh against per-repo config.
7. tar `@epoch` is GNU-tar-specific; rebuilders must tar inside the container (or publish the inner-binary SHA too).
8. dtolnay action vs digest-pinned container = two toolchain sources, unreconciled.
9. Trusting-trust scope honesty: provenance is relative to the pinned rustc image, not full source-bootstrap — state it.
10. cgu=16 'reproducible' was measured same-machine/gnu only; lean to mandating cgu=1 (the spec's own belt-and-suspenders) to kill the cross-machine codegen-ordering variable the A/B gate can't see.

### Re-dispatch instructions
Fold all Critical + Important, then re-run R0 (loop continues after every fold per CLAUDE.md). Specifically: (1) correct the flag form everywhere to `--remap-path-prefix`; (2) decide the remap CHANNEL — env (RUSTFLAGS/CARGO_BUILD_RUSTFLAGS in container, variable part resolvable) vs committed config with LITERAL fixed paths — and stop claiming arbitrary local `cargo install` is reproducible-by-default; (3) re-ground §1.3/§7 on `a759c79`; (4) add the Cross.toml passthrough list + an epoch-honored probe to §5.