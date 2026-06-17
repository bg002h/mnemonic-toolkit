# R0 review — PLAN_C6_mk1_slip0132_derive_from_path (round 1)

- **Plan:** `design/PLAN_C6_mk1_slip0132_derive_from_path_2026-06-17.md`
- **Source SHA verified against:** `1a0d0a9` (HEAD == origin/master == tag `mnemonic-toolkit-v0.58.0`)
- **Reviewer:** opus architect, adversarial R0. Built `target/debug/mnemonic`; ran bitcoind v0.2.4 (Bitcoin Satellite fork) `-chain=main` offline.

## Verdict: RED — 1 Critical, 3 Important

The make-or-break items (2: interop default-change; 4: does the card store the full path / does the approach work) were resolved EMPIRICALLY. Item 4's mechanics work (path IS stored), but item 4's *semantic claim* ("x in → same out") is FALSE for the dominant input class, and item 2's external-interop break is CONFIRMED real. The default-change must be ESCALATED to the user, not silently shipped.

---

## Critical

### C-1 — The default-change to `--to xpub` is BOTH (a) a confirmed external-interop regression AND (b) semantically wrong for neutral-xpub inputs; it must be ESCALATED to the user, not shipped on the architect's authority.

Two independent empirical findings converge to make this Critical.

**(a) External interop break — CONFIRMED on Bitcoin Core's descriptor parser.** Built the binary, ran `bitcoind -chain=main` offline, `getdescriptorinfo`:
- control, neutral xpub: `wpkh(xpub6CatW…/0/*)` → **accepted** (`issolvable: true`).
- C6 default output, zpub: `wpkh(zpub6rFR7…/0/*)` → **REJECTED**: `error code -5 … key 'zpub6rFR7…' is not valid`.

So a user who today pipes `convert --from mk1 --to xpub` into a Core descriptor gets a working xpub; after C6 they get a zpub Core rejects. This is a silent behavior regression on the *default* path of a stable read-back edge.

**(b) The "x in → same out" property the feature is FOR is FALSE for neutral-xpub inputs — and this is the common case.** The card cannot distinguish "user supplied a neutral `xpub` at m/84'" from "user supplied a `zpub` at m/84'": both normalize to the identical neutral xpub + path `84'/0'/0'` on intake (verified: `convert --from xpub=<zpub>` and `bundle --slot @0.xpub=<zpub>` both emit `info: normalized zpub …` and store neutral). Derive-from-path therefore cannot *recover* the original prefix — it can only *guess*, and it guesses "variant". For a neutral-xpub input it guesses WRONG: x in → z out.

This is not hypothetical. The existing bijection contract `tests/cli_standalone_bijections.rs` B1/B2/B3 asserts `xpub → mk1 → xpub` is **byte-identical** for bip84/bip49/multisig templates. Empirically: B1 derives a neutral xpub at `m/84'/0'/0'`, bundles, and `mk1_decode(chunks,"xpub")` returns it byte-identical today. Under C6 it returns `zpub6rFR7…`, so `assert_eq!(got, xpub, "B1: … must be byte-identical")` FAILS. The plan books this under "golden churn … expected, documents the behavior change" — but B1/B2/B3 are not stale goldens, they are the *correctness contract* for the read-back edge, and C6 *inverts* the property the feature claims to deliver for the neutral-input majority.

Empirical confirmation the path IS stored (so the mechanism is sound, only the policy is wrong):
```
$ mnemonic convert --from mk1=<bip84 card> --to path
path: 84'/0'/0'
$ mnemonic convert --from mk1=<bip84 card> --to xpub
xpub: xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V   # == BIP84_REF_XPUB
```
The purpose `84'` is the first component and IS recoverable — `xpub_prefix_from_origin_path` would work mechanically. The problem is policy, not mechanism.

**Why this is the architect's call to ESCALATE, not approve.** The canonical FOLLOWUP (`FOLLOWUPS.md:4206`, status `open`, tier **`product-question`**) frames this exact item as an OPEN PRODUCT QUESTION and instructs: *"decide the product question first … is on-card SLIP-0132 preservation actually wanted, or is normalize-in/re-emit-out (the shipped behavior) sufficient?"* The user's stated decision ("derive from path, no flag, no wire change") chose a *mechanism*; it does not obviously choose "change the DEFAULT of `--to xpub` to a form Bitcoin Core rejects AND invert byte-identity for neutral-xpub inputs." Per CLAUDE.md the R0 gate explicitly requires the architect to "vet the interop default-change risk (is the stderr-note + escape-hatch sufficient, or escalate to the user?)". On the evidence: the stderr note + `--xpub-prefix xpub` escape hatch are NOT sufficient to make a silent default-change safe, because (i) the note does not prevent the break for anyone who pipes stdout (the note goes to stderr; the zpub goes to stdout into the descriptor), and (ii) it cannot fix the x-in→z-out inversion for neutral inputs (no note makes that "same out").

