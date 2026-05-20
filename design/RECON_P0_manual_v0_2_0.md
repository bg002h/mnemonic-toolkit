# P0 recon output — manual-v0.2.0 content-audit cycle

**Phase:** P0 (Setup + recon-grounding) per `design/PLAN_manual_v0_2_0_content_audit.md` §7.
**Executed:** 2026-05-20 against master `0cb3d1e` (post-plan-commit; plan-predecessor was `8977389`).
**Outcome:** All 4 P0 sub-tasks (P0a/P0b/P0c/P0d) complete; P1a may dispatch.

---

## P0a — Tag + SHA confirmation

| Item | Value | Verified by |
|---|---|---|
| Latest `manual-v*` tag | `manual-v0.1.10` | `git tag -l 'manual-v*' \| sort -V \| tail -1` |
| Master HEAD pre-plan-commit | `8977389` (plan predecessor; matches plan §0) | `git rev-parse 8977389` |
| Master HEAD post-plan-commit | `0cb3d1e` (this cycle's P1a working SHA) | `git rev-parse master` |
| Rust toolchain pin | `1.85.0` (channel + rustfmt + clippy) | `cat rust-toolchain.toml` |

**Implication for §12 risk #5 (rustc-version stderr portability):** `rust-toolchain.toml` pins the channel, so local + CI captures using the workspace's vendored toolchain will produce byte-identical stderr. The §12 risk is mitigated by existing infrastructure; the P3 CI extension can use `actions-rust-lang/setup-rust-toolchain` with the workspace pin auto-detected (no explicit version override needed).

## P0b — Binary build

```
$ cargo build --bin mnemonic
   Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.11s
$ target/debug/mnemonic --version
mnemonic 0.28.1
```

Binary at `target/debug/mnemonic` (115 MB debug build, executable). Reports `mnemonic 0.28.1` matching the toolkit-v0.28.1 release SHA the plan predecessor pinned.

## P0c — Fixture-recipe mapping table

Source: `docs/manual/src/30-workflows/39-cross-format-conversion.md` (recipes 1-8) cross-referenced against `crates/mnemonic-toolkit/tests/fixtures/wallet_import/` (60 files).

| Recipe | Source line | --blob in prose | Fixture path | Q10 substitution? |
|---|---|---|---|---|
| 1 BSMS → Bitcoin Core | L52 | `coordinator.bsms.txt` | `bsms-shwsh-2of3.txt` (sh-multi-2of3 — matches recipe intent) | **Yes** (placeholder name) |
| 2 Bitcoin Core → fresh bundle | L83 | `wallet.json` | `core-mainnet-receive-change-pair.json` (realistic listdescriptors output) | **Yes** (placeholder name) |
| 3 BSMS → BIP-388 wallet-policy | L112 | `multisig.bsms` | `bsms-2line-sortedmulti-2of3.txt` | **Yes** (placeholder name) |
| 4 Sparrow → BSMS | L144 | `sparrow-multisig-2of3-p2wsh-sortedmulti.json` | exact match | No |
| 5 Specter → Bitcoin Core | L166 | `specter-singlesig-p2wpkh.json` | exact match | No |
| 6 Coldcard → BIP-388 | L192 | `coldcard-singlesig-bip84-mainnet.json` | exact match | No |
| 7 Jade → BSMS | L216 | `jade-multisig-2of3-p2wsh.json` | exact match | No |
| 8 Electrum → BSMS | L235 | `electrum-multisig-2of3-wsh.json` | exact match | No |
| Multi-entry envelope sample | L258 | `wallet.json` | (same case as recipe 2) | **Yes** (placeholder name) |

**Q10 substitution scope is narrower than plan §1 estimated:** only 3 of 8 recipes (1/2/3) need `$FIXTURES_DIR/` substitution. Recipes 4-8 are exact-named in `tests/fixtures/wallet_import/` and the recipe `.cmd` files can use the bare filename directly with cwd staging. This **simplifies the verify-examples.sh extension** (§2.2): `$FIXTURES_DIR` is still needed (for recipes 1-3), but the recipe `.cmd` shape can be uniform-bare-filename + per-cmd `mktemp -d` cwd staging where the fixture is symlinked or copied in.

**Suggested .cmd shape (uniform across recipes):** each `.cmd` file's setup block does `cp $FIXTURES_DIR/<actual-fixture-name> <bare-filename-used-in-prose>` before invoking the recipe. The recipe-body then uses the bare filename matching the documented prose exactly. This minimizes prose-vs-transcript divergence and keeps the documented teaching-form intact.

## P0d — Chapter-41 inheritance command-block line numbers

Source: `docs/manual/src/40-cli-reference/41-mnemonic.md` (post-P13D state at master `0cb3d1e`).

| Block | Lines | Content |
|---|---|---|
| DESC env-var setup | L100-102 | `DESC='wsh(andor(pkh(@0),after(12000000),or_i(and_v(v:pkh(@1),older(4032)),and_v(v:pkh(@2),older(32768)))))'` |
| Bundle command — text form | L111-118 | `mnemonic bundle ... --slot '@0.phrase=...' --slot '@1.phrase=...' --slot '@2.phrase=...'` (NO `--json`) |
| Bundle command — JSON form | L222-230 | Same as text form but with `--json > /tmp/inheritance-bundle.json` |
| Verify-bundle command | L366-373 | `mnemonic verify-bundle --network mainnet --account 0 --descriptor "$DESC" --slot ... --bundle-json /tmp/inheritance-bundle.json` |

**Confirmed:** the FOLLOWUP body's `:209-216` (engraving-card stderr explainer) and `:351-357` (multisig.cosigners[] explainer) are prose-context paragraphs ABOUT the bundle output and verify-bundle output, NOT the command code blocks themselves. The actual command blocks are at L100-102 + L222-230 + L366-373.

**P1a composite transcript target (`41-inheritance.{cmd,out}`):** drives the JSON-form bundle (L100-102 + L222-230) + verify-bundle (L366-373) end-to-end, with `/tmp/inheritance-bundle.json` replaced by cwd-relative `inheritance-bundle.json` per §2.2 item 5 per-cmd tmpdir invariant. Stdout is the JSON envelope; verify-bundle stderr is the per-cosigner decode report.

The text-form bundle command at L111-118 is **out-of-scope** for the `41-inheritance` transcript per Q6/I2 (single composite pair driving bundle + verify-bundle, not separate text-form + json-form pairs). If text-form coverage is wanted, a separate FOLLOWUP can be filed.

---

## P0 → P1a handoff

P1a can now dispatch with full information:

1. Working SHA: `0cb3d1e` (post-plan-commit master).
2. Binary path: `target/debug/mnemonic` (or rebuild via `cargo build --bin mnemonic` from workspace root).
3. Capture targets (15 total):
   - 6 chapter-45 Round-trip examples at L299/367/443/526/590/684 (Sparrow/Specter/Coldcard-singlesig/Coldcard-multisig/Jade/Electrum). Each is a single command; no fixture dependency (the Round-trip examples already use exact-named fixtures or self-contained input).
   - 8 chapter-30/39 recipes at L45/68/106/136/159/184/208/227. 3 need fixture substitution (recipes 1/2/3); 5 are exact-named.
   - 1 chapter-41 inheritance composite at L100-102 + L222-230 + L366-373.
4. Each capture in a per-cmd `mktemp -d` cwd (§2.2 item 5 invariant).
5. Triple format: `.cmd` + `.out` (stdout) + `.err` (stderr); see §2.1.

Findings beyond P0 scope (defer to P1b classification):

- None at this phase.

P0 is complete; P1a may proceed.
