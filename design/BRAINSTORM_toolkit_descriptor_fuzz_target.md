# BRAINSTORM — toolkit `parse_descriptor` fuzz target (cfg(fuzzing) mount)

Status: DRAFT (pre-mini-R0). 2026-06-12. FOLLOWUP
`toolkit-descriptor-fuzz-target` (descoped from stress Cycle C). Completes
the constellation fuzz program (md/ms/mk codec fuzz shipped; this adds the
toolkit's descriptor-string door).

## Problem

`crates/mnemonic-toolkit/src/parse_descriptor.rs::parse_descriptor`
(`(input: &str, keys: &[ParsedKey], fingerprints: &[ParsedFingerprint]) ->
Result<MdDescriptor, ToolkitError>`) is the toolkit's untrusted
descriptor-string intake (rust-miniscript parse → walk → md-codec Node).
It has NO coverage-guided fuzzing. It is BIN-PRIVATE (declared
`mod parse_descriptor;` in main.rs, NOT exported from lib.rs — the locked
Option C crate shape, `SPEC_secret_memory_hygiene_v0_9_B.md` §4 P2;
`ToolkitError` is binary-private by design). The Cycle-C R0 measured its
transitive `crate::` closure at ~35 modules / ~56k lines, so a fuzz crate
can't path-dep it as-is.

## Approach (Cycle-C R0's design — to be empirically validated by mini-R0)

A `#[cfg(fuzzing)]` mount in lib.rs that exposes parse_descriptor + its
closure to a fuzz crate ONLY during cargo-fuzz builds:

1. **lib.rs:** `#[cfg(fuzzing)] pub mod fuzz_surface { ... }` (or a flat set
   of `#[cfg(fuzzing)] pub mod <m>;` declarations) mounting parse_descriptor
   + every module its `crate::` paths reference. The module FILES are shared
   with the bin crate (a file can be included in both bin + lib crates);
   `crate::` inside them resolves against whichever crate root includes
   them — so lib.rs must mount the FULL closure so the paths resolve.
   **BLAST RADIUS:** `cfg(fuzzing)` is set ONLY by cargo-fuzz (`--cfg
   fuzzing` via RUSTFLAGS); in every normal build (cargo build/test, CI
   rust.yml, the shipped binary) the mount is ENTIRELY ABSENT — zero effect.
2. **Cargo.toml:** `[lints.rust] unexpected_cfgs = { check-cfg =
   ['cfg(fuzzing)'] }` — so clippy `-D warnings` (CI) doesn't flag the
   unknown `cfg(fuzzing)`. This ONE LINE is the only normal-build-visible
   change. (Verify the toolkit CI runs clippy `-D warnings` — rust.yml.)
3. **fuzz/ workspace** (own `[workspace]`, pinned nightly, like the codec
   fuzz dirs): path-deps the toolkit LIB crate; replicates the root
   `[patch.crates-io] miniscript = { git … rev = 95fdd1c }` (Cargo patches
   don't cross workspace boundaries; the closure uses
   `Terminal::SortedMultiA` @ parse_descriptor.rs which exists only in the
   patched rev); committed `fuzz/Cargo.lock`.
4. **fuzz target** `descriptor_parse`: bytes → utf8-lossy → `mnemonic_toolkit
   ::fuzz_surface::parse_descriptor(input, &[], &[])` (empty key/fingerprint
   slices are valid); never-panic (libFuzzer). Optionally a fixed-point
   oracle if a re-emit path is reachable — but the primary is never-panic
   (parse_descriptor on arbitrary strings must return Err, not panic).
5. **CI** `fuzz-smoke.yml` (or extend an existing one): the gnu-target pin
   (`--target x86_64-unknown-linux-gnu`, per the Cycle-C musl/ASan gotcha) +
   nightly + cargo-fuzz; compile gate on push touching parse_descriptor.rs/
   the fuzz dir, smoke on cron + dispatch.

## Open questions for mini-R0 (answer EMPIRICALLY — try the mount)

1. **Does the cfg(fuzzing) lib.rs mount COMPILE?** Try it: add the mount +
   the check-cfg lint, write a trivial fuzz target, `cargo +nightly fuzz
   build --target x86_64-unknown-linux-gnu`. Does the 35-module closure
   compile under cfg(fuzzing) in the LIB crate? Modules may have
   bin-only assumptions, `#[cfg(test)]` items, or `crate::`-root items
   (the clap CLI structs) that don't exist in the lib crate root → report
   what breaks + the minimal fix.
2. **Exact closure:** what is the precise set of modules parse_descriptor
   needs (the `crate::` references, transitively)? Is it really ~35, or can
   a narrower mount work (e.g. parse_descriptor only needs error + template
   + synthesize + a few, not cmd/ or wallet_export)?
3. **Normal-build verification:** confirm `cargo build`/`cargo test
   -p mnemonic-toolkit`/`cargo clippy --all-targets -D warnings` are
   UNAFFECTED with cfg(fuzzing) off (only the lint line present). The
   `--locked` CI guard must still pass (does adding the lint line or the
   fuzz/ dir change the root Cargo.lock? fuzz/ has its own lock).
4. **Patch replication:** does the fuzz workspace need the miniscript-git
   patch, and does it resolve? (The closure uses SortedMultiA.)
5. **Is there a LESS-invasive path** than the 35-module mount — e.g. a
   thin `#[cfg(fuzzing)] pub fn fuzz_parse_descriptor(s: &str)` wrapper in
   ONE existing lib-exposed module that calls into the bin graph? (Probably
   not, since the bin graph isn't in the lib crate — but check.)
6. **Scope:** test/infra-only NO-BUMP? The lint line + cfg(fuzzing) mount
   are library-source changes (cfg-gated dead code in normal builds) — is
   that NO-BUMP (no behavior/API change in any normal build) or does the
   lib-source touch warrant a patch bump? (Lean NO-BUMP: zero normal-build
   effect, like the codec fuzz dirs.)
7. **Local feasibility:** can the implementer verify locally (nightly +
   cargo-fuzz are available, used in Cycles C)? Confirm.

If the mount is too invasive / doesn't compile cleanly, RECOMMEND deferring
(document the blocker) rather than forcing a risky refactor — this is the
lowest-value, highest-complexity remaining item.
