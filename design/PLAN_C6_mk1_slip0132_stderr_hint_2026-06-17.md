# PLAN — C6 (revised): mk1 SLIP-0132 path-implied-variant stderr HINT (2026-06-17)

> Tier-3 item C6. **R0-r1 disproved the derive-from-path DEFAULT-CHANGE** (Critical C-1: the card
> can't tell xpub-at-m/84' from zpub-at-m/84' — both store the same neutral xpub — so emitting a
> guessed variant inverts the `xpub→mk1→xpub` byte-identity contract AND Bitcoin Core rejects the
> zpub). **User decided: "stderr hint, keep neutral."** So: `convert --from mk1 --to xpub` stdout stays
> NEUTRAL xpub (byte-identity + interop intact); ADD a one-line stderr NOTE when the card's stored
> origin path is *conventionally* a SLIP-0132 variant, pointing at the existing `--xpub-prefix`. NO
> wire change, NO new flag, NO stdout change. **Source SHA: toolkit `1a0d0a9`** (HEAD == origin/master
> == v0.58.0; citations grep-verified). **PATCH → v0.58.0 → v0.58.1** (advisory-only stderr note,
> zero clap delta, zero stdout change — older()-advisory v0.55.2 precedent). R0 gate: 0C/0I before code.

---

## What changed from round-1 (why this is now safe)

R0-r1 Critical C-1 killed the default-change. This revised design **never changes stdout** — the
read-back `convert --from mk1 --to xpub` still emits the neutral `xpub` (so `tests/cli_standalone_
bijections.rs` B1/B2/B3 byte-identity holds, and the output stays Bitcoin-Core-acceptable). The only
addition is an **informational stderr note** when the card's path implies a non-neutral SLIP-0132
variant — telling the user "your card's path is conventionally zpub; here's how to emit that form" —
which is exactly the spirit of "x/y/z in → same out" without the impossible-to-distinguish guess.
This dissolves C-1 (no stdout change → no interop break, no byte-identity inversion) and I-2 (no
cross-surface stdout disagreement).

## Citations (grep-verified @ `1a0d0a9`; R0-r1 I-3 prefix/line corrections folded)

| Surface | Location |
|---|---|
| `XpubPrefix` enum (5 variants) | `crates/mnemonic-toolkit/src/slip0132.rs:17-28` |
| info-line precedent (reuse the shape, reverse direction) | `crates/mnemonic-toolkit/src/slip0132.rs::render_slip0132_info_line :132-137` |
| `convert --from mk1` decode + neutral re-emit (note-injection site) | `crates/mnemonic-toolkit/src/cmd/convert.rs:1589` `mk_codec::decode`, **`:1593`** `Xpub => card.xpub.to_string()` |
| `--xpub-prefix` post-process swap (unchanged; the on-demand producer) | `crates/mnemonic-toolkit/src/cmd/convert.rs:1086-1098`; flag `:300-301` |
| byte-identity contract (MUST stay green) | `crates/mnemonic-toolkit/tests/cli_standalone_bijections.rs` B1/B2/B3 (`xpub→mk1→xpub` byte-identical) |
| mk-codec `KeyCard.origin_path` (path IS stored — R0-r1 verified `convert --from mk1 --to path` → `84'/0'/0'`) | crates.io `mk-codec 0.4.0` (`crates/mnemonic-toolkit/Cargo.toml:35`, checksum-pinned) `key_card.rs:42` |
| FOLLOWUP | `design/FOLLOWUPS.md:4206` `mk1-card-slip0132-variant-not-preserved-on-card` (tier product-question) |

## Design

**New pure helper** `pub(crate) fn path_implied_xpub_prefix(path: &DerivationPath) -> XpubPrefix` in
`slip0132.rs` (unit-testable, no I/O): purpose 49 → `Ypub`; 84 → `Zpub`; 48 → read the 4th hardened
"script-type" component → 1'→`YpubMultisig`, 2'→`ZpubMultisig`, else `Xpub`; 44/45/86/empty/non-hardened
→ `Xpub`. (No network needed — this returns the abstract `XpubPrefix`; the note text uses the lowercase
spelling + network is only relevant if the user later passes `--xpub-prefix`.)

**The note (the only behavior addition).** In the `convert --from mk1 --to xpub` decode block
(`convert.rs:1593`), compute `path_implied_xpub_prefix(&card.origin_path)`; if it is **non-default** (a
real SLIP-0132 variant) AND **`args.xpub_prefix.is_none()`** (R0-r2 M-2: exactly `is_none()` — a user
who passes `--xpub-prefix xpub`, the explicit-neutral case, has chosen a form and the note suppresses;
do NOT use `is_some_and(!is_default)`, which would wrongly re-fire on explicit-neutral), surface ONE
stderr note. **Plumbing (R0-r2 M-3):** `compute_outputs` (`convert.rs:1185-1192`) takes no stderr
writer — it returns `(Vec<Output>, Option<&'static str> input_variant, Option<SeedVersion>)` and
`run()` does the stderr writes. Do NOT overload slot-2 (`input_variant`, rendered unconditionally via
`render_slip0132_info_line` at `:1061-1066`). Add a **dedicated 4th return element**
`Option<XpubPrefix>` (the path-implied hint, set only in the `Mk1`→`Xpub` arm); `run()` emits the note
AFTER the existing `input_variant` stderr block, gated on `args.xpub_prefix.is_none()`. The stdout
output stays the unchanged neutral `card.xpub.to_string()`. The note text:
```
note: this card's <m/84'/…> path is conventionally SLIP-0132 zpub; re-emit with --xpub-prefix zpub
      (the engraved mk1 stores the BIP-32-neutral xpub — the variant is a display form, not on the card).
```
The variant spelling (`ypub`/`Ypub`/`zpub`/`Zpub`) is the existing `parse_xpub_prefix_arg` inverse —
add a tiny `xpub_prefix_flag_str(XpubPrefix) -> &'static str` (`Ypub→"ypub"`, `YpubMultisig→"Ypub"`,
`Zpub→"zpub"`, `ZpubMultisig→"Zpub"`, `Xpub→"xpub"`). Best-effort stderr (mirror `emit_advisories`);
never affects exit code or stdout.

