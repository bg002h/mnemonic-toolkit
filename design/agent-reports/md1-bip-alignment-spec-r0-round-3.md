# R0 review — `SPEC_md1_bip_alignment_and_code_honesty.md` (round 3, post-F-A1b-drop re-gate)

**Reviewer:** Fable, adversarial, read-only. Persisted verbatim per CLAUDE.md. Ground truth: `descriptor-mnemonic` @ `origin/main ef1f3e71` (re-confirmed unchanged), toolkit working tree (clean `crates/`, binary v0.84.0), empirical CLI runs.
**Dispatched:** 2026-07-10 (MD SPEC, R0 round 3).

## 1. F-A1b removal — CLEAN, no stale residue

Swept the spec for every F-A1b/gate/refusal token. §F-A1b (lines 31-33) is a pure deferral note: no encode gate, no `encode_md1_string`/`split` gating residue, decoder `MissingExplicitOrigin` reject explicitly unchanged. Acceptance #1 excludes it from the gate and tracks it OPEN; acceptance #5 files `pathless-wallet-backup-partial-decode`; P2a (line 129) and Phase 1 (line 128) are consistent; F-A4's orthogonality note (line 53) is correct. No instruction anywhere would cause an implementer to add the refusal.

## 2. B-C2 softening — honest, BIP stays self-consistent

