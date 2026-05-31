# End-of-Cycle R0 — toolkit v0.37.10 mk1 origin-path

Opus architect, final gate before ff-merge + tag. Full branch diff `master...HEAD`
(`a255060`→). Persisted by controller. (RED 0C/1I → folded → GREEN.)

## Confirmations (file:line)
1. **Helper consistent-by-construction** (`synthesize.rs` `mk1_origin_path`): depth-0→empty;
   else `depth-1` intermediates + terminal `xpub.child_number` ⟹ `len==depth && last==child`
   for every class. Unit-tested incl. encode-no-mismatch + the 3→0 pad.
2. **All 8 `KeyCard::new` sites use the helper; md1 path_decl unchanged** (built from raw path).
   Reject loop removed cleanly, no dead binding.
3. **Overlap-prefix cross-check** keyed off `d`; `.zip()` stops at overlap; depth difference not
   flagged, only shared-prefix disagreement. Parent-fp `full[..d-1]` bounds-safe. No false-positive
   on 3→4/4→3/4→4 (empirical: 74→0).
4. **Import origin override** (`json_envelope.rs envelope_to_resolved_slots`) prefers
   `bundle.origin_path[s]` (full origin), per-cosigner `origin_paths` index, single-sig no-op.
   Necessary so re-imported origin matches the source descriptor.
5. **Rebuilt cross-check fixtures non-vacuous** (overlap-disagree-at-#1; seed-mixing parent-fp).
   `bsms_envelope_mk1_decodes_back_…` confirms the regenerated fixture's semantic identity.
6. **SemVer PATCH; no GUI/manual lockstep** (mk1 chunk bytes change, no clap/JSON-wire change).
   Error-mirror exit-code 2 + friendly arm + test correct.

## CRITICAL — None.
## IMPORTANT (RED → folded GREEN)
- **I1 — missing `CHANGELOG.md [0.37.10]` entry.** Every prior 0.37.x release has one; the file
  asserts "all notable changes documented." **FOLDED:** added the `[0.37.10] — 2026-05-30` entry
  (re-pin + helper + cross-check redesign + import override; PATCH, no lockstep).
## MINOR (folded)
- **M1 — planned 3→0 inspect-pin test absent.** **FOLDED:** added
  `bundle_watch_only_no_origin_xpub_inspect_shows_synthetic_path` pinning `origin_path: m/0/0/0'`.
- **M2 — stale `#[allow(dead_code)]` on `derivation_path_from_envelope`** (now live via the import
  override). **FOLDED:** attribute dropped + comment refreshed; clippy still clean.
- **M3 — plan-vs-impl divergences** (beneficial, verified): helper-fixtures kept depth-4 (helper
  dissolves the panic); cleaner seed-mixing cross-check fixtures. Informational, no action.

## VERDICT: GREEN (0C/0I after fold) — clear to ff-merge `master` + tag `mnemonic-toolkit-v0.37.10`.
Controller pre-gate: full suite 0 failures (was 74); clippy `--all-targets -D warnings` clean.
(Toolkit CI does not run `cargo fmt --check`; the whole-crate drift under newer rustfmt is a
pre-existing non-gate — the repo pins toolchain 1.85.0 and never fmt-gated.)
