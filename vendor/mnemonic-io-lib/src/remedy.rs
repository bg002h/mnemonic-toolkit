//! Purge and remedy text — **`me`'s alone**.
//!
//! **`mt`'s purge text is NOT a source, and this is a disqualification on
//! evidence.** It advises zsh operators `history -d`, which does not delete on
//! zsh 5.9.2 (`-d` prints timestamps), and it tells fish operators to match on
//! the bearer material — typing the secret into history a second time, which is
//! the very thing a purge is for. **That second half is measured rather than
//! argued.** Under fish 4.8.1, `history delete --exact --case-sensitive '<the
//! whole command line>'` does remove the planted entry and does leave the secret
//! in the history file, because the recipe *is* a line containing it — and the
//! spelling without `--case-sensitive` prints a complaint, **exits 0**, and
//! deletes nothing at all. See `tests/fish_history_purge.rs`, where both are run.
//!
//! ## F-264 — the recipe that reported success and purged nothing
//!
//! `me`'s own text was not clean either. It offered, for both bash and zsh:
//!
//! ```text
//! sed -i '/me sysw pack/d' "$HISTFILE"
//! ```
//!
//! The operator puts a secret on argv, `me` refuses and prints that, they run
//! it **immediately** as the message invites — and the shell is still holding
//! the entry **in memory**. `HISTFILE` does not contain it yet. `sed -i` edits
//! a file the secret is not in, **exits 0, prints nothing**, and at session
//! exit the shell writes its in-memory history, secret included, to disk.
//!
//! **That is the same defect as `history -d`, in the message that exists to
//! warn against `history -d`.**
//!
//! ### Measured, not reasoned — and the first proposed fix failed too
//!
//! Under stock zsh 5.9.2 and bash 5, on a real pty, with a control that plants
//! the entry and purges nothing (the secret must reach disk, or the harness is
//! measuring itself):
//!
//! | recipe | outcome |
//! | --- | --- |
//! | *(control — no purge)* | secret on disk |
//! | `sed -i …` alone — **what shipped** | secret on disk |
//! | `fc -W; sed -i …; fc -R` | **secret on disk** |
//! | `fc -W; sed -i …; HISTSIZE=0; HISTSIZE=$h; fc -R` | purged |
//!
//! **The three-step flush-edit-reload fix does not work on its own**, and it is
//! what F-264 and the plan both proposed. `fc -R` *appends* the file to the
//! in-memory list rather than replacing it, so the entry is still in memory and
//! is written back at exit. Zeroing `HISTSIZE` is what actually empties memory;
//! restoring it and re-reading rebuilds the history from the cleaned file, so
//! nothing else in the session is lost.
//!
//! **bash has the identical defect and needed the identical shape** — flush,
//! edit, **clear memory**, reload — which is why the two shells no longer share
//! one line. The shipped text said `bash/zsh:` and was wrong for both.

/// The purge recipes, one per shell, as `(shell, recipe)`.
///
/// Public and structured so a test can **run the emitted recipe** rather than a
/// copy of it. §6 condition 5 asks for a positive test — run it under a real
/// interactive shell and assert the entry is gone — and a test that runs its
/// own hard-coded string proves only that the string works.
///
/// `command` is matched on rather than the secret: quoting the secret into a
/// `sed` pattern is how an operator types it into history a second time.
///
/// **The pattern is word-bounded (`\b…\b`), and that is not decoration.** The
/// bare surface's command is just `me`, and `sed '/me/d'` deletes `make`,
/// `time`, `some`, `name` and `/home/…` — measured on a six-line sample, where
/// plain `/me/d` left ONE line of six standing. `\bme\b` left four, removing
/// only the invocation and `cd /home/me`. GNU `sed` is already assumed here:
/// `-i` without an argument is GNU-only.
///
/// **fish ignores `command`, and that is the whole of F-273.** Every fish
/// `history delete` spelling has to be handed the material to match on, at a
/// prompt that records what is typed, so a targeted fish recipe removes one copy
/// of the secret by writing a second. `history clear-session` is the one that
/// purges unattended without being told what to look for — see
/// [`history_purge_block`] for what that costs and `tests/fish_history_purge.rs`
/// for the five spellings that were run before settling on it.
pub fn history_purge_recipes(command: &str) -> Vec<(&'static str, String)> {
    vec![
        (
            "zsh",
            // fc -W       flush memory to the file, so the entry is IN it
            // sed -i      remove it from the file
            // HISTSIZE=0  empty the in-memory list -- `fc -R` alone APPENDS,
            //             so without this the entry survives in memory and the
            //             shell writes it back at exit (measured)
            // HISTSIZE=$h restore the operator's own size, not a guess
            // fc -R       rebuild memory from the cleaned file
            format!(
                "fc -W; sed -i '/\\b{command}\\b/d' \"$HISTFILE\"; \
                 h=$HISTSIZE; HISTSIZE=0; HISTSIZE=$h; fc -R"
            ),
        ),
        (
            "bash",
            format!(
                "history -w; sed -i '/\\b{command}\\b/d' \"$HISTFILE\"; \
                 history -c; history -r"
            ),
        ),
        (
            "fish",
            // No `command` interpolation, deliberately -- see this function's
            // doc comment. `clear-session` matches on nothing, needs to be told
            // nothing, and so cannot re-type the secret at a recording prompt.
            "history clear-session".to_string(),
        ),
    ]
}