**Fix — escalate with a concrete recommended design.** Take the choice back to the user with these findings, and recommend the non-default-changing variant the plan itself names as the fallback:
- Keep `convert --from mk1 --to xpub` emitting the **neutral xpub by default** (no regression, byte-identity preserved).
- When the card's path implies a SLIP-0132 variant, emit a **stderr note**: `note: this card's m/84' path corresponds to the SLIP-0132 'zpub' form; re-emit it with --xpub-prefix zpub` (mirrors `render_slip0132_info_line`). This surfaces the variant without changing the default wire output.
- The existing `--xpub-prefix zpub|ypub|Ypub|Zpub` flag already produces the variant on demand (verified: `--xpub-prefix zpub --network mainnet` → `zpub6rFR7…`).

This does not meet the user's literal "no-flag, same-out, default re-emits the variant" bar — which is exactly why it is an ESCALATION, not an architect override. If, after seeing the Core-rejects-zpub evidence and the x-in→z-out inversion, the user still wants the default-change, that is their informed product call and R0 can then proceed against a plan that (a) documents the interop break in the manual/CHANGELOG as a known behavior change, (b) rewrites B1/B2/B3 to assert the new variant-emitting contract rather than treating them as incidental churn, and (c) decides the inspect/restore scope consistently (see I-2).

---

## Important

### I-2 — Scope inconsistency: `convert` would emit `zpub`, but `inspect`/`inspect --json`/`restore`/`verify-bundle` keep emitting neutral `xpub` for the SAME card.

`inspect.rs:231` (`xpub: {}` over `card.xpub`) and `:328` (JSON `card.xpub.to_string()`) emit the neutral form; the plan leaves them unchanged. Result: `mnemonic inspect <card>` shows `xpub: xpub6CatW…` while `mnemonic convert --from <same card> --to xpub` shows `zpub6rFR7…`. The plan flags this as "R0 to confirm" and leans toward also changing inspect. Recommendation: a forensic surface (`inspect`) should show what is *actually on the card* — the neutral xpub — not a path-derived guess; so inspect should stay neutral. But that makes the two surfaces disagree, which is itself a defect. This inconsistency is a direct consequence of C-1 and dissolves if C-1 is resolved by NOT changing the convert default (both stay neutral; both can carry the same stderr note). If the user nonetheless approves the default-change, the scope decision (which of convert/inspect/restore flip) must be made explicit and uniform in the re-planned doc, not deferred.

### I-3 — Citations carry a systematic path error and three line-number drifts; must be corrected before any implementation (CLAUDE.md: "Plan-doc + spec citations are grep-verified at write time").

Verified against `1a0d0a9`:
- **Path-prefix error (systematic):** every `src/…` citation omits the crate prefix. Real paths are `crates/mnemonic-toolkit/src/slip0132.rs`, `crates/mnemonic-toolkit/src/cmd/convert.rs`, `crates/mnemonic-toolkit/src/cmd/inspect.rs`. (`slip0132.rs:17-28/108-112/139-153/66/132` line numbers are all CORRECT once the prefix is fixed — spot-checked, exact.)
- **`convert.rs:1589/1591` change site is at `:1593`**, not `:1591`. `mk_codec::decode` is at `:1589` (correct), but `Xpub => card.xpub.to_string()` is at **`:1593`** (the plan says 1591). The `--xpub-prefix` post-process is at `:1086` (correct); the flag at `:300-301` (correct); the `--xpub-prefix`-requires-`--network` refusal is at `:921-926`.
- **`mk-codec key_card.rs:42`** — `origin_path: DerivationPath` confirmed at line 42, BUT the plan cites it as `mnemonic-key/crates/mk-codec/src/key_card.rs` (a git-checkout path). The build actually consumes **crates.io `mk-codec = "0.4.0"`** (`Cargo.toml:35`, `Cargo.lock:714-717` checksum-pinned). Cite the packaged 0.4.0 source, and note this is a published-crate dep (no git wire change is even possible without a crates.io release — which reinforces "NO mk-codec change").
- **`install.sh:32`** is actually **`scripts/install.sh:32`** (`mnemonic-toolkit-v0.58.0`). The version-site checklist's `install.sh:32` path is wrong.
- **`inspect.rs:231`** cited for "forensic mk1" — correct line, but it is the `xpub:` line; the dispatch/`origin_path` lines are `:217-231`.

