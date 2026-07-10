# Cycle-prep RECON — constellation-eval F5 + F6 (GUI recovery-form wiring)

Read-only recon feeding a SPEC. **No edits made** except this file. Verified against
current source; both findings **reproduced live**.

- GUI repo: `/scratch/code/shibboleth/mnemonic-gui` @ `master f5cb11f` (Cargo version **0.57.0**; latest tag `mnemonic-gui-v0.57.0`).
- Toolkit: `/scratch/code/shibboleth/mnemonic-toolkit` @ `master 3d985798` (crate **v0.84.0**; F3 shipped v0.83.0, F1/F2 later).
- Repro binary: `/scratch/code/shibboleth/mnemonic-toolkit/target/debug/mnemonic` = **v0.84.0** (post-F3).
- Eval source: `design/agent-reports/constellation-eval-2026-07-06.md:126-155`.

Both fixes land **GUI-side only** (runner + schema data). **No toolkit change, no toolkit-pin
bump** is required (the flattening lives in the toolkit's `gui-schema` emitter and is
faithfully mirrored — the bug is that the GUI never *reverses* it for the run/copy path).

---

## F5 — GUI emits flattened nested-subcommand names the binary rejects — **ACCURATE (reproduced live)**

### Per-claim verification

| Claim | Verdict | Evidence (current lines) |
|---|---|---|
| GUI pushes `SubcommandSchema.name` as a **single** argv token | **ACCURATE** | `src/form/invocation.rs:161` — `argv.push(subcommand.name.to_string());` (cited `:161`, exact). Sole run/copy assembler. |
| Toolkit CLI is **nested** (`seed-xor combine`, not `seed-xor-combine`) | **ACCURATE** | Toolkit `main.rs:95,122-144` top-level `Commands` has 5 `#[command(subcommand)]` parents each wrapping an Args with its own nested enum (`SeedXorCommand`, `SeedqrAction`, `Slip39Command`, `MsSharesCommand`, `XpubSearchCommand`). |
| The 12 nested forms emit an unrunnable line, clap **exit 64** | **ACCURATE** | Repro below. |
| `schema_mirror` can't catch it (tautological) | **ACCURATE** | `tests/schema_mirror.rs:103-108` compares the **flattened** GUI name against the **flattened** `gui-schema` JSON name — flat-vs-flat, never reversed. |
| Cited `schema/mnemonic.rs:4524-4658` = the 12 nested entries | **ACCURATE** | Exactly that range (`4524`–`4658`). |

### Live reproduction (`target/debug/mnemonic` v0.84.0)

```
$ mnemonic seed-xor-combine --help          → exit 64  "error: unrecognized subcommand 'seed-xor-combine'"
                                                        "tip: some similar subcommands exist: 'seedqr', 'seed-xor'"
$ mnemonic seed-xor combine --help          → exit 0   (works)
$ mnemonic xpub-search-address-of-xpub …    → exit 64  "unrecognized subcommand"
$ mnemonic xpub-search address-of-xpub …    → exit 0   (works)
```

### The EXACT 12 nested subcommands + the split rule (the F5 crux)

The toolkit flattens at `crates/mnemonic-toolkit/src/cmd/gui_schema.rs:1015-1017`:
```rust
let flat_name = format!("{}-{}", s.get_name(), ss.get_name());   // parent-child
```
**Both parent and child names contain hyphens**, so a naive "split on hyphens" is WRONG:

