# IMPLEMENTATION PLAN — `Examples.pdf` modernize + CI gate (cycle `examples-pdf-un-ci-gated`)

- **Status:** DRAFT for mandatory plan-doc R0 (0C/0I required before ANY implementation — CLAUDE.md hard gate).
- **Author:** single-author draft (session 2026-07-05), building on the R0-GREEN spec + spec-R0.
- **Source SHA:** `origin/master` = **`37472e25`** (== local HEAD; working tree clean of tracked drift). Every
  `gen.sh` / `install.sh` / workflow / baseline-`Examples.md` citation below was **re-grepped live at this SHA**.
  `git diff a98559c6..37472e25` = **only `design/agent-reports/examples-pdf-spec-r0-round-1.md`** (+217 lines) —
  i.e. NOTHING under `.examples-build/`, `scripts/`, `.github/` moved since the spec-R0's verified SHA. **No
  citation decay; all spec/R0 line numbers still hold verbatim.** (Folds M3.)

## Inputs (all R0-GREEN / user-locked — NOT re-opened here)

- **Spec:** `design/SPEC_examples_pdf_modernize_and_gate.md` (Fable, R0-GREEN, stamped `9fbd5685`).
- **Spec R0:** `design/agent-reports/examples-pdf-spec-r0-round-1.md` (VERDICT GREEN, 0C/0I; rulings BINDING).
- **Recon:** `cycle-prep-recon-examples-pdf-modernize-and-gate.md`.
- **LOCKED user decisions (do NOT re-litigate):**
  1. **Appendix B EXEMPT via B-static** — delete the `if command -v mnemonic-depth2` runtime branch; keep the
     mainline commands LIVE (incl. the depth-≥2 **refusal** as a standing per-run assertion); freeze ONLY the
     depth2 `restore` reconstruction transcript as a labelled static heredoc; the two `--version` runs fold/drop.
  2. **Gate = A, WIDE + REQUIRED.** Triggers `crates/**` + `Cargo.lock` + `Cargo.toml` (root) + `scripts/install.sh`
     (+ `.examples-build/**` + `docs/Examples.pdf` + the workflow file); **required** branch-protection status
     check. Consequence (accepted): every future toolkit release must re-pin + regenerate in the SAME PR or go red.
  3. **Q1 → (b):** freeze §6.6's Core `deriveaddresses` address as a labelled static capture — **zero bitcoind in
     CI** (B-static-parallel to Appendix B). This SUPERSEDES the spec's D1.4 "keep-live + pinned Core in CI" (a).
  4. **Q3:** keep the labelled frozen depth2 transcript (B-static item 3).
  5. **Q4:** keep the committed `docs/Examples.pdf` + `examples-v*` tag-attach.
  6. **Pin `0.55.3 → 0.75.0`, FIXED, source-built in CI** (`cargo build --bin mnemonic` → `target/debug`; NOT
     `.docbins`, which are stale: `current`=0.72.0, `gui-pinned`=0.70.0 — Q5).

## Fold of the 4 spec-R0 Minors

- **M1 (P1→CI golden-under-Core handoff nit): MOOTED by Q1-(b).** With §6.6 frozen as a static capture and **no
  bitcoind in CI**, there is no "generate the §6.6 golden under the same pinned Core v27.0" concern — the block is
  literal heredoc text, byte-covered by the whole-file diff like any prose. Nothing to reconcile. (The one residual
  obligation — that the frozen §6.6 descriptor+address is a *faithful* capture of what live 0.75.0 emits — is a P1
  determinism check + a P3 whole-diff item, S1-backstopped; see P1 gate (v) and P3.)
- **M2 (trigger completeness): FOLDED.** Root `Cargo.toml` (the `[patch.crates-io]` miniscript-fork rev) is added
  to `examples.yml` triggers as belt-and-suspenders alongside `Cargo.lock` (a `[patch]` rev bump touches both).
- **M3 (SHA re-stamp): FOLDED.** Plan stamped `37472e25`; verified no citation moved (diff above).
- **M4 (required-check gap): FOLDED / RESOLVED by locked-decision 2.** `examples.yml` becomes a **REQUIRED**
  branch-protection status check on toolkit `master` (mechanics in P2, mirroring the GUI `snapshots` precedent:
  required context + `enforce_admins` OFF so admin direct-FF release pushes still function).

---

## GOTCHAS / carry-forward (read before touching anything)

1. **Determinism is THE load-bearing risk.** A byte-diff gate flaps if gen.sh output depends on anything but
   (repo tree, `mnemonic` binary). The spec-R0 stress-tested the two under-flagged vectors (`--help` wrap-width,
   `bundle` card wrap-width) and proved both **width-inert** in this binary; D1.3 closes the path/HOME/locale
   leaks. The P1 gate **re-proves** determinism empirically (double-regen byte-identical + cross-env), and P2's
   born-green PR gives an author≠runner cross-machine proof before the gate governs master. **Do not weaken any
   determinism fix; do not add `$(date)`, `$USER`, `hostname`, `$PWD`, or an unpinned locale anywhere.**
2. **Build ordering.** gen.sh is a single hand-run script (`bash gen.sh > Examples.md`) followed by the pandoc
   line in gen.sh's header (`gen.sh:9-11`). It writes `preamble.tex` itself (`gen.sh:45-64`). No `make html`-class
   pre-build; the only ordering constraint is *binary must exist before regen* (CI step order handles it).
