# v0.13.0 P3 — Canonical `## mnemonic slip39` manual chapter

**Phase:** P3 (manual chapter rewrite — replaces the P2.3 stub)
**Predecessor LOCK:** `ea7c675` (P2.3 R1 LOCK round 1; Opus 0C/0I/0N/0n clean)
**Brainstorm:** 2026-05-14 (5 clarifying questions answered + 1 outline-approval pass + 1 pre-decided-item nudge — `\index{}` markers now in scope)
**Cadence:** R0 architect plan review (Opus, against this file) → GREEN single-commit chapter rewrite + index-marker additions → R1 LOCK reviewer (Opus, against final commit). Two reviewer checkpoints, no RED.
**R0 fold (2026-05-14):** Opus R0 returned 1C/3I/5N/3n at `design/agent-reports/v0_13_0-slip39-p3-r0.md`. Folded inline into this revision: C1 (`\index{}` flat-marker scheme — see §3.2 + §8 fold notes); I1 (Trezor One drop + softened `--backup-type` flag — see §3.10 + §7 part 3); I2 (paired-SPEC-patch mandate on stem drift — see §10.4 + §11.3); I3 (0.3.0 ↔ commit-hash chapter-prose disclosure — see §7 part 2); N1 (`--iteration-exponent` perf advisory note in example 4); N2 (`--from entropy=` variant note in example 1); N3 (env-var advisory naming clarification in §6.2); N5 (§2.1 marker count 5→6). **Deferred:** N4 (LOC upper-bound LOCK criterion — defer to R1); n1 (asymmetric-groups example mention); n2 (§14 commit-count math precision); n3 (PE precedent cite for no-RED rationale).

## §1 Goal

Replace the P2.3 minimum-viable stub at `docs/manual/src/40-cli-reference/41-mnemonic.md` (`## mnemonic slip39` H2, ~42 LOC, currently lines ~419-461 post-P2.3) with a canonical reference chapter (~305 LOC) that:

