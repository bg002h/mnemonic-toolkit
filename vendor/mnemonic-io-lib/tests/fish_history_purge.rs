//! F-273 — **fish gets a recipe that was RUN, not a paragraph describing one.**
//!
//! `remedy` shipped zsh and bash as structured recipes and fish as prose,
//! because the obvious fish command — `history delete --prefix` — prompts, and
//! the P0 implementer could not build a harness whose control passed. A
//! description is not a remedy: the operator at a fish prompt is told what does
//! **not** work and left to invent what does.
//!
//! This file is that harness, and every claim in `remedy`'s fish text is
//! measured here rather than reasoned about.
//!
//! ## The harness, and the two things that make it real
//!
//! `script -qc "fish -i" <typescript>` fed from a command file, under an
//! isolated `HOME` / `XDG_DATA_HOME` / `XDG_CONFIG_HOME`. `script` supplies the
//! pty, so fish believes it is interactive and records history at all; the
//! isolation keeps the operator's own `fish_history` untouched — this test
//! writes a secret into a history file, and it must never be a real one.
//!
//! **`TERM` is set to a real terminal on purpose, and that is load-bearing.**
//! Under `TERM=dumb` the whole session runs in well under a second, the control
//! still passes — and `history delete --prefix` stops prompting, returns 0, and
//! still deletes nothing. A harness tuned for speed that way would measure a
//! different program than the operator runs. The cost is fish's ~10s wait for a
//! Primary Device Attribute response that `script`'s pty never sends, paid once
//! per session.
//!
//! ## What was measured, fish 4.8.1 (all under this harness)
//!
//! | attempt | outcome |
//! | --- | --- |
//! | *(control — no purge)* | secret on disk |
//! | `history delete --prefix '<command>'` | **rc 124, killed at a 30s timeout**, secret on disk, and the prompt re-displays it |
//! | `history delete --contains '<command>'` | rc 124 at 30s, secret on disk — same trap |
//! | `history delete --exact '<full line>'` | *"requires --case-sensitive"*, **`$status` 0**, secret on disk — and now in history TWICE |
//! | `history delete --exact --case-sensitive '<full line>'` | purges — but the secret is in history again, as the recipe |
//! | **`history clear-session`** | **purged, unattended, without naming the secret** |
//!
//! The last two rows are the whole argument. Every `history delete` spelling
//! must be handed the material to match on, and the operator types it at a
//! prompt that records what they type — **the secret goes into history a second
//! time to remove it once.** That is the disqualification this module's own
//! header already levels at `mt`'s fish text. `clear-session` matches on
//! nothing, so it needs to be told nothing.
//!
//! **Its cost is that it matches on nothing.** It clears the current session's
//! entire history, an unrelated neighbouring command included — asserted below,
//! and stated in the emitted text, because a remedy that silently destroys more
//! than it was asked to is how an operator stops trusting the next one.
#![cfg(unix)]

use mnemonic_io_lib::remedy;
use std::process::{Command, Stdio};

/// Planted, and searched for. It never leaves the temporary `XDG_DATA_HOME`.
const SECRET: &str = "ms1SECRETSECRETPLANTED";

/// **Deliberately not a command that exists**, and not `me`'s or `mt`'s.
///
/// Not theirs, because this crate is shared and its own tests must not encode
/// either consumer's surface. Not a real binary, because the harness types this
/// line at a live prompt: `me` and `mt` are both on `PATH` on the machine this
/// was written on, and a harness that invokes them is measuring them too. fish
/// records a command it could not resolve just the same — measured, the control
/// below is exactly that case.
const COMMAND: &str = "example-cli pack";

/// Run alongside the secret so the recipe's COST is measurable, not asserted
/// from the manual page.
const NEIGHBOUR: &str = "echo an-unrelated-neighbouring-command";

/// The timeout the prefix measurement is pinned at. It must comfortably exceed
/// fish's own device-attribute wait (~10s here) or a slow *start* would be
/// mistaken for the hang being measured.
const TIMEOUT_SECS: u32 = 30;

fn require(bin: &str, why: &str) -> String {
    assert!(
        std::path::Path::new(bin).exists(),
        "{bin} is required: {why} This is deliberately a FAILURE and not a skip -- \
         a skipped gate prints ok and exit 0. If CI lacks it, install it there rather \
         than weakening this."
    );
    bin.to_string()
}

struct Session {
    /// `fish_history` after the shell exited.
    history: String,
    /// True if the session had to be killed at `TIMEOUT_SECS` — i.e. something
    /// in it was waiting for an answer nobody was there to give.
    timed_out: bool,
}

