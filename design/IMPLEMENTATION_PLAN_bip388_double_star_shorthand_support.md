# IMPLEMENTATION PLAN — bip388-double-star-shorthand-support

**Executes `design/SPEC_bip388_double_star_shorthand_support.md` (R0-GREEN rev-3, SHA `0964462d`).**

- **Model of execution (CLAUDE.md phase-3):** a SINGLE implementer subagent in a git worktree, TDD, phase by
  phase; per-phase opus R0 (FULL `cargo test -p mnemonic-toolkit` suite) to 0C/0I; mandatory post-impl
  whole-diff review.
- **Status:** DRAFT rev-3 — folded PLAN-R0-round-2 (0C/1I: I3 compare-cost is EQUIVALENCE-only, not acceptance — it rejects all multipath `/<0;1>/*`; kept IN for `/**`≡`/<0;1>/*` error-equivalence + FOLLOWUP `compare-cost-multipath-descriptor-unsupported`). SPEC §0/§6/§7.11 updated in lockstep (rev-5). Pending R0 round-3 scoped convergence on the compare-cost decision BEFORE any implementer dispatch.
- **Target:** `mnemonic-toolkit-v0.78.0` (MINOR); codecs NO-BUMP; NO GUI/`schema_mirror`.
- **Branch:** `feature/bip388-double-star-shorthand` off current `origin/master` (`0964462d`), worktree.

---

## Guard-rails (every phase)
- **TDD:** tests FIRST (RED for the right reason), then implement. The equivalence oracles (§7.3/§7.4) are
  the funds anchor — RED-first.
- **No `git add -A`; NEVER `cargo fmt --all`** (mlock.rs fmt-exempt); `cargo fmt -p mnemonic-toolkit` only;
  clippy `-D warnings` clean each phase.
- **Per-phase gate:** full `cargo test -p mnemonic-toolkit` green + clippy + persist per-phase R0 verbatim to
  `design/agent-reports/cycleC-phase-N-*.md` before advance.

---

## Phase P0 — the `/**` expander + wiring + tests + source-text corrections (the whole implementation)

### P0 Task 1 (FIRST — per SPEC §5/§10.2): grep-verify the COMPLETE minimal call-site set
Enumerate every PRODUCTION (`#[cfg(test)]`-excluded) consumer of raw user descriptor text that reaches a
`/**`-rejecting parser (`lex_placeholders` or `MsDescriptor::<DescriptorPublicKey>::from_str`), and classify
each IN-scope (a §0 user-`/**` intake surface) vs OUT (toolkit-generated / canonical / export form).
**Classified set (corrected per PLAN-R0-round-1 I1/I2; implementer to CONFIRM + prove no further in-scope path):**

**IN (expand the user string before the parser):**
| Site | Surface | Note |
|---|---|---|
| `parse_descriptor.rs:875` (top of `parse_descriptor`, before `lex_placeholders(input)`@884 + `from_str(&substituted)`@897) | ALL concrete-pipeline paths — `import-wallet --format descriptor` (`descriptor.rs:68`), `--format bsms` MAIN parse (`bsms.rs:227`), `bundle --descriptor` Concrete branch, `verify-bundle --descriptor` Concrete fork; ~10 `concrete_keys_to_placeholders` callers funnel here; ALSO `gui-schema --classify-descriptor` (`gui_schema.rs:1319`) — auto-covered here (I2, no separate touch) | single chokepoint; `/**` survives key→@N substitution as a `)`-bounded token |
| `bundle.rs:1389` | `bundle --descriptor "…@N/**…"` AtN direct-lex | |
| `verify_bundle.rs:1375` | `verify-bundle --descriptor` AtN fork | |
| `descriptor_intake.rs:297` (`parse_literal_xpub`) | `xpub-search account-of-descriptor --descriptor` | |
| `wallet_import/bsms.rs:300` | BSMS first-address verification `from_str` — `if let Ok` SKIPS on `/**` (soft gap) | `derive_first_address` handles multipath → `/**`→`/<0;1>/*` safe |
| **`wallet_import/roundtrip.rs:231` `recanonicalize_descriptor` (before from_str@241)** (PLAN-R0 I1) | BSMS `--json` canonicalize (`canonicalize_bsms`, `import_wallet.rs:1458`) re-parses the RAW user body → soft-fails `/**` into the `--json` roundtrip/canonical envelope | single chokepoint for both callers; the bitcoin-core caller (roundtrip.rs:170) is a harmless no-op |
| **`export_wallet.rs:517`** (PLAN-R0 I2) | `export-wallet --descriptor "…xpub/**"` HARD-rejects today (asymmetry vs import) | one-line expander before from_str |
| **`cost/strip.rs:21`** (PLAN-R0 I2/I3 — EQUIVALENCE-only) | `compare-cost --descriptor "…xpub/**"` — compare-cost rejects ALL multipath `/<0;1>/*` (pre-existing limitation, no `into_single_descriptors` split); expander makes `/**` reject IDENTICALLY to `/<0;1>/*`, NOT accept | one-line expander before from_str; §7.11 asserts error-equivalence; FOLLOWUP `compare-cost-multipath-descriptor-unsupported` for the pre-existing gap |

