# R0 Review (Round 4, CONFIRMING) — `BRAINSTORM_reproducible_builds_musl.md`

**Verdict: Critical 0 | Important 0 | Minor 4 | GREEN: TRUE (explicit 0C / 0I) — CONVERGED.**

The R0 loop capped at round-3 with 2 Important + 4 Minor; round-3 folded the 2 Importants but never re-reviewed them. This round-4 review verifies the two folds are correctly+completely resolved and re-scans the whole spec. Every load-bearing external fact was empirically re-verified (not trusted from the draft).

## Round-3 fold verification

**r3-I1 (P4 CI self-rebuild gate must EXERCISE the remap) — RESOLVED, sound.** The spec now requires P4 to rebuild at a path DISTINCT from the release path `/build/src`, concretely running the §5 two-path build (`/build-a/src` + `/build-b/src`, both remapped to `/build`) AND asserting byte-equality to the published `.tar.gz`, with an explicit negative test (a deliberate remap-off run must RED the gate). A same-path same-container rebuild-and-compare is explicitly PROHIBITED as the gate form (§3.14, §6 P4, F5). Matches the recon's proven mechanic (the remap is only load-bearing when the two real `from`-paths differ — recon L25/L34/L44). Threaded coherently through §5 step 1, §5 acceptance note (viii), §6 P1/P2/P4, §7, §8.7. Keeps the remap under continuous test release-over-release.

**r3-I2 (vendoring fork resolved in body) — RESOLVED, internally consistent.** Committed `vendor/` + a committed `.cargo/config.toml [source."git+…rust-miniscript?rev=95fdd1c5…"]` replacement is named the CANONICAL published-artifact path (§3.12, §4, F4, §8, §10); CI-time `cargo vendor` downgraded to a non-canonical fallback with its provenance claim corrected to "compile is offline; vendor still fetches once at vendor-time." The `[source]` stanza is admitted as the ONE committed-config exception to F4's verbatim-safe rule, justified because `[source]` keys are verbatim source-URL→path mappings with no `$PWD`/shell expansion — consistent with F4's C2-driven RUSTFLAGS-via-ENV decision. Empirically confirmed `cargo vendor` on this exact tree emits precisely the named `[source."git+https://github.com/rust-bitcoin/rust-miniscript?rev=95fdd1c5773bd918c574d2225787973f63e16a66"]` stanza, verbatim.

## Whole-spec re-scan — no new Critical/Important

Core recipe (top-level `--remap-path-prefix` via `CARGO_BUILD_RUSTFLAGS`, `--locked`, full env-pin set) empirically reconfirmed on 1.85.0 (`-C` form errors `unknown codegen option`; `--remap-path-prefix` compiles; not in `-C help`). secp256k1-sys §5 gate sufficient: epoch-load-bearing probe (build `.o` with epoch unset → confirm it DIFFERS) + `__DATE__`/`__TIME__`/host-path residue grep + aarch64 direct-residue primary gate + Cross.toml `[build.env]` passthrough. SOURCE_DATE_EPOCH keyed off COMMIT SHA via `git show -s --format=%ct` (robust to retag). gzip-`-n` pin verified byte-identical with zero mtime field; both live `tar` sites (`:50`, `:133`) + hash site (`:135`) confirmed at cited line numbers. Container-by-digest, F6 coupling, md-FOLLOWUP plan (add remap + flip RESOLVED in the shipping commit, no status re-edit — slug confirmed PARTIAL/remap-omitted/cross-citing toolkit @ `a759c79`), NO-BUMP, and trusting-trust posture (committed vendor removes the live-host root; container-by-digest removes the apt root; multi-builder attestation correctly scoped as future hardening) all sound.

## Minor findings (non-blocking; fold at plan-authoring time)

- **m1** — `[source]` stanza shown incomplete: an offline `cargo vendor` build also REQUIRES `[source.crates-io] replace-with = "vendored-sources"` + `[source.vendored-sources] directory = "vendor"`. State the committed config carries the full three-stanza output (all verbatim/expansion-free → F4 exception still covers them).
- **m2** — the miniscript fork is a `[patch.crates-io]` entry (`Cargo.toml:28-29`), not a plain `git=` dep. `cargo vendor` handles it identically (same `[source]` key) → resolution unaffected; correct the one-word characterization.
- **m3** — `mnemonic-man.tar.gz` (`:50`) is a published release asset but is NOT hashed into any `SHA256SUMS` (only the per-arch binary tarballs `:133`→`:135` are). Listing it for the gzip-`-n` pin is fine for determinism hygiene; mark it hygiene-only, not provenance-hashed.
- **m4** — gzip OS-byte under-specified (observed `03`/Unix; a different gzip build could differ). Add the OS byte to the residue assertion or state it's pinned by the same container-digest constraint already invoked for the compressor.

## Convergence verdict

CONVERGED to GREEN (0 Critical / 0 Important). Both round-3 folds correctly, completely, and self-consistently resolved; every load-bearing external fact empirically reconfirmed. The 4 Minors strengthen the plan-doc but block nothing — fold at plan-authoring time, after which implementation proceeds under the per-phase R0 + TDD + whole-diff-review pipeline.
