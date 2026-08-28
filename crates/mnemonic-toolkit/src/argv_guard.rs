//! **THE PRE-PARSER argv GUARD** — `SPEC_constellation_cli_uniformity` §6d,
//! P3 rows 14 and 15.
//!
//! Secret material handed to `mnemonic` on the command line is refused
//! **before `Cli::try_parse()` sees any of it**, and `--allow-argv-secret`
//! proceeds.
//!
//! # Why the ordering is normative and not an implementation note
//!
//! §6d rules it so, and the reason is reproducible in this binary today.
//! `mt`'s own source records the lesson from the other side: when its check
//! lived inside the `encode` subcommand, clap rejected the unexpected
//! positional first — **and clap's error echoed the entire bearer transaction
//! to stderr.** A guard downstream of the parser has already lost.
//!
//! **Measured here, on this binary, before this module existed** (the point is
//! that it was run, not assumed):
//!
//! ```text
//! $ mnemonic convert --from phrase=<12 words> --to xpub --template bip84 --bogus-flag
//! error: unexpected argument '--bogus-flag' found          <- names the FLAG
//! $ mnemonic convert --to xpub --template bip84 "<the 12 words>"
//! error: unexpected argument 'abandon abandon … about' found   <- echoes the PHRASE
//! ```
//!
//! So clap does **not** echo a declared flag's *value* on an unrelated error,
//! and it **does** echo a stray *positional* verbatim, at exit 64. That
//! measurement decides two things this module would otherwise have had to
//! guess at:
//!
//! - The override does **not** need to strip the admitted token out of the argv
//!   handed to clap. `mnemonic`'s material is flag-borne, and a flag's value is
//!   never echoed. (`--allow-argv-secret` is additionally a real, global clap
//!   flag, so clap accepts it rather than erroring on it.)
//! - The stray-positional echo is a **live, pre-existing leak** in a shape this
//!   row's flag-keyed table cannot reach. It is filed, not papered over — see
//!   the P3 log's follow-ups.
//!
//! # It does not invent a recogniser
//!
//! §7 of the spec rules that `mnemonic` **conforms rather than invents**: the
//! toolkit already ships an argv-secret subsystem, and P3 keys the refusal off
//! it instead of building a second one that could drift. So the `<node>=` half
//! of the table below tests its token against
//! [`mnemonic_toolkit::secret_taxonomy::SECRET_NODE_TYPES_ARGV`] — the same
//! `pub const &[&str]` that `NodeType::is_argv_secret_bearing` is held in
//! lockstep with by a parity test — and the `--slot @N.<subkey>=` half tests
//! against [`mnemonic_toolkit::secret_taxonomy::SECRET_SLOT_SUBKEYS`]. **Neither list is
//! copied here.** A tenth node type added to that const is refused by this
//! guard the moment it lands, and `argv_parity_covers_every_secret_node_type`
//! in `tests/cli_p3_argv_refusal.rs` is generated from the const so it cannot
//! silently fall behind.
//!
//! Matching a flag NAME in raw argv is a string comparison, not a parse, which
//! is what makes §6d's ordering achievable at all.
//!
//! # What this row does NOT reach, and why — stated rather than hidden
//!
//! Two of the eleven argv-material shapes the toolkit ships are **deliberately
//! outside** the table, because refusing them would advise a channel that does
//! not exist — the one thing §6h forbids outright:
//!
//! - **`--share <ms1-share>`** (`ms-shares combine`, `slip39 combine`). The
//!   private channel is `--share -`, and the tool accepts **at most one** `-`
//!   per invocation while a K-of-N recovery needs K ≥ 2 shares. Refusing here
//!   would leave the primary recovery path reachable only through the override,
//!   with a remedy line that cannot be followed.
//! - **A positional `ms1`** (`inspect`, `repair`). Catching material that no
//!   flag declares is §6d's *second*, value-shape layer; this row's work is the
//!   first, flag-keyed layer. The shape is also exactly what clap echoes, per
//!   the measurement above.
//!
//! A third exemption is narrower and lives in [`channel_for`] rather than here:
//! `--ms1` is refused on `inspect`, `repair` and the three `xpub-search` verbs,
//! and **not** on `verify-bundle` or `import-wallet`, because those two have no
//! private channel for it — measured, not assumed. Same rule, same reason.
//!
//! All are filed with owning phases rather than left to be rediscovered.