| Flattened GUI name (`SubcommandSchema.name`) | Correct nested argv | naive hyphen-split (BROKEN) |
|---|---|---|
| `seed-xor-split` | `["seed-xor","split"]` | `[seed,xor,split]` ✗ |
| `seed-xor-combine` | `["seed-xor","combine"]` | `[seed,xor,combine]` ✗ |
| `seedqr-encode` | `["seedqr","encode"]` | `[seedqr,encode]` (ok by luck) |
| `seedqr-decode` | `["seedqr","decode"]` | ok by luck |
| `slip39-split` | `["slip39","split"]` | ok by luck |
| `slip39-combine` | `["slip39","combine"]` | ok by luck |
| `ms-shares-split` | `["ms-shares","split"]` | `[ms,shares,split]` ✗ |
| `ms-shares-combine` | `["ms-shares","combine"]` | `[ms,shares,combine]` ✗ |
| `xpub-search-path-of-xpub` | `["xpub-search","path-of-xpub"]` | `[xpub,search,path,of,xpub]` ✗ |
| `xpub-search-account-of-descriptor` | `["xpub-search","account-of-descriptor"]` | ✗ |
| `xpub-search-address-of-xpub` | `["xpub-search","address-of-xpub"]` | ✗ |
| `xpub-search-passphrase-of-xpub` | `["xpub-search","passphrase-of-xpub"]` | ✗ |

**How does the GUI know a name is nested?** It does **not** — there is NO structural marker.
`SubcommandSchema` (`src/schema/mod.rs:30-55`) carries only `name/human_name/flags/positional_args/allows_slots/conditional`.
The flattened name is a single opaque string; the parent/child boundary is **lost** at flatten
time in the toolkit and **not recoverable** by string surgery. `schema_check::json_flag_names`
also matches flat-to-flat — no reverse map exists anywhere in the GUI.

**Distinguishing nested from flat (the crux answer).** The ONLY reliable discriminator is the
closed set of the **5 nested-parent prefixes**: `seed-xor`, `seedqr`, `slip39`, `ms-shares`,
`xpub-search`. A flattened name is nested **iff** it begins with `"<parent>-"` for one of those
5. Match on the **complete parent token + `-`** (not a substring): `seedqr-encode` matches
`"seedqr-"` but not `"seed-xor-"`; there is no ambiguity because the 5 prefixes are distinct
complete strings and **no flat subcommand begins with any of them** (flat set: `convert`,
`bundle`, `verify-bundle`, `restore`, `addresses`, `export-wallet`, `import-wallet`, `inspect`,
`repair`, `decode-address`, `compare-cost`, `derive-child`, `silent-payment`, `word-card`,
`gen-man`, `build-descriptor`, …). `import-wallet`/`export-wallet` are **flat** clap subcommands
(`main.rs`, no `#[command(subcommand)]`), so a hyphen-split would corrupt them — the prefix-set
rule leaves them untouched.

### Confirm both run AND copy paths are affected

`assemble_argv_with_secret_mask` (`invocation.rs:152`) is the **single** source for both:
`assemble_argv` (`:126`) delegates to it, and `app_window.rs:898` calls it once — the result
feeds Run (`spawn_and_capture`, `app_window.rs:1038/1094`) **and** the Copy-command display
(`render_copy_command*`, `app_window.rs:939`). So argv[1] = the flattened name today corrupts
**both** the spawned subprocess AND the "Copy command" text. Fixing the assembler fixes both at
once.

### Fix approach (F5)

Split argv[1] in the assembler. In `assemble_argv_with_secret_mask`, replace the single push at
`invocation.rs:161-162`:
```rust
argv.push(subcommand.name.to_string());  mask.push(false);
```
with a push of the **nested token sequence**. Cleanest options, in order of preference:

- **(A) Prefix-set split helper (recommended, GUI-only, minimal).** A `const NESTED_PARENTS:
  &[&str] = &["seed-xor","seedqr","slip39","ms-shares","xpub-search"]` + a
  `fn subcommand_argv_tokens(name) -> Vec<&str>` that, if `name` starts with `"<p>-"`, returns
  `[p, &name[p.len()+1..]]`, else `[name]`. Push each token (+ `mask.push(false)` per token).
  Mask parallelism preserved (both tokens non-secret). Localised, no schema churn.