### I-4 — Helper signature glosses the `NetworkKind → CliNetwork` bridge; `card.xpub.network` is a `NetworkKind`, not a `CliNetwork`.

`apply_xpub_prefix(&Xpub, XpubPrefix, CliNetwork)` (slip0132.rs:108) needs a `CliNetwork`. But `bitcoin 0.32` `Xpub.network` is a `NetworkKind` (Main/Test) — verified by existing use at `synthesize.rs:555` (`c.xpub.network != network.network_kind()`). `network.rs` only provides `CliNetwork → NetworkKind` (`:30`), not the reverse. The plan's pseudocode `net_from(card.xpub)` hides a missing adapter. It is implementable (`Main→Mainnet`, `Test→Testnet`; signet/regtest collapse to Testnet, which is correct since testnet SLIP-0132 prefixes are shared), and it correctly AVOIDS the `--xpub-prefix`-requires-`--network` refusal (that refusal at `:921` only fires for the *flag* path, not an internal derive) — but the plan must specify this adapter rather than wave at it. (Verified the refusal does not block the internal path: `--xpub-prefix xpub --network mainnet` works, and the auto-derive would never set `args.xpub_prefix`, so `:921-926` is not reached.)

---

## Minor

- **M-1 — Round-trip test 1 is not runnable as written.** TDD §1 says `convert --from xpub=<zpub@m/84'> --to mk1` → … . But `convert … --to mk1` is a hard refusal (`refusal_xpub_to_mk1`, `convert.rs:694`; `if from == Xpub && to == Mk1`). mk1 cards are minted via `bundle`/`synthesize`, not `convert --to mk1`. The test must mint the card via `bundle --slot @0.xpub=<zpub>` (which DOES accept and normalize the zpub — verified) then decode. Fix the test recipe.
- **M-2 — SemVer/lockstep claims (assuming the feature ships in SOME form):** MINOR is defensible for a default-display change; no new flag → correctly no `schema_mirror` surface; no new `ToolkitError` variant; the new exhaustive `match` is only inside the pure helper over `XpubPrefix`/path components (no `error.rs` alphabetical-ordering obligation). Version sites enumerated are otherwise complete (Cargo.toml, both READMEs `:13`/`:9`, `scripts/install.sh:32`, `fuzz/Cargo.lock:575`, main `Cargo.lock:727`, CHANGELOG). FOLLOWUP `mk1-card-slip0132-variant-not-preserved-on-card` at `FOLLOWUPS.md:4206` is `open` and would flip — but its resolution text must record the *actual* product decision reached after escalation (resolve-as-stderr-note vs resolve-as-default-change vs WONTFIX), not pre-assume derive-from-path-as-default.
- **M-3 — Non-vacuity claim is sound** (revert change site → variant test emits neutral → RED), but pair it with a POSITIVE anti-regression cell pinning that a *neutral-xpub-at-m/84'* input still round-trips byte-identical if the user's decision is the stderr-note design — otherwise B1/B2/B3 silently encode the inversion.

## Empirical evidence log
- `bundle --network mainnet --template bip84 --slot @0.phrase="abandon…about" --group-size 0` → 2-chunk mk1 card; decodes to `path: 84'/0'/0'`, `xpub: xpub6CatW…` (== `BIP84_REF_XPUB`).
- `--xpub-prefix zpub --network mainnet` on that card → `zpub6rFR7…` (== `BIP84_REF_ZPUB`); `--xpub-prefix xpub --network mainnet` → neutral. Override path works.
- `convert --from xpub=<zpub>` and `bundle --slot @0.xpub=<zpub>` both emit `info: normalized zpub…` and proceed → toolkit OWN intake re-normalizes (mitigation 2 holds; feeding C6 output back into the toolkit is safe).
- `bitcoind -chain=main` `getdescriptorinfo "wpkh(xpub6CatW…/0/*)"` → accepted; `"wpkh(zpub6rFR7…/0/*)"` → **rejected, "key … is not valid"**. External interop break confirmed.
