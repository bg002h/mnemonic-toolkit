//! **`mnemonic-io-lib` — the IO and safety MECHANISM six m-format binaries
//! share, and none of the policy they disagree about.**
//!
//! `md`, `mk`, `ms`, `mt`, `me` and `mnemonic` each solve the same IO and
//! safety problems their own way. `me` solves them most completely, so `me` is
//! the donor. What makes the extraction risky is not the code — it is that the
//! code already exists twice and **the two copies disagree**, so extracting the
//! wrong half would freeze a disagreement into a shared dependency.
//!
//! ## The seam: mechanism is shared, policy is not
//!
//! The measurement that settles it. `mt` already ships `me`'s
//! `stdout_world_readable_mode` as `world_readable_stdout_guard` — the same
//! code, comments included, down to extracting the mode. Then every policy
//! decision diverges. Run with a valid transaction on stdin:
//!
//! | stdout mode | `me sysw pack` | `mt encode` |
//! | --- | --- | --- |
//! | 0600 — **control** | exit 0, 733 bytes | exit 0, 796 bytes |
//! | **0620** | **exit 0, 733 bytes written** | **exit 1, REFUSED** |
//!
//! The control is load-bearing: both exit 0 at 0600 on the same input, so the
//! 0620 divergence is the mode and nothing else. `mt` refuses a
//! group-*writable* destination because someone else could alter the strings
//! before they are cut; `me` permits it. **`mt` is arguably right, and this
//! crate does not settle it** — hoisting policy would force a decision neither
//! tool has agreed to make, inside a shared dependency where the argument is
//! hardest to have, and the likely resolution is silent adoption of whichever
//! rule shipped here.
//!
//! ## What this crate therefore does NOT hold
//!
//! - **No exit-code integer, and no binary→code mapping.** The binaries agree
//!   on almost nothing: on the same invalid artifact `md` 1, `mk` 2, `ms` 1,
//!   `mnemonic` 2; on a clap usage error `md` 2, `mk` 64, `ms` 64, `mt` 2,
//!   `me` 2. See [`exit`].
//! - **No disqualifying permission mask.** Not `0o044`, not `0o077`.
//!   See [`fd`].
//! - **No record classification.** Deciding what a string IS belongs to each
//!   binary, and **nothing here ever names a record class**. The three
//!   predicates that answer it are *inherent methods* on `me`'s own enum, and a
//!   different crate cannot define an inherent `impl` for a foreign type —
//!   `error[E0116]`, reproduced in a two-crate scratch project. That is a
//!   language rule, not a preference, and the cheapest edit that would make
//!   such a build succeed is dragging a binary's container vocabulary in here.
//! - **No writes to stdio.** Functions return what should be said; the caller
//!   emits it. A library six binaries share cannot be tested without capturing
//!   process stdio and cannot be redirected by a caller — doubly wrong in a
//!   crate whose whole purpose is controlling what reaches stdout.
//!
//! ## What it does hold
//!
//! Mechanism, and the **vocabulary for describing what was measured**. Two
//! shipped defects are the argument for that second half: a refusal that called
//! a 65,536-byte zeros image *BEARER*, and one that called mode 0620
//! *readable by group or others* when `0620 & 0o044 == 0`. Both are messages
//! hard-coded to a rule's NAME rather than derived from the observation, and a
//! message computed from what was observed cannot make either mistake.

/// `--in` / `--out` / `-`, and where a container's bytes are going.
pub mod channel;
/// Refusal decisions, their wording, and the ordering rule. No integers.
pub mod exit;
/// The fd mechanism: the mode that was measured, and nothing about what it
/// means.
pub mod fd;
/// What was measured, as types.
pub mod observation;
/// Record-stream shaping only.
pub mod records;
/// Purge and remedy text.
pub mod remedy;
/// Creating the file a bearer artifact is written into, owner-only.
pub mod write;

// NOT re-exported at the root, and that is a choice rather than an omission.
// The root set below is already partial -- `fd`, `observation` and `remedy` are
// module-qualified only -- so adding `write_private` to it would not make the
// crate consistent, it would move the inconsistency. The second consumer reaches
// `fd` and `remedy` module-qualified already; `write` matches them.
pub use channel::{destination, Destination};
pub use exit::{write_block, WriteBlock};
pub use records::{no_records_guard, split_record_stream};
