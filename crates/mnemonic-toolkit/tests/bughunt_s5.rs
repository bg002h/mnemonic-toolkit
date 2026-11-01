//! Bug-hunt triage RED rows, mnemonic-toolkit slice S5
//! (`repair`, `inspect`, `word-card`): hunt-report bugs #50 and #51, plus the
//! five unnumbered items X1-X5.
//!
//! Each test asserts the CORRECT behaviour through the built `mnemonic`
//! binary, so it fails on unmodified code exactly when the bug is real.
//! Where the product answer is open, the test asserts only that the reported
//! wrong behaviour is absent (the open question is named on the test).
//!
//! Fixtures are public cards (mk1/md1) and signet-free test strings: the ms1
//! values are test vectors (`ms10entr…` zero-entropy card; shares of the
//! all-`00 01 02 … 0f` secret), never wallet secrets.

use assert_cmd::Command;

// ── Fixtures. ───────────────────────────────────────────────────────────────

/// 128-bit zero-entropy `ms1` card (also used by `cli_repair.rs`).
const VALID_MS1: &str = "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
/// A second, different valid `ms1` card: 256-bit entropy `00 01 … 1f`
/// (`ms encode --hex 000102…1f`).
const VALID_MS1_256: &str =
    "ms10entrsqqqqzqsrqszsvpcgpy9qkrqdpc83qygjzv2p29shrqv35xcur50p7q548au6h32276";

/// `abandon…about` bip84 mk1 card, two chunks (also used by `cli_word_card.rs`).
const MK1_C0: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";
const MK1_C1: &str =
    "mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";

/// The same seed's bip84 md1 descriptor card, three chunks.
const MD1_C0: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
const MD1_C1: &str = "md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d";
const MD1_C2: &str = "md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

/// Share 1 of `ms split -k 2 -n 3 --hex 000102030405060708090a0b0c0d0e0f`
/// (ms 0.20.2); with share 2, `ms combine` recovers the hex. A valid codex32
/// string with threshold `2`.
const SHARE_1: &str = "ms12mc8pqf9tqz74mwtgw8g3y86qux3mgf6cq8skgye8ey5fl9";

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// One run's observable result.
struct Run {
    code: i32,
    stdout: String,
    stderr: String,
}

fn run(cmd: &mut Command) -> Run {
    let out = cmd.output().expect("spawn mnemonic");
    Run {
        code: out.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&out.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&out.stderr).into_owned(),
    }
}

/// `s` with the char at byte index `i` removed (all fixtures are ASCII).
fn drop_char(s: &str, i: usize) -> String {
    format!("{}{}", &s[..i], &s[i + 1..])
}

// ── #50 ─────────────────────────────────────────────────────────────────────

/// #50: `word-card --from` says "for a multi-chunk card, pass all chunks
/// joined OR repeat the flag; chunks are auto-grouped by HRP", and "`-` reads
/// one card per line". Repeating the flag (or two stdin lines) with the chunks
/// of ONE card must encode that card, the same as the joined form.
///
/// Open product question: with several multi-chunk cards, how are repeated
/// `--from` chunks grouped (by HRP only, or by chunk-set id)?
#[test]
fn bughunt_tk_50_word_card_from_groups_chunks_of_one_card() {
    let joined_mk1 = run(mnemonic().args([
        "word-card",
        "--json",
        "--from",
        &format!("{MK1_C0} {MK1_C1}"),
    ]));
    assert_eq!(
        joined_mk1.code, 0,
        "joined form must work: {}",
        joined_mk1.stderr
    );

    let mut failures: Vec<String> = Vec::new();

    let repeated =
        run(mnemonic().args(["word-card", "--json", "--from", MK1_C0, "--from", MK1_C1]));
    if repeated.code != 0 || repeated.stdout != joined_mk1.stdout {
        failures.push(format!(
            "mk1 repeated --from: exit {} (want 0 and the joined form's output); stderr: {}",
            repeated.code,
            repeated.stderr.trim()
        ));
    }

    let stdin_lines = run(mnemonic()
        .args(["word-card", "--json", "--from", "-"])
        .write_stdin(format!("{MK1_C0}\n{MK1_C1}\n")));
    if stdin_lines.code != 0 || stdin_lines.stdout != joined_mk1.stdout {
        failures.push(format!(
            "mk1 two stdin lines: exit {} (want 0 and the joined form's output); stderr: {}",
            stdin_lines.code,
            stdin_lines.stderr.trim()
        ));
    }

    let joined_md1 = run(mnemonic().args([
        "word-card",
        "--json",
        "--from",
        &format!("{MD1_C0} {MD1_C1} {MD1_C2}"),
    ]));
    assert_eq!(
        joined_md1.code, 0,
        "joined md1 form must work: {}",
        joined_md1.stderr
    );
    let repeated_md1 = run(mnemonic().args([
        "word-card",
        "--json",
        "--from",
        MD1_C0,
        "--from",
        MD1_C1,
        "--from",
        MD1_C2,
    ]));
    if repeated_md1.code != 0 || repeated_md1.stdout != joined_md1.stdout {
        failures.push(format!(
            "md1 repeated --from: exit {} (want 0 and the joined form's output); stderr: {}",
            repeated_md1.code,
            repeated_md1.stderr.trim()
        ));
    }

    assert!(
        failures.is_empty(),
        "word-card --from does not group chunks of one card, though its help says it does:\n{}",
        failures.join("\n")
    );
}