/// Plant `NEIGHBOUR`, then `COMMAND SECRET`, then `history save`, then
/// `recipe`, in one interactive fish on a pty. Returns the history file **after
/// the shell has exited**, because that is when a shell writes back what it was
/// still holding in memory — the trap F-264 is named for.
fn fish_session(recipe: Option<&str>) -> Session {
    let fish = require(
        "/usr/bin/fish",
        "F-273's gate is 'the emitted recipe, RUN under a real interactive fish, \
         actually removes the entry', and there is no way to run it without fish.",
    );
    let script = require(
        "/usr/bin/script",
        "fish only records history when it believes it is interactive, which needs a pty.",
    );
    let timeout = require(
        "/usr/bin/timeout",
        "one of the measurements IS a hang, so the harness must be able to survive one.",
    );

    let dir = tempfile::tempdir().unwrap();
    let d = dir.path();
    let data = d.join("data");
    let config = d.join("config");
    std::fs::create_dir_all(&data).unwrap();
    std::fs::create_dir_all(&config).unwrap();

    let mut cmds = format!("{NEIGHBOUR}\n{COMMAND} {SECRET}\nhistory save\n");
    if let Some(r) = recipe {
        cmds.push_str(r);
        cmds.push('\n');
    }
    cmds.push_str("exit\n");
    let cmds_path = d.join("cmds.fish");
    std::fs::write(&cmds_path, cmds).unwrap();

    let st = Command::new(timeout)
        .arg(TIMEOUT_SECS.to_string())
        .arg(script)
        .arg("-qc")
        .arg(format!("{fish} -i"))
        .arg(d.join("typescript"))
        .stdin(Stdio::from(std::fs::File::open(&cmds_path).unwrap()))
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .env("HOME", d)
        .env("XDG_DATA_HOME", &data)
        .env("XDG_CONFIG_HOME", &config)
        // See the header: `dumb` would make this ~30x faster and would delete
        // the prompt that half of these measurements are about.
        .env("TERM", "xterm")
        .status()
        .expect("`script` (util-linux) is required to give fish a pty");

    // `timeout` reports a killed child as 124.
    let timed_out = st.code() == Some(124);
    let history =
        std::fs::read_to_string(data.join("fish").join("fish_history")).unwrap_or_default();
    Session { history, timed_out }
}

/// **THE CONTROL, and it runs first for a reason.** A harness that fails to
/// record history at all reports "purged" for every recipe, including one that
/// does nothing — which is exactly how F-273 came to be deferred rather than
/// answered. Nothing below this line means anything until this passes.
#[test]
fn the_harness_records_history_at_all() {
    let s = fish_session(None);
    assert!(
        !s.timed_out,
        "the control session was killed at {TIMEOUT_SECS}s; nothing can be concluded"
    );
    assert!(
        s.history.contains(SECRET),
        "with NO purge attempt the planted secret must reach disk, or this file is \
         measuring itself rather than the recipe. fish_history was:\n{}",
        s.history
    );
    assert!(
        s.history.contains(NEIGHBOUR),
        "the neighbouring command must be recorded too, or the cost assertion below \
         is vacuous. fish_history was:\n{}",
        s.history
    );
}

/// **THE GATE.** The recipe `remedy` actually emits, run under a real
/// interactive fish, removes the entry — unattended, with nobody to answer a
/// prompt.
#[test]
fn the_emitted_fish_recipe_actually_purges_the_entry() {
    let recipes = remedy::history_purge_recipes(COMMAND);
    let (_, fish) = recipes
        .iter()
        .find(|(s, _)| *s == "fish")
        .expect("a fish recipe must exist -- F-273: fish was DESCRIBED and not PRESCRIBED");

    let s = fish_session(Some(fish));
    assert!(
        !s.timed_out,
        "the emitted fish recipe `{fish}` blocked until the session was killed at \
         {TIMEOUT_SECS}s. A recipe that waits for an answer is not one an operator \
         can be handed in a refusal message -- that is F-273 itself."
    );
    assert!(
        !s.history.contains(SECRET),
        "the emitted fish recipe reported success and purged nothing. fish_history \
         after the session exited was:\n{}",
        s.history
    );
}