use mnemonic_toolkit::secret_taxonomy::{SECRET_NODE_TYPES_ARGV, SECRET_SLOT_SUBKEYS};

/// The override, spelled as `me sysw pack` and `mt` spell it. §6d declines to
/// rename it: the predicate is deliberately the union of secret and bearer, and
/// at argv they are the same problem.
pub const OVERRIDE_FLAG: &str = "--allow-argv-secret";

/// Exit code for the refusal.
///
/// **2, not 3 and not 64.** `mnemonic`'s own refusal family is 2 —
/// `ExportWalletSecretInput`, `ModeViolation`, `ConvertRefusal` all exit 2 —
/// while 3 is already taken here by `ToolkitError::FutureFormat` and 64 is
/// clap's usage code, which this is not (the refusal happens *instead of*
/// parsing). §6f's closure condition is that no existing code moves; this adds
/// a path, it renumbers nothing.
pub const EXIT_REFUSED: u8 = 2;

/// The flags whose VALUE is secret material, with the private channel that
/// exists for each.
///
/// `Channel::Stdin` names a companion flag; `Channel::Sentinel` names the `-`
/// value form. Which one is right per flag was measured against the binary's
/// own `gui-schema`, not assumed: `--passphrase-stdin` exists on every
/// subcommand that declares `--passphrase`, and `--ms1-stdin` exists **only**
/// on the three `xpub-search` verbs — where `--ms1 -` in turn does *not* work
/// (it is taken as a literal one-character `ms1` string, measured at exit 1).
/// That asymmetry is why [`channel_for`] takes the argv.
#[derive(Clone, Copy)]
enum Shape {
    /// `--from <node>=<material>`; refuse iff `<node>` is argv-secret-bearing.
    NodeEquals,
    /// `--slot @N.<subkey>=<material>`; refuse iff `<subkey>` is secret-bearing.
    SlotEquals,
    /// The whole value is the material. `sentinel` says whether `-` is a real
    /// stdin channel for this flag (and therefore exempt) or just a
    /// one-character value.
    Whole { class: &'static str, sentinel: bool },
}

struct Entry {
    flag: &'static str,
    shape: Shape,
}

/// The table. Nine of the eleven argv-material shapes the toolkit ships; the
/// two omissions are named in this module's header with their reasons.
const TABLE: &[Entry] = &[
    Entry {
        flag: "--from",
        shape: Shape::NodeEquals,
    },
    Entry {
        flag: "--slot",
        shape: Shape::SlotEquals,
    },
    Entry {
        flag: "--passphrase",
        shape: Shape::Whole {
            class: "a BIP-39 passphrase",
            sentinel: false,
        },
    },
    Entry {
        flag: "--bip38-passphrase",
        shape: Shape::Whole {
            class: "a BIP-38 passphrase",
            sentinel: false,
        },
    },
    Entry {
        flag: "--decrypt-password",
        shape: Shape::Whole {
            class: "a wallet decryption password",
            sentinel: false,
        },
    },
    Entry {
        flag: "--phrase",
        shape: Shape::Whole {
            class: "a BIP-39 phrase",
            sentinel: false,
        },
    },
    Entry {
        flag: "--secret",
        shape: Shape::Whole {
            class: "private key material",
            sentinel: false,
        },
    },
    Entry {
        flag: "--digits",
        shape: Shape::Whole {
            class: "a SeedQR digit string",
            sentinel: true,
        },
    },
    Entry {
        flag: "--ms1",
        shape: Shape::Whole {
            class: "a codex32 (ms1) card -- seed-equivalent",
            sentinel: true,
        },
    },
];

/// The human-readable class of a `<node>=` / `@N.<subkey>=` token.
fn class_of_token(token: &str) -> &'static str {
    match token {
        "phrase" => "a BIP-39 phrase",
        "entropy" => "raw seed entropy",
        "xprv" => "a BIP-32 extended PRIVATE key",
        "wif" => "a WIF private key",
        "ms1" => "a codex32 (ms1) card -- seed-equivalent",
        "bip38" => "a BIP-38 encrypted private key",
        "electrum-phrase" => "an Electrum seed phrase",
        "seedqr" => "a SeedQR digit string",
        "minikey" => "a Casascius mini private key",
        _ => "secret key material",
    }
}