3. **No RUSTUP_TOOLCHAIN override needed.** Unlike the GUI legs, we build only the root-workspace `mnemonic`
   binary; `rust-toolchain.toml` pins 1.85 and `gen.sh:167` requires rustc ≥1.85 — the default toolchain suffices.
4. **No background CI-watchers** (repeated silent-death gotcha across the screenshot/UI cycles). Poll git/`gh`
   ground truth on every completion; resume agents via SendMessage from transcript, never trust a notification.
5. **Opus per-phase R0** (CLAUDE.md hard gate): every phase — plus this plan — passes an opus R0 to 0C/0I BEFORE
   proceeding; fold → **persist the review verbatim to `design/agent-reports/`** → re-dispatch until GREEN. No
   code before the plan-doc is GREEN. Re-dispatch after each fold (folds introduce drift).
6. **Commit trailer = current session model + live session URL** (read from the harness prompt, do not hardcode).
7. **Q5 housekeeping (non-blocking):** flag the stale `.docbins` tiers (`current`=0.72.0, `gui-pinned`=0.70.0) to
   the user to refresh-or-retire, so no future doc cycle regens against 0.72.0 by accident. This cycle already
   bypasses `.docbins` (CI source-builds; local uses `~/.cargo/bin` 0.75.0). Not on the critical path.
8. **Stage paths explicitly** (no `git add -A`); the `.gitignore` flip + `git add` of the new golden are a single
   atomic commit so the gate is **born green** (P2).

## Scope / bump / locksteps

- **NO-BUMP, toolkit-only, docs+CI-only.** Zero `crates/**` source changes; no flag add/remove/rename → GUI
  `schema_mirror` and the manual flag-coverage lint **do not fire** (gen.sh only *re-captures* `--help`; `gen-man`
  + `word-card` already shipped). No sibling-codec or GUI changes; no cross-repo locksteps.
- **New tag family:** `examples-v1.0.0` at ship (attach-only trigger; the toolkit crate is NOT tagged/bumped).
- **FOLLOWUP flips (shipping commit):** `design/FOLLOWUPS.md::examples-pdf-un-ci-gated` → RESOLVED + companion
  `docs/manual-gui/FOLLOWUPS.md::examples-pdf-un-ci-gated` in lockstep; add the gate-B deferral note (Honesty §a).
- **Merge mechanics:** land via **PR** (not direct-FF) — the PR itself exercises `examples.yml` pre-merge (both the
  workflow-file path AND the `crates/**`-adjacent paths fire), giving a live-green proof before the gate governs
  master. Ship = squash/merge PR → tag `examples-v1.0.0` → verify the release asset attached.

---

## Phase list

- **P0 — spec + spec-R0 + THIS plan + plan-R0.** DONE modulo the plan-R0 loop (this document → opus R0 → fold →
  persist verbatim → re-dispatch until 0C/0I). **No code before the plan is GREEN.**
- **P1 — gen.sh modernization** (pin + coupled prose + determinism hardening + the two B-static freezes).
- **P2 — gate + golden** (`examples.yml`, `.gitignore` flip + `git add` the golden atomically, required-check).
- **P3 — regen + ship** (rebuild `docs/Examples.pdf`, whole-diff opus review, flip FOLLOWUPs, tag + verify attach).

Single implementer per phase in a worktree (NOT parallel re-implementations); determinism proofs are the "tests"
(TDD-style). Every phase R0-gated; reviews persist to `design/agent-reports/<cycle>-phase-N-<round>-review.md`.

---

## P1 — pin + prose + determinism hardening

### P1 deliverables

Modernize `.examples-build/gen.sh` so its output is a **pure function of (repo tree, `mnemonic 0.75.0` binary)**,
with every version string / coupled-narration line rewritten in lockstep, and Appendix B + §6.6 restructured to
B-static. Regenerate the golden `.examples-build/Examples.md` (committed in P2). **Zero `crates/**` changes.**

### P1.a — The 11 pin sites (`0.55.3 → 0.75.0`, + the date)

Verified live at `37472e25` (`grep -n '0\.55\.3'` in `.examples-build/`): 9 literal `gen.sh` occurrences +
`.gitignore:5` (= 10 version-string sites) + the coupled `date:` at `gen.sh:88` = **11 edit sites**.

