# IMPLEMENTATION PLAN — cycle-13 Lane A: Coldcard/Jade multisig fidelity PAIR (H11 export + H14 import)

**Status:** DESIGN ONLY — this plan-doc feeds its OWN mandatory R0 loop (plan → plan R0 → fold → re-dispatch until GREEN) BEFORE any code is written. No implementation has begun.
**Spec (R0-GREEN, round 2):** `design/BRAINSTORM_cycle13a_coldcard_multisig_fidelity.md`. Spec R0 reviews: `design/agent-reports/cycle13a-spec-r0-round{1,2}-review.md` (round 2 = GREEN, 0 Critical / 0 Important).
**Source SHA pinned:** `origin/master = 9b2a8ae341e0bd7fe2a75ad8d669830d96b93ccb` (toolkit **v0.65.2** — `crates/mnemonic-toolkit/Cargo.toml:3`). EVERY citation below was re-grepped LIVE against this SHA this session (citations decay every merge; the working tree is on the v0.60.0 own-account branch — its line numbers are NOT trusted). The two round-2 advisory nits are FOLDED: `computed_fp` binds at `:359-360` (the comment is `:358`); `cs.fingerprint` is READ at `:366` (emitted `:367`).
**Plan-R0 fold history:** R1 review (`design/agent-reports/cycle13a-coldcard-multisig-plan-R1-review.md`, NOT GREEN — 0C/1I/3M) FOLDED: **I-1** — P1's fixture-reconciliation was incomplete (named 2 of ≥4 breaking inline fixtures). Decoded the masked xpub consts against `9b2a8ae3`: **`XPUB_A`/`XPUB_B` depth 4, `XPUB_C` depth 3 — ALL depth>0**, so NO depth-0 multisig xpub constant exists in the test module. Folded: §P1 H14-h now enumerates the COMPLETE break set (`:990`, `:1042`, `:1265`, `:1418`) with each fixture's correct post-change outcome (re-point vs split-to-refuse vs supply-XFP), and MANDATES adding depth-0 `XPUB_D0_*`/`FP_D0_*` constants. **M-1** (canonicalizer test blobs must carry per-line `<XFP_master>:` so P1 re-parse does not refuse → #15/#16 RED stays attributable to the canonicalizer) + **M-2** (broader canonicalizer regression cluster `:1410/:1428/:1445/:1485`) folded into P4. **M-3** (#13b XFP=computed to avoid an incidental depth-0 warning) folded into P2. **R2 review (`…-plan-R2-review.md`, NOT GREEN — 0C/1I/0M) FOLDED: I-1 (cont'd)** — the break set was incomplete across the FILE boundary; two CLI integration tests in `tests/cli_import_wallet_coldcard_multisig.rs` (`:166` `coldcard_ms_xfp_header_divergence_warns_byte_exact_template`, `:209` `coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner`) are end-to-end twins of unit `:990`/`:1265`, flip warn→silent under H14-c, and were nowhere in the plan → would surface as a surprise RED at the full-suite gate. Folded: §P1 H14-h now adds these 2 CLI tests (items 5+6) with the depth-0 re-point outcome + a "CLI surface verified CLEAN" enumeration; §6 checklist references the CLI layer. Re-dispatch the plan-R0 after this fold (the reviewer-loop continues after EVERY fold).

---

## 0. SemVer / version / co-ship / lockstep (READ FIRST)

- **DO NOT bump the toolkit version in this lane's commits.** Lane A co-ships with **Lane B (L8/L9)** and **Lane C (M1/M7/L18)** as ONE toolkit **MINOR v0.66.0**. The orchestrator integrates all three file-disjoint lanes and does the SINGLE version bump. **The implementer leaves `Cargo.toml` / `CHANGELOG` / both `README`s / `install.sh` / `fuzz/Cargo.lock` UNTOUCHED.** (Toolkit release-ritual version-sites are not gate-enforced — see MEMORY `project_toolkit_release_ritual_version_sites`; that ritual is the orchestrator's job at integration, not this lane's.)
- **SemVer of the integrated ship:** toolkit MINOR v0.66.0 — H11 changes the export wire-shape for the previously-malformed divergent case; H14 turns a previously-ACCEPTED depth>0/no-XFP intake into a REFUSAL. Both are observable behavior changes; neither adds/removes/renames a clap flag.
- **NO `schema_mirror` impact** — no clap flag/option/subcommand/dropdown-value added/removed/renamed (the gate is flag-NAME parity per CLAUDE.md). No `mnemonic-gui` schema PR required.
- **NO `--json` wire-shape change** — H11 is a text-format emitter; H14 is import-side fp resolution. No GUI `--json`-consumer paired-PR concern.
- **md-codec / mk-codec / ms-codec / md-cli / ms-cli / mk-cli:** NO-BUMP. No cross-repo `FOLLOWUPS.md` companion mirror (toolkit-only fidelity fix).
- **Manual prose (voluntary, same-PR, NOT gate-enforced):** `docs/manual/src/45-foreign-formats.md` + `docs/manual/src/30-workflows/37-wallet-export.md` (both EXIST at origin/master — verified). The manual-mirror lint is flag-name based → it will NOT fire on these prose edits. Ship them in-PR as hygiene; not a blocker.
- **SPEC update (in-repo, same-PR, IN SCOPE):** `design/SPEC_wallet_import_v0_28_0.md` §11.4.1 (table `:419-427`, formula `:429`) — depth-gated truth-table correction.
- **NEVER `cargo fmt` the toolkit** (`mlock.rs` permanently fmt-exempt — MEMORY `project_g6_fmt_exemption_and_asymmetric_pin`). None of the four target source files is `mlock.rs`, but a blanket `cargo fmt` would touch it — DO NOT run it.

---

## 1. Citation table — re-grepped LIVE against `9b2a8ae3` (with the two nit fixes)

All paths under `crates/mnemonic-toolkit/src/` unless noted. Verified this session via `git show origin/master:<path> | grep -n`.

### H11 export — `wallet_export/coldcard.rs`
| What | Line(s) | Verified content |
|---|---|---|
| `emit_coldcard_multisig_text` fn start | `:258` | `pub(crate) fn emit_coldcard_multisig_text(inputs: &EmitInputs) -> Result<String, ToolkitError>` |
| `inputs.template` required (`BadInput` if absent) | `:261` | `let template = inputs.template.ok_or_else(...)` |
| `format_str` match (P2WSH/P2SH-P2WSH) | `:281` | `WshMulti\|WshSortedMulti => "P2WSH"`, `ShWshMulti\|ShWshSortedMulti => "P2SH-P2WSH"` |
| `derivations` built in SLOT order | `:324-328` | `.iter().map(\|s\| normalize_path(&s.origin_path_bare()))` over `inputs.resolved_slots` |
| **collapse to `m/0'/0'`** (the bug) | `:330-336` | `if !empty && windows(2).all(w[0]==w[1]) && !derivations[0].is_empty() { derivations[0].clone() } else { "m/0'/0'".to_string() }` |
| sortedmulti lex-sort by xpub | `:345` | `cosigners.sort_by(\|a,b\| a.xpub.to_string().cmp(&b.xpub.to_string()))` (fires only for `WshSortedMulti`/`ShWshSortedMulti`) |
| single shared `Derivation:` push | `:361` | `lines.push(format!("Derivation: {derivation}"))` |
| cosigner emit loop (xpub-sorted) | `:363` | `for cs in cosigners {` |
| **`cs.fingerprint` READ** (master fp, true per-slot) | `:366` | `let xfp = cs.fingerprint.to_string().to_uppercase();` |
| `<XFP>: <xpub>` emit | `:367` | `lines.push(format!("{xfp}: {}", cs.xpub))` |
| Jade delegation (byte-identical) | `wallet_export/jade.rs:46` | `=> emit_coldcard_multisig_text(inputs)` for all 4 wsh/sh-wsh multisig templates |

### H14 import — `wallet_import/coldcard_multisig.rs`
| What | Line(s) | Verified content |
|---|---|---|
| `parse_text` fn start | `:168` | `pub(super) fn parse_text(...)` |
| `shared_derivation` declared | `:197` | `let mut shared_derivation: Option<String> = None;` |
| `pending_per_cosigner_path` declared | `:205` | `let mut pending_per_cosigner_path: Option<String> = None;` |
| `Derivation` arm (stages shared + pending) | `:233-244` | shared set `:236-237`; `pending_per_cosigner_path = Some(value)` `:243` |
| **`<XFP>:` cosigner arm** (Q1 target) | `:245-256` | builds `RawCosigner{ per_line_path: None }` `:249-253`; **clears** `pending_per_cosigner_path = None` `:256` |
| bare-xpub arm (consumes pending) | `:268-274` | `let path = pending_per_cosigner_path.take()` `:269`; `per_line_path: path` `:273` |
| effective-path resolution (per-line OR shared fallback) | `:338-341` | `raw.per_line_path.as_deref().or(shared_derivation.as_deref())` |
| `path_components_str` | `:355` | `derivation_path_components(path_str)` (path/depth known here) |
| `xpub_parse_result` (`Result<Xpub>`) | `:358` | `let xpub_parse_result = Xpub::from_str(&raw.xpub_str);` |
| **`computed_fp` binding** (nit: `:359-360`) | `:359-360` | `let computed_fp: Option<Fingerprint> = xpub_parse_result.as_ref().ok().map(\|x\| x.fingerprint());` |
| `supplied_fp` (per-line XFP OR header) | `:361` | `raw.per_line_xfp.or(header_xfp)` |
| **5-row truth table** | `:363-399` | `match (supplied_fp, computed_fp)`; Row 2 mismatch warns `:368-380`; Row 4 `(None, Some(computed)) => computed` `:386` (the silent-substitute bug) |
| `effective_fp` stamped into `[fp/path]` | `:415` | `let path_raw = format!("[{}{}]", effective_fp, path_components_str);` |
| **masked fixture consts** (nit: `:945-947`) | `:945-947` | `FP_A="34A3A4F1"` / `FP_B="FF9DFBCF"` / `FP_C="B7F7DFEA"` = `xpub.fingerprint()` of XPUB_A/B/C |
| first masked fixture | `:954` | `fn parse_shared_derivation_no_xfp_header_silent()` (asserts `stderr.is_empty()` `:980`) |
| `.depth` guard | (NONE) | grep-confirmed: zero `.depth` reads in the file — only doc-comment mentions |

### I-1 round-trip canonicalizer — `wallet_import/roundtrip.rs`
| What | Line(s) | Verified content |
|---|---|---|
| `canonicalize_coldcard_multisig` fn | `:361` | `pub(crate) fn canonicalize_coldcard_multisig(blob: &[u8]) -> Result<String, ToolkitError>` |
| re-parse via `parse_text` | `:368` | `let parsed = parse_text(blob, &mut sink)?;` |
| cosigner lines lex-sorted | `:393` | `cosigner_lines.sort();` |
| **"ASSUMES homogeneous derivation"** comment | `:395-398` | `// canonicalization ASSUMES homogeneous derivation ...` |
| **single `Derivation:` from `cosigners[0].path`** | `:399-402` | `format!("m{}", path_components_for_canonical(&parsed.cosigners[0].path))` |
| `Derivation:` emit | `:413` | `out.push_str(&format!("Derivation: {}\n", derivation_str));` |
| Jade canonicalize delegates here | `:570` | `let inner_canonical = canonicalize_coldcard_multisig(multisig_file.as_bytes())?;` |
| baseline idempotence test | `:1397` | `fn canonicalize_coldcard_multisig_idempotent()` (homogeneous; must stay GREEN) |

### Callers / blast-radius
| What | Line(s) | Verified content |
|---|---|---|
| `--round-trip-verify` dispatch (coldcard-multisig arm) | `cmd/import_wallet.rs:1447` | `Some(canonicalize_coldcard_multisig(blob).map_err(\|e\| e.to_string()))` |
| Jade import delegates to shared parser | `wallet_import/jade.rs:133` | `let mut parsed = parse_coldcard_multisig_text(multisig_file.as_bytes(), stderr)?;` |
| Jade delegation doc | `wallet_import/mod.rs:121-122` | "delegates to `coldcard_multisig::parse_text` and re-annotates the inner" |
| H10 unsorted-export refusal (sorted-only reachability) | `cmd/export_wallet.rs:126`,`:134` | guard matches `WshMulti\|ShWshMulti` `:126`; `return Err(ExportWalletUnsortedMultisigUnsupported{..})` `:134` |
| json_envelope NOTICE-substitute (parity comparator) | `wallet_import/json_envelope.rs:388-398` | `origin_fingerprint` absent → `card.xpub.fingerprint()` + stderr NOTICE `:394` |

### Exit codes — `error.rs`
| Variant | Line | Exit |
|---|---|---|
| `BadInput(String)` | `:11`, exit map `:549` | **1** (H11-d export refusal, H11/H14 export-side) |
| `ImportWalletParse(String)` | `:227`, exit map `:582` | **2** (H14-b import refusal) |
| `ExportWalletUnsortedMultisigUnsupported` | `:177`, exit map `:576` | **2** (pre-existing H10 — NOT modified this lane; context only) |

### SPEC + synthesize + vendored bitcoin
| What | Line(s) | Verified content |
|---|---|---|
| SPEC §11.4.1 truth table | `SPEC_wallet_import_v0_28_0.md:419-427` | 5-row table (header present / computed available / matches) |
| SPEC **buggy** computed-fp formula | `:429` | "`Xpub::fingerprint()` on the master xpub (if depth=0) or on the cosigner xpub itself (if depth>0 …)" — **MUST be replaced** |
| `synthesize_multisig_full` (depth-4 leaf + depth-0 master fp, M-2) | `synthesize.rs:597`,`:639`,`:644`,`:650` | `master_fingerprint = master.fingerprint(&secp)` `:639`; `derive_xpub_at_path(...)` (leaf) `:644`; `fp_bytes = master_fingerprint.to_bytes()` `:650` |
| `ResolvedSlot.fingerprint` field (= master fp) | `synthesize.rs:898` | `pub fingerprint: Fingerprint,` |
| vendored `bitcoin 0.32.8` `Xpub::depth` | `bitcoin-0.32.8/src/bip32.rs:111` | `pub depth: u8,` ("master = 0") — the H14 discriminator |
| vendored `Xpub::fingerprint()` | `bip32.rs:840` | HASH160 of `self.public_key` (CURRENT key, not master) — stale doc-comment at `:832` says "chaincode"; cite the BODY |

**Drift noted vs spec body (non-blocking, citations corrected here):** spec §2.1 says H10 guard region `:124-135` → live guard match is `:126`, refusal construct `:134`. Spec §2.5 says canonicalizer comment `:395-401` / `cosigners[0].path :401` → live comment `:395-398`, `cosigners[0].path` read `:401`. Spec §2.1 M-2 says `synthesize_multisig_full :594+` → live `:597`. The cited *content* is unchanged; line anchors above are the live ones.

---

## 2. Execution model

- **Single implementer subagent** (per CLAUDE.md: NOT parallel re-implementations) executes this GREEN plan in a **git worktree off `origin/master = 9b2a8ae3`**. Lanes B and C run in their own file-disjoint worktrees concurrently; the orchestrator integrates.
- **TDD, RED-first.** Each phase: write the phase's RED tests FIRST, confirm they FAIL for the right reason, then implement until GREEN. Tests live in the **BIN target** → run `cargo test -p mnemonic-toolkit` (NOT `--lib`; the toolkit's tests are in the binary crate).
- **Per-phase gate (BOTH must pass before advancing):**
  1. `cargo test -p mnemonic-toolkit` — the **FULL package suite**, never a targeted `--test`/single-test target (MEMORY `feedback_r0_review_run_full_package_suite`: CLI/flag/import phases ripple into argv/schema/version/lint tests outside any one phase's targets; a stale lint stayed RED for three phases because targeted runs masked it).
  2. `cargo clippy --workspace --all-targets -- -D warnings`.
- **NEVER `cargo fmt`** (see §0). If formatting drifts, hand-fix only the lines you touched.
- **No version-site edits** (see §0) — the implementer touches only the four source files + SPEC + fixtures + (voluntary) manual prose.
- **Refusal exit-code discipline:** every new refusal exits non-zero. H11-d empty-origin export refusal → `ToolkitError::BadInput` → **exit 1** (`error.rs:549`). H14-b depth>0/no-XFP import refusal → `ToolkitError::ImportWalletParse` → **exit 2** (`error.rs:582`). Assert the exit class in each refusal test.

---

## 3. Phased plan

Ordered to make every phase's RED→GREEN clean despite the H11↔H14↔canonicalizer round-trip coupling. The coupling: H11's divergent export emits `Derivation: <path>` + `<XFP_master>: <xpub>` per cosigner; that form round-trips only if (a) the parser arm consumes the per-line path (Q1) and (b) H14 silently accepts a supplied XFP at depth>0; and the round-trip-VERIFY surface only passes if the canonicalizer preserves per-cosigner paths (I-1). So we build the **intake/refuse semantics first** (P1), then the **parser arm that makes the per-line form round-trip** (P2), then the **export emit** that produces it (P3), then the **canonicalizer + verify** surface that must not re-collapse it (P4). Each phase's tests are independently RED-then-GREEN against the prior phases' GREEN state.

### P1 — H14 import refuse-vs-substitute matrix + SPEC §11.4.1 + fixture rewrite
**Files:** `wallet_import/coldcard_multisig.rs` (truth-table `:363-399`), `design/SPEC_wallet_import_v0_28_0.md` (§11.4.1 `:419-429`), fixtures (`:945-947` consts + dependents).

**Implementation (Decisions H14-a..h, §3.2 matrix):**
- Compute the depth discriminator from the ALREADY-available `xpub_parse_result` (`:358`): `let computed_depth: Option<u8> = xpub_parse_result.as_ref().ok().map(|x| x.depth);` (vendored `Xpub::depth` `bip32.rs:111`). **`xpub.depth == 0` is the discriminator** (Q3-ratified — the decoded public field, authoritative, independent of declared path). No need to count path components.
- Rewrite the `match (supplied_fp, computed_fp)` block (`:363-399`) into the depth-gated matrix:
  - `(None, Some)` **AND depth==0** → use computed silently (**H14-a**; current Row-4-at-depth-0 is correct).
  - `(None, Some)` **AND depth>0** → **REFUSE** `ToolkitError::ImportWalletParse` (exit 2) with the §3.2 message (cosigner index `i`, the depth `d`, "master fingerprint unrecoverable from an account xpub", "re-export with the device's XFP — a top-level `XFP:` header or a per-cosigner `<XFP>: <xpub>` line") (**H14-b**). Byte-exact message finalized in this phase (R0 to ratify if it tightens).
  - `(Some, _)` **AND depth>0** → accept supplied as authoritative; do NOT compute/compare; do NOT set `xfp_header_disagreed`; NO warning (**H14-c**). This silences the spurious Row-2 warning that fires on EVERY authentic depth>0 export.
  - `(Some, Some)` **AND depth==0** → Row 1/2 unchanged: match→silent, mismatch→`xfp_header_disagreed=true` + WARNING + use supplied (**H14-d**). The existing byte-exact Row-2 warning template (`:370-377`) is preserved for the depth-0 case.
  - `(Some, None)` (xpub malformed, depth unknown) → use supplied silently (**H14-e**, Row 3 unchanged).
  - `(None, None)` → hard error (Row 5 unchanged, `:388-398`).
- **SPEC §11.4.1 (H14-g):** replace the `:429` formula with the depth-aware rule — *"the master fingerprint is `Xpub::fingerprint()` IFF `xpub.depth == 0`; at depth>0 the master fp is conveyed ONLY by a supplied XFP (header or per-line `<XFP>:`) and is otherwise unrecoverable → REFUSE."* Add a depth gating column/preamble to the truth table; gate the `xfp_header_disagreed` warning on depth==0.
- **Fixture rewrite (H14-h) — COMPLETE BLAST-RADIUS (plan-R0 I-1, decoded against `9b2a8ae3`):** the masked consts (`:945-947`) pin `FP_A/B/C` to `xpub.fingerprint()`, and the three xpub consts are **`XPUB_A` depth 4, `XPUB_B` depth 4, `XPUB_C` depth 3 — ALL depth>0** (decoded from the base58 depth byte this session; confirmed via `git show origin/master:…coldcard_multisig.rs`). So under the new depth-gated matrix EVERY inline fixture feeding these consts at depth>0 changes behavior. **MANDATORY first step:** add **depth-0 multisig xpub fixture constants** (`XPUB_D0_A`/`XPUB_D0_B`/… — genuine `m`-root xpubs with depth byte 0, e.g. derived BIP-32 master xpubs) AND their computed `FP_D0_*` consts. **None exist in the test module today** (`XPUB_A/B/C` are depth 3-4), so the "use depth-0 xpubs so the warning stays meaningful" rewrites below are NOT executable without them — adding them is a prerequisite, not optional.
  - **The COMPLETE break set (reconcile each EXPLICITLY — do NOT rely on a catch-all token scan; #4 below cites no `FP_*`/`xfp_header_disagreed` token):**
    1. **`parse_xfp_header_mismatch_warns_uses_header` (`:990`)** — `XFP: DEADBEEF` header + bare `XPUB_A` (depth 4) → `(Some,Some)` depth>0 → H14-c **silent, no `xfp_header_disagreed`** under the new matrix; its asserts (`xfp_header_disagreed=true` `:1012`, warning present) would FAIL. **Correct post-change outcome: re-point to a depth-0 xpub (`XPUB_D0_*`)** so it stays a meaningful Row-2/H14-d warn case (DO NOT delete the asserts — H14-d keeps Row 2 at depth 0).
    2. **`parse_per_cosigner_xfp_divergence_warns` (`:1265`)** — `CAFEBABE: XPUB_A` (depth 4) → `(Some,Some)` depth>0 → H14-c silent; asserts `xfp_header_disagreed` `:1281` + warning `:1283-1284` would FAIL. **Re-point to a depth-0 xpub** to keep it a Row-2/H14-d warn case.
    3. **`parse_no_header_no_per_cosigner_xfp_uses_computed_silent` (`:1042`)** — bare `XPUB_A/B/C` (depth 4) under shared `Derivation:`, **NO XFP anywhere** → `(None,Some)` depth>0 → **H14-b: now REFUSES (exit 2)**. The fixture `.unwrap()`s and asserts computed `FP_A/B/C` `:1074-1076`. **Correct post-change outcome: SPLIT — (a) re-point to depth-0 xpubs (`XPUB_D0_*`) keeping the "row-4-at-depth-0 uses computed silently" assertion (= H14-a, test #9's territory), AND (b) add/convert a depth>0 variant asserting the H14-b REFUSAL (= test #6's territory).** This is the exact silent→refuse flip the lane is built on; the implementer MUST NOT leave it as a silent-parse assertion.
    4. **`parse_heterogeneous_coin_type_rejected` (`:1418`)** — bare `XPUB_A` (depth 4) + bare `tpub_a` (depth 4), **no XFP** → `(None,Some)` depth>0. The per-cosigner truth-table loop (`:336-431`) runs **BEFORE** the coin-type heterogeneity check (`:438-444`, confirmed live), so H14-b refuses with the **master-fingerprint** message before the coin-type validation is ever reached → its assertion `msg.contains("must share a coin-type")` `:1434` would FAIL. **Correct post-change outcome: supply per-line `<XFP>:` master fps on both cosigners** (so the loop passes, depth>0/with-XFP = H14-c) **so the coin-type check `:438-444` is still reached and its assertion stays valid.** (Do NOT re-point to depth-0 here — the test's purpose is the coin-type cross-check, which must remain exercised.)
  - **CLI-LAYER break set (plan-R0 round-2 I-1 — the break set is NOT confined to `coldcard_multisig.rs`; reconcile these EXPLICITLY too):** `tests/cli_import_wallet_coldcard_multisig.rs` has two end-to-end CLI twins of the broken unit fixtures that flip warn→silent under H14-c and would surface as a SURPRISE RED at the full-suite gate (the unit-layer enumeration above does NOT catch them):
    5. **`coldcard_ms_xfp_header_divergence_warns_byte_exact_template` (`:166`)** — feeds `XFP: DEADBEEF` header + a bare depth-4 `m/48'/0'/0'/2'` xpub via stdin; asserts the byte-exact Row-2 warning (4 `stderr.contains(...)` asserts). Under H14-c (depth>0 + supplied) the import goes **SILENT** → all four warning asserts FAIL. CLI twin of unit `:990`. **Correct post-change outcome: re-point the blob to a depth-0 `XPUB_D0_*` xpub** so it stays a meaningful Row-2/H14-d warn case (asserts KEPT, not deleted).
    6. **`coldcard_ms_per_cosigner_xfp_divergence_warns_per_cosigner` (`:209`)** — feeds `CAFEBABE: <depth-4-xpub>`; asserts the Row-2 warning (3 asserts). Goes **SILENT** under H14-c → asserts FAIL. CLI twin of unit `:1265`. **Re-point to a depth-0 `XPUB_D0_*` xpub** (asserts KEPT). (Mechanically identical to the `:990`/`:1265` re-point — the only gap was that the unit-layer pass never looked outside `coldcard_multisig.rs`.)
  - **CLI surface verified CLEAN (no other flips — do not edit):** jade CLI tests use Row-1 fixtures; `cli_import_wallet_snet_network_mismatch.rs::coldcard_multisig_mainnet_xpub_on_cointype1_rejects` supplies matching per-line XFPs (truth-table loop passes → network-mismatch assertion stays reachable; does NOT regress, unlike unit `:1418`); `cli_wallet_cross_format_convergence.rs` asserts only exit-0 + decoded key-material (no stderr-silence assert → the warn→silent shift is immaterial); `cli_import_wallet_format_mismatch_matrix.rs` + `cli_import_wallet_p0c_dispatch.rs` refuse before the truth table. **The COMPLETE cross-file break set = the 4 unit fixtures (`:990/:1042/:1265/:1418`) + these 2 CLI tests (`:166/:209`).**
  - **Fixtures that STAY GREEN (verify, do not edit):** every `{FP_A}: {XPUB_A}` per-line form supplies a per-line XFP that **equals** computed, so `(Some,Some)` depth>0 → H14-c silent → SAME observable outcome as today's Row-1; the on-disk `coldcard-ms-*` fixtures + `parse_testnet_path_sets_network_testnet` (`:1399`) likewise; the canonicalizer cluster (`:1397/:1410/:1428/:1445/:1485`) feeds homogeneous supplied==computed blobs. Enumerate them as the regression-guard set.
  - During the rewrite, **confirm the toolkit's OWN all-agree export round-trips SILENT** under H14-c (M-2: `synthesize.rs` emits depth-4 account xpubs under a depth-0 master fp → fires Row 2 TODAY → must go silent now).

**RED tests (write FIRST):**
- **#6 `import_coldcard_multisig_depth_gt0_no_xfp_refuses`** — depth-4 account xpub (`m/48'/0'/0'/2'`), shared `Derivation:`, no `XFP:` header, bare xpub → REFUSE exit 2, message mentions "depth"/"master fingerprint". RED today (silently substitutes).
- **#7 `import_coldcard_multisig_depth_gt0_with_header_xfp_no_warning`** — same depth-4 xpub + top-level `XFP: <synthetic-master>` (≠ `xpub.fingerprint()`) → resolved `[fp/path]` uses supplied, stderr SILENT, exit 0. RED today (Row 2 warns).
- **#8 `import_coldcard_multisig_depth_gt0_per_line_xfp_no_warning`** — per-line `<XFP_master>: <xpub>` at depth 4, `XFP_master ≠ xpub.fingerprint()` → uses supplied, SILENT, exit 0. RED today.
- **#9 `import_coldcard_multisig_depth0_no_xfp_uses_computed`** — genuine **depth-0 master xpub** (the new `XPUB_D0_*` const), no XFP → uses `xpub.fingerprint()` silently, exit 0 (H14-a). Guard against over-refusing. (Requires the depth-0 const mandated in H14-h.)
- **#10 `import_coldcard_multisig_depth0_xfp_mismatch_warns`** — **depth-0 xpub** (`XPUB_D0_*`), supplied XFP ≠ computed → WARNING + use supplied (Row 2 still meaningful at depth 0). Guard against under-warning. (Requires the depth-0 const.)
- **#11 fixture-rewrite regression** — the rewritten masked fixtures (the complete break set in H14-h: unit `:990`, `:1042`, `:1265`, `:1418` PLUS CLI-layer `cli_import_wallet_coldcard_multisig.rs:166`, `:209`) now reflect the depth-gated matrix (re-pointed to `XPUB_D0_*` for the warn/silent-computed cases; the depth>0/no-XFP case `:1042` split into a depth-0-silent assertion + a depth>0-REFUSE assertion; `:1418` carries per-line XFPs so the coin-type check stays reachable); confirm the toolkit's OWN all-agree shared-path export now round-trips SILENT (M-2).
- **#14 `import_jade_depth_gt0_no_xfp_refuses`** — a Jade `get_registered_multisig` reply whose inner `multisig_file` carries a depth>0 cosigner xpub with NO XFP → REFUSE exit 2 via the shared `coldcard_multisig::parse_text` delegated from `wallet_import/jade.rs:133` (M-1 blast-radius proof). RED today.

**Why P1 first:** the refuse/accept semantics are independently testable against a fixed blob; nothing downstream depends on export/canonicalizer changes. Establishes H14-c (silent depth>0 accept), which P3's round-trip test (#5) relies on.

### P2 — Q1 parser-arm change (per-line `Derivation:` consumed without clearing `shared_derivation`)
**File:** `wallet_import/coldcard_multisig.rs` (`<XFP>:` arm `:245-256`).

**Implementation (Q1 resolution-A, RATIFIED, with the two MANDATORY conditions):**
- Extend the `cosigner_xfp_key if is_xfp_hex(...)` arm (`:245-256`) to ALSO consume a pending per-line path: replace `per_line_path: None` (`:252`) with `per_line_path: pending_per_cosigner_path.take()`, and **DELETE** the defensive clear `pending_per_cosigner_path = None;` (`:256`) — `.take()` already empties it.
- **MANDATORY condition (i):** the arm MUST consume ONLY the per-line pending (`pending_per_cosigner_path.take()`); it MUST NOT clear or disturb `shared_derivation` (set `:236-237`, fallback-consumed for cosigners 2..N at `:341` `.or(shared_derivation.as_deref())`). Clearing the shared path would break the all-agree shape-1 parse (one shared `Derivation:` precedes N `<XFP>: <xpub>` lines).
- This makes the H11-emitted `Derivation: <path>\n<XFP_master>: <xpub>` per-cosigner form round-trip: the path lands in `per_line_path` (overrides shared at `:340`), and the per-line `<XFP_master>` is the supplied fp → H14-c (P1) silently accepts it at depth>0.

**RED test (write FIRST):**
- **#13 `import_coldcard_multisig_shared_derivation_3_cosigners_all_resolve_shared`** — one shared `Derivation: m/48'/0'/0'/2'` + 3 `<XFP>: <xpub>` lines (NO per-line `Derivation:`) → assert ALL 3 cosigners resolve to the SHARED path (proving cosigners 2..N still fall back to `shared_derivation` via `:341` after the arm change). MANDATORY-condition (ii). This is the regression guard that the arm change does NOT break shape-1; it should be GREEN both before AND after the arm change (it asserts an invariant the change must preserve) — verify it stays GREEN through P2.
- ALSO add a **NEW positive RED test (#13b)** in P2 for the interleaved per-line form (since #13 is a non-regression guard, not a RED-for-the-new-behavior): `import_coldcard_multisig_per_line_derivation_plus_xfp_roundtrips_path` — feed `Derivation: m/A\n<XFP_master>: <xpub_depth0>\nDerivation: m/B\n<XFP_master2>: <xpub2_depth0>` (depth-0 xpubs `XPUB_D0_*` so no refusal) → assert cosigner 0 resolves to `m/A`, cosigner 1 to `m/B` (distinct per-line paths preserved). RED today (the arm sets `per_line_path: None` + clears pending → both fall back to whatever shared/first-pending leaks). GREEN after the arm change. **(plan-R0 M-3) Choose the per-line `<XFP_master>` values to EQUAL the computed `xpub.fingerprint()` of the depth-0 xpubs (or have the test ignore stderr) — otherwise H14-d fires an incidental Row-2 warning at depth 0 that muddies the RED→GREEN.** The test asserts only path resolution, so either approach is valid; making the XFPs match computed is the cleaner choice.

**Why P2 before P3:** the export (P3) only round-trips if the parser already consumes the per-line form. Building P2 on P1's GREEN keeps the P3 round-trip test (#5) a clean RED→GREEN: it becomes GREEN exactly when P3's emit lands (P1+P2 already in place).

### P3 — H11 export per-cosigner `Derivation:` emit (sorted-slot pairing) + empty-origin refuse
**Files:** `wallet_export/coldcard.rs` (`:324-367`); `wallet_export/jade.rs` covered by delegation (no edits).

**Implementation (Decisions H11-a..e, mandatory pairing rule H11-b):**
- Replace the collapse (`:330-336`) + the single shared push (`:361`) + the cosigner loop (`:363-367`) with a branch on path homogeneity:
  - **All origins agree (H11-c):** keep the CURRENT shape — single shared `Derivation:` line + `<XFP>: <xpub>` cosigner lines. **Byte-identical to today** (no regression for the common case).
  - **Divergent (H11-a/b):** emit per cosigner `Derivation: <path>` then `<XFP_master>: <xpub>` (shape-2 interleaved). **NEVER emit `m/0'/0'`.**
- **MANDATORY pairing rule (H11-b, the I-2 funds-safety fix):** build a SINGLE sorted vector of slots — sort the `(origin_path, xpub, fingerprint)` tuples TOGETHER by the SAME sortedmulti xpub-lex key already used at `:345` (`a.xpub.to_string().cmp(&b.xpub.to_string())`). Then for EACH sorted slot read `cs.origin_path_bare()` (normalized via the existing `normalize_path`), `cs.fingerprint`, and `cs.xpub` from the **SAME slot**. **NEVER index a separate slot-order `derivations[i]` vector against the sorted cosigner loop** — that scrambles path↔xpub whenever sort-order ≠ slot-order (the §2.1 I-2 hazard, WORSE than `m/0'/0'`). Concretely: do NOT reuse the slot-order `derivations` vector (`:324-328`) inside the divergent branch; read the path off `cs` inside the already-sorted `for cs in cosigners` loop. (Because of cycle-2's H10 refusal — `export_wallet.rs:126`/`:134` refuses UNSORTED export to coldcard/jade — `emit_coldcard_multisig_text` is reachable for divergent paths ONLY via `WshSortedMulti`/`ShWshSortedMulti`, so the `cosigners.sort_by` at `:345` ALWAYS fires on the divergent path. This sorted-only case is the only divergent case that executes.)
- **Empty-origin refuse (H11-d):** if any slot's `origin_path_bare()` is empty (no faithful per-cosigner path possible), **REFUSE** via `ToolkitError::BadInput` (exit 1) — never substitute a placeholder origin. Message names the empty-origin cause. (Today the collapse silently emits `m/0'/0'`.)
- **Jade (H11-e):** covered by `wallet_export/jade.rs:46` delegation — no separate code; add the Jade-path RED test below.

**RED tests (write FIRST):**
- **#1 `export_coldcard_multisig_divergent_paths_emits_per_cosigner_derivation`** — divergent SORTED 2-of-3 (`--template wsh-sorted-multi`) with distinct paths → output contains a `Derivation:` line per cosigner with the real path, NO `m/0'/0'`, each cosigner line `<XFP_master>: <xpub>` carries its real master fp. RED today.
- **#1b `export_coldcard_multisig_sort_order_ne_slot_order_pairs_correctly`** (the load-bearing I-2 pairing test) — divergent SORTED multisig whose **xpub-lex sort order ≠ slot order** AND whose per-cosigner paths differ (construct slots so the xpub that sorts FIRST is at a higher slot index with a distinct path) → each emitted `Derivation:` is paired with the CORRECT `<XFP_master>: <xpub>` for the SAME slot (path↔xpub↔fp all from one sorted slot), NOT scrambled. **RED today AND would stay RED under a naive `derivations[i]`-indexed fix** → this is the test that forces H11-b's same-sorted-slot rule. (#1's `@0`==`@2` shape does NOT exercise this.)
- **#2 `export_coldcard_multisig_shared_path_unchanged`** — all-equal paths → single shared `Derivation:` + `<XFP>: <xpub>` lines, BYTE-IDENTICAL to current. GREEN-preserving regression guard.
- **#3 `export_coldcard_multisig_empty_origin_refuses`** — all slots empty `origin_path_bare()` → refuse exit 1 (BadInput), message names the empty-origin cause, NO `m/0'/0'` anywhere. RED today.
- **#4 `export_jade_divergent_paths_inherits_per_cosigner`** — same as #1 via `--format jade --template wsh-sorted-multi`; delegation covers it. RED today.
- **#5 / #12 `roundtrip_export_coldcard_multisig_divergent_then_import_matches`** (= the headline co-design proof, `roundtrip_divergent_master_fp_and_paths_preserved`) — export divergent (#1) → `import-wallet --format coldcard-multisig` → each cosigner's resolved `[fp/path]` equals the original (divergent path + master fp preserved). RED before this lane; **GREEN only with P1 (H14-c silent accept) + P2 (arm consumes per-line path) + P3 (emit) all landed.** The single assertion that proves H11 and H14 compose.

**Why P3 after P1+P2:** #5 (round-trip) needs the import side already correct (P1 silent-accept + P2 per-line-path parse) so the only remaining variable is the emit. RED→GREEN is then attributable to P3 alone.

### P4 — I-1 round-trip canonicalizer + `--round-trip-verify` surface
**File:** `wallet_import/roundtrip.rs` (`canonicalize_coldcard_multisig` `:361`).

**Implementation (Decision H11-f):**
- Extend `canonicalize_coldcard_multisig` (`:361`) to detect heterogeneous `parsed.cosigners[].path`. When homogeneous → UNCHANGED (single shared `Derivation:` from `cosigners[0].path`, current `:399-402`/`:413`; the `:1397` idempotence baseline stays GREEN). When heterogeneous → emit per-cosigner `Derivation:` lines (mirroring H11's divergent emit), so the per-cosigner paths H11 now preserves survive the canonical form instead of being re-collapsed onto `cosigners[0].path`.
- Replace the `:395-398` "ASSUMES homogeneous derivation" comment with the new heterogeneous-aware contract. The canonical re-emit must be **idempotent** on divergent blobs and must use the SAME per-cosigner ordering as the canonical sort (`cosigner_lines.sort()` `:393`) so `canon(canon(blob)) == canon(blob)`.
- Covers BOTH live surfaces: `--round-trip-verify` (`cmd/import_wallet.rs:1447`) and Jade round-trip (`roundtrip.rs:570` delegates here) — no separate Jade canonicalizer edit.
- **(plan-R0 M-1) Divergent test blobs MUST carry per-line `<XFP_master>:` (the exact shape P3 emits), NOT bare depth>0 xpubs.** `canonicalize_coldcard_multisig` re-parses via `parse_text` (`:368`); after P1, a divergent blob with depth>0 cosigners and NO supplied XFP **refuses at re-parse** (H14-b) — so #15/#16 would RED for the WRONG reason (the P1 refusal, not the canonicalizer collapse). Feed `Derivation: m/A\n<XFP_master_A>: <xpub_A>\nDerivation: m/B\n<XFP_master_B>: <xpub_B>` (per-line master fps supplied) so re-parse succeeds and the RED is attributable to the canonicalizer alone. (If the test uses depth-0 xpubs instead, no XFP is needed — either is acceptable as long as re-parse does NOT refuse.)
- **(plan-R0 M-2) Regression-guard set — verify these stay GREEN, do NOT edit:** `canonicalize_coldcard_multisig_idempotent` (`:1397`), `_with_and_without_xfp_header_match` (`:1410`), `_3of5_stable` (`:1428`), `_cosmetic_variants_match` (`:1445`), `_invalid_blob_returns_parse_error` (`:1485`). All feed homogeneous supplied==computed blobs → the homogeneous-path branch is unchanged → they stay GREEN. List them in the phase's GREEN-verification step, not just `:1397`.

**RED tests (write FIRST):**
- **#15 `canonicalize_coldcard_multisig_divergent_paths_preserves_per_cosigner`** — feed a divergent-path blob (heterogeneous `parsed.cosigners[].path`, **carrying per-line `<XFP_master>:` per M-1** so re-parse does not refuse) → canonical form emits per-cosigner `Derivation:` lines (NOT `cosigners[0].path` on all), and is idempotent (`canon(canon(blob)) == canon(blob)`). RED today (re-emits shared form from `cosigners[0].path` `:401`). Homogeneous blob still canonicalizes to single-shared-`Derivation:` (existing `canonicalize_coldcard_multisig_idempotent` `:1397` stays GREEN — assert this explicitly).
- **#16 `roundtrip_verify_divergent_coldcard_multisig_passes`** — `import-wallet --format coldcard-multisig --round-trip-verify` on a divergent-path blob (**per-line `<XFP_master>:` carried** so the import side does not refuse) → round-trip-verify PASSES (canonical form preserves per-cosigner paths → no spurious mismatch / false pass). Covers the LIVE `import_wallet.rs:1447` surface; the Jade analogue rides `:570`. RED today (canonicalizer collapses).

**Why P4 last:** the canonicalizer must preserve exactly the per-cosigner shape P3 emits; building it on P3's GREEN ensures #16's round-trip-verify exercises the real divergent export. (If sequencing surfaces a cleaner RED→GREEN by folding P4's canonicalizer into P3 — e.g. #16 needing the emit and the canonicalizer together — the implementer may merge P3+P4 into one phase, but MUST keep #15 and #16 as distinct RED-first assertions and gate on the full suite + clippy.)

---

## 4. RED-test inventory (≈16 — mapped to phases)

| # | Test | Phase |
|---|---|---|
| 1 | `export_coldcard_multisig_divergent_paths_emits_per_cosigner_derivation` | P3 |
| 1b | `export_coldcard_multisig_sort_order_ne_slot_order_pairs_correctly` (I-2 load-bearing) | P3 |
| 2 | `export_coldcard_multisig_shared_path_unchanged` (regression) | P3 |
| 3 | `export_coldcard_multisig_empty_origin_refuses` (exit 1) | P3 |
| 4 | `export_jade_divergent_paths_inherits_per_cosigner` | P3 |
| 5/12 | `roundtrip_export_coldcard_multisig_divergent_then_import_matches` (= `roundtrip_divergent_master_fp_and_paths_preserved`, headline) | P3 (needs P1+P2) |
| 6 | `import_coldcard_multisig_depth_gt0_no_xfp_refuses` (exit 2) | P1 |
| 7 | `import_coldcard_multisig_depth_gt0_with_header_xfp_no_warning` | P1 |
| 8 | `import_coldcard_multisig_depth_gt0_per_line_xfp_no_warning` | P1 |
| 9 | `import_coldcard_multisig_depth0_no_xfp_uses_computed` (over-refuse guard) | P1 |
| 10 | `import_coldcard_multisig_depth0_xfp_mismatch_warns` (under-warn guard) | P1 |
| 11 | fixture-rewrite regression (masked `:954+`; M-2 silent all-agree) | P1 |
| 13 | `import_coldcard_multisig_shared_derivation_3_cosigners_all_resolve_shared` (Q1 condition ii) | P2 |
| (13b) | `import_coldcard_multisig_per_line_derivation_plus_xfp_roundtrips_path` (new positive Q1 RED) | P2 |
| 14 | `import_jade_depth_gt0_no_xfp_refuses` (M-1 Jade blast-radius) | P1 |
| 15 | `canonicalize_coldcard_multisig_divergent_paths_preserves_per_cosigner` (I-1, idempotent) | P4 |
| 16 | `roundtrip_verify_divergent_coldcard_multisig_passes` (live `:1447` + `:570`) | P4 |

---

## 5. Affected files (final)

- `crates/mnemonic-toolkit/src/wallet_export/coldcard.rs` — H11 divergent per-cosigner emit + sorted-slot pairing (H11-b) + empty-origin refuse (H11-d).
- `crates/mnemonic-toolkit/src/wallet_import/coldcard_multisig.rs` — H14 depth-gated truth table (H14-a..g) + Q1 arm change (consume pending per-line path, never clear `shared_derivation`) + fixture rewrite (`:945-947` + dependents).
- `crates/mnemonic-toolkit/src/wallet_import/roundtrip.rs` — `canonicalize_coldcard_multisig` per-cosigner `Derivation:` on heterogeneous paths (H11-f); covers `--round-trip-verify` + Jade round-trip.
- `design/SPEC_wallet_import_v0_28_0.md` §11.4.1 — depth-gated truth-table + corrected computed-fp formula (H14-g).
- (`wallet_export/jade.rs` + `wallet_import/jade.rs` — covered by delegation; NO edits, but Jade round-trip + Jade-import-refusal tests added.)
- (voluntary, same-PR) `docs/manual/src/45-foreign-formats.md` + `docs/manual/src/30-workflows/37-wallet-export.md` — divergent per-cosigner export form + depth>0-no-XFP refusal + "supply the device XFP" guidance.

**Untouched (orchestrator's integration job):** `Cargo.toml`, `CHANGELOG`, both `README`s, `install.sh`, `fuzz/Cargo.lock`, `mnemonic-gui` schema. No clap surface change → `schema_mirror` not touched.

---

## 6. MANDATORY whole-diff review (non-deferrable)

Per CLAUDE.md, after the last phase is GREEN, run a **mandatory independent adversarial whole-diff execution review** over the entire Lane-A diff (R0 = plan correctness; this review catches implementation-introduced regressions TDD misses). Persist the verbatim review to `design/agent-reports/cycle13a-coldcard-multisig-whole-diff-review.md` BEFORE any integration/ship. Review focus areas:
- **I-2 scramble:** confirm the divergent emit reads path/xpub/fp from the SAME sorted slot (test #1b GREEN for the RIGHT reason — not accidentally passing because sort==slot in the fixture).
- **Fixture blast-radius (plan-R0 I-1, both rounds):** confirm the COMPLETE cross-file break set was reconciled — UNIT layer (`coldcard_multisig.rs`): `:1042` SPLIT into a depth-0-silent (H14-a) assertion AND a depth>0-REFUSE (H14-b) assertion (NOT left as a silent-parse), `:1418` carries per-line XFPs so the coin-type check (`:438-444`) is still reached, `:990`/`:1265` re-pointed to depth-0 `XPUB_D0_*` (asserts KEPT, not deleted); **CLI layer (`tests/cli_import_wallet_coldcard_multisig.rs`, plan-R0 round-2): `:166` + `:209` re-pointed to depth-0 `XPUB_D0_*` (asserts KEPT)** — these flip warn→silent under H14-c and are NOT caught by the unit-layer enumeration. Confirm depth-0 `XPUB_D0_*`/`FP_D0_*` consts were actually added (none existed at `9b2a8ae3`).
- **Q1 shared-path non-regression:** confirm `shared_derivation` is never cleared by the `<XFP>:` arm (test #13 GREEN; spot-check a 5-cosigner shared blob).
- **H14-c silence:** confirm the toolkit's OWN all-agree export round-trips with EMPTY stderr (M-2; no leftover Row-2 warning).
- **Refusal exit codes:** #3 → exit 1 (BadInput), #6/#14 → exit 2 (ImportWalletParse).
- **Idempotence:** #15 `canon(canon)==canon` on a divergent blob; `:1397` baseline still GREEN.
- **No fmt drift / no version-site edits / no schema_mirror delta.**
If the Agent-API dispatch fails mid-session, FLAG it explicitly and defer the formal review to API recovery — never silently substitute inline self-review (CLAUDE.md per-phase policy item 5).

---

## 7. FOLLOWUP / report-tick (orchestrator does the integrated tick)

- **At SHIP (integrated v0.66.0), tick H11 and H14 in `design/agent-reports/constellation-bughunt-2026-06-20.md`** — the checkbox headers are LIVE at `### - [ ] H11 · …` (`:715`) and `### - [ ] H14 · …` (`:994`) on origin/master this session. **RE-GREP these line numbers at ship time** (the report mutates as Lanes B/C tick their findings in the same integrated commit; line anchors decay). The orchestrator performs the integrated tick for all three lanes in the shipping commit (MEMORY `feedback_followup_status_discipline`: flip status in the shipping commit, verify "open" at decision time).
- **No new `FOLLOWUPS.md` slug** is required for Lane A (the fix is complete, not deferred). If the whole-diff review surfaces a deferral (e.g. a tr(multi_a) coldcard-export edge already fenced by an existing FOLLOWUP), file it then per the post-cycle burndown habit (MEMORY `feedback_post_cycle_followup_burndown`).
- **No sibling-repo companion** (`md`/`ms`/`mk` NO-BUMP; toolkit-only).

---

## 8. MANDATORY R0 GATE (CLAUDE.md hard gate)

**NO code before this plan-doc is GREEN (0 Critical / 0 Important).** This plan-doc → opus architect **R0 review** BEFORE any implementation begins. Fold findings → persist the review verbatim to `design/agent-reports/cycle13a-coldcard-multisig-plan-R<n>-review.md` → re-dispatch → repeat until GREEN (the reviewer-loop continues after EVERY fold — folds themselves introduce drift). Only after plan-R0 GREEN does the single-implementer TDD execution begin (P1→P2→P3→P4), with per-phase full-suite + clippy gates and per-phase reviews persisted to `design/agent-reports/`, then the mandatory whole-diff review (§6). Any agent verifying external protocol facts (Coldcard text format, BIP-32 fingerprint/depth semantics) re-checks authoritative source (the toolkit's own round-trip parser + vendored `bitcoin 0.32.8` `bip32.rs`), not just this doc.