**Also audit (P0 Task 1):** the sibling `canonicalize_*` (coldcard/electrum/sparrow/specter, `roundtrip.rs:305/484/637/807`) for the same raw-body-from_str class — expand any that parse a raw USER body.

**OUT (confirmed toolkit-generated / never-`/**` — NO touch, document the confirmation):**
`restore.rs:2066/2380/2760/3238` (md1-card reconstructions — cards encode `/<0;1>/*`, never `/**`), `nostr.rs:142` (built string), `export_wallet.rs:633/796` (post-517 canonical form), `wallet_export/*` (canonical), `descriptor_builder/gate.rs` (rendered `wsh(M)`), `bitcoin_core.rs:337/1026` + `roundtrip.rs:170`-via-bitcoin-core (Core never emits `/**`).

**Placement decision to lock in P0:** consider a per-command chokepoint (expand `descriptor_str` once earlier in `bundle`/`verify_bundle` before the AtN/Concrete split) vs the per-parser sites above. The chosen set MUST cover: concrete + AtN + xpub-search + bsms-address-check + bsms-canonicalize + export-wallet + compare-cost.

### P0 Task 2: implement `expand_literal_double_star`
Add `fn expand_literal_double_star(desc: &str) -> Cow<str>` (home it in `parse_descriptor.rs` next to
`substitute_nums_sentinel`, or a shared module both `parse_descriptor` and `descriptor_intake` import).
Rewrite a `/**` to `/<0;1>/*` **only** when immediately followed by the terminator set `)` `,` `}` whitespace
`#` EOS (§5/M2) — excludes `/***`, `/**'`. Per-key (all `/**` in a multisig expand); anchored on the
`/**`+terminator boundary, NEVER naive global replace. `/**`-free input → borrowed no-op (idempotent with
`expand_bip388_policy`, which already emits `/<0;1>/*`). Model on `substitute_nums_sentinel`
(`parse_descriptor.rs:373`).

### P0 Task 3: wire the expander at the confirmed IN sites (Task 1).

### P0 Task 4: source-text corrections (SPEC §2/§8 — behavior + comments)
- **Reword the reject message** (`parse_descriptor.rs:206-211`): drop `(or the /** shorthand)` — `/**` is now
  accepted; keep `/0/*` as the un-representable exemplar.
- **Fix comment** `parse_descriptor.rs:189` (wrong BIP + stale residue description).
- **Correct BIP-389→BIP-388** in `cli_import_wallet_descriptor.rs:159`(+`:191`), `sparrow.rs:42`. Leave the
  genuine multipath BIP-389 refs (§2 LEAVE list).

### P0 Task 5: TDD — write ALL §7 tests FIRST (RED), then implement to GREEN
- REPURPOSE the 2 reject-tests (§7.1 `parse_descriptor.rs:1731`, §7.2 `cli_import_wallet_descriptor.rs:191`) →
  accept-with-expansion.
- §7.3 concrete-xpub equivalence oracle (wpkh/sortedmulti/tr; `/**` output == `/<0;1>/*` output: descriptor /
  md1 cards / `--json` / derived addresses).
- §7.4 AtN-form oracle (`bundle --descriptor "wsh(sortedmulti(2,@0/**,@1/**))"` + verify-bundle AtN fork).
- §7.5 xpub-search `/**` parses; §7.6 bsms round-2 `/**`; §7.7 precision (`/***`/`/**'` reject; no-`/**` no-op;
  multisig both expand); §7.8 JSON `@N/**` regression; §7.9 reworded message on a genuine `/0/*` reject;
  §7.10 `/0/**` floor-not-weakened composite (N1).
