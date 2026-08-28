//! **Record-stream SHAPING only.**
//!
//! What a record IS — `Class`, the prefixes, the payload grammar — stays with
//! `me`, and so does the pre-parser argv guard, which asks `me`'s own
//! `classify()`. Siting that guard here while `me` depends on this crate is a
//! reproduced `error: cyclic package dependency`, and it would break the rule
//! that nothing here ever names a `Class` variant.
//!
//! What is left is the shaping: split a stream into records, and refuse an
//! empty one.

/// Split a newline-separated record stream. Blank lines are skipped, so a
/// record's index is its position among the NON-blank lines and not its line
/// number — the `--in` contract, applied to stdin too so the two channels
/// cannot disagree about what record 3 is.
pub fn split_record_stream(raw: &str) -> Vec<String> {
    raw.lines()
        .map(str::to_owned)
        .filter(|l| !l.trim().is_empty())
        .collect()
}

/// Where `me sysw pack` takes its records from, in precedence order:
/// `--in`, then argv, then **stdin** (G-P3.4 / SPEC §1.1).
///
/// stdin is LAST rather than first so no existing invocation changes meaning,
/// and it exists at all because the ruled pipeline is
/// R7, for EVERY channel that can arrive empty — **not just stdin**.
///
/// An empty input is refused rather than packed, because a container built
/// from nothing is a SILENT SUCCESS: 52 bytes of header, exit 0, and a file
/// the operator can flash. The device then offers nothing, which is the same
/// shape as P3's F1 reached from the host side.
///
/// **It was implemented on stdin alone, and `--in` bypassed it.** R7's own
/// reason is why that mattered: `fish` reports a pipeline's status as the LAST
/// command's, so a failed upstream arrives as nothing at all — and a failed
/// upstream also leaves a **0-byte file**. `mt encode --qr > rec.txt`
/// fails exactly that way on an operator's first try, because §8.2h refuses a
/// world-readable stdout and `>` creates 0644 under the usual umask. The
/// stdin half exited 2 and the `--in` half exited 0 for the same situation.
///
/// The message NAMES THE FILE when there is one: "pass them with --in" is
/// advice to do the thing they just did.
///
/// **It returns a `String`, not a `(String, i32)`.** P0's one signature change:
/// this function is the only member of the moving set that referenced one of
/// `me`'s exit-code constants, and a shared IO library may publish no exit
/// integer at all (plan §3). The number is the caller's — `me` maps this
/// refusal onto its own usage code at the two `read_records` call sites, and a
/// different binary would map it onto a different one.
pub fn no_records_guard(
    recs: Vec<String>,
    from: Option<&std::path::Path>,
) -> Result<Vec<String>, String> {
    if !recs.is_empty() {
        return Ok(recs);
    }
    let what = match from {
        Some(p) => format!("no records in {}", p.display()),
        None => "no records on stdin".to_string(),
    };
    Err(format!(
        "{what}: pass them on argv, with --in, or on stdin.\n      \
         An EMPTY input is what a FAILED upstream command leaves behind -- \
         `mt encode --qr > rec.txt` writes nothing when it refuses \
         -- so it is refused here rather than packed into a container that \
         holds nothing and still flashes."
    ))
}