1. Covers the SLIP-39 conceptual model (share / group / member / threshold / identifier / iteration exponent / extendable bit / passphrase) at a depth that lets a new reader use the CLI without consulting SPEC.
2. Walks through 4 progressive worked examples (1-of-1 no-pass → 1-of-1 with-pass → 2-of-3 no-pass → 2-of-3 of 2-of-3 with-pass) showing both `split` and reverse-via-`combine`.
3. Mirrors SPEC §2.5 (24 refusal classes) + §2.6 (6 advisory rows) byte-faithfully so the chapter is a reference table, not just narrative.
4. Documents the JSON envelope schema for both `split` and `combine` operations, byte-faithful with the SHA-pinned `cli_slip39_json.rs` test fixtures.
5. Provides a runnable Trezor-interop smoke recipe via cross-implementation verification against `python-shamir-mnemonic` (the canonical SLIP-0039 reference; toolkit is bit-exact verified against `python-shamir-mnemonic@17fcce14` per SPEC §5).
6. Adds a small set (~3-5) of `\index{}` markers + matching `69-index-table.md` rows so SLIP-39 surface is discoverable from the appendix index — even though existing 40-cli-reference chapters carry zero markers (the user's P3 brainstorm response set this new precedent: "the existence of slip-39 support should be reflected in index too, even if just an entry to first page of chapter").

The flag-coverage lint already passes after P2.3; P3 must NOT regress it.

## §2 File inventory + scope

### §2.1 Modified files (3)

1. `docs/manual/src/40-cli-reference/41-mnemonic.md` — replace the P2.3 `## mnemonic slip39` stub (~42 LOC) with the canonical chapter (~305 LOC). Single contiguous H2 between `## mnemonic seed-xor` and `## mnemonic gui-schema`. NO touch to chapter intro or to other H2s.
2. `docs/manual/src/60-appendices/69-index-table.md` — append rows for the `\index{}` markers added by §8 (6 rows; alphabetically positioned).
3. `crates/mnemonic-toolkit/Cargo.toml` — NO touch (PE bumps version, not P3).

### §2.2 No new files

P3 is content-only on existing manual artifacts. No new `tests/`, `docs/`, or `design/` files (this plan + the R0 + R1 reviewer reports are the only design artifacts).

### §2.3 Out-of-scope at P3 (deferred to PE)

- `Cargo.toml` version bump 0.13.0-dev → 0.13.0
- `CHANGELOG.md` v0.13.0 entry
- Tag `mnemonic-toolkit-v0.13.0` + GitHub release
- Push the now-12 unpushed commits (8 pre-P3 + 3 P3 commits-to-be: R0 report + GREEN + R1 LOCK report) to origin/master
- Companion `slip39-gui-schema-flattening-companion` FOLLOWUP closure in adjacent `mnemonic-gui` working tree

## §3 Chapter structure (section-by-section)

10 H3 sections under the `## mnemonic slip39` H2. LOC budgets are ±20%; total ~305 LOC.

### §3.1 Intro paragraph (~12 LOC)

Replaces the P2.3 stub's "Canonical chapter content lands at P3" disclaimer. Sets up:
- What SLIP-39 is (Trezor's K-of-N share-splitting standard).
- How it differs from `seed-xor` (threshold reconstruction vs all-N XOR).
- That shares are SLIP-39 mnemonics (NOT BIP-39 — different wordlist, different length, different checksum).
- Cross-link to the `## mnemonic seed-xor` H2 above.

Add `\index{SLIP-39}` here so the chapter-top entry resolves.

### §3.2 Concept signposts H3 (~30 LOC)

Mirrors the manual's chapter-level `## Concept signposts` style. Defines (one short paragraph each):
- **Master secret** — the BIP-39 phrase or raw entropy that `split` consumes / `combine` recovers.
- **Share** — a single SLIP-39 mnemonic produced by `split`. Each share is independently secret material.
- **Group / member** — a group is a partition of shares; a member is one share within a group.
- **Group threshold (`G`)** — how many groups must contribute ≥ their member threshold of shares to reconstruct.
- **Member threshold (`T`)** — per-group: how many of that group's shares must combine to reconstruct that group's "group secret".
- **Identifier** — random 15-bit per-secret fingerprint shared across all shares of one split; mismatch on `combine` → refusal row 7.
- **Iteration exponent (`E`)** — PBKDF2 cost; iterations = 10000 × 2^E. Trezor default E=1 (20000 iters); E ≥ 5 emits a perf advisory (§3.9).
- **Passphrase** — SLIP-39 passphrase (NOT the BIP-39 passphrase); empty string = SLIP-39 default.
- **Extendable bit** — 1-bit flag controlling whether the identifier participates in the PBKDF2 salt.

Add `\index{SLIP-39 share}` and `\index{group threshold}` and `\index{member threshold}` here. (R0 C1 fold: flat marker form, NOT LaTeX sub-entry `share!SLIP-39` — the lint's source-side normalizer does not strip `!` so the sub-entry form would fail bidirectional check.)

### §3.3 Synopsis H3 (~10 LOC)

Two `sh` blocks:

```sh
mnemonic slip39 split   --from <phrase=…|entropy=…> --group-threshold G --group N,T [--group N,T]... [OPTIONS]
mnemonic slip39 combine --share <slip39-mnemonic-or-> ... [OPTIONS]
```

### §3.4 `slip39 split` flags H3 (~20 LOC)

Verbatim from the P2.3 stub's split table (already lint-clean). 8 rows + `--help`. Pre-existing wording. NO column changes.

### §3.5 `slip39 combine` flags H3 (~15 LOC)

Verbatim from the P2.3 stub's combine table. 6 rows + `--help`.

### §3.6 Worked examples H3 (~120 LOC = 4 × ~30 LOC)

Four sub-H4 examples, ordered by increasing complexity. Detailed in §4.

Add `\index{K-of-N}` somewhere in this section (the headline SLIP-39 feature).

### §3.7 JSON output H3 (~30 LOC)

Both schemas presented as fenced `json` blocks, byte-faithful with `crates/mnemonic-toolkit/tests/cli_slip39_json.rs` SHA-pin EXPECTED constants. Detailed in §5.

### §3.8 Refusals H3 (~28 LOC)

Full SPEC §2.5 mirror — 24 rows, each one line. Two-column table: `Trigger | Refusal stem`. Detailed in §6 (the byte-faithful row-by-row mirror plan).

### §3.9 Advisories H3 (~10 LOC)

Full SPEC §2.6 mirror — 6 rows, each one line. Two-column table: `Trigger | Stderr advisory`. Detailed in §6.

### §3.10 Trezor interop H3 (~30 LOC)

Cross-impl smoke recipe via `python-shamir-mnemonic`. Detailed in §7. (R0 I1 fold: chapter prose drops the Trezor One mention — SPEC §3 OOS row `OOS-slip39-import-trezor-onev-format` confirms Trezor One predates SLIP-39 — and softens the firmware-version-specific backup-type-flag claim.)

## §4 Worked-example detail

All examples use abstract output (`<share-1 (33 words)>`, etc.) — NOT byte-exact share text — because `split` is CSPRNG-driven. Each example shows: master input + `split` invocation + abstract output + reverse-via-`combine` invocation. Master secret across all 4 examples: `abandon × 23 + art` (canonical BIP-39 zero-entropy 24-word vector; matches the seed-xor chapter's example precedent for reader recognition).

### §4.1 Example 1: 1-of-1 no passphrase (~22 LOC)

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 1,1
```

Stdout: 1 line, a 33-word SLIP-39 share `<share-1>`. Reverse:

```sh
mnemonic slip39 combine --share "<share-1>"
```

Stdout: the original 24-word `abandon × 23 + art` phrase recovered.

Pedagogical point: shows the basic `split`/`combine` mechanic with the simplest possible threshold (degenerate). Sets expectations for share length (33 words for 32-byte master entropy at default iter_exp=0).

R0 N2 fold — also include a one-line `--from entropy=` variant (cheap, closes the entropy-vs-phrase gap that all 4 examples otherwise leave): "alternative master input via raw hex entropy: `mnemonic slip39 split --from entropy=0102030405060708090a0b0c0d0e0f10 --group-threshold 1 --group 1,1` — produces a 20-word share (16-byte entropy at default iter_exp=0); the JSON envelope's `identifier` + `iteration_exponent` shape is the same regardless of `phrase=` vs `entropy=`".

### §4.2 Example 2: 1-of-1 with passphrase (~30 LOC)

```sh
echo "abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon abandon \
abandon abandon abandon abandon abandon abandon abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 1,1 \
    --passphrase "TREZOR"
```

Stdout: 1 share. Reverse with the matching passphrase:

```sh
mnemonic slip39 combine --share "<share-1>" --passphrase "TREZOR"
```

Reverse with the WRONG passphrase (or omitted): silently recovers a DIFFERENT entropy — SLIP-39 passphrase has no authentication tag (same security model as BIP-39 passphrase). Note this explicitly + cross-link to refusal row 11.

Pedagogical point: introduces the SLIP-39 passphrase. Notes the empty-vs-omitted distinction (both = SLIP-39 default; neither emits the inline-secret-on-argv advisory because `--passphrase` only fires the advisory when `is_some()`).

### §4.3 Example 3: 2-of-3 single group, no passphrase (~30 LOC)

```sh
echo "abandon abandon abandon ... abandon art" |
  mnemonic slip39 split --from phrase=- --group-threshold 1 --group 3,2
```

Stdout: 3 shares `<share-1>`, `<share-2>`, `<share-3>`. Reverse with any 2:

```sh
mnemonic slip39 combine --share "<share-1>" --share "<share-2>"
```

Pedagogical point: introduces the K-of-N threshold (the headline SLIP-39 feature). Notes that the 3rd share is "extra resilience" — losing any single share is recoverable. Briefly mentions what `slip39 combine` with only 1 share does (refusal row 12: "insufficient shares for group 0: need 2, got 1").

### §4.4 Example 4: 2-of-3 of 2-of-3 multi-group with passphrase (~38 LOC)

```sh
echo "abandon abandon ... abandon art" |
  mnemonic slip39 split --from phrase=- \
    --group-threshold 2 \
    --group 3,2 --group 3,2 --group 3,2 \
    --passphrase "TREZOR"
```

Stdout: 9 shares (3 groups × 3 members each); group_idx is encoded in each share. Per the SPEC, share order in stdout is group-major:

```
<g0-m0>
<g0-m1>
<g0-m2>
<g1-m0>
...
<g2-m2>
```

Reverse with 2 shares from group 0 + 2 shares from group 1 (groups 2 unused — group_threshold=2 satisfied):

```sh
mnemonic slip39 combine \
  --share "<g0-m0>" --share "<g0-m1>" \
  --share "<g1-m0>" --share "<g1-m1>" \
  --passphrase "TREZOR"
```

Pedagogical point: the comprehensive case. Shows multi-group, separate thresholds, passphrase, and the recovery flexibility (2 of 3 groups, 2 of 3 members each → 4 of 9 total shares suffice; many such 4-share subsets are valid). Briefly notes `--group-threshold 2 --group 3,2 --group 3,2 --group 3,2` is "social-recovery"-style: 3 trustees each hold 3 shares; any 2 trustees with ≥2 of their 3 shares cooperate.

R0 N1 fold — append a "Note:" paragraph: "to exercise the iteration-exponent perf advisory (§3.9), append `--iteration-exponent 5` to the `split` invocation; stderr will print `warning: --iteration-exponent E=5 yields 320000 × PBKDF2-HMAC-SHA-256 iterations; split + combine performance may be observably slow ...`. The exponent is encoded in each share's id_exp field, so the matching `combine` invocation needs no extra flag — it reads the exponent from the shares automatically." Closes the example-coverage gap on `--iteration-exponent` for ~3 LOC.

This example's combine recipe is also the input to §3.10 Trezor interop.

## §5 JSON envelope content

Both `split` and `combine` envelopes are SHA-pinned in `cli_slip39_json.rs`. Pull verbatim. The chapter shows abstract stand-ins for share-text (`<share-1>` etc.) but otherwise byte-exact field names + types + ordering.

Field names + ordering verified against `cli_slip39_json.rs` (HEAD `ea7c675`) — see lines 50-110 for split, 230-325 for combine. Both envelopes have `schema_version` + `operation` first, then operation-specific fields, in source-defined order (the `R0 N4 fold` test `json_split_group_entry_field_order_shares_last` enforces `shares` is last in each group entry).

### §5.1 `split` envelope

```json
{
  "schema_version": "1",
  "operation": "split",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "group_threshold": 2,
  "groups": [
    {"member_count": 3, "member_threshold": 2, "shares": ["<g0-m0>", "<g0-m1>", "<g0-m2>"]},
    {"member_count": 3, "member_threshold": 2, "shares": ["<g1-m0>", "<g1-m1>", "<g1-m2>"]},
    {"member_count": 3, "member_threshold": 2, "shares": ["<g2-m0>", "<g2-m1>", "<g2-m2>"]}
  ]
}
```

Each group entry is `{member_count, member_threshold, shares}` in that exact order (R0 N4 fold; mirrors `seed_xor.rs:352-361`). NO top-level `language` field, NO `master_word_count` field — chapter prose conveys those out of band.

### §5.2 `combine` envelope (both `--to entropy` and `--to phrase` shapes)

`--to entropy` (default):

```json
{
  "schema_version": "1",
  "operation": "combine",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "output_shape": "entropy",
  "entropy_hex": "0102030405060708090a0b0c0d0e0f10",
  "phrase": null
}
```

`--to phrase`:

```json
{
  "schema_version": "1",
  "operation": "combine",
  "identifier": <u64>,
  "iteration_exponent": 0,
  "output_shape": "phrase",
  "entropy_hex": null,
  "phrase": "abandon abandon … art"
}
```

Both `entropy_hex` and `phrase` are always present; one carries the value and the other is `null`, selected by `output_shape`. No `--language` field in the envelope — `--language` only affects whether the `phrase` value uses English / Czech / Korean / etc. (the chapter calls this out in prose).

## §6 SPEC §2.5 + §2.6 mirror tables

### §6.1 Refusals table (24 rows)

Pull verbatim from `design/SPEC_slip39_v0_13_0.md` §2.5. Two-column markdown table: `Trigger | Refusal stem`. Each row's stem text must match SPEC byte-faithfully (substring match against the lib's actual emitted error stems is what makes them "byte-faithful" — verified by `cli_slip39_refusals.rs`). At GREEN-write time, do `awk '/^### §2.5/,/^### §2.6/' design/SPEC_slip39_v0_13_0.md | grep '^| [0-9]'` to capture all 24 rows; transcribe to chapter table.

### §6.2 Advisories table (6 rows)

Pull verbatim from SPEC §2.6. Same approach.

The advisory row for `MNEMONIC_SLIP39_TEST_RNG` should be present even though it's a test-only env-var — readers may encounter the warning text via search and need to know what triggered it.

R0 N3 fold — add a one-line note beside the advisory table: "The warning string names `MNEMONIC_SLIP39_TEST_RNG` even when only the companion `MNEMONIC_SLIP39_TEST_IDENTIFIER` is set — both env-vars trigger the same single-string advisory; see SPEC §6 for both env-var definitions."

## §7 Trezor interop recipe (§3.10 detail)

Single H3 + ~30 LOC. Three parts:

1. **What the smoke recipe proves** — bit-exact compatibility of the toolkit's emitted SLIP-39 shares with the Trezor SLIP-0039 reference, demonstrated by cross-implementation recovery. The toolkit's library is already verified against `python-shamir-mnemonic@17fcce14` (45 fixture vectors, see SPEC §5); the recipe lets readers reproduce that verification themselves.

2. **The recipe** (paste-able, no hardware needed):

   ```sh
   pip install shamir-mnemonic==0.3.0
   python -m shamir_mnemonic.cli combine \
     --passphrase "TREZOR" \
     "<g0-m0>" "<g0-m1>" \
     "<g1-m0>" "<g1-m1>"
   ```

   Reuses example 4's shares + passphrase. Output: the same 32-byte master entropy that `mnemonic slip39 combine` recovered. Note: `python-shamir-mnemonic` outputs hex entropy by default (not BIP-39 phrase); convert via `mnemonic convert --from entropy=<hex> --to phrase` if reader wants the phrase form.

   R0 I3 fold — chapter prose MUST disclose the version-pin caveat: "Recipe pinned to `shamir-mnemonic==0.3.0` (latest released PyPI version at chapter-write 2026-05-14). The toolkit's library compatibility is verified against upstream commit `17fcce14`; if the recipe fails for you, the released PyPI version may have introduced a wire-format change since the toolkit's vendored test vectors. The version-pinned PyPI archive is at <https://pypi.org/project/shamir-mnemonic/0.3.0/>. File a toolkit issue with the failing share text + python error if encountered." Stronger fold (recommended): GREEN-write actually runs the recipe end-to-end against shares from `target/debug/mnemonic slip39 split` and converts the prose to "validated 2026-05-14 against shamir-mnemonic 0.3.0 on Linux x86_64".

3. **Trezor hardware compatibility note** — informational paragraph (R0 I1 fold — drops Trezor One, softens backup-type-flag specificity, distinguishes basic vs advanced modes): "Shares produced by `mnemonic slip39 split` are bit-identical to Trezor SLIP-0039. Users with a Trezor Model T or Safe family device (NOT Trezor One — Trezor One predates SLIP-39 and uses raw BIP-39 only) can verify by importing through the Trezor Suite recovery wizard. SLIP-39 has two modes: `slip39-basic` for single-group splits (e.g., examples 1-3 above; the `2-of-3` shape) and `slip39-advanced` for multi-group splits (example 4's `2-of-3-of-2-of-3` shape). Consult Trezor's current docs for the exact `trezorctl recovery-device --backup-type` flag value, which has historically varied by firmware version."

Add `\index{Trezor SLIP-0039 interop}` here.

## §8 Index marker plan

Total: 6 `\index{}` markers; 6 corresponding rows in `69-index-table.md`.

R0 C1 fold — all markers use FLAT form (no LaTeX `!` sub-entry). The lint's source-side normalizer at `lint.sh:124-125` strips `\_` → `_` only; `!` is not stripped, so any `\index{X!Y}` source-side resolves to literal `X!Y` and cannot match a `| `X, Y` |` table row. Flat form keeps source-side and table-side strings byte-identical.

| Marker | Section | 69-index-table.md row |
|---|---|---|
| `\index{SLIP-39}` | §3.1 intro | `| `SLIP-39` | [mnemonic slip39](#mnemonic-slip39) |` |
| `\index{SLIP-39 share}` | §3.2 concept signposts | `| `SLIP-39 share` | [mnemonic slip39](#mnemonic-slip39) |` |
| `\index{group threshold}` | §3.2 concept signposts | `| `group threshold` | [mnemonic slip39](#mnemonic-slip39) |` |
| `\index{member threshold}` | §3.2 concept signposts | `| `member threshold` | [mnemonic slip39](#mnemonic-slip39) |` |
| `\index{K-of-N}` | §3.6 worked examples | `| `K-of-N` | [mnemonic slip39](#mnemonic-slip39) |` |
| `\index{Trezor SLIP-0039 interop}` | §3.10 Trezor interop | `| `Trezor SLIP-0039 interop` | [mnemonic slip39](#mnemonic-slip39) |` |

The bidirectional lint check at `lint.sh` step 6/6 will enforce: every `\index{}` in `41-mnemonic.md` must match a row in `69-index-table.md`, and vice versa. NO partial migration — ship all markers + rows in the same commit.

NOTE (post R0 C1 fold): the `\index{share!SLIP-39}` LaTeX sub-entry approach was rejected at R0 because `lint.sh:124-125` does not normalize `!`; the marker scheme is now FLAT (no sub-entries) — see fold rationale above the marker table. Three alternatives were considered: (a) literal `\index{share, SLIP-39}` with comma in marker (rejected — `makeindex` PDF render misinterprets the comma as part of the term not a sub-entry); (b) flat `\index{SLIP-39 share}` (CHOSEN — trivial source/table match, no lint patches, no PDF render issues); (c) patching `lint.sh` to strip `!` to `, ` (rejected — out of P3 scope per §2.3, sets non-obvious convention). Future chapter authors adding markers MUST use flat form unless `lint.sh` is extended.

## §9 Phase split

Single GREEN commit + 2 reviewer commits = 3 P3 commits total.

### §9.1 R0 — pre-GREEN architect plan review

Dispatch Opus `general-purpose` agent with `model: "opus"` per memory `feedback_opus_primary_review_agent`. Agent reviews this plan file against the SPEC + the existing P2.3 stub + the analogous chapter precedent (seed-xor + final-word). Produces report at `design/agent-reports/v0_13_0-slip39-p3-r0.md`. Confidence-≥80 findings get folded into this plan before GREEN starts.

### §9.2 GREEN — chapter rewrite + index markers

Single commit. Verbatim commit-message header:

```
docs(manual): v0.13.0 P3 GREEN — canonical 41-mnemonic.md slip39 chapter (4 progressive worked examples + full SPEC §2.5/§2.6 mirror + python-shamir-mnemonic interop recipe + index markers)
```

Files: `docs/manual/src/40-cli-reference/41-mnemonic.md` + `docs/manual/src/60-appendices/69-index-table.md`. Verify `make -C docs/manual lint MNEMONIC_BIN=...` returns `[lint] OK` (all 6 steps; flag-coverage + index-bidirectional are the two gates that could fail).

### §9.3 R1 — post-GREEN LOCK reviewer

Dispatch Opus `general-purpose` agent with `model: "opus"`. Agent verifies the GREEN commit against this plan + LOCK criteria §10. Produces report at `design/agent-reports/v0_13_0-slip39-p3-r1.md`.

If R1 returns 0C/0I → LOCK clean, P3 closes. PE unblocks.
If R1 returns C or I findings → fold inline (or in a fold-commit), re-dispatch R2 if scope warrants.

## §10 Verification gates / LOCK criteria

1. `docs/manual/src/40-cli-reference/41-mnemonic.md` `## mnemonic slip39` section is ≥ 250 LOC and contains all 10 H3 subsections per §3.
2. The chapter's split-flags table contains all 8 flag strings the live `slip39 split --help` emits (lint flag-coverage; already passing post-P2.3 — must NOT regress).
3. The chapter's combine-flags table contains all 6 flag strings the live `slip39 combine --help` emits.
4. Refusals table has 24 rows mirroring SPEC §2.5 byte-faithfully AND mirroring `cli_slip39_refusals.rs` byte-faithfully (R0 I2 fold: lib-AND-SPEC byte-faithful agreement required; if lib and SPEC drifted at GREEN-write time, the GREEN commit MUST also patch SPEC §2.5 — no defer-via-FOLLOWUP). At plan-time R0 verified zero drift between SPEC and lib; the criterion is forward-looking.
5. Advisories table has 6 rows mirroring SPEC §2.6 byte-faithfully.
6. JSON envelope examples for `split` + `combine` are byte-faithful with the SHA-pinned `cli_slip39_json.rs` field names + ordering.
7. Trezor interop H3 includes the runnable `python-shamir-mnemonic` recipe + the hardware compatibility note.
8. `make -C docs/manual lint MNEMONIC_BIN=$(realpath target/debug/mnemonic) ...` returns `[lint] OK` (all 6 steps; index-bidirectional must pass for the new `\index{}` markers).
9. `cargo test --tests --no-fail-fast` totals still 978 / 0 / 8 (P3 touches zero Rust source).
10. `cargo clippy --all-targets -- -D warnings` clean (same).
11. R1 LOCK reviewer returns 0 Critical / 0 Important at confidence ≥ 80.

## §11 Risk areas

1. **`\index{}` first-mover risk** — none of the 4 existing 40-cli-reference chapters use `\index{}` markers; P3 sets new convention. Risk: a future chapter author adds markers in a non-uniform style. Mitigation: low-priority follow-up to retroactively add markers to seed-xor / final-word / etc. — out of P3 scope; file as `cli-reference-index-markers-retrofit` FOLLOWUP if R0 reviewer flags.

2. **JSON field name drift** — chapter's `split` envelope shows `"groups": [{"member_threshold": ..., "shares": [...]}]`. If the actual SHA-pinned shape uses different field names (e.g. `"per_group_member_threshold"` or flat `"shares_by_group_idx"`), chapter is wrong. Mitigation: at GREEN-write time, transcribe directly from `cli_slip39_json.rs::EXPECTED` constants; do NOT copy from this plan's §5 sketch.

3. **Refusal stem byte-drift** — SPEC §2.5 row stems are the canonical reference, but the actual emitted stems are in `cmd/slip39.rs` and verified by `cli_slip39_refusals.rs`. If SPEC and emitted text drifted post-P2.2 fold, chapter could mirror the stale SPEC. R0 verified at plan-time that no drift currently exists (all 24 stems byte-match between SPEC and `cli_slip39_refusals.rs`); the risk is forward-looking. Mitigation (R0 I2 fold; tightened from "lib-wins-on-drift + FOLLOWUP defer"): at GREEN-write time, cross-check SPEC table rows against `cli_slip39_refusals.rs::stderr.contains` assertions. **If lib and SPEC drift detected, the P3 GREEN commit MUST also patch SPEC §2.5 in the same commit** — mirrors P2.2 GREEN's `d40eb0c` 8-SPEC-patch precedent. Do NOT defer SPEC reconciliation to a separate FOLLOWUP — that produces a stale-SPEC trap for the next phase's R0 reviewer.

4. **`python-shamir-mnemonic` API drift** — recipe pins `pip install shamir-mnemonic==0.3.0`. If the upstream API changes between writing and a reader running the recipe, recipe is wrong. Mitigation: pin the version exactly; note in chapter prose that `0.3.0` is the version verified at chapter-write time. R0 reviewer should verify the pinned version matches `python-shamir-mnemonic@17fcce14` SemVer.

5. **Chapter length overrun** — budget ~305 LOC; risk it grows to 400+. Mitigation: at GREEN-write time, after each H3 section is drafted, count LOC and trim if > 1.3× budget.

6. **Stub-replacement diff size** — single GREEN commit will be ~+305 / -42 net + the `69-index-table.md` additions. Reviewer-friendly but a big single hunk. Mitigation: none — splitting the GREEN into multiple commits (one per H3) was rejected at brainstorm in favor of single-commit clarity.

## §12 Open questions — RESOLVED at brainstorm

| Q | Resolution |
|---|---|
| Q1 | **Worked example set:** 4 progressive examples (1-of-1 no-pass / 1-of-1 with-pass / 2-of-3 no-pass / 2-of-3 of 2-of-3 with-pass). User explicit: "Examples are important". |
| Q2 | **Trezor recipe approach:** cross-impl verify via `python-shamir-mnemonic`. Reproducible without hardware. |
| Q3 | **Refusal/advisory coverage:** full SPEC mirror — 24 refusal rows + 6 advisory rows. |
| Q4 | **Cadence:** R0 plan review → GREEN → R1 LOCK reviewer (no RED — no natural failing-test gate for prose). |
| Q5 | **Example output strategy:** abstract `<share-N (33 words)>` placeholders; CSPRNG output isn't reproducible without the env-var wedge, and the wedge's INSECURE warning is ugly tutorial chatter. |
| Q6 | **Index markers:** add 5-6 `\index{}` markers + matching `69-index-table.md` rows. User explicit: "the existence of slip-39 support should be reflected in index too, even if just an entry to first page of chapter". Sets new precedent for 40-cli-reference chapters. |
| Q7 | **Glossary additions:** skip. The `lint.sh` glossary-coverage step uses a fixed term list; chapter prose stands on its own. |
| Q8 | **Master secret choice:** `abandon × N + <last>` zero-entropy canonical vector across all 4 examples. Matches seed-xor chapter precedent. |
| Q9 | **Replace vs grow stub:** wholesale-replace in single GREEN commit. |
| Q10 | **Chapter intro list bump:** none — P2.3 already set "Nine subcommands" + slip39 cross-link. |

## §13 Cross-refs

- Predecessor LOCK: `design/agent-reports/v0_13_0-slip39-p2-3-r1.md`
- Existing chapter: `docs/manual/src/40-cli-reference/41-mnemonic.md` (as of `e68a042` GREEN)
- Analogous chapter precedents: `## mnemonic seed-xor` (~125 LOC, lines 294-417) + `## mnemonic final-word` (~80 LOC, lines 213-291) — both within the same file
- SPEC: `design/SPEC_slip39_v0_13_0.md` §2.3 (JSON), §2.5 (refusals), §2.6 (advisories), §5 (cross-impl verification), §6 (test-only env vars)
- Test fixtures with SHA-pin EXPECTED: `crates/mnemonic-toolkit/tests/cli_slip39_json.rs`
- Prior P2 plan: `design/PLAN_v0_13_0_p2.md` (this plan mirrors its style and §-numbering convention)
- Memory: `[[project-v0-13-0-slip39-in-flight]]` (post-P2.3 LOCK state); `[[feedback-opus-primary-review-agent]]` (R0 + R1 dispatch convention)

## §14 After P3 LOCKs

PE (separate session): release rollup
- `Cargo.toml` 0.13.0-dev → 0.13.0
- `CHANGELOG.md` v0.13.0 entry summarizing: SLIP-39 K-of-N share-splitting CLI (`slip39 split` + `combine`), full library impl, manual chapter, python-shamir-mnemonic interop verified.
- Tag `mnemonic-toolkit-v0.13.0`
- `git push origin master` (pushes the accumulated unpushed commits: 5 P2.2 + 3 P2.3 + 3 P3 + N PE release commits = 11 + N total; PE commits typically include the Cargo.toml bump + CHANGELOG entry, so N ≈ 1-2)
- GitHub release with manual PDF asset (CI workflow at `.github/workflows/manual.yml` auto-attaches)
- Adjacent `mnemonic-gui` working tree: commit + push the `slip39-gui-schema-flattening-companion` FOLLOWUP closure