- §7.11 (PLAN-R0 I1/I2/I3): `export-wallet --descriptor "…xpub/**"` ACCEPTS == `/<0;1>/*`; `compare-cost
  --descriptor "…/**"` **rejects IDENTICALLY** to `/<0;1>/*` (same "multipath key cannot be a
  DerivedDescriptorKey" error+exit, != the pre-fix raw "invalid child number format" — EQUIVALENCE not
  acceptance, PLAN-R0 I3; the pre-existing compare-cost multipath gap is FOLLOWUP
  `compare-cost-multipath-descriptor-unsupported`); `import-wallet --format bsms --json` `/**` yields a CLEAN
  `roundtrip`/canonical field (not "canonicalize: parse failed"); `gui-schema --classify-descriptor "…/**"`
  classifies (chokepoint-covered).
- **TDD ordering (PLAN-R0 M1):** the three FLIPPED cells (§7.1, §7.2, §7.9) go RED the moment they are flipped
  and green only when the expander (Task 2/3) + message reword (Task 4) land — flip them in the SAME
  commit/phase as the impl so the suite is never RED "for the wrong reason."

**Gate:** full `cargo test -p` green; clippy clean; per-phase opus R0 (weighted to the call-site completeness +
the equivalence oracles' non-tautology + the precision/floor-not-weakened guards).

## Phase P1 — docs lockstep

- **Manual prose** (`docs/manual/`, re-grep at impl): `41-mnemonic.md` §"Non-representable use-site steps"
  (`:137-168`, incl. `:164` semantic rewrite) + `:218,659,711-713,1204,1253-1255,1466,3510`;
  `45-foreign-formats.md:127-133`. State `/**` is now ACCEPTED (expanded to `/<0;1>/*`), not rejected; correct
  BIP-389→BIP-388 where `/**` is named (NOT the correct `/<a;b>/*` lines).
- **`verify-examples`:** none expected affected (`.examples-build/` has no `/**`); run `make -C docs/manual
  lint` with the new binary to confirm; regen any newly-added fence.

**Gate:** `make -C docs/manual lint` green with the new binary; per-phase R0.

## Post-impl whole-diff review (MANDATORY endpoint before release)
Fresh opus adversarial review over the whole diff. Weighted to: **call-site completeness — grep for a missed
path, explicitly enumerating the known adjacencies (PLAN-R0 M3): the `canonicalize_*` family
(`roundtrip.rs`), and the `compare-cost`/`export-wallet --descriptor` user surfaces** — plus the expander
precision (terminator anchoring; no descriptor corruption), the equivalence oracles' non-tautology,
idempotence with `expand_bip388_policy`, and the reject-message correctness. Persist to
`design/agent-reports/cycleC-postimpl-whole-diff-review.md`.

## Release ritual (v0.78.0) — per §9
Version sites: `Cargo.toml` + workspace `Cargo.lock` + `fuzz/Cargo.lock` + BOTH READMEs +
`scripts/install.sh:32` self-pin (NOT the frozen sibling pins) + `.examples-build/gen.sh:44` version-check +
embedded gen.sh strings + regen `Examples.md` + CHANGELOG `[0.78.0]` (BIP-388; retire the v0.76.0 `/**`-hard-
fails interim bullet). Codecs NO-BUMP. **Re-vendor N/A this cycle (PLAN-R0 M2)** — no dependency added (`Cow`
is std), so `vendor/` is untouched and vendor-freshness is a no-op-pass (do NOT skip-in-confusion). Direct-FF
+ tag `mnemonic-toolkit-v0.78.0` + push + verify all 14 CI gates green.

## Risk register (for plan R0)
1. **Call-site completeness** (P0 Task 1) — a missed IN site = `/**` still rejects there (fail-closed, soft
   gap). The grep-classification + post-impl grep-sweep are the nets.
2. **Expander precision** — terminator-anchored; multisig all-keys; no stray `**`. §7.7 guards.
3. **Idempotence** with `expand_bip388_policy` (no double-expansion).
4. **Equivalence = funds property** — expanded `/**` ≡ explicit `/<0;1>/*` (§7.3/7.4).
5. **bsms.rs:300 soft gap** — confirm expanding there restores the first-address check for `/**` without
   changing non-`/**` behavior.
