**⚠** r1: 0C/2I/5M/3N

# Review vf spec r1 (Opus, medium)

Scope: 3753b5c7..64361263 excl agent-reports.

## Findings

### I-1 [author cd77c7d9, Opus] Check (4) passes a vendored git source whose Cargo.toml differs from upstream's; regression against toolkit's current gate (false PASS)

Location: SPEC section 6 "(4) Git-source provenance" (spec lines 267-273: "every file except `Cargo.toml` is byte-identical ... Cargo.toml's name and version equal upstream's ... that file's bytes are not compared"); proto `vendor-freshness.sh` lines 288-293 (name_of/version_of only); section 7 removes toolkit's GROUNDED_GIT_SOURCES manifest digest, which today covers Cargo.toml.

Failure (run, scratch copy /scratch/cache/vf-r1-tk = toolkit 3753b5c7 + proto + data file): in
`vendor/miniscript/Cargo.toml` change `default = ["std"]` to `default = ["std", "compiler"]`, rewrite
`files["Cargo.toml"]` in `vendor/miniscript/.cargo-checksum.json` to the new sha256 (the F-354 self-consistent
shape, as G4 does for a .rs file). New gate: all four checks OK, `exit 0`. Toolkit's current gate on the same tree:
`exit 1`, `vendor/miniscript/.cargo-checksum.json: GIT-SOURCE PROVENANCE MISMATCH`. The release build then compiles
miniscript with a feature set that upstream at the Cargo.lock rev does not have. The same hole admits a tree
vendored from another rev when the rev bump changed only Cargo.toml (features, dependency features, edition,
`build`, lints) and kept the version, which is common on a git master pin such as miniscript's (version 13.0.0 across
many revs); (1) stays green when the lock's package set is unchanged. A file that upstream adds at the rev and that
cargo discovers by itself (`build.rs`, `src/bin/*.rs`) is also unseen: (4) compares only files present in vendor/.
The spec's own red heading ("A tree vendored from another rev passes (1)-(3); only this check sees it") is therefore
false for these trees, and the change removes a guarantee toolkit has today.

Fix needs a product choice (architect). Options:
  (a) Semantic Cargo.toml compare: run `cargo metadata --no-deps --format-version 1 --offline --manifest-path
      <checkout>/<field 3>/Cargo.toml` (empty CARGO_HOME) and compare, for the package named by the row, `name`,
      `version`, `features`, `dependencies` (name, req, kind, target, optional, uses_default_features, features,
      rename), `targets` (kind, name, crate_types, src_path relative to the crate dir, edition), `edition`, `links`
      against the same fields of the vendored package from check (1)'s `cargo metadata` output (currently sent to
      /dev/null). Covers build.rs and auto-discovered targets too. Needs a new G row (G10: vendored Cargo.toml feature
      changed, manifest digest rewritten -> `vendor/G: Cargo.toml: ... differs from upstream`) and an M6 mutant.
      Run: that command on descriptor-mnemonic at cf35d61a (crates/md-codec, empty CARGO_HOME, offline) exits 0 and
      yields version 0.47.0, features [default, derive], 9 dependencies, lib + example targets. Caveat for the
      architect: cargo in the upstream checkout honours that repo's rust-toolchain.toml.
  (b) Keep a per-row pinned sha256 of the vendored Cargo.toml as a 4th data-file field (TOFU for that one file,
      re-recorded at each pin bump) -- reintroduces the hand step the spec removes.
  (c) Accept and document: add to section 6 and the script header "NOT covered" a line naming Cargo.toml-only
      differences and upstream-only files, and record in section 7 that toolkit loses this coverage. (Not a fix of
      the false PASS; only makes the claim true.)

### I-2 [author cd77c7d9, Opus; kept by fold 77777ffc, Sonnet] FOLLOWUPS bookkeeping is undecided: four sibling entries are opened and nothing resolves them

