# Spec R0 — round 1 — `examples-pdf-un-ci-gated` (Examples.pdf modernize + gate)

- **Reviewer:** opus architect (adversarial R0 gate; verified live against source).
- **Spec under review:** `design/SPEC_examples_pdf_modernize_and_gate.md` (Fable, stamped `9fbd5685`).
- **Source verified at:** `origin/master` = **`a98559c6`** (advanced from the spec's `9fbd5685`).
  `git diff 9fbd5685..a98559c6` = **only the spec file itself** (+396 lines); `.examples-build/` is
  byte-identical → **every `gen.sh` / `install.sh` / workflow citation in the spec still holds at `a98559c6`**
  (no line decay). The plan-doc should re-stamp to `a98559c6`; no citation moves.

---

## VERDICT: GREEN — 0 Critical / 0 Important. Write the plan-doc.

The spec is architecturally sound, funds-safe, and — the load-bearing concern — its regen is a **reproducible
pure function of (repo tree, `mnemonic` binary, pinned Core)** with **no residual flap vector I could find after
stress-testing the two the spec under-flags** (clap `--help` wrap-width and `bundle` card wrap-width — both proven
width-insensitive below). Findings are Minor only. Several ratifications the spec raises as "open questions" are
genuine **USER decisions owed before P2** (chiefly Q2); they are inputs, not spec defects, and the spec correctly
surfaces + STOP-ledger-gates them.

**USER decisions owed before the relevant phase:**
- **Q2 (before P2):** the `crates/**` + `Cargo.lock` *leading* trigger → **every future toolkit release goes RED
  on examples.yml until it re-pins + regenerates in the same PR**, AND whether examples.yml becomes a **required**
  branch-protection check (blocking) vs advisory. This changes the release ritual for all future work. USER call.
  (My recommendation: include the leading triggers + make it required — it is the whole anti-rot point — and
  accept the bundled-regen release ritual.)
- **Q1 (before P2), softer:** whether to install pinned Bitcoin Core in a *docs* gate at all. My lean is the
  lighter alternative the spec did not enumerate (freeze §6.6's Core address as a labelled static capture,
  B-static-style) — architect-callable; only the show-only *downgrade* (S2) needs user sign-off.
- **Q5 (housekeeping, non-blocking):** flag the stale `.docbins/current` = 0.72.0 (and `gui-pinned` = 0.70.0) to
  the user to refresh/retire; the cycle itself already correctly bypasses `.docbins`.

---

## #2 (LOAD-BEARING) — Determinism / portability completeness: **COMPLETE. No residual flap vector found.**

I enumerated every nondeterminism vector and tested the two the spec under-flags. Ruling: the D1.3 fixes close
the set.

| Vector | Status after D1.3 | Evidence |
|---|---|---|
| Author-machine `REPO` in **displayed** cmd (`gen.sh:179-180`) | Fixed — single-quote so display shows literal `sh "$REPO/scripts/install.sh"`, `eval` still expands at run time | Only two `run`/`show` lines reference `$REPO`/absolute paths in display; sweep confirmed no others (`grep '$REPO\|/scratch\|/home\|$HOME' `). Internal `$BUILD`/`$WORK` (cp/cat/cd) never reach captured output (cwd = `$WORK`, seed files are relative). |
| `$HOME`/`$XDG_DATA_HOME`/`$CARGO_INSTALL_ROOT` in `install.sh --list`/`--dry-run` **output** | Fixed — `unset XDG_DATA_HOME CARGO_INSTALL_ROOT; export HOME=/home/user` | **Verified install.sh derives its path output from EXACTLY those three** (`MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/..."` :75; `install root: ${CARGO_INSTALL_ROOT:-$HOME/.cargo/bin}` :336; dry-run mkdir/man lines :324). No fourth env var leaks a path. **The REPO path does NOT leak into install.sh OUTPUT** (only into the display line, handled above). |
| `date:` YAML field | Fixed — static string, D1.1 keeps it hardcoded, **never `$(date)`** | `grep` for `$(date`/`date +` in gen.sh = **zero hits**. |
| `$USER` / `whoami` / `hostname` / `uname` / `sort` / `$PWD` | None present | full sweep = zero hits. |
| `mktemp` path leak | None — the one `mktemp -d` (`gen.sh:657`, bitcoind datadir) is used only as `-datadir` and is **not echoed** (the `printf` at :662 emits only `$RECV` + `$CORE`) | verified. |
| locale-sensitive tool output (28 `python3`/`jq`/`sed` calls) | Fixed — `LC_ALL=C LANG=C` | jq preserves input order; python/sed made C-locale. |
| **clap `--help` wrap-width** (re-captured block, `gen.sh:186`) — *spec under-flags this* | **Not sensitive** — proven | `mnemonic --help` is byte-identical (4299 B) under `COLUMNS=40` vs `200` vs unset, captured non-TTY. `bundle --help` also width-stable. **This retires the biggest un-addressed flap candidate.** |
| **`bundle` card wrap-width** (the 4 centerpiece blocks) — *spec under-flags* | **Not sensitive** — proven | `bundle` output byte-identical under `COLUMNS=40` vs `200`; md1 emitted once, space-separated, single-line per card (matches D2). |
| Bitcoin Core `deriveaddresses` address (§6.6) | Deterministic + pinned v27.0 | `deriveaddresses` is a pure descriptor→address function, **stable across Core versions**; pinned v27.0 in CI. See Minor-1 for the P1→CI handoff nit. |
| pandoc/xelatex PDF bytes | **Not gated** (disclosed §6.2/§7c) — only the `.md` golden is gated | correct: xelatex output is non-reproducible; PDF *buildability* re-proven each run. |

**Conclusion:** the recon under-flagged environment stability, but the spec's D1.3 **caught and closed it**, and
the two vectors D1.3 didn't explicitly name (`--help`/`bundle` width) are empirically inert in this binary. The
gate will **not** flap. This is the strongest part of the spec.

---

## B-static ruling (#3, D4): **SOUND.**

- **Mainline 0.75.0 still refuses depth-≥2 — verified live**, exit=2, message byte-identical to the recon's
  capture: `error: taproot tree depth ≥2 (≥3 leaves) is not yet restorable … the engraved card remains a faithful
  backup`. The "standing proof" does **not** invert; keeping `run 'mnemonic restore …'` (`gen.sh:749`) LIVE is
  legitimately a per-CI-run assertion that the cap still holds.
- The restructure is correct: the `show` build-steps block (`gen.sh:719-722`) is **already unconditional** (above
  the `if`); the live/gated mainline commands (`cat taproot-4leaf.desc` :735, `bundle … | jq > depth2.md1` :742,
  `cat depth2.md1` :743, mainline refusal :749) need only the shipped binary and are recon-verified stable; the
  **only** non-CI-reproducible piece — `mnemonic-depth2 restore` output (`gen.sh:756`) — becomes a labelled static
  heredoc. The two `--version` runs (:729-730) exist solely for the now-obsolete "same version string" framing and
  are correctly folded/dropped, which also removes the last stray "0.55.3" that could read as the doc's version.
- The exemption is **structural** (gen.sh never invokes `mnemonic-depth2` → zero CI carve-out logic; the frozen
  block is byte-covered by the whole-file diff like any prose). The proposed label is loud + honest. **Whole doc
  stays reproducible in CI with no depth2 binary.** Confirmed.

---

## Gate-A + trigger-scope ruling (#4): **DESIGN CORRECT; trigger scope is a USER decision (per-release redness).**

- **Precedents verified reusable.** `manual.yml`: `cargo build --bin mnemonic` (debug; "sufficient for
  transcript-replay byte-comparison"), tag-triggers-ignore-paths comment (:1-6), release-attach pattern (ensure
  release exists → `gh release upload … --clobber`, :147-168). `bitcoind-differential.yml`: pinned Core v27.0 =
  `BITCOIND_SHA256` + `actions/cache` keyed on the hash + `sha256sum -c` + extract + offline `-connect=0 -listen=0`
  spin-up (:57-105). Both transplant cleanly.
- **CI installs NO sibling CLIs** — confirmed gen.sh invokes only `mnemonic` (+ python3/jq/sed/bitcoind); zero
  `md`/`ms`/`mk` direct calls. examples.yml is simpler than manual.yml (no `cargo install --tag` sibling steps).
- **Golden = tracked `.examples-build/Examples.md`**, `git diff --exit-code` scoped to that path. Correct — the
  scoping avoids false positives from the still-ignored `preamble.tex`/`work/`. The `.gitignore` flip (remove
  line 9) + `git add` in the same atomic P2 commit makes the gate born-green. Sound.
- **CI builds from source, not `.docbins`** — verified: step 4 `cargo build --bin mnemonic` + `EXAMPLES_BIN_DIR=
  target/debug`; the source tree's `Cargo.toml` = 0.75.0 → the built binary reports `mnemonic 0.75.0` → satisfies
  the hardcoded `gen.sh:22` pin. Header-version and gate move together on each intentional regen (exactly recon
  item 6's "fixed-pin-in-header + gate-against-current-source").
- **Trigger scope — USER FLAG (Q2).** Including `crates/**` + `Cargo.lock` + `scripts/install.sh` makes gate A a
  **leading** indicator (a PR that changes any captured output goes red in that PR). The unavoidable consequence:
  **every future release that bumps the crate version turns examples.yml red** (built binary reports the new
  version → `gen.sh:22` FATAL; independently the install.sh self-pin advances → captured `--list`/`--dry-run`
  drift) **until the same PR re-pins gen.sh + regenerates the golden.** That is the loud-rot discipline the
  FOLLOWUP asks for AND the user's standing directive (`feedback_docs_cli_output_binary_identical`), but it
  imposes a recurring per-release cost on ALL future toolkit work and effectively adds an Examples-regen line to
  the release-version-sites ritual. **This belongs to the user.** Additionally the spec does not state whether
  examples.yml becomes a **required** status check — required = release PRs are *blocked* until the regen is
  bundled; advisory = red is a loud nudge. The plan must encode this; it too is a USER call. Recommendation:
  include the leading triggers, keep `Cargo.lock` (a clap/dep bump can change `--help`/error text with no
  `crates/**` edit), make the check required, and accept the bundled-regen ritual.

---

## Funds-stability confirmation (#5): **CONFIRMED — display-only change; no derivation drift.**

Re-verified beyond the recon: the depth-2 refusal is byte-identical at 0.75.0 (above), and the `bundle`
centerpiece delta is purely presentational — each card printed **once, space-separated, grouped** (`md1fg dxlpq
pqpm6 …`), vs 0.55.3's twice/unbroken + hyphenated md1. All funds-critical surface (fingerprints, xpubs,
descriptors, `#4wup4at0` checksum, `bc1q…`/`bc1p…` addresses, BSMS, refusal messages) is seed-deterministic and
byte-stable 0.55.3→0.75.0 per the recon's per-command diff. The regen risk is **presentation/narrative, not
derivation**. The spec's **S1 STOP-ledger** (any delta outside §3's enumeration, especially any
address/xpub/descriptor/checksum/fingerprint change, halts + escalates) is the correct backstop, and P3's
whole-diff review checks the actual diff against the §3 enumeration. Adequate.

