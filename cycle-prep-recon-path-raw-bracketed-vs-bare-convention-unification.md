# cycle-prep recon — 2026-05-28 — path-raw-bracketed-vs-bare-convention-unification

**Origin/master SHA at recon time:** `dd7c228`
**Local branch:** `master`
**Sync state:** `up-to-date (0 ahead / 0 behind)`
**Untracked:** `.claude/` (only)

Slug verified: `path-raw-bracketed-vs-bare-convention-unification`. **Verdict: citations individually ACCURATE-but-DRIFTED, but the SCOPE FRAMING is structurally wrong — the FOLLOWUP names ONE bracketed producer (`mk1_card_to_resolved_slot`) when there are ~8.** Approach (a) as literally scoped would fix 1 of 8 and leave the convention *more* inconsistent, not less.

---

## Per-slug verification

### `path-raw-bracketed-vs-bare-convention-unification`

- **WHAT (from FOLLOWUPS.md):** `ResolvedSlot.path_raw` is overloaded — bracketed `[fp/path]` from the envelope/import path vs bare derivation path from `resolve_slots`. F5 (v0.37.7) was band-aided at the export-wallet boundary; unify the underlying convention. A latent cosmetic bug: `bundle --import-json --json` emits `multisig.cosigners[].origin_path` of shape `"m/[fp/path]"`.

- **Citations:**

  - `wallet_import/json_envelope.rs:282` (`mk1_card_to_resolved_slot` producing bracketed `path_raw`) — **DRIFTED-by-~50.** Function now at `json_envelope.rs:332`; the bracketed populate is `let path_raw = format!(\n "[{}/{}]", ...)` at **`:350-351`**; `ResolvedSlot { … path_raw … }` literal at `:358`. The bracketed-producer claim is ACCURATE.

  - `cmd/bundle.rs:547,628` (`resolve_slots` producing BARE `path_raw`) — **DRIFTED (partial).** `fn resolve_slots` now at `bundle.rs:427`. Bare populates: `let path_raw = path.to_string();` at **`:503`** and **`:628`** (`:628` still exactly matches), plus the multisig branch `let (path, path_raw) = match …` at `:539` with `path_raw,` field at `:585`. The cited `:547` drifted into the `:539–585` block. Bare-producer claim ACCURATE.

  - `bundle.rs:767,1000-1004` (latent cosmetic: polluted `origin_path`) — **ACCURATE + EMPIRICALLY CONFIRMED LIVE.** `:767` is `origin_path: normalize_origin_path(&s.path_raw)` inside the `CosignerEntry` map (the `n != 1` multisig JSON branch); `normalize_origin_path` (`:731`) prepends `m/` to any string not already starting `m/`, so a bracketed `[fp/path]` becomes `"m/[fp/path]"`. `:1000-1003` is the `SlotCardBlock` branch `origin_path: if s.path_raw.is_empty() { None } else { Some(s.path_raw.clone()) }` (no normalize → would emit bare `"[fp/path]"`). **Live run** (`./target/release/mnemonic bundle --import-json tests/fixtures/wallet_import/envelope_v0_27_0.json --network mainnet --json`) yields `multisig.cosigners[].origin_path = "m/[b8688df1/48'/0'/0'/2']"` (×3). The fingerprint is *also* duplicated in the sibling `master_fingerprint` field, so the embedded fp is redundant pollution.

  - `import_wallet.rs::origin_path_from_bracket` callers — **ACCURATE.** Def at `import_wallet.rs:1945`; callers at `:1409` and `:1431`. It strips `[fp/…]` → `m/…`. **Ripple hazard (NEW finding):** its impl assumes the inner content begins with `fp` then `/path` (`inner.find('/')` → `format!("m{}", &inner[slash..])`). It is roughly idempotent for input already starting `m/`, but for a *bare-without-`m`* form like `48'/0'/0'/2'` it returns `m/0'/0'/2'` (drops `48'`). So "make producers bare" is not consumer-transparent — the exact bare form and this consumer must be co-designed.

  - `coldcard_multisig.rs:658` (framed as a "bracket-consumer to update") — **STRUCTURALLY-MISLEADING.** `:658` is `format!("{}{}/<0;1>/*", c.path_raw, c.xpub)` where `c` is a *local* parser struct (`path_raw` field declared `:534`, populated bracketed at `:422`). This builds a **descriptor key fragment** `[fp/path]xpub/<0;1>/*` — a legitimate, *required* bracketed use that should STAY bracketed. More importantly, the FOLLOWUP MISSED that the same file is itself a **bracketed PRODUCER of `ResolvedSlot.path_raw`**: `coldcard_multisig.rs:485` builds `ResolvedSlot { … path_raw: c.path_raw.clone() … }` (`:489`) copying the bracketed local form straight into the slot. So coldcard-multisig is an *unlisted producer*, not just a consumer.

- **Action for brainstorm spec:** Refresh all line numbers to SHA `dd7c228` (`mk1_card_to_resolved_slot` → `:332`/populate `:350-351`; `resolve_slots` → `:427`/bare `:503`,`:628`; cosmetic emit `:767` + `:1000-1003`; `origin_path_from_bracket` def `:1945` callers `:1409`,`:1431`). **Re-scope from "fix `mk1_card_to_resolved_slot`" to "unify a convention used by ~8 producers."** Treat the `origin_path_from_bracket` bare-form sensitivity and the *legitimate* descriptor-key bracket use (`coldcard_multisig:658`, plus descriptor synthesis) as first-class constraints, not afterthoughts. Cite source SHA `dd7c228`.

