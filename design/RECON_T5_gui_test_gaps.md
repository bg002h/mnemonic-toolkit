# RECON — T5 (GUI test gaps) sub-cycle prep

**Purpose:** read-only recon feeding a SPEC for test-hardening sub-cycle T5. No code/tests
modified. Source: `design/agent-reports/constellation-eval-2026-07-06.md` §2 (test-improvement
program) + ground-truth against `/scratch/code/shibboleth/mnemonic-gui` at HEAD
(`5d88286`, 2026-07-10; branch protection + pin state as observed at recon time).

---

## 1. Full 15-item reconciliation (§2 of the eval)

The eval's §2 numbers 1-15 continuously across two subsections: a "Cross-cutting" pair (1-2,
process/CI-wiring, not new test code) followed by 13 numbered test/harness items (3-15) grouped
under four headings, the last of which is literally titled `### GUI` and contains #13, #14, #15.

| # | One-line | Repo(s) | Sub-cycle | Notes |
|---|---|---|---|---|
| 1 | No repo's test suite gates a merge — add required-check contexts | all 5 repos | **not T1-T5** (process, not a test) | See §4 below: **already true for GUI** (branch protection now requires `schema-mirror gate`, `clippy`, `headless`, `snapshots`, `x86_64-unknown-linux-gnu`) — remediated independently of any T-cycle, presumably folded into Cycle E or a prior process pass. Toolkit/md/mk/ms status not re-verified here (out of T5 scope). |
| 2 | wc-codec suite + fuzz targets run in no CI workflow | toolkit | **not T1-T5** (CI-wiring, not new test code) | Toolkit-only; unrelated to GUI. Not reconciled further here (out of T5 scope). |
| 3 | Word-Card wire golden (`tests/wire_golden.rs`) | toolkit (wc-codec) | **T3** | user-confirmed mapping |
| 4 | md1 frozen corpus omits production-default shapes | toolkit/md | **T3** | user-confirmed mapping |
| 5 | `payload_bits` mutation gap | toolkit | **T3** | user-confirmed mapping |
| 6 | toolkit `prop_repair_never_wrong.rs` | toolkit | **T2** | user-confirmed mapping |
| 7 | md `bch_exhaustive_sweep.rs` + fix `parity_smoke` silent skip | md | **T2** | user-confirmed mapping |
| 8 | mk `bch_correct_ok_implies_valid_codeword` proptest + 3rd fuzz target | mk | **T2** | user-confirmed mapping |
| 9 | mk address rendering pinned to sibling constants, not official BIP-84/86/49 vectors | mk | **T4** | user-confirmed mapping |
| 10 | ms derive: only bip84 pinned; bip44/49/86 purpose constants untested | ms | **T1** | user-confirmed mapping |
| 11 | ms BIP-39 language mapping: only en/ja/fr exercised, Czech↔Portuguese swap uncaught | ms | **T1** | user-confirmed mapping |
| 12 | ms K-of-N share index pool untested above n=11 (secret-at-`s` disclosure risk, n≥17) | ms | **T1** | user-confirmed mapping |
| **13** | **GUI `default_value`/dropdown-`choices` parity gate** | **gui** | **T5** | **RECONCILED: GUI-scoped, belongs in T5 alongside #14/#15 — see verdict below. Status: already SHIPPED (2026-07-10, `schema_mirror_defaults_drift.rs`).** |
| 14 | GUI canonicity-classifier corpus — h-notation gap | gui | **T5** | user-confirmed mapping. Status: **partially open** — see §3. |
| 15 | GUI funds-core bundle→restore round-trip, spec-independent oracle | gui | **T5** | user-confirmed mapping. Status: **fully open**. |

### The #13 reconciliation verdict