/// **THE RECIPE'S COST, measured and then required to be stated.**
///
/// `history clear-session` matches on nothing, which is why it needs to be told
/// nothing — and is also why it takes the whole session with it. An operator who
/// finds unrelated commands gone from their history, unwarned, learns not to run
/// the next remedy this tool prints.
///
/// The second half is the half that rots: the measurement stays true on its own,
/// while the sentence describing it is one tidy-up away from being deleted.
#[test]
fn the_recipe_costs_the_whole_session_and_the_text_says_so() {
    let recipes = remedy::history_purge_recipes(COMMAND);
    let (_, fish) = recipes
        .iter()
        .find(|(s, _)| *s == "fish")
        .expect("a fish recipe must exist -- F-273: fish was DESCRIBED and not PRESCRIBED");

    let s = fish_session(Some(fish));
    assert!(
        !s.history.contains(NEIGHBOUR),
        "this assertion exists to FAIL if fish ever gains a targeted purge that does \
         not name the secret -- at which point the recipe should become that, and the \
         cost sentence should go. fish_history was:\n{}",
        s.history
    );

    let block = remedy::history_purge_block(COMMAND);
    assert!(
        block.contains("whole session"),
        "the emitted text must SAY that the fish recipe clears the session's entire \
         history, because it measurably does:\n{block}"
    );
}

/// **F-273's own finding, reproduced by an independent harness and kept.**
///
/// `history delete --prefix` is the command an operator reaches for, and it is
/// the reason fish shipped as prose. **It purges nothing.** That is the whole
/// finding, and it is what this test asserts.
///
/// **It fails in two different ways, and the assertion deliberately covers
/// both** — measured 2026-08-27 on two fish versions:
///
/// | fish | what `delete --prefix` does | secret afterwards |
/// | --- | --- | --- |
/// | 4.8.1 (this machine) | prompts, listing the matches **including the secret**, and never returns — killed at the harness timeout | still on disk |
/// | 3.7.0 (ubuntu-noble, CI) | returns unattended, exit 0, no prompt | **still on disk** |
///
/// An earlier version of this test asserted the **hang**, and so it passed
/// locally and went RED the first time CI ran it — not because the finding was
/// wrong, but because the hang is a *mechanism* and the finding is *the secret
/// survives*. Asserting the mechanism made the test version-specific and
/// strictly weaker: it could not have caught the 3.7.0 behaviour at all, which
/// is arguably the worse of the two. A command that hangs is visibly unfinished;
/// one that returns 0 having done nothing looks exactly like success, and that
/// is the shape this whole module exists to warn about.
///
/// **If this test ever fails, fish has started actually deleting** — and only
/// then may the recipe become a targeted one. Re-measure before rewriting it.
#[test]
fn history_delete_prefix_purges_nothing_however_it_fails() {
    let s = fish_session(Some(&format!("history delete --prefix '{COMMAND}'")));
    assert!(
        s.history.contains(SECRET),
        "`history delete --prefix` DELETED the entry. That is a behaviour change \
         in fish, not a broken test: every version measured leaves the secret \
         exactly where it was, whether it hangs at a prompt (4.8.1) or returns \
         unattended at exit 0 (3.7.0). Re-measure both modes before making the \
         emitted recipe a targeted delete. fish_history was:\n{}",
        s.history
    );
    // Which of the two failure modes this fish took is recorded rather than
    // asserted -- both leave the secret, and pinning one of them is exactly the
    // mistake that sent this test RED in CI while the finding was sound.
    println!(
        "delete --prefix mode: {}",
        if s.timed_out {
            "hung at a prompt, killed at the timeout (the 4.8.1 shape)"
        } else {
            "returned unattended, purging nothing (the 3.7.0 shape)"
        }
    );
}

/// **`--exact` is the plausible fix, and it is F-264 wearing a fish hat.**
///
/// It is the one `history delete` spelling the manual says does not prompt, so
/// it is what a reader reaches for on being told `--prefix` hangs. Measured:
/// fish prints *"builtin history delete --exact requires --case-sensitive"*,
/// **exits 0**, and the entry is still in memory and still on disk. A recipe
/// built on it would report success and purge nothing — the exact defect this
/// module's header exists to warn about.
///
/// And the spelling that *does* work, `--exact --case-sensitive`, has to be
/// handed the whole command line, secret included, at a prompt that records it.
/// **It removes one copy of the secret by typing a second.** That is why the
/// recipe is `clear-session` and not a targeted delete, and this test is the
/// evidence for it — without which the next reader tidies `clear-session` into
/// something "more precise" and reintroduces both defects at once.
#[test]
fn history_delete_exact_reports_success_and_purges_nothing() {
    let s = fish_session(Some(&format!(
        "history delete --exact '{COMMAND} {SECRET}'"
    )));
    assert!(
        !s.timed_out,
        "`--exact` is the non-prompting spelling; if it now blocks, the manual and \
         the binary have diverged further, not less"
    );
    assert!(
        s.history.contains(SECRET),
        "if `history delete --exact` has started working without --case-sensitive, \
         re-measure the whole table in this file's header before changing the recipe. \
         fish_history was:\n{}",
        s.history
    );
}
