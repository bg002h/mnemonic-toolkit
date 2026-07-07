# Cycle-Prep Recon — `bip389-double-star-shorthand-support`

**P0 STRICT-GATE recon. Recon only — no implementation, no source edits.**

## Header

- **origin/master SHA:** `0964462d` (`design: flip bitcoin-core-receive-change-pair-merge → RESOLVED (v0.77.0) + GUI-companion note`)
- **Local branch:** `master`
- **Sync state:** `git rev-list --left-right --count HEAD...origin/master` → `0  0` (local `master` == `origin/master`, fully synced)
- **Shipped tip:** `mnemonic-toolkit-v0.77.0` (tag on `44e55c4e`; `0964462d` is a 1-line FOLLOWUP-status flip on top, untagged) — Cycle B (Bitcoin Core receive/change pair-merge) just shipped, confirming the "Cycle B shipped v0.77.0" premise.
- **Untracked note:** 38 untracked files in the worktree (all pre-existing `cycle-prep-recon-*.md` / `design/*.md` / `design/agent-reports/*.md` / `docs/manual-gui/design/agent-reports/*.md` scratch artifacts from prior sessions — none touched by this recon, none relevant to the `bip389-double-star-shorthand-support` slug).

---

## Slug: `bip389-double-star-shorthand-support`

### WHAT (FOLLOWUPS.md:44-47)

> Accept the BIP-389 `/**` combined shorthand (expand to `<0;1>/*`) instead of rejecting it. Surfaced 2026-07-06 during `descriptor-use-site-collapse` (plan-R0 I-D). Since v0.76.0 `xpub/**` hard-fails. Status OPEN — `/**` is claimed as "the BIP-389 canonical combined-descriptor shorthand emitted by common wallets (Sparrow/Nunchuk/Core `doc/descriptors.md` combined form)"; today it hard-fails with a `<0;1>/*` workaround pointer. plan-R0 I-D flagged this as possibly HIGHER user-impact than the pair-merge follow-up (which itself just shipped as v0.77.0). Fix: "a pre-lex expansion `xpub/**` → `xpub/<0;1>/*` (or lexer support for `/**` as an alias)". Tier: toolkit.

### Citation-by-citation verification (against `0964462d`)

**1. "Since v0.76.0 `xpub/**` hard-fails (the `wild` group eats `/*`, leaving residue `*` → reject)."**
**ACCURATE.**
- `lex_placeholders` — `crates/mnemonic-toolkit/src/parse_descriptor.rs:60`.
- Regex (all-named capture groups) — `parse_descriptor.rs:97-99`:
  ```
  r"(?:\[(?P<pfx_fp>[0-9a-fA-F]{8})(?P<pfx_path>(?:/\d+(?:'|h)?)*)\])?@(?P<idx>\d+)(?:\[(?P<sfx_fp>[0-9a-fA-F]{8})(?P<sfx_path>(?:/\d+(?:'|h)?)*)\])?(?:/<(?P<mpath>[^>]*)>)?(?P<wild>/\*(?:'|h)?)?"
  ```
  The `wild` group is `/\*(?:'|h)?` — matches `/*`, `/*'`, `/*h` only (2-3 chars). Against a trailing `/**`, `wild` consumes exactly `/*`, leaving one stray `*` byte unconsumed.