/// One refused occurrence. The value is **never** carried in it — only its
/// class, its length, and where it sat.
pub struct Finding {
    /// The flag as the operator wrote it, e.g. `--from phrase=`.
    pub flag: String,
    pub class: &'static str,
    /// Characters, not bytes: this is for a human comparing against what they
    /// typed.
    pub len: usize,
    /// The private channel that exists for this flag on this invocation.
    pub channel: String,
    /// Index in argv. argv[0] is the binary itself.
    pub position: usize,
}

/// The subcommand word — the first token at `argv[1..]` that does not begin
/// with `-`.
///
/// **THE LITERAL READ IS SAFE HERE, AND THE REASON IS SPECIFIC RATHER THAN
/// GENERAL.** `me`'s equivalent had to reject exactly this shape, because a
/// flag VALUE that followed a `-`-prefixed token could pose as the surface and
/// grant an override. `mnemonic` has **two** root-level global flags —
/// `--no-auto-repair` and `--allow-argv-secret` — and **both are booleans**, so
/// nothing that precedes the subcommand consumes a value, and no value can
/// stand where this function looks. `subcommand_word_survives_a_global_flag`
/// pins that; if a root-level global ever takes a value, this becomes
/// spoofable and must move to an allowlist.
fn subcommand_word(argv: &[String]) -> Option<&str> {
    argv.iter()
        .skip(1)
        .map(String::as_str)
        .find(|t| !t.starts_with('-'))
}

/// The private channel for a (subcommand, flag) pair — or `None` where the tool
/// has none.
///
/// **`None` MEANS "DO NOT REFUSE HERE", AND THAT IS §6h RATHER THAN A GAP IN
/// NERVE.** *"The remedy must not forward-reference a channel that does not
/// exist"* is the rule this file is most exposed to, and a refusal whose remedy
/// cannot be followed is worse than the advisory it replaced: it stops the
/// operator without telling them what to do instead. Every `None` below was
/// **measured on the binary**, and every one is filed:
///
/// - **`--ms1` on `verify-bundle`**: `--ms1 -` is not accepted, and the verb
///   needs `--slot @N.phrase=` at the same time — only one input can be `-`, so
///   even a working sentinel would name an impossible combination.
/// - **`--ms1` on `import-wallet`**: same, no sentinel.
/// - **`--slot` on `export-wallet`**: the verb refuses secret slots itself
///   (*"export-wallet is watch-only by definition"*), so there is nothing to
///   route and no channel to name.
fn channel_for(flag: &str, sub: Option<&str>, subkey_form: Option<&str>) -> Option<String> {
    match (flag, sub) {
        // Measured: `--ms1 -` reads stdin on `inspect` and `repair`;
        // `--ms1-stdin` exists ONLY on the three `xpub-search` verbs, where
        // `--ms1 -` is instead taken as a literal one-character ms1 string;
        // neither exists on `verify-bundle` or `import-wallet`.
        ("--ms1", Some("inspect" | "repair")) => Some("--ms1 -".into()),
        ("--ms1", Some("xpub-search")) => Some("--ms1-stdin".into()),
        ("--ms1", _) => None,
        // Measured: `--slot @N.<subkey>=-` reads stdin on `bundle` and
        // `verify-bundle`; on `import-wallet` the `-` is taken literally and
        // the channel is the `@env:` sentinel, which its own shipped advisory
        // already names and which resolves (measured).
        ("--slot", Some("import-wallet")) => Some(format!(
            "--slot {}@env:VAR",
            subkey_form.unwrap_or("@N.phrase=")
        )),
        ("--slot", Some("export-wallet")) => None,
        ("--slot", _) => Some(format!("--slot {}-", subkey_form.unwrap_or("@N.phrase="))),
        ("--from", _) => Some(format!("--from {}-", subkey_form.unwrap_or("phrase="))),
        ("--digits", _) => Some("--digits -".into()),
        (other, _) => Some(format!("{other}-stdin")),
    }
}