The canonical-default table in B-C2 (line 74) matches `canonical_origin.rs` @ ef1f3e71 exactly (pkh→44', wpkh→84', tr-keyonly→86', wsh(multi/sortedmulti)→48'/2', sh(wsh(multi/sortedmulti))→48'/1', everything else None), plus the in-cycle sh(wpkh)→49' arm. The non-canonical case is marked "under separate design" with the current decoder's reject stated truthfully; no encoder-MUST, no MUST-reject locked in. B-I3's `MissingExplicitOrigin` enumeration is consistent. (One pre-commitment nit → M-B.)

## 3. M-6 fold — named sites verified, but list INCOMPLETE and the central safety claim is FALSE

The six named sites all check out at source (`synthesize.rs:349-368`, `:50-52`, `:1092-1112` [region extends ~:1127], `restore.rs:303` vs `:317-320`, `cli_bundle_md1_template_form.rs:239`, `gui_schema.rs:1317-1320` + missing classify cell). The bip49 refusal stays pinned through `cli_template_from_tree`/`template_admissible`; the round-2 no-flip story holds for the synthesize/mutation-3 path and the `wsh(or_d)` fixtures.

**But "No runtime flip (verified)" is empirically FALSE.** The list misses the **descriptor-mode canonicity probes** — `bundle.rs:1416-1418` and `verify_bundle.rs:1408-1414` — where `is_non_canonical = canonical_origin(tree).is_none()` feeds real gates + path binding. Demonstrated live against v0.84.0:

- `mnemonic bundle --descriptor "sh(wpkh(@0/<0;1>/*))" --slot @0.phrase=…` → **succeeds today** non-canonical: `info: non-canonical descriptor; defaulting origin path for @0 to m/48'/0'/0'/1'`. Post-re-pin `is_non_canonical` flips false → `bind_descriptor_mode_paths` early-returns (bundle.rs:2262-2266), canonical 49' supplies the path → **same invocation silently emits a DIFFERENT WALLET** (notice disappears). Same-command→different-wallet — the highest-severity pattern.
- `--account 5` variant → succeeds today (`m/48'/0'/5'/1'`); post-re-pin **refused** by §4.12.g canonicity guard (bundle.rs:1421-1427).
- `--slot @0.path=m/49'/0'/0'` override (the exact workaround today's notice recommends) → succeeds today; post-re-pin **refused** by §6.6 row-4 canonical-mode rejection (bundle.rs:1432ff; exemption non-canonical-only, `slot_input.rs:293-300`).
- `verify-bundle --descriptor`: a pre-re-pin elided-sh(wpkh) bundle re-verified post-re-pin byte-mismatches → previously-passing verify **fails** (fail-loud, not false-pass — but a verify regression on existing artifacts).
- **Zero test coverage:** no sh(wpkh) cell in `tests/cli_non_canonical_descriptor.rs` or any descriptor-mode test — the re-pin suite will NOT catch this.

### IMPORTANT

- **I-1. Ripple bullet asserts "No runtime flip (verified)" while F-A1 flips four descriptor-mode behaviors at re-pin, one wallet-changing, zero test coverage.** Not Critical: nothing in this repo's diff is unsafe; both old and new wallets stay seed-recoverable (old cards carry explicit origins, still decode); verify fails loud; new 49' semantics is arguably the correct target (BIP-48-cosigner-leaf-1' for nested single-sig was itself dubious). But the spec would dispatch the re-pin leg under a false safety assertion. **Fold:** (a) scope the "No runtime flip" claim to the template/routing surfaces it was verified on; (b) add `bundle.rs:1416-1418` (+`:1421` §4.12.g, `:1432` row-4, `:2262-2266` early-return) and `verify_bundle.rs:1408-1414` to the site list; (c) record a decision — either accept the flip as the intended post-F-A1 semantics (then pin with new toolkit tests for elided-sh(wpkh)-descriptor→canonical/49'/no-notice, `--account≠0`→refuse, `[Phrase,Path]`→refuse; CHANGELOG/manual note; migration note that old elided-mode bundles verify via inline `[fp/48'/0'/0'/1']@0` origins since the `--slot @N.path=` route now refuses) — or explicitly defer the re-pin with the flip documented. Either acceptable; silence is not.

### MINOR

- **M-A.** `crates/mnemonic-toolkit/src/error.rs:343-349` — `TemplateFormUnsupportedShape` doc defines the refusal class as "`canonical_origin(&tree)` is `None` … e.g. bip49 nested segwit"; post-F-A1 bip49 is `Some` yet still refused → doc turns actively wrong. Add to the M-6 site list.
- **M-B.** B-C2's BIP insert embeds the brainstorm's candidate mechanism ("loud encode advisory + decoder partial-decode-with-placeholders") in a public spec before that design converged. Recommend the BIP carry only "under separate design; current reference decoder rejects `MissingExplicitOrigin`", keeping the candidate mechanism in the FOLLOWUP/brainstorm doc.
- **M-C.** Spec header "R0 history" (line 5) cites round 1 only; add the round-2 report path.

## 4. M-7 fold — CLEAN

Line 91 states the four emitted files, `path` surfaces via `.descriptor.json`, and the BIP §Test Vectors table pins template+path pairs for NUMS/tr_with_leaf — exactly the round-2 ask.

## 5. Recovery-safety re-check after the F-A1b drop

Dropping F-A1b is a literal encode-side status quo: no card that decoded before stops decoding; nothing worse than ef1f3e71. Retained fixes unaffected + individually sound: F-A1 strictly additive (decode-gate no-op on `Some`; encoder writes `path_decl` verbatim), F-A2 dispatch-bit additive ({4,8,12} even), F-A8 rejects only malformed non-zero pads, F-A3 refusal exit≠0, F-A5/F-A9 cosmetic, F-A4 stderr-only. The one regression vector (I-1) is a consequence of F-A1 at the toolkit re-pin — not of the F-A1b removal — fully covered by the fold above.

## Verdict

The F-A1b excision is clean and drift-free, B-C2 is honest, M-7 is folded, and recovery-safety within this repo holds. The gate item is the ripple bullet's false "No runtime flip (verified)" claim over an incomplete M-6 site list — the descriptor-mode canonicity probes flip four behaviors at re-pin, one wallet-changing, none tested.

**VERDICT: OPEN (0C / 1I / 3M)** — fold I-1 (plus the Minors), then re-dispatch for convergence.