- Terminator/residue-reject block — `parse_descriptor.rs:183-213` (doc-comment 183-201, code 202-213). The check (line 203-204): after the match, if the next char is not `)`/`,`/`}`/whitespace, reject. The stray `*` fails this, so the residue-reject fires.
- **Exact mechanism confirmed via git history too** — `design/agent-reports/cycleA-plan-r0-round-2.md:13` (persisted verbatim from the Cycle A plan-R0 loop) already nails the full mechanism: `concrete_keys_to_placeholders`'s `push_str(&descriptor[last_end..])` (**still exactly `pipeline.rs:391`** — re-grepped live, zero drift since Cycle A despite the intervening v0.77.0 cycle) preserves any residual text (including a trailing `/**`) verbatim into the `@N`-placeholder form; `lex_placeholders`'s `wild` capture then eats only `/*`, leaving `*` as residue → reject. "Sparrow excepted (self-expands before lexing)" — confirmed independently below.
- Shipped in commit `be280c00` ("impl: cycleA Phase 1 — residue-reject floor + 22-cell migration (GREEN)"), 2026-07-06.
- Regression tests exist on BOTH layers: unit `lex_rejects_double_star_shorthand` (`parse_descriptor.rs:1731-1738`) and CLI-integration `descriptor_double_star_shorthand_rejected_with_multipath_remedy` (`crates/mnemonic-toolkit/tests/cli_import_wallet_descriptor.rs:191-217`, asserting exit 2 + stderr contains `descriptor`, `/**`, `multipath`, `<a;b>`).