/// Is this value one of the tool's EXISTING private channels? Those are exempt
/// — refusing them would remove the very remedy the refusal points at.
fn is_private_channel_value(value: &str, sentinel_ok: bool) -> bool {
    value.is_empty() || value.starts_with("@env:") || (sentinel_ok && value == "-")
}

/// Split `--flag=value` into its halves, or `None` for a bare `--flag`.
fn inline_value<'a>(token: &'a str, flag: &str) -> Option<&'a str> {
    token
        .strip_prefix(flag)
        .and_then(|rest| rest.strip_prefix('='))
}

/// The verdict for one argv.
pub enum Verdict {
    /// Nothing secret on argv.
    Clean,
    /// Secret material on argv, and `--allow-argv-secret` was supplied.
    Admitted,
    /// Secret material on argv, with no override.
    Refuse(Vec<Finding>),
}

/// Scan raw argv. **Pure** — reads no file, resolves no environment variable,
/// and never sees a parser.
pub fn inspect(argv: &[String]) -> Verdict {
    let mut findings = Vec::new();
    // Tokens consumed as a flag's VALUE cannot also be read as the override.
    // Without this, `mnemonic convert --passphrase --allow-argv-secret` would
    // grant the override with the flag's own value — the value-spoofs-the-
    // surface defect `me` reproduced from the other direction.
    let mut consumed_as_value = vec![false; argv.len()];

    let mut i = 1;
    while i < argv.len() {
        let token = &argv[i];
        let mut matched = None;
        for e in TABLE {
            if let Some(v) = inline_value(token, e.flag) {
                matched = Some((e, v.to_string()));
                break;
            }
            if token == e.flag {
                if let Some(v) = argv.get(i + 1) {
                    consumed_as_value[i + 1] = true;
                    matched = Some((e, v.clone()));
                }
                break;
            }
        }
        if let Some((entry, value)) = matched {
            if let Some(f) = classify(entry, &value, argv, i) {
                findings.push(f);
            }
        }
        i += 1;
    }

    if findings.is_empty() {
        return Verdict::Clean;
    }
    let overridden = argv
        .iter()
        .enumerate()
        .any(|(i, t)| t == OVERRIDE_FLAG && !consumed_as_value[i]);
    if overridden {
        Verdict::Admitted
    } else {
        Verdict::Refuse(findings)
    }
}

fn classify(entry: &Entry, value: &str, argv: &[String], position: usize) -> Option<Finding> {
    let sub = subcommand_word(argv);
    match entry.shape {
        Shape::NodeEquals => {
            let (token, material) = value.split_once('=')?;
            if !SECRET_NODE_TYPES_ARGV.contains(&token) {
                return None;
            }
            if is_private_channel_value(material, true) {
                return None;
            }
            Some(Finding {
                flag: format!("--from {token}="),
                class: class_of_token(token),
                len: material.chars().count(),
                channel: channel_for("--from", sub, Some(&format!("{token}=")))?,
                position,
            })
        }
        Shape::SlotEquals => {
            // `@<index>.<subkey>=<material>`
            let (head, material) = value.split_once('=')?;
            let (index, subkey) = head.strip_prefix('@')?.split_once('.')?;
            if !SECRET_SLOT_SUBKEYS.contains(&subkey) {
                return None;
            }
            if is_private_channel_value(material, true) {
                return None;
            }
            Some(Finding {
                flag: format!("--slot @{index}.{subkey}="),
                class: class_of_token(subkey),
                len: material.chars().count(),
                channel: channel_for("--slot", sub, Some(&format!("@{index}.{subkey}=")))?,
                position,
            })
        }
        Shape::Whole { class, sentinel } => {
            if is_private_channel_value(value, sentinel) {
                return None;
            }
            Some(Finding {
                flag: entry.flag.to_string(),
                class,
                len: value.chars().count(),
                channel: channel_for(entry.flag, sub, None)?,
                position,
            })
        }
    }
}