**Pin-site completeness (#1):** grep `0\.55\.3` across `.examples-build/` = **10 literal occurrences** (`gen.sh`
:3, :22, :87, :104, :531, :696, :709, :716, :724 + `.gitignore:5`) **+ the coupled date at `gen.sh:88`**. **All 11
are addressed** — the spec's "8" is a grouping (D1.1 items 1-7 + item 8 bundling the Appendix-B framing
:709/:716-717/:724; :531 handled in the historical-attribution paragraph + the D3 table; :88 is item 4; :666 is
`v0.49.1`, correctly left as history). **No missed occurrence → no mixed-tier regen risk.** COMPLETE.

---

## Rulings on Q1–Q5

- **Q1 (pinned Core in the docs gate) — ARCHITECT CALL; I recommend the lighter option the spec omits.** The spec
  frames it as keep-live (a) vs show-fallback (c) and rules (a). There is a cleaner **(b): freeze §6.6's Core
  address as a labelled static capture** (B-static-style), which the spec does not consider. Rationale: the
  `deriveaddresses` address is a deterministic function of the *fixed* descriptor AND is **already redundant** with
  what `restore` reports ("Byte-for-byte the same bc1p…") — it is a confirmation of a fixed mathematical fact, not
  a rot vector. Freezing it (honest "recorded from Bitcoin Core v27.0" label) keeps the external-oracle content,
  needs **zero bitcoind in CI**, and is philosophically identical to the Appendix-B B-static the spec already
  adopts — the most "minimal gate A" outcome the ratified scope asks for. **(a) keep-live is acceptable** given the
  in-repo precedent (bounded cost, deterministic), but it is the single largest CI-complexity add and slightly
  strains "minimal." **Only (c) show-only is a content downgrade needing user sign-off (already S2).** Ruling: let
  the plan pick (b) or (a); default to (b). Not blocking either way.

