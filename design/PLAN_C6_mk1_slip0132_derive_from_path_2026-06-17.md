# PLAN — C6: mk1 SLIP-0132 variant re-emit, derived from the card's path (2026-06-17)

> Tier-3 item C6, user-decided: **preserve "x/y/z in → same out" by DERIVING the SLIP-0132 variant
> from the origin path the mk1 card already stores — NO wire change, NO new flag.** (The breaking
> wire-generation option and the per-decode-surface override flag were both rejected by the user.)
> **Source SHA: toolkit `1a0d0a9`** (HEAD == origin/master == tag mnemonic-toolkit-v0.58.0;
> citations grep-verified). **MINOR → v0.58.0 → v0.59.0** (changes a default display output). Toolkit
> git-tag only. R0 gate: **no code until R0 → GREEN (0C/0I).**

---

## Gap

The mk1 card normalizes a SLIP-0132 input (`ypub`/`zpub`/`Ypub`/`Zpub` + testnet `u/v/U/V`) to the
neutral `xpub`/`tpub` on intake (`slip0132.rs::normalize_xpub_prefix :66`, wired at `convert.rs:1415`,
`bundle.rs:579`), and the card stores only the neutral key (mk-codec `xpub_compact` is network-only;
the variant is genuinely NOT on the card — confirmed: preserving it on-card would be a breaking mk1
wire-generation bump, REJECTED by the user). So reading the card back (`convert --from mk1 --to xpub`,
`convert.rs:1589-1591` `card.xpub.to_string()`) always yields neutral `xpub` — the cosigner's
original SLIP-0132 form is lost. Today the only recovery is the explicit `--xpub-prefix <variant>`
flag (v0.6.1) — which the user does not want to require.

**Key insight (user-confirmed approach):** the card DOES store the **derivation path**
(`mk-codec KeyCard.origin_path: DerivationPath`, `key_card.rs:42`), and the SLIP-0132 variant is, by
the SLIP-0132 convention, a *restatement* of the path purpose:

| origin-path purpose | SLIP-0132 variant | `XpubPrefix` |
|---|---|---|
| `m/49'/…` (BIP-49 single-sig) | ypub / upub | `Ypub` |
| `m/84'/…` (BIP-84 single-sig) | zpub / vpub | `Zpub` |
| `m/48'/coin'/acct'/1'` (BIP-48 P2SH-P2WSH multisig) | Ypub / Upub | `YpubMultisig` |
| `m/48'/coin'/acct'/2'` (BIP-48 P2WSH multisig) | Zpub / Vpub | `ZpubMultisig` |
| `m/44'/…`, `m/45'/…`, `m/86'/…`, anything else | xpub / tpub (neutral) | `Xpub` |

So the variant is **re-derivable from the already-stored path** with no wire change — for every input
that followed the SLIP-0132 convention (prefix matches path), this gives back exactly what went in.

## Citations (grep-verified @ `1a0d0a9`)

| Surface | Location |
|---|---|
| `XpubPrefix` enum (5 variants) | `src/slip0132.rs:17-28` |
| re-emit renderer (REUSE) | `src/slip0132.rs::apply_xpub_prefix(&Xpub, XpubPrefix, CliNetwork)->String` `:108-112` |
| variant→version-bytes table | `src/slip0132.rs::swap_target_for` `:139-153` |
| `--xpub-prefix` post-process swap (source-agnostic, over `NodeType::Xpub` outputs) | `src/cmd/convert.rs:1086-1098` (flag `:300-301`) |
| convert `--from mk1` decode + neutral re-emit (the change site) | `src/cmd/convert.rs:1589` `mk_codec::decode`, `:1591` `Xpub => card.xpub.to_string()` |
| mk-codec `KeyCard.origin_path` | `mnemonic-key/crates/mk-codec/src/key_card.rs:42` |
| input-normalization info-line precedent (reverse direction) | `src/slip0132.rs::render_slip0132_info_line :132` |

## Design

**New helper** `pub(crate) fn xpub_prefix_from_origin_path(path: &DerivationPath) -> XpubPrefix` in
`slip0132.rs` (pure; unit-testable). Reads the purpose component (1st, hardened) → `Ypub`(49)/`Zpub`(84);
for purpose 48 reads the 4th hardened "script-type" component → `YpubMultisig`(1')/`ZpubMultisig`(2');
everything else → `Xpub` (neutral). Network (mainnet vs testnet variant) is taken from
`card.xpub.network` at apply time (NOT the `--network` flag — the auto-derive must not require it).

**Behavior change (the change site):** at `convert --from mk1 --to xpub` (`convert.rs:1591`), when the
user gave **no explicit `--xpub-prefix`**, render the xpub output via
`apply_xpub_prefix(&card.xpub, xpub_prefix_from_origin_path(&card.origin_path), net_from(card.xpub))`
instead of the neutral `card.xpub.to_string()`. An explicit `--xpub-prefix` still wins (the existing
post-process at `:1086` overrides) — so `--xpub-prefix xpub` is the neutral escape hatch.

