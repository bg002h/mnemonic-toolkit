//! P6 — `mnemonic word-card` end-to-end integration tests.
//!
//! Realizes `design/IMPLEMENTATION_PLAN_word_card_encoding.md` §7 P6 KATs:
//!   1. e2e round-trip (mk1 — assert on recovered xpub, NOT literal string;
//!      md1 — string-deterministic, literal string asserted too),
//!   2. RAID r=1/r=2 via the CLI (drop r plates → reconstruct),
//!   3. corruption survived within budget; beyond budget → clean refuse,
//!   4. `--json` shape stable + error/exit-code mapping.
//!
//! Fixtures are PUBLIC material (xpub / descriptor) from fixed BIP-39 seeds —
//! no secrets. mk1/md1 strings produced by `mnemonic bundle --network mainnet
//! --template bip84` over the `abandon…about` (and two more) deterministic seeds.

use assert_cmd::Command;
use predicates::prelude::*;

// ── Fixtures (deterministic; PUBLIC xpub/descriptor material). ──────────────

/// The `abandon…about` seed's bip84 mk1 card (2 chunks) + its xpub identity.
const MK1: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const MK1_XPUB: &str = "xpub6CatWdiZiodmUeTDp8LT5or8nmbKNcuyvz7WyksVFkKB4RHwCD3XyuvPEbvqAQY3rAPshWcMLoP2fMFMKHPJ4ZeZXYVUhLv1VMrjPC7PW6V";

/// The same seed's bip84 md1 descriptor card (3 chunks). md1 is fully
/// string-deterministic, so the round-trip re-emits the SAME chunks.
const MD1: &str = "md1fgdxlpqpqpm6jzzqqvqpdqw0za5zs4gyy55aq4vsmnhy4s6wyaypu34c7raqu8np md1fgdxlpqf2zcgefcpupmel75q5435j7seugaj5jr7qyur6vt76es5cdeyrq7zdy0d md1fgdxlpq3xa2dk8vwpj7gx74hwqxqdp083jehp5tdrfa0n5zdfkqcdlrvnh5r62jn";

/// Three distinct mk1 cards (different seeds) for a RAID array.
const RAID_MK1: &str = "mk1qprsqhpqqsq3cqtsleeutks2qvzg3vs70mejhk622ws2kgdemj2cd8zwj2skzx2wq0qw70l4q99vdyh5x0z8v4yslsp8qp3yxg3dpe854wq4 mk1qprsqhpp0f30mtxzd65mvwcur9usdatwuqvq6z70r9nwrgk6xn6l8gy6nwa2n977sw6zh34rma0nh";
const RAID_MK2: &str = "mk1qp075gpqqsqhl2y9jkux3r03qvzg3vs7afghae0rhwz39k4sk9ejeku6jn6z5ng97tlv6kn0ru5kswgtdzmrgpk7l5pz735pjry2ursns6sk mk1qp075gpp450k4vmqvakpywv7pkdanhr5g2a605szqzpgggzzg55fzuu4z3ehm2ud3c88udv56yu6n";
const RAID_MK3: &str = "mk1qp8laepqqsqnl7usj55xg5qxqvzg3vs76psuyrqg8vt6w7wmgj73n889zv2eymp4zxqs9x6du0nfrz8e7qgymg03kcptxlndsx9jxaajlmtj mk1qp8laeppudnky9jhqsh5zpemaskg7ht32a5xh89h8wkwwt7ke08nrvkdfntj97pkmr5uvarcaep3t";
const RAID_MK1_XPUB: &str = MK1_XPUB;
const RAID_MK2_XPUB: &str = "xpub6DNfJehqF1LUs9kwaqDu12Ajpz9psYVtbGhTykQo1CYdkkqV2vAyR4DiWXSTTDujWHzVy1AtV6ENGKWgwbLWqa4wXMZR4ZmdpRjQBG5EgTV";
const RAID_MK3_XPUB: &str = "xpub6DBbzvudcQg2nS4tHSkrm7FGkXL6arWxEYoZ1g1GWbaNdWRobcJmxH8KQdjuzTZJtwDveibGd83eGzdGCR6NjSqpU8p2Xx4wWQK7iGBycAm";

