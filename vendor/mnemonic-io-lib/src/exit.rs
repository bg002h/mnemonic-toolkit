//! **Refusal DECISIONS, their wording, and the ordering rule — and NO
//! INTEGERS.**
//!
//! ## Why the crate publishes no exit constant at all
//!
//! Calling this "the exit-code table" implies there is one table. Measured on
//! the same invalid-artifact input: **`md` 1, `mk` 2, `ms` 1, `mnemonic` 2**;
//! and for a clap usage error **`md` 2, `mk` 64, `ms` 64, `mt` 2, `me` 2**. The
//! binaries agree on almost nothing, the usage-code split is ruled out of
//! scope, and `ms` maps clap errors to 64 deliberately, with a comment citing
//! its own spec's carve-out.
//!
//! **A CONSTANT IS A MAPPING**, and "meanings, not a table" does not separate
//! the two. A published `EXIT_USAGE = 2` — the donor's number — would leave
//! `ms` two choices: adopt it, which is out of scope, or ignore it, which makes
//! the constant decorative. So:
//!
//! | this module holds | this module does NOT hold |
//! | --- | --- |
//! | the refusal DECISION types | any exit integer |
//! | the wording of each refusal | any binary→code mapping |
//! | the ordering rule — which gate outranks which | a "usage error" number |
//!
//! Each binary maps a decision onto its own code. Publishing an integer would
//! do to the exit codes exactly what publishing `0o044` would do to the mask.
//!
//! ## Decision, not announcement
//!
//! `write_block` decides; the caller emits. Every `refuse_*` stays in the
//! binary, because a library six binaries share cannot write to stdio
//! unconditionally — it cannot be tested without capturing process stdio and a
//! caller cannot redirect it, which is doubly wrong in a module whose whole
//! purpose is controlling what reaches stdout.

use super::channel::{destination, Destination};
use super::observation::PayloadKind;

/// Why a write cannot proceed — **the single decision**, so the early check and
/// `emit`'s cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WriteBlock {
    /// Nothing in the way.
    None,
    /// stdout is a terminal (F-253), **carrying what the payload is** so the
    /// refusal can say something true about it rather than assert the gate's
    /// name. F-259: this arm used to be a bare `Terminal`, and the only fact
    /// that could have corrected its message was riding a `bool` it never
    /// consulted.
    Terminal(PayloadKind),
    /// stdout is a file whose mode grants group/other read (F-252), carrying
    /// the mode so the refusal can quote what it measured.
    WorldReadable(u32),
}

/// **F-246 binds every gate, including ones added later.** The rule is that no
/// line describing a container may print until every gate that can abort the
/// write has run — so both gates are decided here, once, and consulted at the
/// top of `pack` as well as inside `emit`.
///
/// Terminal is checked FIRST because the mode check structurally cannot see it:
/// a TTY is a character device, and those are exempt there (that exemption is
/// load-bearing for `/dev/null`, mode 0666).
pub fn write_block(
    out_given: bool,
    kind: PayloadKind,
    allow_world_readable: bool,
    stdout_is_tty: bool,
    world_readable_mode: Option<u32>,
) -> WriteBlock {
    match destination(out_given, stdout_is_tty) {
        // `--out`: `me` creates the file 0600 itself, so neither gate applies.
        Destination::File => WriteBlock::None,
        // `--allow-world-readable` does NOT override this. It says "this file's
        // permissions are my problem"; it is not a request to paint a payload
        // across a scrollback, and the message offers a file route.
        //
        // **The KIND rides along rather than deciding here (F-259).** A fill
        // image is refused too — 64 KB of binary in a scrollback is worth
        // refusing whatever the secrecy — but the refusal has to be able to say
        // WHICH it is refusing, and before this the fact simply was not here.
        Destination::Terminal => WriteBlock::Terminal(kind),
        // The mode gate asks about EXPOSURE, so it is the one gate the kind
        // genuinely settles: there is nothing to expose in a fill image, and
        // `wipe` used to buy this outcome by passing `true` in the FLAG's seat.
        Destination::Stream => match world_readable_mode {
            Some(mode) if !allow_world_readable && kind.exposure_matters() => {
                WriteBlock::WorldReadable(mode)
            }
            _ => WriteBlock::None,
        },
    }
}