| # | Site | Current | Action |
|---|---|---|---|
| 1 | `gen.sh:22` | `[ "$VER" = "mnemonic 0.55.3" ] \|\| … exit 1` | **The enforcement point** → `"mnemonic 0.75.0"`. Built 0.75.0 binary passes; any other version FATALs. |
| 2 | `gen.sh:3` | header comment "…v0.55.3 binary" | → v0.75.0 |
| 3 | `gen.sh:87` | `subtitle: "mnemonic-toolkit v0.55.3 …"` | → v0.75.0 |
| 4 | `gen.sh:88` | `date: "2026-06-15"` | → **the regen date** (static literal; NEVER `$(date)` — determinism). Set to the P1 regen day; P3 re-confirms if it slips a day. |
| 5 | `gen.sh:104` | "`mnemonic` **v0.55.3** on Linux" intro | → v0.75.0 |
| 6 | `gen.sh:696` | Appendix A "Generated with `mnemonic` v0.55.3 on Linux" | → v0.75.0 |
| 7 | `.gitignore:5` | "Run with mnemonic 0.55.3 (+ mnemonic-depth2 …)" | → v0.75.0 pin **and** reword the "Everything else here is a build artifact" framing (lines 1–6): `Examples.md` becomes a **tracked CI golden**, not an ignored artifact (P2.b). |
| 8 | `gen.sh:531` | "the v0.55.3 non-NUMS feature" | Reword (keep history, kill version-ambiguity): "the non-NUMS internal-key feature (shipped in v0.55.3)". |
| 9 | `gen.sh:709` | "shipped `mnemonic` v0.55.3 documented everywhere else …" | → v0.75.0 (mainline still refuses depth-≥2 at 0.75.0 — recon+R0 verified). |
| 10 | `gen.sh:716-717` | "both report the same version string `0.55.3`, so the command name is the only thing that tells them apart" | **Obsolete under B-static** (D4): rewrite — the twin was built from the 0.55.3-era experimental branch; the static capture below is from that build. Removes the last stray "0.55.3" that could read as the doc's version. |
| 11 | `gen.sh:724` | `if command -v mnemonic-depth2 … = "mnemonic 0.55.3"` | **Deleted** wholesale by the B-static restructure (P1.d) — the runtime branch goes away. |

`gen.sh:666` ("which v0.49.1 routes *around* the codec") stays **unchanged** — clearly historical attribution, no
ambiguity. (R0 §Pin-site-completeness: all 11 addressed, no missed occurrence, no mixed-tier regen risk.)

### P1.b — Determinism / portability hardening (D1.2–D1.3; required for a byte-diff gate)