**Suppression:** no note when (a) the path implies neutral `Xpub` (44'/86'/etc. — nothing to say), or
(b) the user already passed `--xpub-prefix` (they've chosen the form; the existing post-process at
`:1086` handles it). This keeps the note signal-only.

**Scope:** `convert --from mk1 --to xpub` only (the read-back surface the user round-trips). `inspect`
is a reasonable secondary home for the same note but is NOT in v1 (keep the change minimal; R0 may
green-light adding it). `verify-bundle`/`restore` unchanged (internal byte-match, no user-facing
"here's your key" moment). NO mk-codec/mk-cli change, NO publish, NO wire change, NO stdout change.

## TDD

`tests/cli_convert_mk1_slip0132_hint.rs` (new) + module unit tests in `slip0132.rs`:
1. **Note fires, stdout neutral.** Build a real mk1 card via `bundle --slot @0.xpub=<zpub>` (R0-r1 M-1:
   `convert --to mk1` is a hard refusal — cards mint via bundle), extract the mk1 from `--json`, then
   `convert --from mk1=<card> --to xpub`: assert STDOUT is the neutral `xpub…` (NOT zpub — byte-identity
   intact) AND STDERR contains `--xpub-prefix zpub`.
2. **Neutral path = no note.** A `m/44'`/`m/86'` card → stderr has NO `--xpub-prefix` note.
3. **Explicit `--xpub-prefix` suppresses the note** (and produces the variant on stdout via the existing
   path) — confirms no double-speak.
4. **Anti-regression (R0-r1 M-3):** a neutral `xpub` engraved at `m/84'` → `convert --from mk1 --to xpub`
   STDOUT is byte-identical neutral `xpub` (the note still fires — "conventionally zpub" — but stdout is
   unchanged). Pins that the note never touches stdout. (Also: confirm B1/B2/B3 stay green unchanged.)
5. **Module unit tests** for `path_implied_xpub_prefix`: 49→Ypub, 84→Zpub, 48'/…/1'→YpubMultisig,
   48'/…/2'→ZpubMultisig, 44/45/86/empty→Xpub; and `xpub_prefix_flag_str` inverse.

**Non-vacuity:** remove the note emit → test 1's stderr assertion fails (RED). The byte-identity tests
(B1/B2/B3 + cell 4) are the guard that the note never leaks into stdout.

## Lockstep / SemVer

- **PATCH → v0.58.1.** Advisory-only stderr note, **no stdout change, no new flag** → no GUI
  `schema_mirror` surface, **no new `ToolkitError` variant**. (older()-advisory v0.55.2 set the
  advisory-only=PATCH precedent.)
- Version sites (full checklist): `Cargo.toml`, BOTH READMEs (`README.md:13` +
  `crates/mnemonic-toolkit/README.md:9`), `scripts/install.sh:32`, `fuzz/Cargo.lock`, main `Cargo.lock`,
  CHANGELOG `[0.58.1]`. fmt gate (`cargo +1.95.0 fmt --all` then revert `mlock.rs`).
- **Manual:** the `convert` `--xpub-prefix` section gets a sentence — reading an mk1 card now prints a
  stderr hint naming the SLIP-0132 variant the card's path conventionally uses (the card itself stores
  the neutral xpub). `make -C docs/manual lint`.
- **FOLLOWUP:** `mk1-card-slip0132-variant-not-preserved-on-card` (FOLLOWUPS.md:4206) → `resolved`,
  recording the OUTCOME of the product-question: on-card preservation is NOT pursued (would be a
  breaking wire bump); the shipped resolution is a non-breaking stderr hint pointing at `--xpub-prefix`,
  keeping stdout neutral (Core-interoperable, byte-identity-preserving).

## Execution

1. R0 architect review of THIS revised plan → GREEN (0C/0I), persist to
   `design/agent-reports/c6-mk1-slip0132-plan-r0-round2-review.md`. (Round-1 RED is the prior file.)
2. TDD: write cells 1-5; confirm RED (note-missing) + the byte-identity guards GREEN.
3. Implement the helper + the note at the convert site.
4. Per-phase impl review → 0C/0I, persist.
5. Version bump v0.58.1 + lockstep + manual + CHANGELOG + FOLLOWUP. fmt gate. Commit, tag
   `mnemonic-toolkit-v0.58.1`, push master, CI green.