Item #13 sits under the eval's `### GUI` heading (report lines 281-294), textually and
structurally identical in kind to #14 and #15 — all three are numbered consecutively immediately
under that heading, with no other item interleaved. There is no ambiguity in the source document:
**#13 is GUI-scoped**, exactly like #14/#15, and belongs in T5. The prior notes' gap ("#13
unmapped") was very likely an off-by-one skim past #12 (the last ms item, a genuinely
higher-severity secret-disclosure finding) rather than a real classification question — the
eval's own heading placement settles it.

Net: **T5 = eval items #13, #14, #15, all three.** Cross-cutting items #1/#2 are process/CI-wiring
work, not new-test-code items, and are not part of any T1-T5 sub-cycle by the eval's own framing
("largely non-code" — see eval §3 Cycle F description). Nothing from the 15-item list is silently
dropped: 1-2 are cross-cutting/process (out of T1-T5 scope entirely), 3-12 map to T1-T4 exactly as
already known, 13-15 map to T5.

---

## 2. Ground-truth: item #13 — default_value/choices parity gate

**Eval text (verbatim):**
> 13. **`default_value` / dropdown-`choices` parity gate** — the pinned binary's `gui-schema` v5
> JSON carries both, but no test compares the hand-mirrored strings against it (`schema_mirror`
> gates flag *names* only). This is exactly what makes F3/F6's silent flag materialization
> possible, and what let the `export-wallet --timestamp 'now'→'0'` drift slip (caught by hand).
> Add `schema_mirror_defaults_drift.rs` asserting mirror `default_value`/`choices` == live JSON.
> **Highest-leverage single GUI test.**