**Interop guard (R0 MUST vet — the central risk).** Bitcoin Core's descriptor parser + rust-miniscript
+ BIP-388 reject non-`xpub` prefixes. Changing the DEFAULT of `convert --from mk1 --to xpub` to emit
`ypub`/`zpub`/`Zpub` means a user piping that into Core/a descriptor breaks. Mitigations in this plan
(R0 to confirm sufficiency, or escalate to the user): (1) emit a one-line **stderr note** whenever a
non-neutral variant is rendered — e.g. `note: emitted zpub (SLIP-0132, from the m/84' path); pass
--xpub-prefix xpub for the BIP-32-neutral form required by Bitcoin Core / descriptors.` (mirrors the
existing reverse info-line `render_slip0132_info_line`); (2) the toolkit's OWN descriptor/bundle
intake re-normalizes variants→neutral (`normalize_xpub_prefix`), so feeding the result back INTO the
toolkit is unaffected — only direct external Core/descriptor piping is affected; (3) the ecosystem
(Sparrow/Specter/Coldcard) DOES use `Zpub` for `m/48'/…/2'` cosigners, so for segwit paths the variant
is the more-compatible form for those tools. **If R0 judges the default-change too risky, the fallback
design is: keep `--to xpub` neutral and surface the derived variant only as a stderr note + via the
existing `--xpub-prefix` — but that does not meet the user's "no-flag, same-out" bar, so escalate.**

**Scope:** the `convert --from mk1 --to xpub` read-back is the primary round-trip surface and the v1
scope. `inspect --mk1` (forensic, `inspect.rs:231`) — the plan proposes ALSO re-emitting the derived
variant for display consistency, but R0 to confirm (forensic output may prefer the neutral stored
form; low-risk either way). `verify-bundle`/`restore` MATCH cosigner keys by raw bytes (identical
across variants) — they are NOT changed (no display-driven correctness, and changing them risks
golden churn for zero round-trip benefit). NO mk-codec/mk-cli change, NO publish, NO wire change.

## TDD

`tests/cli_convert_mk1_slip0132.rs` (new) + module unit tests in `slip0132.rs`:
1. **Round-trip per variant.** `convert --from xpub=<zpub@m/84'> --to mk1` → `convert --from mk1=<card>
   --to xpub` → emits **zpub** (was xpub). Same for ypub@49', Zpub@48'/2', Ypub@48'/1'. Each: the
   re-emitted prefix == the input prefix (the user's "same out" property holds for convention inputs).
2. **Neutral path stays neutral.** A card at `m/44'/…` or `m/86'/…` → `convert --from mk1 --to xpub`
   emits plain `xpub` (no spurious variant).
3. **Explicit override wins.** `convert --from mk1 --to xpub --xpub-prefix xpub --network mainnet` on a
   zpub-path card → emits neutral `xpub` (the interop escape hatch).
4. **Testnet.** A `vpub`/`upub`-path card re-emits the testnet variant (network from `card.xpub`).
5. **stderr note** fires on a non-neutral emit, names the neutral escape hatch.
6. **Module unit tests** for `xpub_prefix_from_origin_path`: 49→Ypub, 84→Zpub, 48'/1'→YpubMultisig,
   48'/2'→ZpubMultisig, 44/45/86/empty→Xpub.

**Non-vacuity:** revert the change site → round-trip test 1 emits neutral xpub → RED.
**Golden churn:** existing `convert --from mk1 --to xpub` tests whose fixture cards use a 49'/84'/48'
path will now emit a variant — audit + update those goldens (TDD phase; expected, documents the
behavior change).

## Lockstep / SemVer

- **MINOR → v0.59.0** (changes a default display output = behavior change; precedent: input-format /
  output-display changes are MINOR). **NO new flag → no GUI `schema_mirror` surface.** **No new
  `ToolkitError` variant.**
- Version sites (the full checklist — [[project_toolkit_release_ritual_version_sites]] equivalent):
  `Cargo.toml`, BOTH READMEs (`README.md:13` + `crates/mnemonic-toolkit/README.md:9`), `install.sh:32`,
  `fuzz/Cargo.lock`, main `Cargo.lock`, CHANGELOG `[0.59.0]`. fmt gate (revert mlock.rs).
- **Manual:** the `convert` `--from mk1`/`--xpub-prefix` section — document that reading an mk1 card
  now re-emits the SLIP-0132 variant implied by the card's path by default, with `--xpub-prefix xpub`
  for the neutral form. `make -C docs/manual lint`.
- **FOLLOWUP:** `mk1-card-slip0132-variant-not-preserved-on-card` (FOLLOWUPS.md:4206) → `resolved`,
  recording the derive-from-path approach (NOT the breaking-wire option) + the convention-only caveat.

## Execution

1. R0 architect review of THIS plan → GREEN (0C/0I), persist to
   `design/agent-reports/c6-mk1-slip0132-plan-r0-round{N}-review.md`. **R0 MUST vet the interop
   default-change risk** (is the stderr-note + escape-hatch sufficient, or escalate to the user?) and
   the inspect-scope question.
2. TDD: write the round-trip + unit + neutral + override + testnet cells; confirm RED.
3. Implement the helper + the convert change site (+ inspect if R0 says so) + the stderr note.
4. Per-phase impl review → 0C/0I, persist.
5. Version bump v0.59.0 + lockstep + manual + CHANGELOG + FOLLOWUP. fmt gate. Commit, tag
   `mnemonic-toolkit-v0.59.0`, push master, CI green.