/// The purge paragraph as it is printed, indented to sit inside a refusal.
///
/// **`history -d` is NAMED here and never OFFERED**, and the distinction is the
/// gate. The donor's own test file records the trap:
///
/// > *"NOT `!err.contains("history -d")` — the message deliberately NAMES that
/// > command in order to warn against it, so the naive negative fails on the
/// > warning itself. The requirement is that it is never OFFERED."*
///
/// So the recipes are structured data and the warning is prose, and the test
/// asserts `history -d` appears in **no recipe** while still appearing in the
/// text. A gate written as "does not contain the string" goes RED against the
/// correct text and can only be made green by deleting the warning — recreating
/// the exact defect that disqualifies `mt`'s wording.
///
/// **fish is now PRESCRIBED, and its cost is printed with it — F-273.**
///
/// It shipped as prose because `history delete --prefix` prompts: reproduced by
/// an independent harness, it is killed at a 30-second timeout with the entry
/// still on disk, and the prompt lists the matching commands — the secret with
/// them. `--contains` does the same. `--exact` is the spelling the manual says
/// does not prompt, and without `--case-sensitive` it complains, **exits 0** and
/// deletes nothing; with it, it works, and it works only by being handed the
/// whole command line at a prompt that records it.
///
/// `history clear-session` purges unattended and never names the secret, so it
/// is what is offered. **It clears the current session's entire history**, an
/// operator's unrelated commands included, and reaches no earlier session —
/// measured: an entry planted in one session survives `clear-session` run in the
/// next. Both limits are stated in the emitted text rather than left to be
/// discovered, because a remedy that silently destroys more than it was asked to
/// is how an operator learns to ignore the next one.
pub fn history_purge_block(command: &str) -> String {
    let mut s = String::new();
    s.push_str(
        "TO PURGE WHAT ALREADY LEAKED -- match on the COMMAND, never on the \
         secret, or you type it into history a second time. Run ALL of the \
         steps: your shell is still holding that entry in MEMORY, so editing \
         the history FILE alone changes nothing and the entry is written back \
         when you exit.\n",
    );
    for (shell, recipe) in history_purge_recipes(command) {
        s.push_str(&format!(
            "      \x20   {:<7} {recipe}\n",
            format!("{shell}:")
        ));
    }
    s.push_str(
        "      \x20   (fish's recipe clears the whole session's history, not \
         just the leaked line -- unrelated commands from this shell go with it, \
         and entries from EARLIER fish sessions are not reached, so run it in \
         the shell that leaked. Every `history delete` spelling has to be handed \
         the secret to match on at a prompt that records it, and --prefix blocks \
         on that prompt without deleting anything.)\n",
    );
    s.push_str("      \x20   and `shred -u` any file you pasted it from.\n");
    s.push_str(
        "      (On zsh, `history -d` does NOT delete -- -d prints timestamps. \
         It would report success and purge nothing. Editing the file on its \
         own is the SAME trap, measured on stock zsh 5.9.2 and bash 5: the \
         entry is only in memory, sed exits 0 having changed a file that never \
         held it, and the shell saves it at exit anyway.)",
    );
    s
}
