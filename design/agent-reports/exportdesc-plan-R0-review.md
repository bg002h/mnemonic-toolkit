# export-wallet --format descriptor — Plan R0 Review (round 0)
**Verdict:** GREEN (0C/0I)

Plan faithfully + implementably realizes the R0-GREEN SPEC; all 5 dispatch sites, emitter, partition test, v0.42.0 release set, and paired GUI v0.23.0 lockstep confirmed against live source at `a3aaeba`. No Critical/Important. 5 Minors (implementer-hint quality); m1/m2/m4 folded into the plan.

## SPEC→plan coverage — COMPLETE
§1→P1; §2 emitter (3 trait methods incl extension, no trailing \n) → 1.2 (vs mod.rs:395-399 + green.rs:25-50); §3 enum + 5 dispatch + Descriptor=>false → 1.1/1.2 (all 5 sites exhaustive, no `_`); §4 decisions → 1.2/1.3; §5 lockstep → 2.1 + GUI cycle; §6 phasing; §7 tests 1-7 → 1.1/1.3/partition/GUI; §8 MINOR → 2.2; §9 recipe → 2.1; §10 re-grep. No unmapped item, no decision divergence.

## Minor (5; m1/m2/m4 folded, m3/m5 noted)
- **m1 (folded):** test must not shell `convert` for the xpub — it label-prefixes `xpub: …` (convert.rs:1100). Use a hardcoded known account-xpub literal (account-depth correct). → plan Task 1.1 Step 1 now uses the bip84 literal.
- **m2 (folded):** round-trip must reuse the existing P11A helper `run_export_from_import_envelope` (cli_export_wallet_from_import_json.rs:491) + fixtures (core-bip84-mainnet.json / sparrow-multisig-2of3...), NOT `bundle --descriptor` (wrong producer). → plan Task 1.3 updated.
- **m4 (folded):** GUI `src/schema/mnemonic.rs:1` module-doc header v0.41.0→v0.42.0 added to the GUI version-literal sweep.
- **m3 (noted):** the dispatch brief's R0-review path had a typo; actual files exportdesc-spec-R0/R1-review.md confirmed GREEN. No plan change.
- **m5 (noted):** enum variant appended at `:43` (lowest line-shift); downstream cites shift +1 — covered by the plan's re-grep mandate.

## TDD ordering — coherent
1.1 writes failing smoke; 1.1 Step 2 fails at clap; Step 3 adds enum + format_requires_template arm (still fails — 4 dispatch arms non-exhaustive E0004); 1.2 wires emitter + arms → green; commit 1.1+1.2 together (no broken intermediate). partition_is_exact (logic test, literal arrays :840/:843) needs the Descriptor entry — captured.

## round-trip feasibility — confirmed
from_import_json re-emit ends at `canonical_descriptor = parsed_ms.to_string()` (:692) → DescriptorEmitter returns the same → canonical body == envelope's (modulo checksum). Single-sig + wsh-multisig fixtures exist. Taproot refused on the from-import-json leg (:672-682) for all formats → correctly routed via direct --descriptor passthrough only.

## GUI sequencing — correct
GUI mini-cycle bumps Cargo pin (:42) + pinned-upstream (:22) + pinned_version (:3452) + EXPORT_FORMATS (:61-72) + (m4) docstring (:1), AFTER the toolkit tag is pushed (git-dep resolves against remote). pin_coherence guard (Cargo tag == pinned-upstream tag) satisfied by bumping both. import-sniff (:1989-1998) correctly untouched. Matches the v0.22.0 cycle pattern.

## new-issues
make audit binary set correct (4 BINs + FIXTURES_DIR, Makefile:44-50). Emitter registration mod/re-export alphabetical (between coldcard + electrum). No #[allow(dead_code)] needed (wired into live dispatch same phase). No error.rs variant. README markers README.md:13 + crates/.../README.md:9; install.sh self-pin :32. No missed file.

## Verdict rationale
Every SPEC item + 5 match sites + 7 tests + release set + GUI lockstep implementable against live source; TDD ordering coherent (no broken intermediate); round-trip feasible via the P11A helper; SPEC R0 blockers closed. 5 Minors are hints; m1/m2/m4 folded. GREEN (0C/0I) — implementation may proceed.