// ── #51 ─────────────────────────────────────────────────────────────────────

/// #51: `inspect <ms1A> <ms1B>` decodes only the first ms1. The second must
/// not be silently ignored: it is either decoded and reported, or the command
/// refuses. Asserted two ways: (a) a CORRUPT second ms1 must not give exit 0;
/// (b) a valid but different second ms1 must either show up in the output
/// (256-bit strength) or make the command fail.
///
/// Open product question: report both cards, or refuse a second ms1?
#[test]
fn bughunt_tk_51_inspect_second_ms1_not_ignored() {
    // (a) second ms1 is corrupt (one substituted character, checksum broken).
    let corrupt_b = "ms10entrsqqqqqqqqqqqpqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let a = run(mnemonic().args([
        "--allow-argv-secret",
        "inspect",
        "--no-auto-repair",
        VALID_MS1,
        corrupt_b,
    ]));
    assert_ne!(
        a.code, 0,
        "a corrupt second ms1 was silently ignored: exit 0, stdout:\n{}",
        a.stdout
    );

    // (b) second ms1 is valid and different (256-bit); A is 128-bit.
    let b = run(mnemonic().args([
        "--allow-argv-secret",
        "inspect",
        "--no-auto-repair",
        VALID_MS1,
        VALID_MS1_256,
    ]));
    assert!(
        b.code != 0 || b.stdout.contains("bit_strength: 256"),
        "a valid second ms1 was silently ignored: exit 0, stdout:\n{}",
        b.stdout
    );
}

// ── X1 ──────────────────────────────────────────────────────────────────────

/// X1: a mixed-case card is not a valid codex32/bech32 string. BIP-93: "a
/// codex32 string MUST be entirely uppercase or entirely lowercase"; BIP-173:
/// "Decoders MUST NOT accept strings where some characters are uppercase and
/// some are lowercase". `mnemonic repair` must refuse a mixed-case chunk of
/// any kind, not report it as valid (exit 0) or repair it.
///
/// (BIP-93 also says implementations "optionally interpret ... characters with
/// incorrect case, as erasures", so repair MAY correct one; the test asserts
/// only that a mixed-case card is not accepted as already valid, exit 0.)
#[test]
fn bughunt_tk_x1_repair_refuses_mixed_case_chunks() {
    let mixed_ms1 = "ms10ENTRsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f";
    let mixed_md1_c0 = "md1FGDxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np";
    let mixed_mk1_c0 = "mk1QPRSQhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4";

    let mut accepted: Vec<String> = Vec::new();

    let ms1 = run(mnemonic().args(["--allow-argv-secret", "repair", "--ms1", mixed_ms1]));
    if ms1.code == 0 {
        accepted.push(format!(
            "ms1 mixed case accepted, exit 0, stdout: {}",
            ms1.stdout.trim()
        ));
    }
    let md1 = run(mnemonic().args([
        "repair",
        "--md1",
        mixed_md1_c0,
        "--md1",
        MD1_C1,
        "--md1",
        MD1_C2,
    ]));
    if md1.code == 0 {
        accepted.push(format!(
            "md1 mixed case accepted, exit 0, stdout: {}",
            md1.stdout.trim()
        ));
    }
    let mk1 = run(mnemonic().args(["repair", "--mk1", mixed_mk1_c0, "--mk1", MK1_C1]));
    if mk1.code == 0 {
        accepted.push(format!(
            "mk1 mixed case accepted, exit 0, stdout: {}",
            mk1.stdout.trim()
        ));
    }

    assert!(
        accepted.is_empty(),
        "repair accepted mixed-case chunks as valid:\n{}",
        accepted.join("\n")
    );
}

