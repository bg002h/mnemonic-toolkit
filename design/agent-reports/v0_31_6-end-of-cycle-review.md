# v0.31.6 end-of-cycle architect review

**Reviewer:** opus (feature-dev:code-reviewer)
**Round:** end-of-cycle
**Cycle:** Cycle 13 (seedqr-digits-from-input-unification)
**Date:** 2026-05-21
**Pre-tag SHA:** `922bd34` (Phase 3-4; Phase 5 uncommitted)

## Verdict

**GREEN.** All 10 verification items pass. 0 Critical / 0 Important / 1 Minor (cosmetic, addressed inline).

## Design pivot note

R0 I1 proposed "substitute Seedqr→Phrase before compute_outputs". This was DISCARDED in Phase 2 because `(Phrase, Phrase)` is an identity barrier in `classify_edge` — the canonical `--from seedqr= --to phrase` decode would have been wrongly refused. The shipped implementation wires `Seedqr` as a first-class input node through `is_supported_direct_edge` + `compute_outputs` (`Seedqr | Phrase | Entropy` arm decodes digits→phrase→entropy then projects). Reviewer confirmed the pivot is sound.

## Verifications

1. **`compute_outputs` Seedqr arm**: inner `match from` — Seedqr decodes digits→phrase→entropy; `(Seedqr, Phrase)` re-encodes entropy→phrase (the decoded phrase), not refused. `debug_assert_eq!(from, Entropy)` on the ElectrumPhrase target arm is unreachable for Seedqr (not a supported edge). No misfire.
2. **`is_supported_direct_edge`**: 9 `(Seedqr, *)` edges present incl. `(Seedqr, Phrase)`. `(Seedqr, {Mk1, Path, MiniKey, ElectrumPhrase})` correctly absent.
3. **classify_edge**: Seedqr falls through cleanly to the catch-all (not Bip38/Address/MiniKey/Xpub source, not ElectrumPhrase pair, not codec_set).
4. **seedqr `run_decode`**: 4 `(digits, from)` cases handled — both→unreachable (clap-guarded), neither→required-input (exit 1), digits→deprecation notice + argv advisory, from→node-check. Notice precedes advisory.
5. **secret_taxonomy parity**: `Seedqr` in `is_secret_bearing` + `SECRET_NODE_TYPES` + `declare_node_type_variants!` macro. Parity test exhaustive.
6. **`--to seedqr` rejection**: absent from `--to` PossibleValuesParser → clap rejects (raw exit 2 remapped to exit 64 by main.rs sysexits wrapper).
7. **Manual mirror**: convert + seedqr-decode sections updated; flag-coverage lint green.
8. **Cargo.toml / install.sh / CHANGELOG**: all 0.31.6.
9. **SemVer PATCH**: additive + deprecation-warning; correct.
10. **GUI lockstep**: `--from` on seedqr-decode is the sole net-new flag; CHANGELOG flags the paired GUI v0.16.2.

## Minor (addressed)

Inline comment at `convert.rs:857` said "exit 2"; clarified to note the sysexits wrapper remaps to exit 64. Fixed pre-tag.

## Cleared for tag.