**Status: SHIPPED.** `mnemonic-gui/tests/schema_mirror_defaults_drift.rs` (191 lines) exists at
HEAD, added in commit `3c5c7dd` (2026-07-10, "F5+F6 recovery-form wiring — nested-argv split +
virgin-dropdown inference (funds-safety)"), merged via PR #32 into `mnemonic-gui-v0.58.0`. This is
literally the eval's recommended filename, landed as part of Cycle E (F5+F6+M1 GUI wiring)
per the eval's own remediation grouping — the F6 fix (virgin-dropdown materialization) and this
drift gate are the same "materialized argv must match the toolkit's real default" concern.

**What it does** (`tests/schema_mirror_defaults_drift.rs:93-158`):
- Iterates every `mnemonic` subcommand/flag in the hand mirror (`schema::mnemonic::SCHEMA`),
  compares `FlagSchema.default_value` and `FlagKind::Dropdown` choices against the live pinned
  `mnemonic gui-schema` v5 JSON via `schema_check::{json_flag_defaults, json_flag_choices}`.
- Handles the `""` "(none)"/inference display-sentinel GUI convention (stripped from both sides
  before comparing) and a tiny explicit `DEFAULT_VALUE_ALLOWLIST` for value-format-only
  divergences (`compare-cost --feerate`: `"1.0"` vs toolkit's `"1"`).
- `choices` comparison is scoped to flags whose live JSON carries non-null `choices` (skips
  text-kind-rendered-as-Dropdown GUI conveniences like `--separator`).
- A second unit test (`parse_accessors_normalize_typed_defaults_and_choices`) pins the JSON
  accessor's typed→string normalization independent of any binary.

**CI wiring — confirmed a required check, not advisory.** `.github/workflows/schema-mirror.yml`'s
`cargo-test-full-suite` step (`:127-133`) runs `cargo test --workspace` with `MNEMONIC_BIN=mnemonic`
set (binary installed at the pinned tag in an earlier step), which picks up this test automatically
via cargo's default `tests/*.rs` discovery. `gh api repos/bg002h/mnemonic-gui/branches/master/protection`
confirms `required_status_checks.contexts` includes `"schema-mirror gate"` (job name `schema-mirror`,
step name `schema-mirror gate`) alongside `clippy`, `headless (no-default-features)`, `snapshots`,
`x86_64-unknown-linux-gnu`; `enforce_admins: false`. So this test **already gates PRs into
`mnemonic-gui` master**, not just runs advisory.

**Residual (not required for T5, optional follow-on):** the docstring (`:28-31`) explicitly scopes
the gate to `mnemonic` only, calling extension to `md`/`ms`/`mk` "a natural follow-on… deliberately
out of this cycle to stay a bounded add." No FOLLOWUP currently tracks that extension (grepped
both repos' `FOLLOWUPS.md`, zero hits for `schema_mirror_defaults_drift` outside this test file).
T5 does not need to do this — item #13 is closed as written — but the SPEC skeleton below flags it
as an optional in-scope-if-cheap addendum.

**Verdict: #13 requires NO new work for T5.** It is fully shipped, CI-gated as a required check,
and matches the eval's own described fix exactly. T5's real work is #14 (partial) and #15 (open).

---

## 3. Ground-truth: item #14 — canonicity-classifier corpus (h-notation gap)

**Eval text (verbatim):**
> 14. **Canonicity-classifier corpus** covers only apostrophe-notation origins; an h-notation
> regex/toolkit disagreement silently coerces `--account` to 0 and changes derived BIP-48 paths.
> Add a deterministic shape-grid differential against `mnemonic gui-schema --classify-descriptor`.

**Status: the differential *harness* already exists (pre-dates the eval); the h-notation *fixture
grid* the item calls for does not.**

- `mnemonic-gui/tests/canonicity_drift.rs` (224 lines) is exactly "a deterministic shape-grid
  differential against `mnemonic gui-schema --classify-descriptor`" — it already exists, predates
  the 2026-07-06 eval by weeks (`bdecfff` 2026-06-08 "per-fixture Expect table replaces lenient 50%
  floor"; `1aa3030` 2026-06-21 "canonicity regex accepts suffix-origin form `@N[fp/path]`"). It
  shells out per-fixture to `MNEMONIC_BIN gui-schema --classify-descriptor <STR>` and asserts four
  independent invariants per fixture (`disagreements`, `regressed`, `wrong_verdict`,
  `newly_parsed` — `:139-223`), against a 19-row `FIXTURES` table (`:105-136`; 12 Canonical + 4
  NonCanonical + 3 ParseFails covering the BIP-388 `/**`-shorthand toolkit-parser gap).
- `tests/canonicity_classifier.rs` (150 lines, pre-dates the eval further: `ac9d8a0` 2026-05-16) is
  the GUI-only unit-test corpus for `classify_descriptor_canonicity` in isolation (no toolkit
  shell-out).
- **Every single fixture in both files uses apostrophe notation** (`44'`, `84'`, `86'`, `48'`) —
  confirmed by grep: zero hits for `84h`/`86h`/`48h`/`44h`/`"hardened"`/`"apostrophe"` anywhere in
  either test file or `design/SPEC_canonicity_drift_per_fixture_table.md`.
- The GUI's classifier regex (`mnemonic-gui/src/form/conditional.rs:99-129`) already contains
  h-notation support structurally: each origin-fingerprint bracket and use-site path segment
  matches `(?:/\d+'?h?)*` / `/\*+'?h?` — i.e., the code *claims* to accept `44h` as well as `44'`,
  but this claim is **never exercised against the toolkit's actual parser** by any test. This is
  precisely the untested-claim shape the item's "silently coerces" framing describes.

**Gap, precisely:** the existing harness's oracle discipline (shell out to the toolkit, assert
GUI-regex agreement) is sound and reusable — it is the *fixture set* that is one-notation-narrow,
not the *infrastructure*. Fix scope is a fixture-table extension, not new harness code.

**RED-under-mutation (the concrete regression each new fixture proves):**
1. **Regex h-support regression.** Swap `conditional.rs`'s `'?h?` for a bare `'?` (drop h-support
   entirely) — the full existing 19-fixture `canonicity_drift.rs` table stays 100% green (none of
   its fixtures contain an `h` origin/use-site segment), silently shipping a GUI that
   misclassifies every h-notation descriptor as NonCanonical. A parallel h-notation shape grid
   (mirroring the existing apostrophe rows: `pkh([deadbeef/44h/0h/0h]@0/<0;1>/*)`,
   `wpkh(@0/<0;1>/*)`-with-h-origin, `wsh(multi(2,@0,@1,@2))`-with-h-origin cosigner brackets, a
   `tr(@0)` h-origin row, at minimum one deliberately-mixed apostrophe+h row) would RED the
   instant this regresses.
2. **The "--account coerced to 0" consequence, not just the raw verdict.** `is_descriptor_non_canonical`
   (`conditional.rs:139-144`) is what actually drives the `--account → PinValue(0)` lift/pin
   decision consumed by `bundle()`. A fixture that only checks the raw `Canonicity` enum verdict
   does not prove the *consuming* pin-lift logic agrees for an h-notation input — a bug that
   flips `is_descriptor_non_canonical`'s truth table specifically on h-input (e.g. an
   off-by-one in how the bracket capture group is consumed downstream) could pass a
   verdict-only fixture while still corrupting the account pin. T5's fixture set should add at
   least one direct `is_descriptor_non_canonical(...)` assertion (or equivalent
   `bundle()`-visibility-effect assertion) on an h-notation non-canonical descriptor, not stop at
   `classify_descriptor_canonicity`.