// ── X2 ──────────────────────────────────────────────────────────────────────

/// X2: the GUI help says `mnemonic repair --ms1` takes `@env:VAR`, and the
/// argv guard exempts the sentinel, but `repair` never resolves it. Every
/// other `--ms1` surface (verify-bundle, bundle, ms_shares ...) resolves it
/// via `env_sentinel::resolve_env_var_sentinel`.
///
/// Open product question: resolve `@env:` in `repair`/`inspect`, or drop the
/// GUI claim (mnemonic-gui `src/schema/mnemonic.rs:2545`)?
#[test]
fn bughunt_tk_x2_repair_ms1_resolves_env_sentinel() {
    let r = run(mnemonic()
        .env("S5_XCARD", VALID_MS1)
        .args(["repair", "--ms1", "@env:S5_XCARD"]));
    assert!(
        r.code == 0 && r.stdout.contains(VALID_MS1),
        "`repair --ms1 @env:S5_XCARD` did not resolve the variable: exit {} (want 0 and the card on stdout); stderr: {}",
        r.code,
        r.stderr.trim()
    );
}

// ── X3 ──────────────────────────────────────────────────────────────────────

/// X3: a K-of-N `ms1` share with ONE substituted character is a codex32
/// string one substitution from valid; `repair` must correct it and print the
/// clean share (as ms1 corrections always exit 4, the VERIFY-ME candidate:
/// `cli_repair.rs` cell 9). A clean share is a valid codex32 string too and
/// must not be refused.
///
/// Open product question: does the correction belong in `mnemonic repair`
/// only (the native `ms repair` fails the same way, in ms-codec's
/// full-decode `decode_with_correction`)?
#[test]
fn bughunt_tk_x3_repair_corrects_one_substitution_in_kofn_share() {
    // Substitute char 21 ('8') with 'x'.
    let damaged = format!("{}x{}", &SHARE_1[..21], &SHARE_1[22..]);
    assert_ne!(damaged, SHARE_1);

    let mut failures: Vec<String> = Vec::new();

    let d = run(mnemonic()
        .args(["repair", "--ms1", "-"])
        .write_stdin(format!("{damaged}\n")));
    if d.code != 4 || !d.stdout.contains(SHARE_1) {
        failures.push(format!(
            "damaged share: exit {} (want 4 with the clean share on stdout); stdout: {:?}; stderr: {}",
            d.code,
            d.stdout.trim(),
            d.stderr.trim()
        ));
    }

    let c = run(mnemonic()
        .args(["repair", "--ms1", "-"])
        .write_stdin(format!("{SHARE_1}\n")));
    if c.code != 0 || !c.stdout.contains(SHARE_1) {
        failures.push(format!(
            "clean share: exit {} (want 0 with the share echoed); stderr: {}",
            c.code,
            c.stderr.trim()
        ));
    }

    assert!(
        failures.is_empty(),
        "repair does not handle a K-of-N ms1 share:\n{}",
        failures.join("\n")
    );
}

// ── X4 ──────────────────────────────────────────────────────────────────────