// ── Helpers. ────────────────────────────────────────────────────────────────

fn mnemonic() -> Command {
    Command::cargo_bin("mnemonic").unwrap()
}

/// Encode one card → return the JSON `cards` array's first plate's word list,
/// space-joined.
fn encode_solo_words(from: &str, parity_words: usize) -> String {
    let out = mnemonic()
        .args([
            "word-card",
            "--from",
            from,
            "--parity-words",
            &parity_words.to_string(),
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let words: Vec<String> = v["cards"][0]["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect();
    words.join(" ")
}

// ── KAT 1 — e2e round-trip. ──────────────────────────────────────────────────

#[test]
fn mk1_e2e_round_trip_recovers_same_xpub_not_literal_string() {
    let words = encode_solo_words(MK1, 8);

    let out = mnemonic()
        .args(["word-card", "--decode", "--json", "-"])
        .write_stdin(words)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();

    // The load-bearing assertion (plan §7 P6): assert on the recovered XPUB, not
    // the literal mk1 string — re-encode draws a fresh CSPRNG chunk_set_id.
    assert_eq!(v["source_kind"], "mk1");
    assert_eq!(v["recovered"]["kind"], "mk1");
    assert_eq!(v["recovered"]["xpub"], MK1_XPUB);
    assert_eq!(v["truncated"], false);

    // The re-emitted mk1 string is NOT byte-identical to the original (fresh
    // chunk_set_id), but it MUST decode back to the same xpub.
    let mstring: Vec<String> = v["recovered"]["mstring"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    assert!(!mstring.is_empty());
    // Re-emitted chunk-set differs from the original chunk-set string.
    assert_ne!(mstring.join(" "), MK1);
}

#[test]
fn md1_e2e_round_trip_is_string_deterministic() {
    let words = encode_solo_words(MD1, 10);

    let out = mnemonic()
        .args(["word-card", "--decode", "--json", "-"])
        .write_stdin(words)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(v["source_kind"], "md1");
    assert_eq!(v["recovered"]["kind"], "md1");
    // md1 IS string-deterministic — the re-emitted chunks equal the original.
    let mstring: Vec<String> = v["recovered"]["mstring"]
        .as_array()
        .unwrap()
        .iter()
        .map(|s| s.as_str().unwrap().to_string())
        .collect();
    assert_eq!(mstring.join(" "), MD1);
}

#[test]
fn encode_text_form_labels_role_kind_and_word_count() {
    mnemonic()
        .args(["word-card", "--from", MK1, "--parity-words", "8"])
        .assert()
        .success()
        .stdout(predicate::str::contains("# solo plate [0] (mk1,"))
        .stdout(predicate::str::contains("words)"));
}

// ── KAT 2 — RAID via the CLI. ────────────────────────────────────────────────

/// Encode the 3-card array at `r`, return the plate JSON objects.
fn raid_encode(r: u8) -> Vec<serde_json::Value> {
    let out = mnemonic()
        .args([
            "word-card",
            "--from",
            RAID_MK1,
            "--from",
            RAID_MK2,
            "--from",
            RAID_MK3,
            "--raid",
            &r.to_string(),
            "--parity-words",
            "6",
            "--json",
        ])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["raid"], r);
    v["cards"].as_array().unwrap().clone()
}

fn plate_words(plate: &serde_json::Value) -> String {
    plate["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap())
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn raid1_reconstructs_one_dropped_data_plate() {
    let plates = raid_encode(1);
    // 3 data + 1 recovery-a.
    assert_eq!(plates.len(), 4);
    let roles: Vec<&str> = plates.iter().map(|p| p["role"].as_str().unwrap()).collect();
    assert_eq!(roles, vec!["data", "data", "data", "recovery-a"]);

    // Drop data plate 0; supply data[1], data[2], recovery-a → reconstruct.
    let surviving = [&plates[1], &plates[2], &plates[3]];
    let mut cmd = mnemonic();
    cmd.args(["word-card", "--decode", "--json"]);
    for p in surviving {
        cmd.args(["--decode-plate", &plate_words(p)]);
    }
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let out = output.stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(v["schema_version"], "2");
    assert_eq!(v["n"], 3);
    assert_eq!(v["reconstructed"], serde_json::json!([0]));
    // The recovered array's xpubs match the originals in order.
    let plates_json = v["plates"].as_array().unwrap();
    let xpubs: Vec<&str> = plates_json
        .iter()
        .map(|p| p["xpub"].as_str().unwrap())
        .collect();
    assert_eq!(xpubs, vec![RAID_MK1_XPUB, RAID_MK2_XPUB, RAID_MK3_XPUB]);

    // (c, F2) the reconstructed plate [0] carries the verify advisory; the
    // present plates [1], [2] do NOT (G4 — advisory only on MDS-solved plates).
    assert!(
        plates_json[0]["verify_advisory"]
            .as_str()
            .unwrap()
            .contains("independently verify"),
        "the *recovered plate must carry a verify advisory"
    );
    assert!(plates_json[1]["verify_advisory"].is_null());
    assert!(plates_json[2]["verify_advisory"].is_null());
    // The loud stderr advisory fires even in --json mode.
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("WARNING") && stderr.contains("reconstructed from RAID parity"),
        "a loud stderr advisory must fire on reconstruction; got: {stderr}"
    );
}

#[test]
fn full_present_raid_decode_has_no_recovery_advisory() {
    // (c, F2 / G4) Supply ALL plates (no drop) → nothing is MDS-solved → NO plate
    // carries a verify advisory and NO stderr warning fires.
    let plates = raid_encode(1);
    let surviving = [&plates[0], &plates[1], &plates[2], &plates[3]];
    let mut cmd = mnemonic();
    cmd.args(["word-card", "--decode", "--json"]);
    for p in surviving {
        cmd.args(["--decode-plate", &plate_words(p)]);
    }
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let v: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(v["reconstructed"], serde_json::json!([]));
    for p in v["plates"].as_array().unwrap() {
        assert!(
            p["verify_advisory"].is_null(),
            "no advisory on an all-present decode"
        );
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !stderr.contains("WARNING"),
        "no stderr advisory when nothing was reconstructed; got: {stderr}"
    );
}

#[test]
fn raid1_text_mode_prints_verify_advisory_on_recovered_plate() {
    // (c, F2 / whole-diff M-1) Text mode (no --json): a *recovered plate carries
    // the `! verify:` advisory on stdout + a loud stderr WARNING; an all-present
    // decode carries NEITHER (G4 — advisory only on MDS-solved plates). Closes
    // the text-path coverage gap the post-impl review flagged.
    let plates = raid_encode(1);
    let surviving = [&plates[1], &plates[2], &plates[3]]; // drop data[0]
    let mut cmd = mnemonic();
    cmd.args(["word-card", "--decode"]); // NO --json → text mode
    for p in surviving {
        cmd.args(["--decode-plate", &plate_words(p)]);
    }
    let assert = cmd.assert().success();
    let output = assert.get_output();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stdout.contains("! verify:") && stdout.contains("independently verify"),
        "text mode must print the `! verify:` advisory under the recovered plate; got: {stdout}"
    );
    assert!(
        stderr.contains("WARNING") && stderr.contains("reconstructed from RAID parity"),
        "text mode must fire the loud stderr advisory; got: {stderr}"
    );

    // All-present text decode → NO advisory anywhere.
    let all = [&plates[0], &plates[1], &plates[2], &plates[3]];
    let mut cmd2 = mnemonic();
    cmd2.args(["word-card", "--decode"]);
    for p in all {
        cmd2.args(["--decode-plate", &plate_words(p)]);
    }
    let out2 = cmd2.assert().success();
    let output2 = out2.get_output();
    let stdout2 = String::from_utf8_lossy(&output2.stdout);
    let stderr2 = String::from_utf8_lossy(&output2.stderr);
    assert!(
        !stdout2.contains("! verify:"),
        "no advisory on an all-present text decode; got: {stdout2}"
    );
    assert!(
        !stderr2.contains("WARNING"),
        "no stderr WARNING on an all-present text decode; got: {stderr2}"
    );
}

#[test]
fn raid2_reconstructs_two_dropped_data_plates() {
    let plates = raid_encode(2);
    // 3 data + recovery-a + recovery-b.
    assert_eq!(plates.len(), 5);
    let roles: Vec<&str> = plates.iter().map(|p| p["role"].as_str().unwrap()).collect();
    assert_eq!(
        roles,
        vec!["data", "data", "data", "recovery-a", "recovery-b"]
    );

    // Drop data plates 0 AND 1; supply data[2] + both recovery plates → reconstruct.
    let surviving = [&plates[2], &plates[3], &plates[4]];
    let mut cmd = mnemonic();
    cmd.args(["word-card", "--decode", "--json"]);
    for p in surviving {
        cmd.args(["--decode-plate", &plate_words(p)]);
    }
    let out = cmd.assert().success().get_output().stdout.clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();

    assert_eq!(v["n"], 3);
    assert_eq!(v["reconstructed"], serde_json::json!([0, 1]));
    let xpubs: Vec<&str> = v["plates"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["xpub"].as_str().unwrap())
        .collect();
    assert_eq!(xpubs, vec![RAID_MK1_XPUB, RAID_MK2_XPUB, RAID_MK3_XPUB]);
}

#[test]
fn raid1_refuses_when_two_plates_lost() {
    // r=1 can only recover 1 missing plate; dropping 2 data plates is
    // underdetermined → clean refuse (funds-safety net), non-zero exit.
    let plates = raid_encode(1);
    // Supply only data[2] + recovery-a (data[0], data[1] both missing).
    let surviving = [&plates[2], &plates[3]];
    let mut cmd = mnemonic();
    cmd.args(["word-card", "--decode"]);
    for p in surviving {
        cmd.args(["--decode-plate", &plate_words(p)]);
    }
    cmd.assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("word-card:"));
}

// ── KAT 3 — corruption within / beyond the RS budget. ───────────────────────

#[test]
fn within_budget_substitution_is_repaired() {
    let words = encode_solo_words(MK1, 8);
    // Substitute one data word (budget m=8 → corrects ⌊8/2⌋ = 4 substitutions).
    let mut tokens: Vec<String> = words.split_whitespace().map(String::from).collect();
    tokens[10] = if tokens[10] == "zoo" { "zebra" } else { "zoo" }.to_string();
    let corrupted = tokens.join(" ");

    let out = mnemonic()
        .args(["word-card", "--decode", "--json", "-"])
        .write_stdin(corrupted)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    // Still recovers the EXACT original xpub.
    assert_eq!(v["recovered"]["xpub"], MK1_XPUB);
}

#[test]
fn beyond_budget_corruption_refuses_with_exit_2_and_no_wrong_payload() {
    let words = encode_solo_words(MK1, 8);
    // Wreck 40 words — far beyond the m=8 budget. Must refuse, never emit a
    // wrong xpub (the funds-safety net).
    let mut tokens: Vec<String> = words.split_whitespace().map(String::from).collect();
    for t in tokens.iter_mut().take(45).skip(5) {
        *t = "zoo".to_string();
    }
    let wrecked = tokens.join(" ");

    mnemonic()
        .args(["word-card", "--decode", "-"])
        .write_stdin(wrecked)
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("word-card:"))
        // The original xpub must NEVER appear on a refusal.
        .stdout(predicate::str::contains(MK1_XPUB).not());
}

#[test]
fn unknown_word_refuses_with_exit_2() {
    mnemonic()
        .args(["word-card", "--decode", "-"])
        .write_stdin("notabip39word abandon ability able")
        .assert()
        .failure()
        .code(2)
        .stderr(predicate::str::contains("word-card:"));
}

// ── KAT 4 — JSON shape + error/exit-code mapping. ───────────────────────────

#[test]
fn encode_json_envelope_shape_is_stable() {
    let out = mnemonic()
        .args(["word-card", "--from", MK1, "--parity-words", "8", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["schema_version"], "2");
    assert_eq!(v["mode"], "encode");
    assert_eq!(v["raid"], 0);
    let card = &v["cards"][0];
    assert_eq!(card["role"], "solo");
    assert_eq!(card["index"], 0);
    assert_eq!(card["source_kind"], "mk1");
    let wc = card["word_count"].as_u64().unwrap();
    assert!(wc > 0);
    assert_eq!(card["words"].as_array().unwrap().len() as u64, wc);
    // Every emitted symbol is a valid BIP-39 word (lowercase, non-empty).
    for w in card["words"].as_array().unwrap() {
        let s = w.as_str().unwrap();
        assert!(!s.is_empty());
        assert_eq!(s, s.to_ascii_lowercase());
    }
}

#[test]
fn decode_json_envelope_shape_is_stable() {
    let words = encode_solo_words(MK1, 8);
    let out = mnemonic()
        .args(["word-card", "--decode", "--json", "-"])
        .write_stdin(words)
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    assert_eq!(v["schema_version"], "2");
    assert_eq!(v["mode"], "decode");
    assert_eq!(v["source_kind"], "mk1");
    assert!(v["erasures_filled"].is_u64());
    assert!(v["truncated"].is_boolean());
    assert!(v["recovered"]["mstring"].is_array());
    // mk1 recovered surfaces the policy-stub count.
    assert!(v["recovered"]["policy_id_stub_count"].is_u64());
    // (c, F2 / G4) a SOLO (all-present) decode is never MDS-solved, so no advisory.
    assert!(v["recovered"]["verify_advisory"].is_null());
}

#[test]
fn ms1_input_refused_as_unknown_hrp_exit_2() {
    // ms1 is a SECRET entropy card — intentionally NOT word-card-able.
    mnemonic()
        .args([
            "word-card",
            "--from",
            "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f",
        ])
        .assert()
        .failure()
        .code(2);
}

#[test]
fn encode_without_input_refuses_exit_1() {
    mnemonic()
        .args(["word-card"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("at least one --from"));
}

#[test]
fn integrity_bits_below_floor_refuses() {
    mnemonic()
        .args(["word-card", "--from", MK1, "--integrity-bits", "16"])
        .assert()
        .failure()
        .code(1)
        .stderr(predicate::str::contains("below the"));
}

#[test]
fn parity_pct_budget_round_trips() {
    // --parity-pct as an alternative to --parity-words must also round-trip.
    let out = mnemonic()
        .args(["word-card", "--from", MK1, "--parity-pct", "25", "--json"])
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v: serde_json::Value = serde_json::from_slice(&out).unwrap();
    let words: Vec<String> = v["cards"][0]["words"]
        .as_array()
        .unwrap()
        .iter()
        .map(|w| w.as_str().unwrap().to_string())
        .collect();

    let out2 = mnemonic()
        .args(["word-card", "--decode", "--json", "-"])
        .write_stdin(words.join(" "))
        .assert()
        .success()
        .get_output()
        .stdout
        .clone();
    let v2: serde_json::Value = serde_json::from_slice(&out2).unwrap();
    assert_eq!(v2["recovered"]["xpub"], MK1_XPUB);
}