**Fix scope for T5:** extend `canonicity_drift.rs`'s `FIXTURES` const with an h-notation grid
(and ideally the mixed-notation edge) using the SAME `Expect` enum / harness; add one
`is_descriptor_non_canonical`-level assertion cell. No new module, no new CLI surface, no toolkit
change (`gui-schema --classify-descriptor` already normalizes h/apostrophe identically at the
toolkit side — this is purely testing the GUI regex against it).

---

## 4. Ground-truth: item #15 — GUI funds-core bundle→restore round-trip

**Eval text (verbatim):**
> 15. **One funds-core round-trip with a spec-independent oracle** — GUI-assembled `bundle` →
> `restore` asserting recovered entropy == the all-zero vector and first address == the published
> BIP-84 address. Today the only end-to-end GUI bundle coverage is a snapshot-of-own-pipeline
> regression pin.

**Status: fully open — confirmed no existing test matches this description.**

Three places that could plausibly cover this, all ruled out by direct inspection:

1. **`tests/wire_shape_snapshot.rs`** (670 lines) — pins `--json` **key-set shape** (not values,
   except a few scalar spot-checks) for `xpub-search` (3 modes) and `import-wallet` (2 formats)
   only. Grep for `"bundle"`/`"restore"` in this file hits only the *string* `"bundle"` as a JSON
   key inside `import-wallet`'s envelope (`bundle.descriptor`, `bundle.multisig`, …) — `mnemonic
   bundle` and `mnemonic restore` are never invoked as subcommands here.
2. **`tests/ui_harness_i4_realcli.rs`** (279 lines) — the I4 "curated real pinned-CLI functional
   cells" tier, structurally the closest analog to what #15 wants (drives `assemble_argv` +
   `runner::run` against the real pinned binary, asserts real fields). But its four cells are all
   **decode-style, deterministic-pure-function reads**: `decode-address`, `md decode`, `ms decode`,
   `mk decode` (`:16-22`). `bundle`/`restore` are not among the four CLIs/cells and there is no
   fifth "mnemonic bundle→restore" cell.
3. **`tests/tutorial/*`** (manifest.rs, mod.rs) — **does** drive `bundle`→`restore` through the
   real GUI pipeline end-to-end (stems `tut-j1-01-bundle-single-sig`, `tut-j2-06-bundle-watch-only`,
   `tut-j2-07-bundle-all-seeds`, `tut-j2-restore-feed-bundle-json`, the J4 NUMS bundle/restore
   chain per `manifest.rs:5-18`). But its oracle is **transcript/PNG byte-gating against a
   committed golden** (`mod.rs::transcript_files`/`check_allowlist` — "byte-gates its transcript,"
   `:246`) — i.e. it re-captures and pins whatever the pipeline currently emits, not an
   independent published value. This is exactly the eval's "snapshot-of-own-pipeline regression
   pin" characterization, confirmed verbatim.

**RED-under-mutation:** any *symmetric* bug that shifts what the pipeline emits and what the
tutorial golden expects **in lockstep** (e.g., corrupt a BIP-84 path-derivation constant shared
by both `bundle`'s encode path and `restore`'s decode path, or corrupt the GUI's own template-mode
default wiring) regenerates a self-consistent new golden and stays green forever under
`tests/tutorial/*` and `wire_shape_snapshot.rs` alike — because nothing in the current suite pins
against a value that exists independent of the tool under test. Only a hardcoded published vector
(the exact BIP-84 first-receive address the eval's own C1 finding cites,
`bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`) RED-proves this class.

**Fix scope for T5:** one new test cell, modeled on I4's shape but adding `bundle`+`restore`:
1. Build a minimal single-sig `bundle` `FormState` (Template=`bip84`, all-zero-entropy BIP-39 seed
   via `--phrase-stdin`, no passphrase, network=mainnet) through the schema/`assemble_argv` path
   (same idiom as `ui_harness_i4_realcli.rs`), run via `runner::run` against the pinned
   `MNEMONIC_BIN`, parse `--json` to extract the emitted `md1` card.
2. Feed that `md1` into a `restore --md1` `FormState`, assembled/run the same way.
3. Assert: recovered entropy hex == the all-zero BIP-39 vector (`00000000000000000000000000000000`,
   the same constant `ui_harness_i4_realcli.rs::MS1_ALL_ZERO_ENTROPY_HEX` already uses), AND first
   derived address == the hardcoded published BIP-84 vector address — a literal in the test file,
   never derived from any tool's own output.

**Scoping note (load-bearing for the dependency check, §5):** the eval's own phrasing ("recovered
entropy == the all-zero vector and first address == the published BIP-84 address") describes a
**`--template bip84`** flow, not a `--descriptor`-mode flow. This matters: `--template` mode never
touches `lex_placeholders` (the C1 use-site-collapse bug's locus), so a template-mode round-trip is
correct against the GUI's *currently pinned* toolkit tag (`mnemonic-toolkit-v0.75.0`, pre-dating
Cycle A's C1 fix at v0.76.0) with zero dependency risk. A `--descriptor`-mode variant of the same
round-trip would be a natural, higher-value companion (it would exercise the exact C1 fix
end-to-end through the GUI) but would require the GUI's toolkit pin to bump to ≥v0.76.0 first —
out of scope for a NO-BUMP-parallel T5. **Recommend the T5 SPEC explicitly scope this cell to
`--template bip84`** (matching the eval's literal text) and file the `--descriptor`-mode variant as
a follow-on FOLLOWUP contingent on the next GUI pin bump.

---

## 5. GUI-specific gates + mechanics (confirmed applicable/not)

| Convention | Confirmed? | Evidence |
|---|---|---|
| PR + CI-before-tag (NOT direct-FF) | Applies | `schema-mirror.yml` fires on `pull_request` to `master`/`release/**`; branch protection is live and enforced (`enforce_admins: false` but required contexts present — a PR cannot merge without them). |
| MSRV | 1.88 (unchanged) | `Cargo.toml:8` `rust-version = "1.88"`. Neither new test touches MSRV-sensitive code (subprocess/regex only). |
| kittest / lavapipe / `RUSTUP_TOOLCHAIN=stable` | **Does NOT apply to either T5 test.** | Both new cells are pure subprocess-driven (`runner::run` / `Command::new`), no `egui_kittest` rendering, no wgpu/vulkan surface. `runner.rs:1-13` confirms `runner::run` is a stdlib `Command`/`wait_with_output` wrapper — no GPU dependency. Only the `snapshots` job (kittest pixel tests) needs lavapipe; T5's cells belong in the plain `cargo-test-full-suite` step, same tier as `ui_harness_i4_realcli.rs`/`canonicity_drift.rs`. |
| `schema_mirror` gates flag-NAMES | Confirmed, and item #13 (already shipped) is the companion gate for flag *values*/`choices` — T5 doesn't touch either gate, it's a consumer of the same `schema_check` module for #13's already-closed work only. |
| NO fmt gate — do NOT `cargo fmt` the GUI | Confirmed | No `cargo fmt` step in any `.github/workflows/*.yml`; per standing user preference (`project_g6_fmt_exemption_and_asymmetric_pin` / repo convention), any T5 implementer must NOT run `cargo fmt --all` on this repo. |
| clippy `-D warnings` → `#![allow(dead_code)]` convention | Confirmed present | `build.yml:29-30,59-60` runs `cargo clippy --all-targets -- -D warnings` (both default-features and `--no-default-features`) as a required check. New test-only code is unlikely to trip this, but any new shared helper added outside `tests/` (unlikely for T5) would need the existing allow-convention. |