1. **Script-relative `REPO`** — `gen.sh:15`:
   `REPO=/scratch/code/shibboleth/mnemonic-toolkit` → `REPO="${REPO:-$(cd "$(dirname "$0")/.." && pwd)}"`
   (works from any checkout; CI's `$GITHUB_WORKSPACE` clone resolves correctly).
2. **Binary override hook** — `gen.sh:18`:
   `export PATH="$HOME/.cargo/bin:$PATH"` → `export PATH="${EXAMPLES_BIN_DIR:+$EXAMPLES_BIN_DIR:}$HOME/.cargo/bin:$PATH"`.
   CI sets `EXAMPLES_BIN_DIR="$GITHUB_WORKSPACE/target/debug"` so the source-built binary wins over any stale
   cargo-installed one. Displayed commands stay bare `mnemonic` (display-faithful, execution-pinned).
3. **Single-quote the two `install.sh` `run` calls** — `gen.sh:179-180`. Today they are double-quoted, so the
   **display** leaks the absolute `REPO` (baseline `Examples.md:98,109` show `/scratch/…/scripts/install.sh`):
   - `gen.sh:179` `run "sh '$REPO/scripts/install.sh' --list"` → `run 'sh "$REPO/scripts/install.sh" --list'`
   - `gen.sh:180` `run "sh '$REPO/scripts/install.sh' --no-gui --dry-run"` → `run 'sh "$REPO/scripts/install.sh" --no-gui --dry-run'`
   `run` prints `$1` **verbatim** then `eval`s it (`gen.sh:78`), so the display shows the literal
   `sh "$REPO/scripts/install.sh" …` while execution still expands `$REPO`. Add a half-line of prose (P1.c table):
   "`$REPO` = your clone root".
4. **Pin the output-visible environment** — insert AFTER the PATH line (order matters: PATH is computed from the
   real `$HOME` first, then we scrub), before the `mnemonic --version` gate at `gen.sh:21`:
   `unset XDG_DATA_HOME CARGO_INSTALL_ROOT; export HOME=/home/user; export LC_ALL=C LANG=C TZ=UTC`.
   `install.sh` derives its path output from EXACTLY those three (R0-verified: `MAN_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/…"`
   `install.sh:75`; `install root: ${CARGO_INSTALL_ROOT:-$HOME/.cargo/bin}` `install.sh:336`; dry-run mkdir/gen-man
   at `install.sh:322,324,327`) → `--dry-run` then deterministically prints `/home/user/…` on every machine. `LC_ALL=C`
   pins the 28 `python3`/`jq`/`sed` outputs. (Residual "some tool needs the real `$HOME`" risk is retired by the P1
   varied-HOME determinism proof — gate (ii).)
5. **Strict preflight (always on)** — add near the top of `gen.sh` (after the version gate): FATAL up-front with a
   clear message if `jq` or `python3` is missing. Today a missing `jq` would silently capture `command not found`
   *into the document body* via `run`'s `eval … 2>&1` (`gen.sh:78`). **Note (Q1-b): `bitcoind`/`bitcoin-cli` are NO
   LONGER required** — §6.6 is now a static capture (P1.e), so drop them from the preflight requirement set (the
   spec's D1.3.4 listed them under the keep-live design; superseded).

TTY note (carry): `run` captures via `$( … 2>&1)` — never a TTY, identical local vs CI — so `is_terminal`-gated
text cannot diverge.

### P1.c — Coupled prose rewrites (the narration-blind-spot class; SAME commit as the pin bump)

| Site | Today | Rewrite |
|---|---|---|
| `gen.sh:209-211` | "Each card is printed twice: once unbroken, once grouped into 5-character blocks (`ms10e ntrsq qqqqq …`) -- the grouped form is what you punch or engrave." | "Each card is printed once, grouped into 5-character blocks (`ms10e ntrsq qqqqq …`) -- exactly the form you punch or engrave." **The primary time-bomb** — flatly false at ≥ v0.56.0. (`ms10e ntrsq` stays true at 0.75.0; verified. Only "twice/unbroken" prose site — `grep -niE 'twice\|unbroken\|grouped\|hyphen'` = exactly `gen.sh:209-210`.) |
| `gen.sh:179-181` install prose | — | + "`$REPO` = your clone root" half-line (P1.b.3). |
| `.gitignore:1-6` | "…Everything else here is a build artifact…; Run with mnemonic 0.55.3…" | New pin (0.75.0) + `Examples.md` now tracked as the CI golden (P2.b). |
| Appendix B / §6.6 framing | (0.55.3-era, `if`-conditional) | Per P1.d / P1.e restructures. |

**Stays true, no change** (checked at 0.75.0): §5.4/§6.1 depth-≥2 refusal narration (`gen.sh:455-459` +
`Examples.md` body), `note: stdout is watch-only`, `CONFIRM/UNVERIFIED` descriptions, the ms1/mk1/md1 card table,
the secret-on-argv narration (§3.4), `gen.sh:666` v0.49.1 attribution.

### P1.d — Appendix B → B-static (D4; LOCKED decision 1 + Q3)

Delete the `if command -v mnemonic-depth2 … then … else … fi` runtime branch (`gen.sh:724` … `else` at `:764` …
`fi` at `:774`) and emit Appendix B **unconditionally**:

- **Keep as-is** (already unconditional, above the old `if`): the framing prose (`gen.sh:701-717`, reworded per
  P1.a items 9–10) + the `show` build-steps block (`gen.sh:719-722` — how a reader builds the twin themselves).
- **Keep LIVE / gated** (need only the shipped binary; recon-verified stable): `run 'cat taproot-4leaf.desc'`
  (`gen.sh:735`), the `bundle --json | jq … > depth2.md1` (`gen.sh:742`) + `run 'cat depth2.md1'` (`gen.sh:743`),
  and the mainline **refusal** `run 'mnemonic restore …'` (`gen.sh:749`). **Keeping the refusal LIVE is a feature:**
  the gate then proves on EVERY CI run that mainline still refuses depth-≥2 — if a future miniscript bump silently
  lifts the cap, `examples.yml` goes red and forces a doc decision. (Standing per-run assertion.)
- **Freeze as ONE static committed heredoc:** the `mnemonic-depth2 restore` reconstruction output
  (`gen.sh:756`; the transcript at baseline `Examples.md:1596-1617` — EXPERIMENTAL advisory + descriptor
  `tr([73c5da0a/84'/0'/4']…{{…}})#5trrgdg0` + `first recv: bc1p6yc7kzttzsafprr6hwsaefuyqxvee4j…` + 12-cosigner
  block + `UNVERIFIED:` + `note: stdout is watch-only`), replaced with a `cat <<'MD'` literal heredoc carrying an
  explicit in-document label, e.g.:
  > *(STATIC CAPTURE — recorded 2026-06-15 from the experimental `mnemonic-depth2` build at v0.55.3. This block is
  > NOT regenerated by `gen.sh` and is not reproducible with a released binary or in CI; build the branch as shown
  > above to reproduce it. Everything else in this document is live-captured and CI-gated.)*
- **Fold/drop the two `--version` runs** (`gen.sh:729-730`) + their intro heredoc (`gen.sh:725-727`): they exist
  solely for the now-obsolete "same version string" framing; dropping them removes the last stray "0.55.3".

Result: the exemption is **structural** — gen.sh never invokes `mnemonic-depth2`; the frozen block is literal text,
byte-covered by the whole-file diff like any prose (cannot drift except by intentional edit); the CI differ needs
**zero skip/carve-out logic** (D5.3 / gate §6.3).

### P1.e — §6.6 Core `deriveaddresses` → B-static (Q1-(b); SUPERSEDES spec D1.4)

Under the locked Q1-(b) decision, freeze §6.6's Core capture the SAME way (zero bitcoind in CI):

- **Delete** the `if command -v bitcoind … then … else … fi` block (`gen.sh:656-679`, incl. the `mktemp -d`
  datadir, the offline `bitcoind` spin-up, the `else` `show`-fallback). **Also drop the now-unused live
  `RECV=$(mnemonic restore … --format bitcoin-core …)` assignment (`gen.sh:649`)** — verified `$RECV` is referenced
  ONLY inside the deleted block (`gen.sh:660,662,673`), nowhere else.
- **Keep** the lead-in prose (`gen.sh:651-655`, "`restore` reports a `bc1p…`. Confirm it against Bitcoin Core's
  **independent C++** derivation: `deriveaddresses` on the receive (`.../0/*`) descriptor …") — stays true.
- **Replace** the deleted block with a single labelled static heredoc emitting the exact captured pair
  (baseline `Examples.md:1474-1475`): the `$ bitcoin-cli -chain=main deriveaddresses "tr(50929b74…sortedmulti_a(2,
  …))#mk8vdqmt" "[0,0]"` command line + `["bc1p550zvnachy40z6hh8llka93mkm0c3635samp264ck6rfd0dcdc8s00n8c8"]`,
  followed by the existing "Byte-for-byte the same `bc1p…` that `restore` reported…" prose (`gen.sh:664-671`), with
  a label, e.g.:
  > *(STATIC CAPTURE — recorded from Bitcoin Core v27.0. `deriveaddresses` is a deterministic descriptor→address
  > function of the fixed descriptor above; this line is NOT regenerated by `gen.sh` and needs no node in CI.)*
- **Rationale (R0 Q1 ruling):** the `deriveaddresses` address is a deterministic function of the *fixed* descriptor
  AND is already redundant with what `restore` reports ("Byte-for-byte the same `bc1p…`") — a confirmation of a
  fixed mathematical fact, not a rot vector. Freezing keeps the external-oracle content, needs zero bitcoind, and
  is philosophically identical to the Appendix-B B-static. **This retires the spec's D1.4 pinned-Core-in-CI plan
  and its S2 STOP** (both mooted — see STOP ledger).
- **Faithfulness obligation (M1 residual):** at P1, verify the frozen descriptor+address equals what a live 0.75.0
  `mnemonic restore … --format bitcoin-core` + a local `bitcoin-cli deriveaddresses` produce (author has both
  locally), so the freeze is a TRUE current capture, not a stale one. Re-checked in the P3 whole-diff review;
  S1-backstopped (a mismatch = funds-surface delta = STOP).

### P1 GATE (phase gate — determinism proofs are the "tests")

Run in the worktree, all must pass before P2:

- **(i) Double-regen byte-identical:** `bash .examples-build/gen.sh > /tmp/a.md; bash … > /tmp/b.md; diff /tmp/a.md /tmp/b.md` → empty.
- **(ii) Cross-env byte-identical:** regen under two different **real** `$HOME` values (e.g. `HOME=/x` vs `HOME=/y`,
  each with `~/.cargo/bin/mnemonic` reachable via `EXAMPLES_BIN_DIR`) → byte-identical. Proves the D1.3 env-scrub
  fully decouples output from the author machine.
- **(iii) depth2-absent-vs-present byte-identical:** regen with `mnemonic-depth2` on PATH and with it removed →
  byte-identical (the recon author *had* it installed; CI never will — this proves B-static removed the
  author-machine dependence).
- **(iv) Diff ⊆ enumerated classes:** `diff` the fresh 0.75.0 golden vs the Jun-26 baseline `.examples-build/Examples.md`;
  every hunk must fall in the spec-§3 enumeration ONLY — (1) the 4 `bundle` card blocks (twice→once, md1 hyphen→space,
  `gen.sh:206,288,308,443`); (2) `mnemonic --version` `0.55.3→0.75.0` (`gen.sh:185`); (3) `mnemonic --help` +2 rows
  `gen-man`,`word-card` (`gen.sh:186`); (4) `install.sh --list` toolkit line `-v0.73.3→-v0.75.0` (`gen.sh:179`);
  (5) `install.sh --dry-run` tag lines + path normalization (`/home/bcg/…→/home/user/…`, `/scratch/…→"$REPO/…"`,
  `gen.sh:180`); (6) Appendix-B + §6.6 restructure (framing + the two static blocks); (7) the D3 prose edits.
  **Any unenumerated hunk — especially any address/xpub/descriptor/checksum/fingerprint change — is a STOP (S1).**
- **(v) §6.6 freeze faithfulness:** the frozen descriptor+address == live-0.75.0 `restore --format bitcoin-core`
  → `deriveaddresses` (P1.e obligation).
- **(vi) pandoc renders clean:** the `gen.sh:9-11` line (cwd `.examples-build`) produces `Examples.pdf` with no
  LaTeX error; file exists, non-trivial size.

### P1 opus R0

Independent opus R0 over the P1 diff (gen.sh + `.gitignore`) + the six gate proofs: confirms (a) all 11 pin sites
edited + the coupled prose rewritten with no missed narration; (b) determinism fixes complete, no new nondeterminism
introduced; (c) B-static restructures (Appendix B + §6.6) delete the runtime branches cleanly with faithful frozen
captures + loud labels; (d) diff ⊆ §3 classes. Full `.examples-build` regen must be run in-review (not a targeted
subset). Fold → persist verbatim → re-dispatch until 0C/0I.

---

## P2 — gate + golden

### P2.a — `.github/workflows/examples.yml` (gate A, WIDE + REQUIRED)

> **⚠ SUPERSEDED by the OQ1 architect ruling — `design/agent-reports/examples-pdf-branch-protection-ruling.md` §7 is the AUTHORITATIVE config.** The YAML below had a latent trap (a **path-filtered `pull_request`** trigger + a **required** check = docs-only PRs wedge forever at "Expected"). The ruling resolves it: **(1)** DROP the `pull_request` `paths:` filter so the `examples` job ALWAYS runs + reports on every PR (no wedge), and gate the heavy steps (build/regen/diff) behind a hand-rolled, fail-safe `git diff --name-only` **guard** (PR-event only; docs-only PR → guard false → seconds-long green no-op; code PR → full gate). Keep the `push:` path filter as-is. **(2)** Governance = minimal, byte-identical to the live GUI `snapshots` protection: `required_status_checks={strict:false, contexts:["examples"]}`, `enforce_admins:false` (admin direct-FF releases still work); do NOT enroll `rust.yml`. **(3)** The `push: [master]` run is LOAD-BEARING, not redundant — it's the sole gate for the toolkit's dominant direct-to-master pushes (which bypass PR checks). Implement §7's config block verbatim; the YAML below is retained only for the non-trigger scaffolding (build/regen/diff/attach steps).

One job, modeled on `manual.yml` (debug binary build `manual.yml:92-96`; tag release-attach `manual.yml:147-168`;
tag-triggers-ignore-paths comment `manual.yml:3-6`). **NO bitcoind** (Q1-(b) froze §6.6).

```
name: examples
# `paths` filters apply only to branch pushes / PRs; tag pushes matching
# `examples-v*` always trigger (release-asset upload must run on every tag).
on:
  push:
    branches: [master, main]
    paths:
      - '.examples-build/**'
      - 'docs/Examples.pdf'
      - 'scripts/install.sh'
      - 'crates/**'
      - 'Cargo.lock'
      - 'Cargo.toml'                 # root [patch.crates-io] miniscript rev (M2)
      - '.github/workflows/examples.yml'
    tags: ['examples-v*']
  pull_request:
    paths: [ … same set … ]
jobs:
  examples:
    runs-on: ubuntu-latest
    steps:
      1. actions/checkout@v6 (fetch-depth: 0)
      2. apt install: pandoc texlive-xetex texlive-latex-recommended
         texlive-latex-extra texlive-fonts-recommended texlive-fonts-extra
         fonts-dejavu jq          # monofont "DejaVu Sans Mono" (gen.sh:94);
                                  # jq explicit though runner-preinstalled
      3. dtolnay/rust-toolchain@1.85.0 + Swatinem/rust-cache@v2
      4. cargo build --bin mnemonic          # debug; manual.yml precedent
      5. regen:  EXAMPLES_BIN_DIR="$GITHUB_WORKSPACE/target/debug" \
                 bash .examples-build/gen.sh > .examples-build/Examples.md || exit 1
                 # a gen.sh FATAL (pin mismatch / missing preflight dep) fails the job
      6. GATE:   git diff --exit-code -- .examples-build/Examples.md
                 # fail-closed; drift shows verbatim in the CI log
      7. build PDF: the exact pandoc line from gen.sh:9-11 (cwd .examples-build);
                    assert Examples.pdf exists
      8. upload workflow artifacts: Examples.pdf + Examples.md
      9. on examples-v* tag: ensure release exists
         (gh release view || gh release create --generate-notes) then
         gh release upload "$REF_NAME" docs/Examples.pdf --clobber
         (verbatim manual.yml:147-168 pattern)
```

**Trigger-scope rationale (WIDE = leading indicator, LOCKED):** `manual.yml`/`quickstart.yml` scope to
`docs/<book>/**`, making their gates *lagging* (only re-run when the book is touched). Including `crates/**` +
`Cargo.lock` + `Cargo.toml` + `scripts/install.sh` makes gate A **leading**: a PR that changes any captured output
(or bumps the crate version, or advances the install.sh self-pin, or bumps a dep/`[patch]` rev that reshapes
`--help`/error text) goes red *in that PR*, forcing the lockstep regen. Cost: one debug `cargo build` + gen.sh
(~seconds of CLI runs) + one pandoc render on crates-touching PRs — small next to the `rust.yml` build the same PR
already pays. **`Cargo.lock` earns its place** (a clap/dep bump can change `--help`/error text with no `crates/**`
edit); `Cargo.toml` is M2 belt-and-suspenders.

### P2.b — The committed golden

- **Golden = `.examples-build/Examples.md`**, newly git-tracked: remove line 9 (`Examples.md`) from
  `.examples-build/.gitignore` and reword the header comment (P1.a item 7). It is the byte-exact P1 generator output
  the gate diffs against, and the single source for the PDF. `git diff --exit-code` is scoped to that ONE path, so
  the still-ignored `preamble.tex` / `work/` / `*.err` never false-positive.
- **`docs/Examples.pdf` stays committed** (Q4): 215 KB pure text, no raster figures — the gui_example attach-only
  rationale (~32 MiB screenshot corpus) does not apply; users already find it in-tree. Additionally attached to the
  `examples-v*` release (parity with the other books). The PDF is **NOT byte-gated** (xelatex output is
  non-reproducible across texlive versions — timestamps/IDs); the golden `.md` is the gated artifact and CI
  re-proves PDF *buildability* every run. Committed-PDF ↔ golden sync is convention (§ Release-ritual); Honesty §c.
- `preamble.tex` stays gitignored (gen.sh writes it deterministically, `gen.sh:45-64`).
- **Atomic born-green commit:** the `.gitignore` flip + `git add .examples-build/Examples.md` (the P1 0.75.0
  output) land in the SAME commit as `examples.yml`, so the very first CI run on the PR is green.

### P2.c — Required status check (M4 / locked-decision 2)

After the PR's `examples` job is green at least once, add **`examples`** to toolkit `master` branch-protection
required contexts (mirror the GUI `snapshots` precedent: required context + **`enforce_admins` OFF** so admin
direct-FF release pushes still function). This is an admin/`gh api` action; if toolkit master has no branch
protection yet, this cycle establishes it with `contexts=[examples]` only (do not over-scope to unrelated jobs
without a separate user decision). **Encode the answer in the plan of record:** required (blocking) per locked
decision 2 — release PRs are blocked until the bundled regen is present.

### P2 GATE (phase gate)

- **Positive (born-green):** the PR's `examples` job passes on the first run (proves author≡runner determinism
  cross-machine — the ultimate backstop for any unforeseen vector).
- **Negative (red-CI proof — fail-closed demonstrated, not assumed):** on a scratch commit, perturb ONE byte of
  the golden (or of a captured output) → push → confirm `examples` goes **RED** at step 6 (`git diff --exit-code`)
  with the drift in the log → drop the scratch commit. Do NOT ship without observing the red.

### P2 opus R0

Independent opus R0 over `examples.yml` + `.gitignore` + the golden add + the negative/positive proofs: confirms
triggers match the WIDE+REQUIRED lock (incl. M2 root `Cargo.toml`), no bitcoind step remains, the diff is scoped to
the golden path, the release-attach mirrors `manual.yml:147-168`, and the required-check mechanics are correct.
Fold → persist verbatim → re-dispatch until 0C/0I.

---

## P3 — regen + ship

### P3 deliverables

1. **Rebuild `docs/Examples.pdf`** from the P1/P2 golden via the exact `gen.sh:9-11` pandoc line (cwd
   `.examples-build`; gen.sh has already written `preamble.tex`). Commit the refreshed `docs/Examples.pdf`
   alongside the golden.
2. **Mandatory post-implementation adversarial whole-diff opus review** (independent agent, FULL diff incl. the
   regenerated golden vs the §3 enumeration AND the P1.c prose table): the **funds-derivation byte-stability
   backstop (S1)** — no address/xpub/descriptor/checksum/fingerprint changed anywhere; the regen faithfully
   re-captured; the prose stopped lying; the two static captures are faithful + loudly labelled. This catches
   implementation-introduced regressions TDD misses (CLAUDE.md post-impl gate).
3. **Flip FOLLOWUP status → RESOLVED** in the shipping commit: `design/FOLLOWUPS.md::examples-pdf-un-ci-gated` +
   companion `docs/manual-gui/FOLLOWUPS.md::examples-pdf-un-ci-gated`, in lockstep; add the **gate-B deferral note**
   (Honesty §a) to the toolkit entry.
4. **Merge the PR**, then **tag `examples-v1.0.0`**.
5. **Verify the tag run** attaches `docs/Examples.pdf` to the `examples-v1.0.0` release (`gh release view
   examples-v1.0.0 --json assets`), and that the tag-triggered `examples` job is green.

### P3 GATE (phase gate)

- Whole-diff opus review = GREEN (0C/0I); S1 confirmed not tripped (funds surface byte-stable).
- PR merged; `examples-v1.0.0` tag pushed; the tag `examples` run green; the PDF asset present on the release.
- Both FOLLOWUP entries flipped in the shipping commit (verify with `scripts/followup-reconcile.sh` if applicable).

### P3 opus R0

The mandatory whole-diff review IS the P3 opus gate. If Agent-API dispatch fails mid-session, **flag it explicitly
and defer the formal review to API recovery — never silently substitute inline self-review** (CLAUDE.md). Any fold
after the review RE-ENTERS the loop (re-dispatch a scoped convergence review; "mechanical fix" is not an exception).

---

## STOP ledger (halt the phase, ask the user)

- **S1 (funds surface — LIVE, primary):** the P1 regen (or the P3 whole-diff) surfaces ANY delta outside §3's
  enumeration — **especially any address / xpub / descriptor / checksum / fingerprint change**. The funds-critical
  surface is expected byte-stable (recon + R0 verified display-only); a violation invalidates the recon's premise.
  Do NOT ship; escalate with the diff. (Also covers P1.e §6.6-freeze faithfulness and P1.d depth2-freeze
  faithfulness — a static capture that disagrees with the live derivation is an S1.)
- **S2 (pinned-Core-in-CI): MOOTED by Q1-(b).** The spec's S2 (fallback show-only §6.6 downgrade needs sign-off if
  pinned Core proves infeasible) no longer applies — Q1-(b) removed bitcoind from CI entirely and froze §6.6 as a
  labelled static capture (no content downgrade: the external-oracle line is preserved verbatim). Retained here only
  to record that the decision was made and its STOP retired.
- **S3 (loud-rot discipline — LIVE):** any pressure to weaken the loud per-release redness (the D1.1 consequence /
  locked-decision 2), e.g. by normalizing version strings out of the diff or downgrading `examples` from required to
  advisory — re-opens the silent-rot hole and reverses a design premise. User call only.
- **S4 (new — required-check establishment):** if adding `examples` as a required context would (a) require enabling
  branch protection on toolkit master where none exists, or (b) risk blocking the codec/toolkit **direct-FF release**
  flow (memory gotcha (d)) because `enforce_admins` cannot be set OFF — surface to the user before flipping the
  branch-protection setting. (Expected resolution: mirror the GUI `snapshots` precedent, `enforce_admins` off; only
  a divergence from that precedent is a STOP.)

---

## Honesty — what gate A catches, and what it does NOT (carry forward)

Gate A catches **regen-drift, NOT narration-truth.** It diffs regenerated-output vs golden; gen.sh regenerates prose
*heredocs* verbatim, so prose is gated only for "golden matches gen.sh", never for "prose tells the truth about the
adjacent output block".

- **(a) Narration blind spot (the deferred gate B).** A future edit that changes an output block (with a faithful
  regen) but forgets the coupled narration — or edits narration to say something false about unchanged output —
  passes gate A as long as gen.sh and golden agree. Exactly the class that produced `gen.sh:209` ("printed twice").
  Gate A *shrinks* the window (output can't drift silently; prose drift now needs an *active* false edit passing
  review, not passive rot) but does not close it. The full answer is deferred **gate B**: per-command `.cmd`/`.out`
  transcripts + `include-transcript.lua`-style build-time inclusion (the four sibling books' "prose == .out by
  construction" model, `docs/manual/tests/verify-examples.sh` + `docs/manual/pandoc/filters/include-transcript.lua`).
  **Deferred v1.1+; file as a note on the FOLLOWUP entry at ship (P3.3).**
- **(b) Sibling-CLI drift.** gen.sh invokes only `mnemonic`; md/ms/mk behavior changes never surface here (gated by
  the four books).
- **(c) Committed-PDF staleness.** The PDF is convention-synced (P2.b); a regen commit that updates the golden but
  not `docs/Examples.pdf` passes CI. Mitigations: the release ritual + the PDF path in the workflow triggers (a
  PDF-only hand-edit at least *runs* the gate, proving the golden still regens — though it cannot verify PDF bytes).
- **(d) Doc coverage gaps.** New subcommands appear in the `--help` capture but the gate never demands worked
  examples for them (editorial).

## Release-ritual addition (gate-enforced from now on)

Every toolkit version bump must, in the SAME change, re-pin `gen.sh` (P1.a sites) + regenerate
`.examples-build/Examples.md` + rebuild `docs/Examples.pdf` (expected diff: version lines + install-pin lines only).
`examples.yml` enforces the first two (red until done); the PDF is convention (Honesty §c). **Add this line to the
toolkit release-version-sites checklist** (`project_toolkit_release_ritual_version_sites`).

---

## Citation appendix (all live at `origin/master` = `37472e25`, re-grepped this session)

- `gen.sh` = `.examples-build/gen.sh` (774 lines): pin/prose sites 3, 15, 18, 21-22, 27-29, 43-64, 78, 81, 87-88,
  94, 104, 123, 167, 179-181, 185-186, 206, 209-211, 288, 308, 314, 443, 455-459, 531, 649, 651-679 (§6.6 block;
  bitcoind `if` 656-679, `deriveaddresses` 660, printf capture 662, `$RECV` 649/660/662/673), 666, 696, 701-722
  (Appendix-B framing), 724-775 (depth2 `if…else…fi`; `--version` runs 729-730, `cat taproot-4leaf.desc` 735,
  `bundle|jq` 742, `cat depth2.md1` 743, mainline refusal 749, `mnemonic-depth2 restore` 756, else-fallback 764-773).
- Baseline `.examples-build/Examples.md` (gitignored build artifact, local, 111 575 B, byte-source of committed
  `docs/Examples.pdf` @ 215 270 B, same Jun-26 build): install leaks 98,109,111-115; `--list` toolkit pin 101;
  `--version` 143; `--help` 158-180 (no `gen-man`/`word-card`); §2 cards 225-246 (twice + md1 hyphens 244-246);
  §6.6 `deriveaddresses` 1474-1475 (address `bc1p550…00n8c8`); depth-2 export refusal ~1191; mainline depth-2
  refusal 1588-1589; depth2 static-capture transcript 1596-1617.
- `.examples-build/.gitignore:1-13` (pin comment :5, `Examples.md` ignore :9).
- `scripts/install.sh:75,322,324,327,336` (path-deriving lines).
- `.github/workflows/manual.yml:3-6,92-96,147-168`; `.github/workflows/bitcoind-differential.yml:57-105`
  (pinned-Core pattern — NO LONGER used per Q1-(b); cited only to record the superseded plan).
- `docs/manual/tests/verify-examples.sh`; `docs/manual/pandoc/filters/include-transcript.lua` (gate-B model, deferred).
- `design/FOLLOWUPS.md:155-161`; `docs/manual-gui/FOLLOWUPS.md:175-187`; `README.md:89` (gui_example attach-only).
- Live verifications (this session + banked): `~/.cargo/bin/mnemonic --version` = 0.75.0; crate `Cargo.toml`
  version = 0.75.0; `.docbins/current` = 0.72.0, `.docbins/gui-pinned` = 0.70.0 (STALE — Q5).

---

## Open question for the plan-R0 / user

- **OQ1 (P2.c mechanics):** does toolkit `master` already have branch protection? If not, this cycle *establishes*
  it with `contexts=[examples]` only, `enforce_admins` off (GUI `snapshots` precedent). Confirm that's the intended
  scope (vs. also enrolling `rust.yml`/other existing jobs, which would be a broader, separately-owned decision) —
  encoded as S4. Everything else in this plan is derived from R0-GREEN + user-locked inputs; no other open items.
