# Fold check: Fable I1 (49bb9ce) and docs commit (1612ed3)

Repo /home/user/mnemonic-toolkit, branch claude/project-thread-utm1aa, HEAD 1612ed3. Read-only check (no commits).

**Verdict: 0 Critical, 0 Important, 1 Minor, 2 Nit. I1 is closed.**

## What the fold does

`bundle_run_unified` (bundle.rs:460-481) now refuses any `--slot @N.path=` that parses (`DerivationPath::from_str`) to an empty path. The check runs after the descriptor-mode return, so only `--template` mode reaches it. It returns `SlotInputViolation { kind: "conflict" }` (exit 2) with the same text as the descriptor-mode twin (bundle.rs:2811-2818), except that "descriptor shape" becomes "template".

## Is it mint-only?

Yes, by construction. `bundle_run_unified` has one caller, `cmd::bundle::run` (bundle.rs:382), and `bundle::run` is called only for `Command::Bundle` (cli.rs:197). The diff touches no other function. `resolve_slots`, verify-bundle, export-wallet and restore are unchanged. `--import-json` returns before this point (bundle.rs:282) and takes the descriptor route, so it is not a template mint.

## Tests

- `cargo test --locked -j 2 -p mnemonic-toolkit --test all -- bughunt_1125` (CARGO_INCREMENTAL=0): 32 passed, 0 failed. This includes `tk1_explicit_master_path_in_template_mode_is_refused`, `tk4_*` and `tk2_*`.
- Full `--test all`, run once: **2940 passed, 0 failed, 19 ignored**. No regression.

## Binary probes (target/debug/mnemonic, mainnet unless noted)

| Input | Result |
|---|---|
| bip44 / bip49 / bip84 / bip86 with `@0.xpub=<master>` and `@0.path=m` | exit 2, I1 text |
| bip84 `@0.path=m/` | exit 2, I1 text (the text echoes `m`, see n1) |
| bip84 `@0.path=M`, `M/`, `" m"`, `"m "`, `/` | exit 1, "--slot @0.path parse: invalid child number format". This refusal was already there; `resolve_slots` would also fail on these. No card is minted. |
| bip84 `@0.path=` (empty) | exit 64, rejected by clap ("slot input value is empty") |
| testnet bip84 `path=m` | exit 2, I1 text |
| wsh-multi, wsh-sortedmulti, sh-wsh-multi, sh-wsh-sortedmulti, tr-multi-a, tr-sortedmulti-a, `--threshold 1`, `@0` xpub with `path=m` | exit 2, I1 text (all six) |
| The same six templates, `@0` seed and `@1` xpub with `@1.path=m/` | exit 2, I1 text naming `@1` |
| The same six templates, both typed non-empty paths (`m/87h/0h/0h` and `m/87'/0'/0'`) | exit 0, card at `m/87'/0'/0'` |
| bip84 typed `m/84'/0'/0'` | exit 0 |
| bip84 typed non-standard `m/0`, with the xpub at m/0 | exit 0. The typed-path tolerance still holds. |
| Seed slot (`@0.phrase=`) with `@0.path=m` on bip84 | exit 2, refused before any card is minted (see m1) |
| fingerprint and `path=m` with no key | exit 2, refused by the §6.6.b validity matrix, as before |
| Descriptor twin `wpkh(@0/<0;1>/*)` with xpub and `path=m` | exit 2, same kind and text. Template and descriptor modes agree on exit code and error kind. |

A seed slot cannot mint an empty origin in template mode. A seed with any `.path` is refused (tk4), and a seed slot is always derived at the template or multisig-family path, which is never empty. With no `.path`, an xpub slot takes the template default, which is non-empty. The typed empty path was the only way to mint one, and it is now closed for all 10 templates, single-sig and multisig, on mainnet and testnet. No valid non-empty path is refused.

## Findings

### Minor

**m1: A seed slot with `path=m` now gets the I1 text instead of the tk4 text.**
- Repro: `bundle --template bip84 --slot @0.phrase=<A> --slot @0.path=m`. It exits 2 with "the master node cannot be recorded ... Give the key's real origin path".
- With any other path, the tk4 refusal says that `--template` ignores `@0.path=` beside a seed. The I1 check runs first.
- The exit code is the same (2) and no card is minted. But the advice "Give the key's real origin path" steers the user to an input that tk4 then refuses. This is the m3 class.
- Fix: limit the loop to slots that also carry `SlotSubkey::Xpub`, so tk4 owns seed slots. Or leave it as is and note it in `tk-bh1125-fable-m3-notice-advises-refused-override`.

### Nit

- **n1:** `@0.path=m/` is refused with text that says `` `--slot @0.path=m` ``. The text names the canonical spelling, not the typed value. This is harmless.
- **n2:** The test covers bip84 and wsh-sortedmulti with `m` only. The `m/` spelling and an `@1` (non-first) slot are untested. The probes above show both are refused.

## 1612ed3

1612ed3 contains no code. It adds `design/FOLLOWUPS.md` (four open entries: fable m1, m2, m3, nits) and the review file under `design/agent-reports/`. The manual paragraph in 49bb9ce (41-mnemonic.md:652-654) matches the behaviour.
