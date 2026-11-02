**⚠ Critical 1 (graded up from proposed Important), Important 1 (proposed, new edge, unconfirmed by test), Minor 1.**

architect grade-convert-path: 3 answered; 0 prior rulings changed

# Grade + ruling: tk `convert --path` ignored (triage 7f5a535e)

Worktree tk-worktrees/triage-convert-path, HEAD 7f5a535e (base origin/master 6ccfdebc); binary
`target/debug/mnemonic` built 2026-11-02 01:35:57, after the HEAD commit at 01:35:20. Seed: BIP-39
"abandon x11 about", signet. The controller's settled facts (RED test fails; convert.rs 1303-1321
never reads `args.path`) were not re-derived. Every other claim below was run or read by me.

## Extra facts I checked (binary runs)
- `--template wsh-sortedmulti --to xpub --path "m/99'/1'/5'" --json` exits 0 and prints
  `{"schema_version":"1","from_node":"phrase","to":[{"node":"xpub","value":"tpubDCChhoz5...merw"}]}`.
  This is the template-path key. Neither the human output nor the JSON shows a path, so the holder
  has no way to see that the wrong path was used.
- `--template bip84 --to xprv --path "m/99'/1'/5'"` prints the template-path xprv, exit 0.
- `--to wif --path bip84` and `--to wif --path 0x54` both exit 1 with `error: --path parse: invalid
  child number format`. Every `--path` read in convert.rs (1372, 1427, 1470, 1531) uses
  `bip32::DerivationPath::from_str`, so no convert edge accepts the named or hex forms that
  `--help` lists (convert.rs:234-240).
- `--from xprv=<bip84 account tprv> --to xpub --path m/0` prints the same xpub as without
  `--path`. The Xprv arm (convert.rs:1494) never reads `args.path`.
- convert has no `--multisig-path-family` (it is not in convert.rs). For multisig templates,
  `derive_bip32_from_entropy` uses the BIP-87 fallback (derive_slot.rs:51-57). So `--path` is the
  only way to get a BIP-48 cosigner xpub out of `convert`, and that is the call it drops.
- Existing tests that pass `--path` to convert are cli_convert_refusals.rs:113 (wif source, already
  refused, exit 2) and cli_convert_happy_paths.rs:272/299/508 (single `--to` leaf edges). None
  combines `--path` with an xpub, xprv or fingerprint target from a seed source, so the ruling
  below changes no pinned output.

## Q1. Grade of the CONFIRMED finding: Critical
The bug fails open. It exits 0 and prints a plausible xpub or xprv for a path other than the one
typed. The trigger is one ordinary documented flag (manual 41-mnemonic.md:1102 says "derivation
path override", and `--account` help at convert.rs:286 says the template path is used "when
`--path` is omitted"). It fails all three limbs of the Minor test. I grade it Critical rather than
Important because it is a wrong result in the workspace's own terms ("Critical (wrong result / data
loss / ...)"), and the holder cannot detect it: no output form shows the path. The realistic case is
a holder building a BIP-48 cosigner key (`--path m/48'/1'/0'/2'`). They get the BIP-87 key, the
descriptor's origin will not match the key, and a signer at that origin cannot sign until someone
rediscovers the real path. This repo has graded the same pattern as funds-safety before: a silently
dropped origin path, FOLLOWUPS.md:4694 `descriptor-prefix-form-origin-annotation-ignored`. The
triage argued "not Critical: no key loss". That argument covers data loss, not wrong results.

## Q1b. New finding (proposed Important, UNRESOLVED: binary-reproduced, no RED test yet)
`--from xprv=... --to xpub|fingerprint --path P` silently ignores P (exit 0). This is the same class
on a different arm (convert.rs:1494), with one condition. I have not run the xpub and mk1 source
arms. Rule R4 below covers all of them. Per the triage rule this needs its own RED test before it
can be called CONFIRMED.

## Q1c. Minor: `--help` lists `--path` forms that convert never accepts
convert.rs:234-236 lists `named: bip44/...` and `hex: 0xNN`. Both are refused with exit 1 (shown
above). This fails closed, and it is help text, not a printed command, so it is Minor. R5 fixes it.