/// X4: `repair --max-indel` must recover a card with a dropped character
/// whether it is typed in lowercase or in all capitals (BIP-93: "it's also
/// valid to use all uppercase"; QR cards are uppercase). Lowercase exits 5
/// with the clean card; all-capitals exits 2 (measured), though the same tool
/// repairs a capitals card without `--max-indel`.
#[test]
fn bughunt_tk_x4_repair_max_indel_accepts_all_capitals() {
    let mut failures: Vec<String> = Vec::new();

    // ms1: drop char 20.
    let ms_dropped = drop_char(VALID_MS1, 20);
    let low = run(mnemonic().args([
        "--allow-argv-secret",
        "repair",
        "--max-indel",
        "1",
        "--ms1",
        &ms_dropped,
    ]));
    assert!(
        low.code == 5 && low.stdout.contains(VALID_MS1),
        "baseline broke: lowercase ms1 with a dropped char must exit 5: exit {} stderr {}",
        low.code,
        low.stderr.trim()
    );
    let up = run(mnemonic().args([
        "--allow-argv-secret",
        "repair",
        "--max-indel",
        "1",
        "--ms1",
        &ms_dropped.to_uppercase(),
    ]));
    if up.code != 5 || !up.stdout.to_lowercase().contains(VALID_MS1) {
        failures.push(format!(
            "ms1 all-capitals: exit {} (want 5 with the card); stderr: {}",
            up.code,
            up.stderr.trim()
        ));
    }

    // mk1: drop char 30 of chunk 1.
    let mk_dropped = drop_char(MK1_C1, 30);
    let up = run(mnemonic().args([
        "repair",
        "--max-indel",
        "1",
        "--mk1",
        &MK1_C0.to_uppercase(),
        "--mk1",
        &mk_dropped.to_uppercase(),
    ]));
    if up.code != 5 || !up.stdout.to_lowercase().contains(MK1_C1) {
        failures.push(format!(
            "mk1 all-capitals: exit {} (want 5 with the card); stderr: {}",
            up.code,
            up.stderr.trim()
        ));
    }

    // md1: drop char 30 of chunk 1.
    let md_dropped = drop_char(MD1_C1, 30);
    let up = run(mnemonic().args([
        "repair",
        "--max-indel",
        "1",
        "--md1",
        &MD1_C0.to_uppercase(),
        "--md1",
        &md_dropped.to_uppercase(),
        "--md1",
        &MD1_C2.to_uppercase(),
    ]));
    if up.code != 5 || !up.stdout.to_lowercase().contains(MD1_C1) {
        failures.push(format!(
            "md1 all-capitals: exit {} (want 5 with the card); stderr: {}",
            up.code,
            up.stderr.trim()
        ));
    }

    assert!(
        failures.is_empty(),
        "repair --max-indel rejects all-capitals cards it repairs in lowercase:\n{}",
        failures.join("\n")
    );
}

// ── X5 ──────────────────────────────────────────────────────────────────────

/// X5: `repair --ms1 -` must not silently ignore stdin lines after the first
/// (FOLLOWUPS `toolkit-repair-ms1-stdin-second-line-ignored`). Line 1 is a
/// clean card; line 2 is a DIFFERENT, damaged card. An implementation that
/// ignores line 2 exits 0 and never mentions it. Joining the lines, refusing
/// naming the second line, or repairing it as its own chunk all pass.
///
/// Open product question: join or refuse? (A single card wrapped over two
/// lines is today handled as two separate strings and exits 2 with
/// "chunk 0 has too many errors".)
#[test]
fn bughunt_tk_x5_repair_ms1_stdin_second_line_not_ignored() {
    // VALID_MS1_256 with one substituted character (position 17).
    let damaged_second = format!("{}p{}", &VALID_MS1_256[..17], &VALID_MS1_256[18..]);
    assert_ne!(damaged_second, VALID_MS1_256);
    let r = run(mnemonic()
        .args(["repair", "--ms1", "-"])
        .write_stdin(format!("{VALID_MS1}\n{damaged_second}\n")));
    assert!(
        r.code != 0 || r.stdout.contains(VALID_MS1_256),
        "second stdin line silently ignored: exit 0, stdout:\n{}",
        r.stdout
    );
}