Location: SPEC section 11 item 4 ("Every PR in this section carries these items ... A `design/FOLLOWUPS.md` entry,
slug `vendor-freshness-shared`, status `open`, with one Companion line per repo PR. The last PR to merge sets it to
`resolved`.") against section 12 step 3 ("PR-T2 ... the FOLLOWUPS entry is resolved").

Failure: item 4 applies to PR-T1, PR-R, PR-K, PR-S and PR-D, so five repos each gain an `open` entry. Only toolkit's
is ever resolved (PR-T2, which section 12 schedules after PR-K/S/D but not necessarily after PR-R; "the last PR to
merge" may be PR-R, which cannot edit toolkit's entry). The mr, key, secret and dm entries stay `open` forever, or
the builder must invent a fifth/sixth follow-up PR per repo. "One Companion line per repo PR" also cannot be written
in PR-T1, the first PR, because the other PRs do not exist yet. Each repo's FOLLOWUPS format is
`resolved <COMMIT>`, which the merging PR cannot know about itself. An implementer must choose.

Fix text (bookkeeping, no product point). Old, spec lines 1127-1129, matches once:

    4. A `design/FOLLOWUPS.md` entry, slug `vendor-freshness-shared`, status
       `open`, with one Companion line per repo PR. The last PR to merge sets it
       to `resolved`.

New:

    4. A `design/FOLLOWUPS.md` entry, slug `vendor-freshness-shared`. In toolkit
       (PR-T1) it is the primary entry, status `open`, with a Companion line naming
       each of mr, key, secret and dm; PR-T2 sets it to `resolved <PR-T2 head SHA>`.
       In mr, key, secret and dm the PR adds a companion entry already marked
       `resolved` (the change ships in that PR) with one Companion line:
       `mnemonic-toolkit design/FOLLOWUPS.md::vendor-freshness-shared`.

Section 12 step 3 ("the FOLLOWUPS entry is resolved") then stands as written.
Applied to a scratch copy (/scratch/cache/vf-r1-specfix.md): old matched once; `diff` shows only spec lines
1127-1132 changed, outside every `<!-- vf:... -->` fenced block, so no gate check (C2-C15 compare only blocks) can
change; the full gate was not re-run for this prose-only fix.

## Minor

- M-1 [author] mr runner rsync (spec 13 "NOT covered"; test line 39): mr's Linux VM install line (ci/self-hosted/README.md:46) has no rsync and nothing in mr uses rsync today; if absent, PR-R's CI goes red with no decided remedy (fails closed; needs a VM without rsync). Fix: proto test line 39 `| ( cd "$REPO_ROOT" && rsync -a --from0 --files-from=- ./ "$PRISTINE/" )` -> `| ( cd "$REPO_ROOT" && tar --null -T - -cf - ) | tar -xf - -C "$PRISTINE"`, drop rsync from the test header, spec section 8/13 and item 3's purpose text, and re-gate (hashes change). Run: `git ls-files -z` copied both ways in /scratch/cache/spec-vf gave identical trees (`diff -r` empty).
- M-2 [author] check (4) passes the caller's whole environment to git (proto line 232 `genv = dict(os.environ, ...)`), so an exported GIT_DIR (a git hook, a wrapper) redirects `git init`/`fetch`/`checkout` into the caller's repo. Run: with GIT_DIR=<victim>/.git, the script's fetch sequence wrote `<victim>/.git/shallow` and left `up/` empty. Spec section 3 "No other variable is read" is also false for GIT_* and CARGO_*. Fix: `genv = {k: v for k, v in os.environ.items() if not k.startswith("GIT_")}` then `genv.update(GIT_TERMINAL_PROMPT="0", GIT_CONFIG_NOSYSTEM="1", GIT_CONFIG_GLOBAL=os.devnull)` (run: victim untouched, fetch/checkout rc 0, HEAD = rev). Conditions: GIT_DIR exported by the caller; no repo runs the gate from a hook today.
- M-3 [author] section 11 line 1199 "After that step, add:" follows "before the step `Resolve ...`"; "that step" can read as the pin step, putting the mutation test before the real gate (order only). Fix: "After the step `Resolve Cargo.lock against committed vendor/ (offline, locked)`, add:".
- M-4 [author] PR-T1 leaves toolkit vendor-freshness.yml:62 ("`cargo metadata` resolution only — no compile...") and :5-6 ("REDs iff the committed vendor/ cannot satisfy Cargo.lock") stale while key/secret/dm get the new wording; give toolkit the same two edits as section 11's key/secret/dm bullets.
- M-5 [author] section 11 item 4 vs section 12: PR-C may merge after PR-T2, so "the last PR to merge" is not PR-T2 (absorbed by I-2's fix).

## Nit

- N-1 spec:1204 old-string carries markdown-escaped backticks (`\``) inside a code span; the file text has none.
- N-2 item 3 test purpose "needs rsync, git, cargo and network" omits python3 (section 8 lists it).
- N-3 dm scripts/README.md row 9 uses backticked cells; item 3 does not say whether to keep that style.

## Design note (ungraded)

- mr's copies are outside every drift comparison (settled by vf-s1); a coordinated edit of mr's script and its two hash lines passes mr CI.

## Fold accounting

All findings are from the author commit cd77c7d9 (Opus); I-2 text was kept unchanged by fold 77777ffc (Sonnet, architect vf-s1). 0 findings introduced by the fold. Gate checks spot-read (C1-C8, C11-C15 descriptions and C1-C7 code): each compares against an extracted block and fails on an empty extraction where it matters (C5 `-s`, C6 `-n want`).

C / I / M / N: 0 / 2 / 5 / 3