---

## 6. T2/T3/T4-independence confirmation

**Confirmed: T5 has zero dependency on T2/T3/T4, and can run fully in parallel.**

- T2 (#6/#7/#8) and T3 (#3/#4/#5) are toolkit/md/mk-internal **property-test and golden-fixture
  additions** — new `tests/*.rs` files and fixture data within those repos' own test trees. None
  add, remove, or rename a clap flag/subcommand (confirmed by the eval's own descriptions: proptest
  harnesses over existing `repair`/`bch_correct_*` functions, a new golden `tests/wire_golden.rs`,
  corpus additions to existing frozen-vector files). Per repo convention (CLAUDE.md), pure test
  additions with no CLI-surface change are **NO-BUMP** — no new tag, no crate-version bump.
- T4 (#9) is an mk-internal test-oracle swap (pin to official BIP-84/86/49 vectors instead of
  sibling-toolkit-derived constants) — also test-file-only, no CLI surface change, NO-BUMP.
- The GUI's toolkit dependency is pinned by **tag**, not by floating branch
  (`Cargo.toml:76` `mnemonic-toolkit = { git = "…", tag = "mnemonic-toolkit-v0.75.0" }`, mirrored
  in `pinned-upstream.toml:22`). A NO-BUMP test-only toolkit change produces no new tag at all —
  there is nothing for the GUI's pin to even move to. The GUI's CI (`schema-mirror.yml:56-62`)
  `cargo install --git … --tag "$TAG"` will keep resolving the exact same `v0.75.0` commit
  regardless of what merges into toolkit `master` under T2/T3/T4.
- Both of T5's own test additions exercise CLI surfaces that have existed at the GUI's pinned tag
  for a long time already: `gui-schema --classify-descriptor` shipped at toolkit **v0.20.0**
  (`design/FOLLOWUPS.md` entry `gui-schema-classify-descriptor-subcommand`, "resolved 14c8119 —
  shipped at mnemonic-toolkit-v0.20.0"); `bundle`/`restore --md1` have existed since the earliest
  toolkit releases. Neither new T5 cell needs any toolkit change, let alone a pin bump.
- The one caveat is the *optional* `--descriptor`-mode companion to #15 flagged in §4 above — that
  would need the GUI's pin to move to ≥v0.76.0 (Cycle A's C1 fix). It is explicitly **not** part
  of the recommended T5 scope (the eval's #15 text describes `--template bip84`, not
  `--descriptor` mode), so it does not break independence; it is called out only as a natural
  follow-on for whoever owns the next GUI pin-bump cycle.

**Conclusion: T5 is genuinely parallel-safe against T1-T4 as currently scoped**, and additionally
against #1/#2 (the cross-cutting process items, already substantially done for GUI per §1/§5's
branch-protection finding).