/// The refusal, as printed.
///
/// **NAMES THE CLASS AND THE LENGTH, NEVER THE BODY.** Printing the value back
/// would put the material in a *second* public place — the defect this refusal
/// exists to name.
///
/// The purge paragraph comes from the shared crate
/// (`mnemonic_io_lib::remedy::history_purge_block`) rather than being written
/// again here: `mt`'s wording is disqualified on evidence (it advises zsh
/// operators `history -d`, which does not delete, and tells fish operators to
/// match on the bearer material — typing the secret into history a second
/// time), and the crate's text was measured against real shells.
///
/// It matches on the bare word `mnemonic`, and the pattern is fixed rather than
/// derived from argv. `me`'s equivalent has to build a subcommand allowlist
/// because a bare `me` would make `sed '/\bme\b/d'` eat `make`, `time` and
/// `/home/me`; `mnemonic` is a distinctive enough word to need no such widening,
/// and a fixed pattern cannot admit material into a `sed` expression at all.
pub fn refusal_message(findings: &[Finding]) -> String {
    let mut s = String::new();
    for f in findings {
        s.push_str(&format!(
            "argument {} on ARGV ({}) is {} -- {} characters. Nothing about it is \
             printed here.\n      ",
            f.position, f.flag, f.class, f.len,
        ));
    }
    s.push_str(
        "Refused BEFORE the command line was parsed; nothing was read and \
         nothing was written.\n      \
         argv is public: /proc, `ps` and your shell history all keep a copy, so \
         the argument parser must not be allowed to echo it back in an error \
         message.\n      \
         Use a private channel instead -- these exist today:\n",
    );
    let mut channels: Vec<&str> = findings.iter().map(|f| f.channel.as_str()).collect();
    channels.sort_unstable();
    channels.dedup();
    for c in channels {
        s.push_str(&format!("      \x20   {c}\n"));
    }
    s.push('\n');
    s.push_str("      ");
    s.push_str(&mnemonic_io_lib::remedy::history_purge_block("mnemonic"));
    s.push_str(&format!(
        "\n      If argv is safe where you are -- a single-user air-gapped box, \
         an amnesic Tails session -- `{OVERRIDE_FLAG}` proceeds. It is greppable, \
         so a reviewer can find every place a script opted in."
    ));
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn argv(v: &[&str]) -> Vec<String> {
        std::iter::once("mnemonic")
            .chain(v.iter().copied())
            .map(String::from)
            .collect()
    }

    fn findings(v: &[&str]) -> Vec<Finding> {
        match inspect(&argv(v)) {
            Verdict::Refuse(f) => f,
            Verdict::Clean => panic!("expected a refusal, got Clean"),
            Verdict::Admitted => panic!("expected a refusal, got Admitted"),
        }
    }

    #[test]
    fn node_equals_is_refused_for_every_argv_secret_node_type() {
        // Generated from the const, so a tenth token cannot be added without
        // this test seeing it.
        for token in SECRET_NODE_TYPES_ARGV {
            let f = findings(&[
                "convert",
                "--from",
                &format!("{token}=material"),
                "--to",
                "xpub",
            ]);
            assert_eq!(f.len(), 1, "{token}: expected exactly one finding");
            assert_eq!(f[0].flag, format!("--from {token}="));
            assert_eq!(f[0].len, "material".len());
            assert_eq!(f[0].channel, format!("--from {token}=-"));
        }
    }

    #[test]
    fn a_watch_only_node_type_is_not_refused() {
        // `xpub` is not in SECRET_NODE_TYPES_ARGV, and watch-only material stays
        // on argv by ruling (spec section 4).
        assert!(matches!(
            inspect(&argv(&[
                "convert",
                "--from",
                "xpub=xpub6...",
                "--to",
                "address"
            ])),
            Verdict::Clean
        ));
    }

    #[test]
    fn the_existing_private_channels_are_exempt() {
        for value in ["phrase=-", "phrase=@env:SEED"] {
            assert!(
                matches!(
                    inspect(&argv(&["convert", "--from", value, "--to", "xpub"])),
                    Verdict::Clean
                ),
                "{value} is an existing private channel; refusing it removes the remedy"
            );
        }
        assert!(matches!(
            inspect(&argv(&["repair", "--ms1", "-"])),
            Verdict::Clean
        ));
        assert!(matches!(
            inspect(&argv(&[
                "convert",
                "--passphrase",
                "@env:PW",
                "--from",
                "xpub=x"
            ])),
            Verdict::Clean
        ));
    }

    /// `-` is a stdin sentinel on `--ms1` and `--digits`, and is a one-character
    /// VALUE on `--passphrase`. Exempting it there would be exempting a real
    /// leak on the strength of a channel that flag does not have.
    #[test]
    fn a_dash_passphrase_is_a_value_not_a_channel() {
        let f = findings(&["convert", "--passphrase", "-", "--from", "xpub=x"]);
        assert_eq!(f[0].flag, "--passphrase");
        assert_eq!(f[0].channel, "--passphrase-stdin");
        assert_eq!(f[0].len, 1);
    }

    #[test]
    fn slot_subkeys_are_keyed_off_the_taxonomy() {
        for subkey in SECRET_SLOT_SUBKEYS {
            let f = findings(&["bundle", "--slot", &format!("@2.{subkey}=material")]);
            assert_eq!(f[0].flag, format!("--slot @2.{subkey}="));
            assert_eq!(f[0].channel, format!("--slot @2.{subkey}=-"));
        }
        // `xpub` is a slot subkey and is NOT secret-bearing.
        assert!(matches!(
            inspect(&argv(&["bundle", "--slot", "@0.xpub=xpub6..."])),
            Verdict::Clean
        ));
    }

    #[test]
    fn the_inline_equals_form_is_seen_too() {
        let f = findings(&["convert", "--from=phrase=material", "--to", "xpub"]);
        assert_eq!(f[0].flag, "--from phrase=");
        let f = findings(&["convert", "--passphrase=hunter2", "--from", "xpub=x"]);
        assert_eq!(f[0].flag, "--passphrase");
        assert_eq!(f[0].len, 7);
    }

    #[test]
    fn every_occurrence_is_reported_not_just_the_first() {
        let f = findings(&[
            "bundle",
            "--slot",
            "@0.phrase=one",
            "--slot",
            "@1.phrase=two",
            "--passphrase",
            "pw",
        ]);
        assert_eq!(f.len(), 3, "one finding per leak site");
    }

    #[test]
    fn the_override_admits() {
        assert!(matches!(
            inspect(&argv(&[
                "convert",
                OVERRIDE_FLAG,
                "--from",
                "phrase=material",
                "--to",
                "xpub"
            ])),
            Verdict::Admitted
        ));
    }

    /// A flag's own VALUE must not be able to spoof the override.
    #[test]
    fn a_value_equal_to_the_override_does_not_grant_it() {
        assert!(matches!(
            inspect(&argv(&[
                "convert",
                "--passphrase",
                OVERRIDE_FLAG,
                "--from",
                "phrase=material",
                "--to",
                "xpub"
            ])),
            Verdict::Refuse(_)
        ));
    }

    #[test]
    fn the_message_never_carries_the_material() {
        let secret = "correct horse battery staple";
        let f = findings(&["convert", "--passphrase", secret, "--from", "xpub=x"]);
        let msg = refusal_message(&f);
        assert!(!msg.contains(secret), "the refusal echoed the value: {msg}");
        assert!(!msg.contains("staple"));
        assert!(msg.contains("28 characters"));
        assert!(msg.contains("--passphrase-stdin"));
    }

    /// Section 6h: the remedy must not forward-reference a channel that does not
    /// exist. `--in` exists on NO `mnemonic` verb -- checked against the whole
    /// binary's flag surface by `tests/cli_p3_argv_refusal.rs` -- and must never
    /// be advised.
    #[test]
    fn the_message_never_advises_a_nonexistent_channel() {
        let f = findings(&["convert", "--from", "phrase=x", "--to", "xpub"]);
        let msg = refusal_message(&f);
        assert!(!msg.contains("--in "), "`--in` exists on no mnemonic verb");
        assert!(!msg.contains("--in\n"));
    }

    /// The purge text is the crate's, and the zsh trap is NAMED without being
    /// OFFERED. A naive `!contains("history -d")` would fail on the warning
    /// itself, so the assertion is that no RECIPE offers it.
    #[test]
    fn the_purge_block_warns_about_history_d_without_offering_it() {
        let f = findings(&["convert", "--from", "phrase=x", "--to", "xpub"]);
        let msg = refusal_message(&f);
        assert!(msg.contains("history -d"));
        for (shell, recipe) in mnemonic_io_lib::remedy::history_purge_recipes("mnemonic") {
            assert!(
                !recipe.contains("history -d"),
                "{shell} recipe offers the trap"
            );
        }
    }

    #[test]
    fn ms1_channel_follows_the_subcommand_that_has_one() {
        let f = findings(&["repair", "--ms1", "ms1abc"]);
        assert_eq!(f[0].channel, "--ms1 -");
        let f = findings(&["inspect", "--ms1", "ms1abc"]);
        assert_eq!(f[0].channel, "--ms1 -");
        let f = findings(&["xpub-search", "path-of-xpub", "--ms1", "ms1abc"]);
        assert_eq!(f[0].channel, "--ms1-stdin");
    }

    /// §6h in its hardest form: where a (subcommand, flag) pair has NO private
    /// channel, the guard does not refuse. Refusing would print a remedy the
    /// operator cannot follow, which is worse than the advisory it replaced —
    /// and the post-clap advisory still fires on these paths.
    #[test]
    fn a_pair_with_no_private_channel_is_not_refused() {
        // `verify-bundle --ms1 -` is not accepted (measured), and the verb needs
        // `--slot @N.phrase=` at the same time, so only one input could be `-`.
        assert!(matches!(
            inspect(&argv(&["verify-bundle", "--ms1", "ms1abc"])),
            Verdict::Clean
        ));
        // `import-wallet --ms1 -` likewise has no sentinel.
        assert!(matches!(
            inspect(&argv(&["import-wallet", "--ms1", "ms1abc"])),
            Verdict::Clean
        ));
        // ... but the SAME flag on a verb that HAS a channel is refused.
        assert!(matches!(
            inspect(&argv(&["inspect", "--ms1", "ms1abc"])),
            Verdict::Refuse(_)
        ));
    }

    /// `--slot`'s channel is the `-` sentinel on `bundle`/`verify-bundle` and
    /// the `@env:` sentinel on `import-wallet`, where `-` is taken literally.
    /// `export-wallet` refuses secret slots itself, so there is nothing to name.
    #[test]
    fn slot_channel_follows_the_subcommand_too() {
        let f = findings(&["bundle", "--slot", "@0.phrase=material"]);
        assert_eq!(f[0].channel, "--slot @0.phrase=-");
        let f = findings(&["verify-bundle", "--slot", "@0.phrase=material"]);
        assert_eq!(f[0].channel, "--slot @0.phrase=-");
        let f = findings(&["import-wallet", "--slot", "@0.phrase=material"]);
        assert_eq!(f[0].channel, "--slot @0.phrase=@env:VAR");
        assert!(matches!(
            inspect(&argv(&["export-wallet", "--slot", "@0.phrase=material"])),
            Verdict::Clean
        ));
    }

    /// The subcommand read must survive a leading global flag, and both of
    /// `mnemonic`'s root globals are booleans — which is the whole safety
    /// argument for reading the first non-`-` token literally.
    #[test]
    fn subcommand_word_survives_a_global_flag() {
        let a = argv(&["--no-auto-repair", "inspect", "--ms1", "ms1abc"]);
        assert_eq!(subcommand_word(&a), Some("inspect"));
        let f = findings(&["--no-auto-repair", "inspect", "--ms1", "ms1abc"]);
        assert_eq!(f[0].channel, "--ms1 -");
    }
}