**2. "The current reject MESSAGE already names `/**`."**
**ACCURATE.** Exact quote, `parse_descriptor.rs:206-211`:
```
"@{i}: derivation steps after the placeholder are not representable in md1; \
 the use-site path must be a multipath `/<a;b>/*` (or bare `/*`) as the final \
 step — a fixed single step like `/0/*` (or the `/**` shorthand) is \
 un-representable (found residue near `{residue}`)"
```

**3. Fix-direction claim: "a pre-lex expansion `xpub/**` → `xpub/<0;1>/*` (or lexer support for `/**` as an alias)".**
**PARTIALLY ACCURATE — needs significant refinement; the FOLLOWUP undercounts both the existing prior art and the true blast radius.**

- **The pre-lex-expansion pattern is not hypothetical — it is ALREADY SHIPPED, for one specific intake shape.** `wallet_import/pipeline.rs::expand_bip388_policy` (`pipeline.rs:282-302`) does EXACTLY this substitution today for BIP-388 wallet-policy JSON (`--format bip388`, and internally for Sparrow/Specter native JSON `description_template`/`defaultPolicy.miniscript.script` fields): `@N/**` → `keys_info[N] + "/<0;1>/*"`, called BEFORE the string ever reaches `lex_placeholders` (`verify_bundle.rs:1337-1341`, `bundle.rs` equivalent, `descriptor_intake.rs:195` for xpub-search). This is why the manual (`docs/manual/src/40-cli-reference/41-mnemonic.md:155-156, 166-168`) states "**Sparrow is unaffected** — Sparrow's own `@N/**` template placeholder is expanded to the multipath form internally before this check runs." Confirmed live: `wallet_export/sparrow.rs:196-199` (`CliTemplate::Bip84 => "wpkh(@0/**)"` etc. — the toolkit's OWN Sparrow exporter emits `/**`), `wallet_import/sparrow.rs` / `specter.rs` / `pipeline.rs::is_bip388_policy_shape` (all handle `@0/**` routinely; dozens of `/**` occurrences across `tests/cli_import_wallet_sparrow*.rs`, `tests/cli_bip388_policy_intake.rs`, `tests/cli_export_wallet*.rs`).
- **The FOLLOWUP's actual, narrower gap:** a BARE/literal descriptor-text surface — a concrete `[fp/path]xpub…/**` with NO BIP-388-JSON wrapper — funneled through `concrete_keys_to_placeholders` (`pipeline.rs:330-400`) → `lex_placeholders`. This is the shape the two existing reject-tests exercise, and the shape that needs a NEW pre-lex expansion (or `wild`-group extension) since `expand_bip388_policy` never sees it (it isn't JSON).
- **Two viable hook points**, worth the brainstorm resolving explicitly:
  (a) a string pre-pass inside `concrete_keys_to_placeholders` recognizing a literal `/**` immediately after an xpub match and rewriting it to `/<0;1>/*` before the placeholder substitution runs; or
  (b) extending the `lex_placeholders` regex's `wild` alternation to recognize `/\*\*` directly and, on that branch, synthesizing `multipath_alts = vec![0, 1]` in the same code that currently reads the `mpath` capture (`parse_descriptor.rs:146-178`) instead of populating `wildcard_hardened` alone. Option (b) is more central/single-sourced (touches one function, is exercised by every caller of `lex_placeholders` uniformly) and is architecturally closer to how the v0.76.0 Cycle-A fix itself was framed (fail-closed inside the lexer). Option (a) mirrors the `expand_bip388_policy` precedent but adds a second parallel expansion site.
- **Caller-surface audit (grepped `lex_placeholders`/`parse_descriptor(` call sites against `0964462d`) — the FOLLOWUP's claimed surface list is INCOMPLETE and one entry is WRONG:**
  | Surface | Path today | In scope for a `lex_placeholders`-level fix? |
  |---|---|---|
  | `bundle --descriptor` / `--import-json` replay | `concrete_keys_to_placeholders` → `lex_placeholders` (`cmd/bundle.rs:1389,1408,1723`) | Yes |
  | `import-wallet --format descriptor` | same pipeline (`wallet_import/descriptor.rs:68`) | Yes |
  | `import-wallet --format bsms` | same pipeline (`wallet_import/bsms.rs:227`) — BSMS Round-2's plaintext descriptor line is concrete-key text, not JSON | Yes |
  | `import-wallet --format bitcoin-core` | pair-merge pre-pass (`wallet_import/bitcoin_core.rs`) then same pipeline | **Moot for `/**` specifically** — the v0.77.0 cycle's own verified finding is that Core *never* emits `/**` or combined multipath at all (only split `/0/*` + `/1/*`, confirmed vs. bitcoin/bitcoin PR #22838 per `CHANGELOG.md` v0.77.0 entry) |
  | `import-wallet --format specter` | Specter's native JSON also uses BIP-388 `@0/**` — **already handled** via `is_bip388_policy_shape`/`expand_bip388_policy` (confirmed fixture `wallet_import/specter.rs:643`) | Already works — no fix needed |
  | `verify-bundle --descriptor` | BIP-388-JSON branch already expands (`cmd/verify_bundle.rs:1332-1341`); the **concrete/"bare-concrete fork"** branch (`classify_descriptor_form == Concrete` → `descriptor_concrete_to_resolved_slots`, `pipeline.rs:406+`) also funnels through `concrete_keys_to_placeholders` | Yes, same gap |
  | `xpub-search account-of-descriptor --descriptor` | **STRUCTURALLY SEPARATE parser** — `descriptor_intake.rs::parse_literal_xpub` (line 291-298) calls `miniscript::Descriptor::<DescriptorPublicKey>::from_str` **directly**, entirely bypassing `lex_placeholders`/`concrete_keys_to_placeholders`. Its BIP-388-JSON funnel (`parse_bip388_json`, line 189-199) already delegates to the shared `expand_bip388_policy` and therefore already works. | **NOT reached by a `lex_placeholders`-scoped fix** for the bare literal-xpub-text case — needs an independent touch in `descriptor_intake.rs` (or a shared pre-expansion helper both call). No existing test exercises this exact shape (a literal `--descriptor` string with concrete xpub + `/**`, no JSON wrapper, via `xpub-search`); `rust-miniscript 13.0.0`'s own descriptor parser almost certainly rejects raw `/**` with its own non-toolkit error text, but this was **not executed/confirmed** in this recon — flag as a spike item for the brainstorm/plan phase, not asserted as fact. |

  **Correction to the FOLLOWUP's surface list:** it names `import-wallet --format specter` and `bitcoin-core` as affected surfaces requiring the fix; specter's native-JSON path is already unaffected (works today) and bitcoin-core's split-export never emits `/**` at all (moot). The two genuinely-affected, currently-broken surfaces the FOLLOWUP doesn't call out precisely enough are (i) the **bare literal-descriptor-text** shape (`--format descriptor`, `--format bsms`, `bundle --descriptor`, `verify-bundle --descriptor` concrete fork) and (ii) **`xpub-search`'s literal-xpub funnel**, which is architecturally untouched by any `lex_placeholders`-level fix and needs its own pass.

### Primary-source verification (BIP attribution — mandatory per constellation protocol-fact gate)

**STRUCTURALLY WRONG: the FOLLOWUP (and the shipped source it's citing) attributes `/**` to the wrong BIP.**

- **Independently verified via WebFetch of both `bitcoin/bips bip-0389.mediawiki` and `bitcoin/bitcoin doc/descriptors.md`:** neither document defines or mentions `/**` anywhere. BIP-389 ("Multipath Descriptors") defines only the explicit `/<0;1>/*` multipath syntax; Core's `descriptors.md` likewise only shows the explicit form (`XPUB1/<0;1>/*`).
- **Independently confirmed via a second, targeted research pass:** `/**` is defined in **BIP-388** ("Wallet Policies for Descriptor Wallets"), whose Formal Definition section states verbatim: *"The `/**` in the placeholder template represents commonly used paths for receive/change addresses, and is equivalent to `/<0;1>/*`."* Real-world usage confirmed via the original bitcoin-dev mailing-list post (Salvatore Ingala, "Wallet policies for descriptor wallets," May 2022) and Ledger's `app-bitcoin-new/doc/wallet.md` — `/**` is a live, currently-used convention in hardware-wallet / Core wallet-policy tooling (Sparrow, Specter, Ledger), just normatively homed in BIP-388, not BIP-389.
- **The technical equivalence claim itself is CORRECT** (`/**` ≡ `/<0;1>/*`, receive=chain-0 / change=chain-1) — only the BIP-number attribution is wrong.
- **This misattribution is not confined to the FOLLOWUP doc — it is already baked into shipped source, shipped tests, a shipped tagged CHANGELOG entry, and shipped manual prose**, all at `0964462d`:
  - `parse_descriptor.rs:1733` (test doc-comment: "trap #5 / plan-R0 I-D: BIP-389 `/**` shorthand")
  - `tests/cli_import_wallet_descriptor.rs:191` (doc-comment: "The BIP-389 combined `/**` shorthand")
  - `CHANGELOG.md` v0.76.0 entry (main paragraph + "Notes/interim limitations" bullet, both say "BIP-389 combined-wildcard shorthand" / "BIP-389 `xpub/**` shorthand") — **this is a shipped, tagged release changelog entry**; convention in this repo is to correct terminology going forward rather than rewrite tagged history, so the brainstorm should decide explicitly whether to (a) leave the v0.76.0 entry as-shipped and just get the *new* entry right, or (b) add a small addendum note.
  - `docs/manual/src/40-cli-reference/41-mnemonic.md:145` ("is the BIP-389 **combined-wildcard shorthand** `/**`") — note line 141 immediately above, "the BIP-389 **multipath** form `/<a;b>/*`", is CORRECT and must NOT be touched; only the `/**` attribution on line 145 is wrong. Other `/**` mentions in the same file needing the same correction: lines 157, 164, 1253-1255.
  - `docs/manual/src/45-foreign-formats.md:130` ("the `/**` shorthand — rather than the multipath `/<a;b>/*`… is refused") — doesn't itself say "BIP-389" but sits directly beside the reject-documentation and should be reviewed/updated in the same pass regardless.
  - `design/agent-reports/cycleA-phase-1-r0-round-1.md` and `cycleA-plan-r0-round-2.md` (historical review artifacts — leave as historical record, no action needed).
- **Confidence:** HIGH on both the technical equivalence and the misattribution finding (two independent research passes concur, plus direct primary-source WebFetch of both candidate BIPs).

### Action items for the brainstorm spec (cite source SHA `0964462d`)

1. Retitle/reslug the FOLLOWUP and the resulting spec/plan to **BIP-388** (e.g. `bip388-double-star-shorthand-support`), while keeping the old slug greppable as a forwarding note for continuity.
2. Resolve the two-hook-point design question (regex-level `wild`-alternation extension inside `lex_placeholders` vs. a separate string pre-pass in `concrete_keys_to_placeholders`) — recommend (b)/regex extension as the single-sourced option, but let the brainstorm weigh it against the `expand_bip388_policy` precedent already in the codebase.
3. Explicitly scope in: `xpub-search account-of-descriptor`'s literal-xpub funnel (`descriptor_intake.rs::parse_literal_xpub`) as a SEPARATE, independently-necessary touch — do not assume a `parse_descriptor.rs`-only fix reaches it.
4. Explicitly scope out (or at least de-prioritize with a documented reason): `import-wallet --format bitcoin-core` (Core never emits `/**`) and `import-wallet --format specter` native-JSON (already works) — correct the FOLLOWUP's implied surface list.
5. Bundle the BIP-388/BIP-389 misattribution correction into the same cycle (same files/lines are being touched anyway) rather than filing it as a separate follow-up.
6. Both existing reject-tests (`parse_descriptor.rs:1731-1738`, `cli_import_wallet_descriptor.rs:191-217`) must be **repurposed** (flipped from expect-reject to expect-accept-with-expansion), not just supplemented — flag this explicitly per the "tests are spec" convention.

---

## Cross-cutting observations

- **SemVer: MINOR (pre-1.0 convention), with a direct, immediately-preceding precedent.** `CHANGELOG.md`'s own stated convention: "pre-1.0 convention that the second component (`0.X`) is the breaking-change axis." The just-shipped `v0.77.0` (HEAD) is the exact same *shape* of change — accepting a previously-hard-rejected import form (Bitcoin Core's split receive/change pair) via a pre-pass — and was versioned SemVer-MINOR, toolkit-only, codec NO-BUMP. The `/**` fix should follow the identical precedent: MINOR bump, `md-codec`/`mk-codec`/`ms-codec` NO-BUMP (no wire-format change — `UseSitePath`'s existing `{multipath: Option<Vec<Alternative>>, wildcard_hardened}` shape already natively represents `/<0;1>/*`; unlike Cycle A's collapse-fix, this is not adding a new *representable* shape, just accepting a new *spelling* of an already-representable one).
- **Lockstep doc flags — LARGER than the FOLLOWUP implies.** At least 8 manual locations reference `/**` and need review/update in lockstep with the implementing PR, not just the one the FOLLOWUP-recon prompt named:
  - `docs/manual/src/40-cli-reference/41-mnemonic.md` lines 137-168 (the canonical "Non-representable use-site steps" section — the authoritative prose block), 218, 659, 711-713, 1204, 1253-1255, 1466, 3510.
  - `docs/manual/src/45-foreign-formats.md` lines 127-133 (BSMS use-site residue-reject cross-reference).
  - `CHANGELOG.md` — new entry needs to retire the v0.76.0 "Notes/interim limitations" `/**`-hard-fails bullet, mirroring exactly how v0.77.0's own entry retired the Bitcoin-Core-split-pair interim-limitations bullet (good writing template to reuse).
- **`verify-examples`-gated transcripts: none affected.** Grepped `.examples-build/` for `/**` — no hits; the existing two reject-tests are unit/CLI-integration level, not manual golden transcripts, so no doc-transcript flip is expected from THIS repo's example corpus. (The manual prose sections above are hand-written narrative, not golden-gated fences, per the existing `manual-output-blocks-non-gateable-residual` FOLLOWUP taxonomy — categorize any new/changed fence per that taxonomy if one is added.)
- **GUI / `schema_mirror` impact: NONE**, confirmed. No clap flag/subcommand/dropdown-value addition, removal, or rename — this is purely a descriptor-string-parsing/acceptance behavior change. No `mnemonic-gui/src/schema/mnemonic.rs` update required.
- **Scope: toolkit-only**, confirmed — no sibling-codec (`md-codec`/`mk-codec`/`ms-codec`) touch expected, matching the v0.77.0 precedent's NO-BUMP pattern for the identical reason (pure toolkit parse-layer; `UseSitePath` already representable).
- **LOC estimate:** small-to-medium. Core fix (regex/lexer extension in `parse_descriptor.rs`, or pipeline pre-pass): ~20-60 LOC. Extending to the `xpub-search` literal-xpub funnel: ~15-40 LOC (separate touch, likely a shared helper factored out to avoid a third parallel `/**`-detection implementation — worth considering during the brainstorm given there would then be THREE independent `/**`-aware code paths: `expand_bip388_policy`, the new lexer/pipeline extension, and `descriptor_intake.rs`). Repurposing/adding tests across ~6 files (`parse_descriptor.rs` unit tests, `cli_import_wallet_descriptor.rs`, `cli_bundle_*`, `cli_verify_bundle_*`, `cli_xpub_search_account_of_descriptor.rs`) is likely the bulk of the diff. Doc prose across 2 manual files + `CHANGELOG.md` + ~5 misattribution-correction sites. **Total estimate: ~200-350 LOC**, comparable to or somewhat smaller than the Cycle A residue-reject-floor buildout itself.
- **Ordering:** this is a good candidate for the NEXT cycle — it's flagged (by the Cycle A plan-R0 loop itself) as possibly higher user-impact than the pair-merge follow-up that just shipped as v0.77.0, and the pair-merge cycle's `wallet_import/bitcoin_core.rs` pre-pass is a fresh, directly-analogous architectural precedent (extract via rust-miniscript, detect a rewritable pattern, rewrite to canonical form before the lexer) that the team just built and tested — reuse-while-fresh is a good reason to sequence this next rather than let the pattern go cold.

---

## Recommended brainstorm-session scope

**Single-slug cycle** (no other FOLLOWUP grouped in — this one has enough internal complexity: 2 candidate hook-point designs, a previously-unaccounted third code path (`xpub-search`), and a naming correction to fold in).

- **Title:** retitle to reflect the correct BIP (`bip388-double-star-shorthand-support`), forwarding-noted from the old slug.
- **SemVer:** MINOR, toolkit-only, all 3 sibling codecs NO-BUMP.
- **Lockstep:** `docs/manual/` (8+ locations across 2 files) + `CHANGELOG.md`; NO GUI/`schema_mirror` touch.
- **Design decision the brainstorm must resolve up front:** hook point (a) pipeline string pre-pass vs. (b) `lex_placeholders` regex/synthesis extension — recommend leaning (b) for single-sourcing, but this needs an explicit architect call, not an implementer default.
- **Must explicitly scope in:** `xpub-search account-of-descriptor`'s literal-xpub funnel as a structurally separate fix target (currently silently unreached by any lexer-level change).
- **Must explicitly scope out / de-prioritize with rationale:** `bitcoin-core` format (Core never emits `/**`) and `specter` native-JSON (already works) — don't let the brainstorm spend cycles "fixing" surfaces that are already correct or moot.
- **Fold in:** the BIP-388/BIP-389 misattribution correction across source comments, test names, and manual prose (mechanical, ~10 sites, same files being touched regardless).
- **R0 gate:** standard — spec R0 (opus) → plan R0 (opus) → per-phase implementation (single subagent, TDD, existing 2 reject-tests repurposed not just supplemented) → post-impl whole-diff adversarial review (opus). Given 3 independent code paths (BIP-388-JSON expander, lexer, xpub-search funnel) all touching `/**` semantics, the post-impl review should specifically probe for a 4th path being missed (grep sweep for any other direct `miniscript::Descriptor::from_str` / `Regex` literal-descriptor consumer not enumerated above).