- **Q2 (`crates/**` + `Cargo.lock` leading trigger + required-vs-advisory check) — USER DECISION, owed before P2.**
  Recommendation above (include + required + bundled-regen ritual). `Cargo.lock` earns its place (dep-bump-only
  output changes). This is the one decision that changes the project's release process, so it is squarely the
  user's.

- **Q3 (ship the frozen 0.55.3-era depth2 transcript, labelled, vs drop) — ARCHITECT CALL; keep-labelled.** The
  proposed label is more than sufficient (states: static, dated, from the experimental build, not regenerated, not
  CI-reproducible, everything-else-is-live). The frozen block's only version-dependent line (`mnemonic-depth2
  --version`) is dropped under D4, so the staleness is cosmetic and clearly fenced as an experimental aside.
  Keeping it preserves Appendix B's teaching payload with full honesty and a structural (carve-out-free)
  exemption. Keep it. (User may override editorially; not blocking.)

- **Q4 (keep committed `docs/Examples.pdf` + `examples-v*` attach) — ARCHITECT CALL; keep-committed + attach.**
  Well-justified: 215 KB text-only, no raster figures — the gui_example attach-only rationale (~32 MiB screenshot
  corpus) does not apply; users already find it in-tree. The disclosed gap (committed PDF not byte-gated → can lag
  the golden; mitigated by the §8 ritual + PDF-in-triggers) is acceptable. Low stakes; FYI to the user only.

- **Q5 (`.docbins/current` = 0.72.0) — cycle already resolves it correctly; housekeeping FLAG only.** Verified:
  `.docbins/current` = 0.72.0, `.docbins/gui-pinned` = 0.70.0, `~/.cargo/bin/mnemonic` = 0.75.0, crate `Cargo.toml`
  = 0.75.0. The spec correctly pins 0.75.0, builds from source in CI, and bypasses `.docbins`. The only action is
  to **tell the user the stale tiers exist** (refresh or retire) so no future doc cycle regens against 0.72.0 by
  accident. Out of this cycle's critical path.

---

## Honesty / scope / phasing audit (#7): **ACCURATE.**

- §7 correctly states gate A catches **regen-drift** (output can no longer silently drift; the three-era patchwork
  "could not have survived this gate") but **not** the narration-vs-truth blind spot — because gen.sh emits prose
  heredocs verbatim and the diff only asserts golden==regen, never prose==adjacent-output-truth. Verified against
  the mechanism (`run`/`show` + `cat <<'MD'` heredocs). The gate-B deferral (per-command `.cmd`/`.out` +
  `include-transcript.lua`, the sibling-books model) is the correct future close and is honestly scoped as v1.1+.
  Blind spots (b) sibling-CLI drift and (d) coverage gaps are correctly named.
- **NO-BUMP / toolkit-only / no-locksteps: CONFIRMED.** Zero `crates/**` src changes; no flag changes → GUI
  `schema_mirror` and the manual flag-coverage lint do not fire (gen.sh only *re-captures* `--help`; `gen-man` +
  `word-card` already shipped). Both FOLLOWUP entries (`design/FOLLOWUPS.md` + `docs/manual-gui/FOLLOWUPS.md`)
  exist, are `open / catalog / v1.1+`, and are ready to flip RESOLVED in the shipping commit (minor line-citation
  drift of a few lines vs the spec's `:155`/`:175` — expected, non-blocking).
- **Phasing + STOP ledger: SOUND.** P0(spec+R0) → P1(modernize + the environment-independence proofs as the
  "tests": twice-byte-identical, HOME-varied byte-identical, depth2-present-vs-absent byte-identical, diff ⊆ §3
  classes, pandoc-clean) → P2(gate + golden + **negative proof**: mutate one golden byte → red) → P3(refresh PDF,
  flip FOLLOWUPs, **mandatory post-impl whole-diff review**, tag, verify asset). S1/S2/S3 are the right escalation
  points. The P2 "born green on the PR" + negative-proof gives a live author==runner cross-machine proof before
  the gate governs master, which backstops any unforeseen determinism vector.

---

## Findings by severity

**Critical:** none.
**Important:** none.

**Minor:**
- **M1 (determinism belt-and-suspenders).** The §9 P1 phase-gate proves HOME-varied byte-identity but does **not**
  require generating the committed golden under the **same pinned Core v27.0** CI uses. `deriveaddresses` is
  Core-version-stable so a mismatch is essentially impossible, and P2's born-green PR would catch any real
  divergence — but to make the P1→CI handoff airtight, the plan should either (a) generate the §6.6 golden line
  under pinned Core v27.0, or (b) adopt Q1-option-(b) (static-labelled Core capture), which moots it entirely.
- **M2 (trigger completeness).** Root `Cargo.toml` (the `[patch.crates-io]` miniscript-fork rev) is not in the
  trigger paths. A `[patch]` rev change updates `Cargo.lock` too, so the `Cargo.lock` trigger already covers it;
  adding root `Cargo.toml` to triggers is a cheap completeness nicety, not required.
- **M3 (spec SHA re-stamp).** Re-stamp the plan-doc to `a98559c6`; no citations move (the only intervening change
  was adding the spec file itself).
- **M4 (required-check gap).** §6.1/§8 do not state whether examples.yml becomes a required branch-protection
  check. Fold into the Q2 user decision and have the plan encode the answer (this determines whether release PRs
  are blocked vs merely reddened).

---

## Bottom line

**GREEN, 0C/0I — proceed to the plan-doc.** The determinism story (the #1 risk of a flapping gate) is airtight and
empirically stress-tested; B-static is verified against a live mainline refusal; the funds surface is display-only;
gate A's precedents transplant cleanly. Before P2, obtain the user's Q2 decision (leading triggers + required-check
+ bundled-regen ritual — the one process-changing call) and surface the Q1 lean (prefer static-labelled Core over
bitcoind-in-CI) and the Q5 `.docbins` housekeeping flag. Q3/Q4 are architect calls already ruled here. Per
CLAUDE.md, re-dispatch this review after folding to confirm the fold introduces no drift.