- **(B) Explicit schema field.** Add `argv_path: &'static [&'static str]` (or
  `nested_parent: Option<&'static str>`) to `SubcommandSchema`; the 12 nested entries set it, the
  rest default `&[name]`. Self-documenting + drift-proof, but Rust `const` struct literals force
  editing **all ~60** `SubcommandSchema` entries (mechanical churn).
- **(C) Toolkit `gui-schema` v6 emits the argv path** (single source of truth in the clap tree),
  GUI mirrors it. Most robust long-term but requires a toolkit release + schema-version bump +
  GUI-pin bump — **out of scope** for a GUI-runner fix.

Recommend **(A)** with a co-located test asserting the split set == the 5 known parents (a
tripwire that fires if a future toolkit adds a 6th nested group — mirror it in the same PR).

### Tests the F5 fix must UPDATE (they assert the flattened token today)

- `tests/xpub_search_widgets.rs:78,154,224,306` — `assert!(argv.contains(&"xpub-search-path-of-xpub"…))`
  etc. After the fix argv holds two tokens `["xpub-search","path-of-xpub"]`, so
  `contains(&"xpub-search-path-of-xpub")` (single flattened token) **fails**. Update to assert the
  adjacent pair (`argv.windows(2).any(|w| w == ["xpub-search","path-of-xpub"])`).
- `schema_mirror` is **unaffected**: it keys off `sub.name` (still the flattened lookup key), not
  the assembled argv.

---

## F6 — virgin dropdowns materialize `opts[0]` → false-negative ownership — **ACCURATE (reproduced live)**

### Per-claim verification

| Claim | Verdict | Evidence |
|---|---|---|
| Rendering a form writes `Dropdown(opts[0])` into `state.values` | **ACCURATE** | `src/form/widget.rs:220-228`: virgin flag → `default_flag_value_for_flag(flag)` pushed into `state.values`. For a `Dropdown` with `default_value:None`, `flag_defaults.rs:77-79 → default_flag_value_for` returns `Dropdown(opts.first())` (`flag_defaults.rs:30-32`) = `opts[0]`. Cited `:220-229`, exact. |
| `default_value:None` ⇒ `is_at_default` can't suppress | **ACCURATE** | `invocation.rs:45-48`: `None` default ⇒ `is_at_default` returns `false` ⇒ `emit_one` emits (`invocation.rs:398,415-421`). Only escape is `opts[0]==""` (the Dropdown arm's `if !v.is_empty()`). |
| `address-of-xpub` emits `--address-type p2pkh --network mainnet` untouched | **ACCURATE** | `XPUB_SEARCH_ADDRESS_OF_XPUB_FLAGS` (`schema/mnemonic.rs:3182`): `--address-type` (`:3243`, `Dropdown(XPUB_SEARCH_ADDRESS_TYPES)`, `default_value:None`) → materialises `"p2pkh"` (const `:2846 = ["p2pkh","p2sh-p2wpkh","p2wpkh","p2tr"]`); `--network` (`:3256`, `Dropdown(NETWORKS)`, `default_value:None`) → materialises `NETWORKS[0]="mainnet"` (`:29`). |
| Toolkit gives explicit flags precedence over inference, no agree-check ⇒ **exit 4** | **ACCURATE** | Reproduced below. |

### Live reproduction (BIP-84 vector; `zpub…`, receive-0 `bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu`; vector from `crates/mnemonic-toolkit/tests/cli_network_fail_open.rs:63-64`)

```
(A) INFERENCE (virgin-correct, NO --address-type/--network):
    → exit 0   "match: bc1qcr8… → 0/0  (script_type=p2wpkh, chain=external, index=0)"
(B) GUI-MATERIALIZED opts[0]  (--address-type p2pkh --network mainnet):
    → exit 4   "no match: bc1qcr8… (searched 0/0..19 + 1/0..19)"
               "error: no match in searched set: mode=address-of-xpub, paths searched=40; …"
(C) --network mainnet ALONE (matches the mainnet zpub → F3 does NOT fire):
    → exit 0
```

