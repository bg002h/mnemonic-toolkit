//! `mnemonic derive-child` subcommand — BIP-85 deterministic derivation.
//!
//! Realizes `design/SPEC_derive_child_v0_7.md` §2 (grammar), §3 (primitive),
//! §4 (in-scope apps), §5 (out-of-scope refusal), §7 (refusal taxonomy).

use crate::bip85;
use crate::cmd::convert::{parse_from_input, FromInput, NodeType};
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::language::CliLanguage;
use crate::network::CliNetwork;
use bitcoin::bip32::Xpriv;
use clap::Args;
use std::io::Write;
use std::str::FromStr;

#[derive(Args, Debug)]
pub struct DeriveChildArgs {
    /// Master xpriv source. Only `--from xprv=<value>` is accepted.
    #[arg(long = "from", value_parser = parse_from_input, required = true)]
    pub from: FromInput,

    /// BIP-85 application. The 6 in-scope tokens map to apps `39'`, `2'`,
    /// `32'`, `128169'`, `707764'`, `707785'`. The 3 out-of-scope tokens
    /// (`rsa`, `rsa-gpg`, `dice`) parse here and surface the SPEC §7
    /// byte-exact refusal at runtime (per SPEC §5 + plan deviation note).
    #[arg(long = "application", required = true)]
    pub application: String,

    /// Per-app `--length` validator (range varies; see SPEC §4).
    /// Required at clap level for grammar-uniformity (SPEC §2). For
    /// `hd-seed` / `xprv` the value is irrelevant unless non-zero, in which
    /// case the SPEC §7 not-applicable refusal fires; pass `--length 0` as
    /// the sentinel to satisfy clap without triggering the refusal.
    #[arg(long = "length", required = true)]
    pub length: u32,

    /// Hardened child index (`0..2^31`).
    #[arg(long = "index", required = true)]
    pub index: u32,

    /// Network for emitted xprv / WIF (defaults to source xprv's network).
    /// Reserved (BIP-85 spec test vectors pin mainnet); in v0.7 the WIF
    /// and xprv emitters always render mainnet to match the spec vectors,
    /// and this flag is held for v0.8 testnet-vector support.
    #[arg(long)]
    #[allow(dead_code)]
    pub network: Option<CliNetwork>,

    /// BIP-39 language. v0.7 supports English only; reserved for v0.8.
    #[arg(long)]
    #[allow(dead_code)]
    pub language: Option<CliLanguage>,
}

pub fn run<W: Write, E: Write>(
    args: &DeriveChildArgs,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<(), ToolkitError> {
    // SPEC §2 — `--from xprv=<value>` only.
    if args.from.node != NodeType::Xprv {
        return Err(ToolkitError::BadInput(format!(
            "derive-child: --from must be xprv=<master-xprv>; got {}",
            args.from.node.as_str(),
        )));
    }
    let master = Xpriv::from_str(&args.from.value)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;

    // SPEC §5 + §7 — out-of-scope apps surface byte-exact refusal here.
    match args.application.as_str() {
        "rsa" | "rsa-gpg" | "dice" => return Err(ToolkitError::DeriveChildUnsupportedApp),
        _ => {}
    }

    let output = match args.application.as_str() {
        "bip39" => {
            let words = args.length;
            if !matches!(words, 12 | 15 | 18 | 21 | 24) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "bip39",
                    length: words,
                    valid_text: "12 | 15 | 18 | 21 | 24 words",
                });
            }
            // SPEC §4 — English only in v0.7 (language code 0).
            bip85::format_bip39_phrase(&master, 0, words, args.index)?
        }
        "hd-seed" => {
            reject_length(args.length)?;
            bip85::format_hd_seed_wif(&master, args.index)?
        }
        "xprv" => {
            reject_length(args.length)?;
            bip85::format_xprv_child(&master, args.index)?
        }
        "hex" => {
            let n = args.length;
            if !(16..=64).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "hex",
                    length: n,
                    valid_text: "16..=64 bytes",
                });
            }
            bip85::format_hex_bytes(&master, n, args.index)?
        }
        "password-base64" => {
            let n = args.length;
            if !(20..=86).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "password-base64",
                    length: n,
                    valid_text: "20..=86 chars",
                });
            }
            bip85::format_password_base64(&master, n, args.index)?
        }
        "password-base85" => {
            let n = args.length;
            if !(10..=80).contains(&n) {
                return Err(ToolkitError::DeriveChildLengthOutOfRange {
                    app: "password-base85",
                    length: n,
                    valid_text: "10..=80 chars",
                });
            }
            bip85::format_password_base85(&master, n, args.index)?
        }
        other => {
            return Err(ToolkitError::BadInput(format!(
                "derive-child: --application {other:?} is not recognized; \
                 expected one of: bip39, hd-seed, xprv, hex, password-base64, \
                 password-base85 (or out-of-scope: rsa, rsa-gpg, dice)",
            )));
        }
    };

    writeln!(stdout, "{output}").ok();
    // SPEC §4 — every in-scope app emits secret material; warn on stdout.
    let _ = writeln!(
        stderr,
        "warning: secret material on stdout — consider redirecting (e.g., '> file.txt' or '| age -e ...')",
    );
    Ok(())
}

/// SPEC §7 — `hd-seed` / `xprv` ignore `--length 0` (sentinel for grammar-
/// uniformity); any non-zero value triggers the not-applicable refusal.
fn reject_length(length: u32) -> Result<(), ToolkitError> {
    if length != 0 {
        return Err(ToolkitError::DeriveChildLengthNotApplicable);
    }
    Ok(())
}
