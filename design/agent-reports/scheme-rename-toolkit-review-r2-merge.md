# Scheme rename (PR #229): review r2, merge commit cf39a185

Scope: `git diff dcc0f62a cf39a185`, limited to the files #227 touched since 40217d8b, plus the restore.rs conflict resolution (`git show --remerge-diff cf39a185`). The source was not edited.

## Verdict: GREEN (0 Critical, 0 Important)

## Checks

1. **restore.rs conflict.** The only hunk in conflict was in `run_multisig` (restore.rs:4347-4351). The resolution takes #227's side: the early single-sig refusal is dropped, and the check moves to `asserted_origins`. Only the comment was respelled. In `asserted_origins` (4029-4111) and `missing_origin_refusal` (4141-4145), the logic is the same token for token. Repeatable `--origin`, the refusal of a key whose card already carries an origin, the duplicate refusal, the script-match check, the "both … give one" conflict, and the master-key handling all match #227. Only `restore_flag("template")` changed to `restore_flag("scheme")` (2 sites), and the arg's id is `"scheme"` (restore.rs:231-238). No restore lookup by the id "template" is left: grep finds none, and `restore_flag` panics on an unknown id, so a stale lookup would have failed the tests that ran.
2. **`redact_and_quote` (bundle.rs:3429-3432).** This is correct and safe.
   - The rewrite branches come after all the secret, `--descriptor` and `--slot`/`--from`/`--passphrase` branches. Those consume their value token first, so a value that is literally `--template` is redacted or passed to the descriptor hook, never rewritten. Redaction is unchanged.
   - It matches only the exact token `--template` and the `--template=` prefix, so `--template-x` is left alone. A quoted descriptor containing the text is one token and is not matched.
   - No bundle/restore arg has `allow_hyphen_values`, positionals, `trailing_var_arg` or `infer_long_args`. So a `--template` in value position, or after `--`, is refused by clap before the program runs.
   - Only bundle.rs:3109/3117/3294 and restore.rs:4174 call it. build-descriptor never reaches it, so `--archetype` does not matter here.
   - The text says "to state this choice explicitly, run:", so it is a command to run, not a quote of what the user typed. The canonical spelling is right, and both spellings give byte-identical output, which `origin_ruling_refusals_name_scheme_under_either_spelling` asserts.
3. **Printed text.** No printed `--template` meaning the scheme is left in src. The remaining one, `taproot_override_classify.rs:96` (`md descriptor --template`), is md's flag and is correct, and so is manual 41-mnemonic.md:1946. The leftovers in manual-gui (42-bundle.md, 4d-restore.md) are deliberate and recorded in FOLLOWUPS `manual-gui-scheme-rename`.
4. **Tests.**
   - bughunt_1125_origin_rulings.rs: the merge respells 9 places and drops no assertion. At :1117, `!contains("--template")` became `!contains("--scheme")`. That keeps the "does not steer to a scheme" intent, which would otherwise have become vacuous.
   - The new alias test checks byte-identity across both spellings and exit codes 4 and 2. It requires that `--scheme` text is present and `--template` is absent.

## Minor

- M1 (bundle.rs:3429): no unit test covers the `--template=bip84` (equals) form, or restore's echo of the alias at restore.rs:4174. A row in `rerun_command_never_echoes_an_inline_secret` would pin both.
- M2 (bundle.rs mode_text): prose still says "pick descriptor passthrough or template" and "single-sig templates ignore threshold". These are byte-pinned strings, the flag names in them are correct, and this is cosmetic only.

## Runs (logs in /home/user/scratch-tk-review2/)

- `cargo test --locked -p mnemonic-toolkit --test all -- bughunt_1125 cli_scheme_flag_alias cli_restore_multisig cli_verify_bundle_md1_template`: 158 passed, 0 failed.
- `--lib -- rerun_command redact`: 5 passed.

## Fold (controller)

- M1: new unit test `rerun_command_prints_the_scheme_spelling_for_the_hidden_alias` covers `--template VALUE`, `--template=VALUE` and leaves `--template-other` alone. Restore reuses the same `redact_and_quote`.
- M2: cosmetic prose, left.