---

## Cross-cutting observations

1. **The bracketed convention is the DOMINANT one, not the exception.** Bracketed `[fp/path]` is produced into `ResolvedSlot.path_raw` by *every* import/foreign-format parser, ~8 sites total:
   `json_envelope.rs:350-351` (mk1 re-decode at bundle time) · `coldcard_multisig.rs:422→489` · `specter.rs:387` · `sparrow.rs:607` · `coldcard.rs:534` · `electrum.rs:946` · `bitcoin_core.rs:438` · `bsms.rs:389`.
   The BARE convention is produced only by `bundle.rs::resolve_slots` (`:503`,`:585`,`:628`) — the seed-/descriptor-mode path. So the overload is: **import surface = bracketed; native synth = bare.** The FOLLOWUP's "fix the one producer" framing undercounts producers **1 → ~8** — directly the `feedback_fix_the_class_hunt_for_second_instance` lesson. A partial fix (only `mk1_card_to_resolved_slot`) would make 1 producer bare while 7 stay bracketed → consumers could assume *neither* → strictly worse.

2. **`path_raw` is genuinely serving two semantically distinct needs**, which is why a blanket "strip to bare" is unsafe: (a) the **origin_path JSON field** wants bare `m/…`; (b) **descriptor-key construction** (`coldcard_multisig:658`; likely `synthesize.rs` origin annotations) wants the bracketed `[fp/path]` form. This is the actual design tension. It points toward FOLLOWUP **option (b) — a typed wrapper** (carry `fingerprint` + bare `DerivationPath` separately, render bracketed/bare on demand) being the structurally-correct fix, over option (a)'s string-normalization. The brainstorm should weigh (a) "normalize all producers to bare + a `bracketed()` render helper for descriptor consumers" vs (b) "typed origin field, delete the overloaded string."

3. **No wire-shape break required for the cosmetic fix.** The cosigner JSON entry already emits `master_fingerprint` separately; correcting `origin_path` from `"m/[fp/path]"` to `"m/48'/0'/0'/2'"` is a *bug-fix* of an unasserted field (no test pins the polluted form — confirmed: `cli_bundle_import_json.rs` Cell 2 asserts only fingerprints + count). It is still a user-visible JSON output change → behavior-change PATCH; own R0; GUI `--json` wire-shape is NOT schema_mirror-gated (CLAUDE.md: gate is flag-NAME parity only), so the GUI consumer self-updates via the paired-PR rule (worth a heads-up, not a hard lockstep).

4. **F5 boundary fix interaction.** `export_wallet.rs:647` (`s.path_raw = format!("m/{}", s.path)`) is the v0.37.7 band-aid. If producers are unified to bare, this boundary normalization becomes redundant and should be removed in the same cycle (else double-normalization risk). The brainstorm must enumerate it as a fold target.

5. **Verification debt to retire in-cycle.** The cosmetic pollution is currently unasserted by ANY test. Whatever the fix, add a regression cell pinning `bundle --import-json --json` cosigner `origin_path` to the clean bare `m/…` form (single-sig path at `:755`/`origin_path_for_json` too), plus the `SlotCardBlock` branch (`:1000-1003`).

6. No crypto/protocol claims in this slug (pure internal-representation refactor) — nothing to verify against a primary spec.

---

## Recommended brainstorm-session scope

- **One cycle, but bigger than the FOLLOWUP implies.** Not a 1-function edit; it touches the ~8 bracketed producers + the consumer contract. Rough sizing: producer normalization is mechanical (~8 small edits) but the *design decision* (option (a) string-normalize-to-bare + render helper, vs option (b) typed origin field) is the load-bearing work and gates LOC. Option (a): ~150–300 LOC incl. tests. Option (b): larger (touches the `ResolvedSlot` struct + every literal site + serde) — likely 400+ LOC and closer to a `v0.37+-refactor` proper.
- **SemVer:** PATCH if internal-only + the only user-visible change is the `origin_path` cosmetic correction (no flag/subcommand/value change). `ResolvedSlot` is `pub(crate)` (`synthesize.rs:591`) so a typed-field change is not a public-API break. Confirm no `pub` re-export before locking (likely PATCH either way).
- **Locksteps:**
  - GUI `schema_mirror`: **NOT triggered** (no clap flag-NAME/value/subcommand change).
  - Manual mirror (`docs/manual/src/40-cli-reference/`): **NOT triggered** by code, BUT if any chapter-45/40 prose shows a `bundle --import-json --json` sample with the polluted `origin_path`, that sample must be re-captured (check `transcripts/` + `verify-examples` corpus — the manual-prose gate will catch drift).
  - Sibling-codec FOLLOWUP companions: **none** — this is toolkit-internal (`md-codec`/`mk-codec` not touched).
- **Ordering / dependencies:** (1) Decide option (a) vs (b) at brainstorm. (2) Unify ALL ~8 producers in lockstep (not just one). (3) Remove the `export_wallet.rs:647` boundary band-aid. (4) Audit consumers: keep the bracketed render for descriptor-key sites (`coldcard_multisig:658`, `synthesize.rs`), bare for `origin_path` JSON. (5) Add the missing regression cells (obs. 5). No inter-slug dependency; standalone cycle.
- **Mandatory R0 gate** (CLAUDE.md): brainstorm spec + plan-doc must reach opus-architect 0C/0I before ANY code. The option-(a)-vs-(b) decision and the consumer-contract table are exactly what R0 should stress.