(B) is the eval's exact false-negative ("searched 0/0..19 + 1/0..19", exit 4). Confirmed: the
corruption is driven by **`--address-type p2pkh`** overriding the inferred `p2wpkh`; the
materialised `--network mainnet` is inert here because it agrees with the mainnet zpub.

### Coupling / latency vs F5 — **ACCURATE**

F6 is **latent behind F5**: today the run path dies at clap **exit 64** (F5) before the toolkit
sees the materialised flags, so the exit-4 false-verdict never surfaces via **Run**. But the
wrong `--address-type p2pkh --network mainnet` **already rides "Copy command" today** (same argv,
`app_window.rs:898/939`), and Run goes fully live the instant F5 is fixed. **Fix F6 in the same
PR as F5** (eval's directive), else fixing F5 activates a silent wrong-verdict.

### F6 / F3 (v0.83.0 network fail-close) interaction — clarified

- **Mainnet zpub** (the proven case): materialised `--network mainnet` *agrees* → F3 does **not**
  fire (repro C, exit 0); the exit-4 false-negative stands, driven purely by `--address-type
  p2pkh`. F3 does **not** mask F6.
- **Testnet-relevant search** (tpub/vpub target, or a materialised network that mismatches):
  post-F3 the materialised `--network mainnet` now **errors** (exit 2 `NetworkMismatch`, per
  `cli_network_fail_open.rs`) rather than silently false-negativing — a *different* loud failure,
  but still blocks the correct ownership verdict. Either way the virgin-materialised opts[0]
  flags corrupt the search; the fix (emit neither flag untouched) is correct under both.

### The `(none)`-sentinel mechanism (shipped v0.57.0 for restore) and how F6 differs

Mechanism (identical): a `""` entry in the Dropdown's choice list stores as `Dropdown("")`,
which (a) **displays** as `(none)` via `display_or("(none)", "")` (`widget.rs:528,535`), and (b)
**emits nothing** — `emit_one`'s Dropdown arm guards `if !v.is_empty()` (`invocation.rs:415`), so
no token, no copy-command artifact (proven by `tests/restore_template_none.rs` +
`tests/export_wallet_template_none.rs`).

**Critical distinction the SPEC must nail — PREPEND, not APPEND.** Restore/export-wallet
**APPENDED** `""` (`RESTORE_TEMPLATES` `schema/mnemonic.rs:141-153`, `opts.last()==""`,
`opts[0]="bip44"` **kept as a real virgin default**) — because `bip44` *is* a sensible default and
`(none)` is an opt-in escape hatch for md1 mode. `tests/restore_template_none.rs:264-313` even
pins this as a **PREPEND tripwire**. F6 needs the **opposite**: for `address-of-xpub`
`--address-type`/`--network` there is **no safe default** (the toolkit must infer from the
SLIP-0132 prefix), so `""` must be at **`opts[0]`** so the virgin form materialises
`Dropdown("")` → emits **nothing**.

### F6 fix approach + scope

Introduce **two GUI-only choice consts with `""` prepended**, pointed at by the two
`address-of-xpub` flags (keep `default_value:None`; the empty seed is argv-identical to "no flag"):
```rust
const XPUB_SEARCH_ADDRESS_TYPES_INFER: &[&str] = &["", "p2pkh", "p2sh-p2wpkh", "p2wpkh", "p2tr"];
const NETWORKS_INFER:                  &[&str] = &["", "mainnet", "testnet", "signet", "regtest"];
```
Do **not** mutate the shared `NETWORKS`/`XPUB_SEARCH_ADDRESS_TYPES` consts — `NETWORKS` is used by
15 `Dropdown(NETWORKS)` sites, most of which *want* the concrete default. Only `address-of-xpub`'s
two dropdowns switch to the `_INFER` variants.

**Scope decision for the SPEC — surgical vs broad.** A schema scan found the `opts[0]`
materialisation is a **general class**: ~20 non-repeating `Dropdown` flags carry
`default_value:None` and no `""` sentinel (e.g. `--template TEMPLATES`, `--format`/`--to`/
`--script-type`/`--application`/`--electrum-version`/`--multisig-path-family`, plus 6 more
`--network NETWORKS` sites), each of which emits its `opts[0]` untouched. **Only `address-of-xpub`
is a *proven* funds-relevant bug** (the toolkit **infers** there; elsewhere `opts[0]` is usually
the toolkit's own default ⇒ benign noise). Discriminating criterion: a dropdown needs the
`""`-prepend **iff the toolkit has no default and infers** (so emitting `opts[0]` overrides
inference). Recommend:
- **This cycle (coupled with F5): surgical** — the 2 `address-of-xpub` dropdowns only (matches the
  eval, minimal blast radius).
- **Follow-up: a broad `Dropdown+None` materialisation audit** — enumerate the ~20 and classify
  each opts[0] as "== toolkit default (benign)" vs "overrides inference (bug)"; file as a FOLLOWUP.

---

## Eval §2 #13 — `schema_mirror` defaults-drift test — **clean add, but must tolerate GUI-only `""` sentinels**

- Where: mirror is `src/schema/mnemonic.rs`; the existing gate is `tests/schema_mirror.rs` (flag
  **NAMES** only, via `schema_check::json_flag_names` reading the pinned toolkit's `gui-schema`
  JSON; binary from `MNEMONIC_BIN` / `pinned-upstream.toml`). The toolkit `gui-schema` v5 JSON
  **does** carry per-flag `default_value` (`gui_schema.rs:249`) and dropdown `choices`
  (`:243,1266-1269`) — so a `schema_mirror_defaults_drift.rs` asserting the hand-mirror's
  `default_value`/`choices` == the live JSON is a **clean add** (same JSON already parsed).
- **Caveat:** the GUI deliberately diverges from toolkit `choices` for display sentinels — the
  `""` `(none)` rows in `RESTORE_TEMPLATES`, `EXPORT_WALLET_TEMPLATES`, `ARCHETYPES`, and (post-F6)
  the two `_INFER` consts are **GUI-only**. A choices-drift assertion must **strip `""` before
  comparing** (compare `choices.filter(|c| !c.is_empty())`), else it red-flags every sentinel.
- Placement: extend `schema_check` with a `json_flag_defaults`/`json_flag_choices` accessor
  (mirrors `json_flag_names`), then a new integration test. Modest effort; couplable if low-cost,
  but **not required** to fix F5/F6.

---

## Kittest test plan (F5 + F6)

The I1 form→argv round-trip pattern lives in `tests/ui_harness_i1_roundtrip.rs` (+ helpers in
`tests/ui_harness/`): render the real per-flag path via `render_with_dispatch`, widget-inject a
value, `run()` to settle, `assemble_argv`, assert the value is argv-bound. `tests/xpub_search_widgets.rs`
and `tests/restore_template_none.rs` are the closest existing templates. Add:

**F5 (nested-argv):** for each of the 12 nested subcommands, render a filled form → `assemble_argv`
→ assert argv[1..3] == the **nested pair** (e.g. `["seed-xor","combine"]`, `["xpub-search",
"address-of-xpub"]`), and assert argv[1] is a bare parent token (never the flattened
`seed-xor-combine`). Plus a `render_copy_command` cell asserting the Copy string starts
`mnemonic seed-xor combine …`. Plus the split-helper unit tripwire (parents == the 5 known; flat
names like `import-wallet` pass through unsplit).

**F6 (virgin dropdown emits nothing):** mirror `tests/restore_template_none.rs`. On a **virgin**
`address-of-xpub` form (render + settle, no interaction): assert `assemble_argv` carries **no**
`--address-type` and **no** `--network` token, and the masked copy-command (both flavors) carries
neither + no `""` artifact. Add the inverse: selecting a real value (e.g. `p2wpkh`) **does** emit.
Add a **PREPEND census** (opposite of restore's APPEND pin): `XPUB_SEARCH_ADDRESS_TYPES_INFER[0]==""`,
`NETWORKS_INFER[0]==""`, virgin settled form ⇒ `Dropdown("")` for both flags. (Optional I4 real-CLI
cell: virgin address-of-xpub against the BIP-84 vector ⇒ exit 0 match — the end-to-end anti-regression.)

---

## Release / CI surface (mnemonic-gui)

- **Current version 0.57.0** → bump to **`mnemonic-gui-v0.58.0`** (behavior fix touching the run
  surface; MINOR). Fix is **GUI-only**; **no toolkit-pin bump** (Cargo pin stays
  `mnemonic-toolkit-v0.75.0` at `Cargo.toml:76`; `pinned-upstream.toml` `tag = v0.75.0`
  unchanged — F5/F6 are runner/schema-data fixes, not clap-surface changes).
- **PR + CI, not direct-FF** (GUI convention). Branch protection (just set by user):
  contexts `[snapshots, clippy, headless (no-default-features), schema-mirror gate,
  x86_64-unknown-linux-gnu]`, `enforce_admins:false`. Jobs live in `.github/workflows/build.yml`
  (`clippy`, `headless (no-default-features)`, `snapshots`, the `cargo check --locked` =
  `x86_64-unknown-linux-gnu`) + `.github/workflows/schema-mirror.yml` (`schema-mirror gate`). Ship
  via a PR (or admin bypass push, `enforce_admins:false`) + the version tag.
- **No `cargo fmt`** on GUI (project rule); **clippy `-D warnings`** is gated (both default and
  `--no-default-features`) — the split helper + new consts must be clippy-clean, and any new
  test-only helpers behind `#![allow(dead_code)]` per harness convention.
- **schema_mirror stays green** (F5 doesn't change `sub.name`; F6's `""`-prepend changes choices,
  which the NAME gate ignores). If eval #13's defaults-drift test is added this cycle, it must
  strip `""` sentinels (above).

---

## SUMMARY (≤250 words)

**Both F5 and F6 are ACCURATE and reproduced live** against `mnemonic v0.84.0`. **F5:** the GUI
run/copy assembler (`invocation.rs:161`) pushes the flattened `SubcommandSchema.name` as one argv
token; the toolkit CLI is nested (5 `#[command(subcommand)]` parents), so all 12 nested-mode forms
emit e.g. `mnemonic seed-xor-combine …` → **exit 64 unrecognized subcommand** (verified); the
nested `seed-xor combine` works. The toolkit flattens as `format!("{parent}-{child}")` and both
sides contain hyphens, so a naive hyphen-split corrupts `ms-shares`/`seed-xor`/`xpub-search-*` AND
would break flat `import-wallet`. **Crux:** no structural marker survives; the only reliable split
is the closed 5-parent-prefix set (`seed-xor, seedqr, slip39, ms-shares, xpub-search`). Fix =
split argv[1] in the assembler (recommend a prefix-set helper); this fixes Run **and** Copy at
once. Must update `xpub_search_widgets.rs` assertions. **F6:** virgin non-repeating dropdowns with
`default_value:None` materialize `opts[0]` and emit it (`is_at_default` can't suppress); proven for
`address-of-xpub` — materialized `--address-type p2pkh --network mainnet` turns an exit-0
`script_type=p2wpkh` match into **exit 4 "no match"** (false-negative ownership; reproduced). F6 is
latent behind F5 but already rides Copy-command; **fix in the same PR.** Fix = the v0.57.0 `(none)`
sentinel, but **PREPEND `""` at opts[0]** (opposite of restore's APPEND) via two GUI-only `_INFER`
choice consts. Scope surgically to address-of-xpub; file the broader `Dropdown+None` class as a
follow-up. Ships GUI-only as **`mnemonic-gui-v0.58.0`** via PR+CI (5 checks), **no toolkit-pin
bump**. Eval #13 defaults-drift test is a clean add but must strip `""` sentinels.