## Q2. Ruling: honour `--path` as an override; refuse it wherever nothing reads it
Why: the manual already promises "override", and `--account`'s help already implies that `--path`
takes precedence over the template path. `md encode --path` and `restore --path` are overrides too
(42-md.md:34, :621). Honouring the flag gives holders their only `convert` route to a BIP-48 key,
which is fewest steps. Refusing everywhere would leave no route and contradict two shipped texts.
The wif and address edges already read `--path` as a literal full path from master (convert.rs
1367-1381, 1463-1478), so one meaning covers every seed-source target. The `--from wif` refusal
(convert.rs:509-513, exit 2) is the precedent for refusing where the flag cannot apply. Warning
only (option 3) still prints a wrong-path key and is rejected.

Exact behaviour (sources phrase / seedqr / entropy unless stated):
- R1 `--to xpub`, `--to xprv`: with `--path P`, print the extended key at P, derived from master
  (`derive_bip32_from_entropy_at_path`, derive_slot.rs:66). `--template` is not required when
  `--path` is given. When both are given, `--path` sets the key and the template's path is not
  used. If `--to wif|address|bip38` is also requested, every target uses the same P (one node).
- R2 `--to fingerprint`: still the master fingerprint. `--path` and `--template` do not change it.
  `--path` satisfies the derivation-target requirement in place of `--template`. The existing text
  changes to `--template or --path is required for derivation targets (xpub/xprv/fingerprint)`
  (BadInput, exit 1, as today).
- R3 `--account N` with N != 0 together with `--path`, on any edge: refuse with exit 2
  (ConvertRefusal, error.rs:653), text:
  `--account has no effect when --path is given; put the account in --path (e.g. m/84'/1'/3') or drop --path.`
  Limitation: clap's `default_value = "0"` means an explicit `--account 0` cannot be told apart from
  the default. It is accepted, and is harmless.
- R4 `--path` given when no requested target reads it (from a seed source that is fingerprint,
  phrase, entropy, ms1 and the like; from xprv, xpub (except address) and mk1 sources, every
  target): refuse with exit 2, text:
  `--path is not used by any requested --to target from this --from source; drop --path.`
  The `--from wif` refusal keeps its own text and fires first (convert.rs:950-952).
- R5 Accepted forms: literal `m/...` only (including bare `m`), the same parser as wif/address.
  Parse errors keep `--path parse: <e>` (BadInput, exit 1). No coin-type/`--network` cross-check
  is added (wif and address have none). If the operator wants one, it is a separate product point.

Text changes:
- convert.rs:234-240 help becomes:
  `BIP-32 derivation path from the master key, literal form `m/...`. Overrides the
  --template/--account path for --to xpub/xprv; required for --to wif, --to address
  (from a seed) and composite --to bip38. Refused where no requested target uses it.`
- convert.rs:285-286 `--account` help: append `Refused (non-zero) together with --path.`
- Manual 41-mnemonic.md:1102 row becomes:
  `| `--path <PATH>` | BIP-32 path from the master key, literal `m/...` only. For `--to xpub`/`--to xprv` it overrides the `--template`/`--account` path (and makes `--template` optional); required for `--to wif`, `--to address` from a seed, and composite `--to bip38`. Does not change `--to fingerprint` (always the master fingerprint). Refused (exit 2) with a non-zero `--account`, or when no requested target uses it |`
- Manual :1103 `--account` row: append `; refused (exit 2) when non-zero together with `--path``.
- Tests: keep the triage RED test (honour satisfies it). Add rows for R1 (xpub at
  `m/48'/1'/0'/2'` matches the `--to wif`/bip32 derivation at that path; xprv likewise; no
  `--template`), R2, R3 (exact text, exit 2), R4 (xprv source and seed->fingerprint only; exact
  text, exit 2), R5 (named form exit 1).

Not a tie: honouring costs one extra derivation call compared with refusing, but refusing breaks
two shipped promises and leaves the BIP-48 case unreachable. Fold needed: yes (code + help + manual
in lockstep, per RULES "printed-text change updates the toolkit manual").

## Q3. Small-fix track? Yes
It is one repo (mnemonic-toolkit; the manual lives in the same repo), adds no new flag, adds two
refusal strings, rewrites one help block, and changes two manual rows. The spec delta is about five
sentences (R1-R5), all in one file (convert.rs) plus tests.

## Open / Brian's alone
None. Whether `--path` should be checked against `--network`'s coin type (R5) is left out on
purpose. It is not needed for this fix.

Counts: Critical 1, Important 1 (proposed, unconfirmed), Minor 1, Nit 0; questions answered 3; prior rulings changed 0.