---

## 7. Proposed T5 SPEC skeleton

**Title:** T5 — GUI canonicity h-notation coverage + funds-core bundle→restore independent-oracle
round-trip.

**Scope (in):**
- S1. Extend `mnemonic-gui/tests/canonicity_drift.rs`'s `FIXTURES` table with an h-notation shape
  grid mirroring the existing apostrophe-notation rows (origin-fingerprint h-notation, use-site
  h-notation, at least one mixed apostrophe+h row), reusing the existing `Expect`
  (Canonical/NonCanonical/ParseFails) harness and its four-invariant assertion machinery
  unchanged.
- S2. Add ≥1 fixture-level assertion of `is_descriptor_non_canonical(...)` (or the equivalent
  `bundle()` visibility-effect) on an h-notation NonCanonical descriptor, closing the
  verdict-vs-consequence gap noted in §3.
- S3. New test cell (new file or appended to `ui_harness_i4_realcli.rs`) driving GUI-assembled
  `bundle --template bip84` (all-zero seed, `--phrase-stdin`, no passphrase, mainnet) →
  `restore --md1` through `assemble_argv`/`runner::run`, asserting recovered entropy ==
  `00000000000000000000000000000000` and first address ==
  `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu` (both hardcoded literals, independent of any
  tool-under-test output).
