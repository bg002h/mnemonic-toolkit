//! v0.53.4 — the friendly mapper's `ms_codec::codex32::Error::InvalidChecksum` arm must
//! NOT echo the embedded full input on stderr (leak-hardening; FOLLOWUP
//! `friendly-ms1-invalidchecksum-echoes-full-input`).
//!
//! An uncorrectable lowercase ms1 (known HRP, bad checksum) renders through the
//! friendly mapper's `Codex32(...)` catch-all, whose `{:?}` Debug-print of
//! `InvalidChecksum { string }` previously dumped the FULL near-secret. The
//! decode path MUST be deterministically repair-DISABLED (`--no-auto-repair`):
//! the repo suite sets `MNEMONIC_FORCE_TTY=1`, under which most corruptions
//! get auto-repaired (exit 5) and never reach the friendly render.

use assert_cmd::Command;

// A valid TREZOR-12-zero ms1, then a single data char corrupted (`q`→`p` at
// a payload position) so the BCH checksum fails uncorrectably enough to reach
// the friendly render under `--no-auto-repair`.
const CORRUPTED_MS1: &str = "ms10entrspqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";

#[test]
fn invalid_checksum_does_not_echo_input_on_stderr() {
    let out = Command::cargo_bin("mnemonic")
        .unwrap()
        .args([
            "--no-auto-repair",
            "convert",
            "--from",
            &format!("ms1={CORRUPTED_MS1}"),
            "--to",
            "phrase",
        ])
        .assert()
        .failure()
        .code(1);
    let stderr = String::from_utf8(out.get_output().stderr.clone()).unwrap();
    assert!(
        !stderr.contains(CORRUPTED_MS1),
        "stderr must NOT echo the full ms1 input (secret-adjacent); got {stderr:?}"
    );
    // The data part past `ms1` chars 9+ is payload; assert a payload slice is
    // absent (catches a partial-echo regression the whole-string check misses).
    assert!(
        !stderr.contains(&CORRUPTED_MS1[9..30]),
        "stderr must NOT echo any payload slice of the ms1 input; got {stderr:?}"
    );
    assert!(
        stderr.contains("invalid") && stderr.contains("checksum") && stderr.contains("withheld"),
        "stderr must name the checksum failure with the input withheld; got {stderr:?}"
    );
}