- S4. (Optional, cheap-if-folded-in) extend item #13's `schema_mirror_defaults_drift.rs` scope
  from `mnemonic`-only to also cover `md`/`ms`/`mk` — genuinely optional; only include if the R0
  reviewer judges it in-scope-cheap, since it was deliberately deferred by the original author.

**Scope (out):** item #13 core work (already shipped, §2); a `--descriptor`-mode variant of S3
(needs GUI pin ≥v0.76.0, §4/§6 caveat — file as a follow-on FOLLOWUP instead); items #1/#2
(process/CI, not T1-T5); any toolkit/md/ms/mk-side change (none needed).

**Repo:** `mnemonic-gui` only. **Gate model:** PR + CI-before-tag, required contexts already live
(`schema-mirror gate`, `clippy`, `headless`, `snapshots`, `x86_64-unknown-linux-gnu`) — the new
tests will gate merges from the moment they land, no separate branch-protection work needed.
**No fmt.** **No lavapipe/kittest dependency** for S1-S3 (pure subprocess tests); S4 likewise.
**No toolkit pin bump required** — runs against the current `mnemonic-toolkit-v0.75.0` pin
unchanged.

**R0 gate:** per CLAUDE.md, brainstorm spec → mandatory opus/fable R0 loop to 0C/0I before any
test code is written, same as every other sub-cycle.

---

## Sources consulted

- `design/agent-reports/constellation-eval-2026-07-06.md` (§2, lines 200-296) — item text, exact
  quoting.
- `mnemonic-gui` @ `5d88286` (2026-07-10): `tests/schema_mirror_defaults_drift.rs`,
  `tests/canonicity_classifier.rs`, `tests/canonicity_drift.rs`, `tests/wire_shape_snapshot.rs`,
  `tests/ui_harness_i4_realcli.rs`, `tests/tutorial/mod.rs`, `tests/tutorial/manifest.rs`,
  `src/form/conditional.rs`, `src/runner.rs`, `src/schema_check.rs`, `Cargo.toml`,
  `pinned-upstream.toml`, `.github/workflows/schema-mirror.yml`, `.github/workflows/build.yml`.
- `gh api repos/bg002h/mnemonic-gui/branches/master/protection` (live, recon-time).
- `git log` dates for the above test files (to establish pre-eval vs post-eval provenance).
- `design/FOLLOWUPS.md` (both toolkit + GUI repos) — grepped for existing tracking of the
  h-notation gap / bundle-restore independent-oracle gap / defaults-drift md-ms-mk extension: zero
  hits in all three cases, confirming genuine open gaps.
