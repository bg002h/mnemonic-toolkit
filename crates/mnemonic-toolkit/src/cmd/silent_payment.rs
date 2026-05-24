//! `mnemonic silent-payment` — derive the BIP-352 receiver static address
//! (`sp1…`/`tsp1…`, base + labeled m≥1) from a seed-bearing secret, plus the
//! scan/spend pubkeys, derivation paths, and the scan/spend private keys
//! (advisory-gated). Sender output construction + chain scanning are OUT of
//! scope (no tx inputs / chain / signing). See `recon-silent-payments.md`.

use crate::error::ToolkitError;
use crate::network::CliNetwork;
use crate::secret_advisory::{secret_in_argv_warning, secret_on_stdout_warning_unconditional};
use bitcoin::bip32::Xpriv;
use clap::{ArgGroup, Args};
use std::io::{Read, Write};
use std::str::FromStr;
use zeroize::Zeroizing;

#[derive(Args, Debug)]
#[command(group(
    ArgGroup::new("secret_src")
        .required(true)
        .multiple(false)
        .args(["secret", "secret_file", "secret_stdin"]),
))]
pub struct SilentPaymentArgs {
    /// Seed-bearing secret: BIP-39 phrase / ms1 / entropy-hex / master xprv.
    /// A single private key (WIF/minikey) is refused — it cannot derive `m/352'`.
    /// SECRET — leaks via argv; prefer --secret-file / --secret-stdin.
    #[arg(long)]
    pub secret: Option<String>,

    /// Read the seed-bearing secret from a file (avoids argv exposure).
    #[arg(long = "secret-file")]
    pub secret_file: Option<std::path::PathBuf>,

    /// Read the seed-bearing secret from stdin.
    #[arg(long = "secret-stdin")]
    pub secret_stdin: bool,

    /// BIP-39 mnemonic-extension passphrase ("25th word"). Applies to phrase /
    /// ms1 / entropy-hex inputs; ignored (with a warning) for an xprv input
    /// (the xprv IS the master). SECRET — leaks via argv; prefer --passphrase-stdin.
    #[arg(long)]
    pub passphrase: Option<String>,

    /// Read the BIP-39 passphrase from stdin (whitespace-preserving; mutually
    /// exclusive with --passphrase, and with --secret-stdin — one stdin per run).
    #[arg(long = "passphrase-stdin", conflicts_with = "passphrase")]
    pub passphrase_stdin: bool,

    /// Bitcoin network: mainnet → `sp` address + coin-type 0; testnet/signet/
    /// regtest → `tsp` address + coin-type 1.
    #[arg(long, value_enum, default_value_t = CliNetwork::Mainnet)]
    pub network: CliNetwork,

    /// BIP-32 account index (`m/352'/coin'/account'/…`).
    #[arg(long, default_value_t = 0)]
    pub account: u32,

    /// Emit a labeled silent-payment address for label m (repeatable). m≥1;
    /// m=0 is the reserved BIP-352 change label and is refused (never publish it).
    #[arg(long = "label")]
    pub label: Vec<u32>,

    /// Also emit the BIP-352 m=0 CHANGE address. For the wallet's OWN change
    /// detection ONLY — never hand it out as a receiving address.
    #[arg(long = "change-address")]
    pub change_address: bool,

    /// Emit JSON instead of the human-readable block.
    #[arg(long)]
    pub json: bool,
}

#[derive(serde::Serialize)]
struct LabeledAddr {
    m: u32,
    address: String,
}

#[derive(serde::Serialize)]
struct SilentPaymentJson {
    network: String,
    account: u32,
    address: String, // base (unlabeled)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    labeled: Vec<LabeledAddr>,
    // BIP-352 m=0 change address (only when --change-address). The warning
    // sibling is emitted in the same envelope so a JSON consumer can't surface
    // `change_address` as a receive target.
    #[serde(skip_serializing_if = "Option::is_none")]
    change_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    change_address_warning: Option<&'static str>,
    scan_pubkey: String,
    spend_pubkey: String,
    scan_path: String,
    spend_path: String,
    scan_priv: String,  // online / hot
    spend_priv: String, // COLD — full spending authority
}

/// Resolve a seed-bearing secret string → master `Xpriv`. Value-sniff order:
/// xprv/tprv → ms1 → BIP-39 phrase → entropy-hex → else refuse (covers WIF/minikey).
/// `passphrase` is the BIP-39 mnemonic-extension passphrase ("25th word"), applied
/// to the phrase/ms1/entropy paths; the xprv path is passphrase-independent (the
/// xprv IS the master) and warns if a passphrase was nonetheless supplied.
fn resolve_master_xpriv<E: Write>(
    secret: &str,
    passphrase: &str,
    network: CliNetwork,
    stderr: &mut E,
) -> Result<Xpriv, ToolkitError> {
    let s = secret.trim();
    let to_master = |entropy: &[u8]| -> Result<Xpriv, ToolkitError> {
        let mnemonic = bip39::Mnemonic::from_entropy_in(bip39::Language::English, entropy)
            .map_err(|e| ToolkitError::SilentPayment(format!("entropy → BIP-39 mnemonic: {e}")))?;
        let seed = crate::derive_slot::derive_master_seed(&mnemonic, passphrase);
        Xpriv::new_master(network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::SilentPayment(format!("master xpriv: {e}")))
    };

    // 1. xprv/tprv master (unambiguous base58check + version prefix).
    if let Ok(xpriv) = Xpriv::from_str(s) {
        if !passphrase.is_empty() {
            writeln!(
                stderr,
                "warning: --passphrase ignored — an xprv/tprv input is already the master key \
                 (BIP-39 passphrase applies only to phrase/ms1/entropy inputs)"
            )
            .map_err(ToolkitError::Io)?;
        }
        return Ok(xpriv);
    }
    // 2. ms1 → entropy (unambiguous bech32 `ms` HRP).
    if s.starts_with("ms1") {
        let (_tag, payload) = ms_codec::decode(s).map_err(ToolkitError::from)?;
        let entropy: Zeroizing<Vec<u8>> = match payload {
            ms_codec::Payload::Entr(b) => Zeroizing::new(b),
            _ => {
                return Err(ToolkitError::SilentPayment(
                    "ms1 decoded to a non-entropy payload".into(),
                ))
            }
        };
        return to_master(&entropy);
    }
    // 3. BIP-39 phrase (whitespace-separated words; checksum-validated).
    if s.split_whitespace().count() >= 2 {
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, s)
            .map_err(|e| ToolkitError::SilentPayment(format!("BIP-39 phrase: {e}")))?;
        let seed = crate::derive_slot::derive_master_seed(&mnemonic, passphrase);
        return Xpriv::new_master(network.network_kind(), &seed[..])
            .map_err(|e| ToolkitError::SilentPayment(format!("master xpriv: {e}")));
    }
    // 4. entropy hex (BIP-39-valid lengths only).
    if let Ok(bytes) = hex::decode(s) {
        if matches!(bytes.len(), 16 | 20 | 24 | 28 | 32) {
            let entropy = Zeroizing::new(bytes);
            return to_master(&entropy);
        }
    }
    // 5. refuse single-key / unrecognized.
    Err(ToolkitError::SilentPayment(
        "expected a seed-bearing secret (BIP-39 phrase / ms1 / entropy-hex / xprv); \
         a single private key (WIF/minikey) cannot derive m/352'"
            .into(),
    ))
}

pub fn run<R: Read, W: Write, E: Write>(
    args: &SilentPaymentArgs,
    stdin: &mut R,
    stdout: &mut W,
    stderr: &mut E,
) -> Result<u8, ToolkitError> {
    // m=0 is the reserved change label — refuse before any derivation.
    if args.label.contains(&0) {
        return Err(ToolkitError::SilentPayment(
            "--label 0 is the reserved BIP-352 change label and must never be published; use m≥1"
                .into(),
        ));
    }
    // Single stdin per invocation — refuse the two-readers case BEFORE any read.
    if args.passphrase_stdin && args.secret_stdin {
        return Err(ToolkitError::SilentPayment(
            "--passphrase-stdin cannot be combined with --secret-stdin (single stdin per invocation)"
                .into(),
        ));
    }

    // Resolve the seed-bearing secret (with argv-leak advisory for inline).
    let secret: Zeroizing<String> = if let Some(s) = &args.secret {
        secret_in_argv_warning(stderr, "--secret", "--secret-stdin");
        Zeroizing::new(s.clone())
    } else if let Some(path) = &args.secret_file {
        Zeroizing::new(std::fs::read_to_string(path).map_err(ToolkitError::Io)?.trim().to_string())
    } else if args.secret_stdin {
        let mut buf = String::new();
        stdin.read_to_string(&mut buf).map_err(ToolkitError::Io)?;
        Zeroizing::new(buf.trim().to_string())
    } else {
        return Err(ToolkitError::SilentPayment(
            "exactly one of --secret / --secret-file / --secret-stdin is required".into(),
        ));
    };
    let _pin = mnemonic_toolkit::mlock::pin_pages_for(secret.as_bytes());

    // Resolve the BIP-39 passphrase. Whitespace is SIGNIFICANT (PBKDF2 salt) —
    // read via read_stdin_passphrase (NOT .trim()) for the stdin path.
    let passphrase: Zeroizing<String> = if let Some(p) = &args.passphrase {
        secret_in_argv_warning(stderr, "--passphrase", "--passphrase-stdin");
        Zeroizing::new(p.clone())
    } else if args.passphrase_stdin {
        Zeroizing::new(crate::cmd::convert::read_stdin_passphrase(stdin)?)
    } else {
        Zeroizing::new(String::new())
    };
    let _pin_pass = mnemonic_toolkit::mlock::pin_pages_for(passphrase.as_bytes());

    let secp = bitcoin::secp256k1::Secp256k1::new();
    let master = resolve_master_xpriv(&secret, &passphrase, args.network, stderr)?;
    let coin = args.network.coin_type();
    let (b_scan, b_spend) = crate::silent_payment::derive_scan_spend(&secp, &master, coin, args.account)?;
    let b_scan_pub = b_scan.public_key(&secp);
    let b_spend_pub = b_spend.public_key(&secp);
    let hrp = crate::silent_payment::sp_hrp(args.network.to_bitcoin_network());

    let base = crate::silent_payment::encode_sp_address(hrp, &b_scan_pub, &b_spend_pub);
    let mut labeled: Vec<LabeledAddr> = Vec::with_capacity(args.label.len());
    for &m in &args.label {
        let b_m = crate::silent_payment::labeled_spend_key(&secp, &b_scan, b_spend_pub, m)?;
        labeled.push(LabeledAddr { m, address: crate::silent_payment::encode_sp_address(hrp, &b_scan_pub, &b_m) });
    }

    // BIP-352 m=0 CHANGE address (opt-in; additive). Internal change-detection
    // ONLY — must never be handed out as a receiving address.
    const CHANGE_WARNING: &str =
        "BIP-352 m=0 change label — internal change detection only; never publish as a receiving address";
    let change_address: Option<String> = if args.change_address {
        let b_m0 = crate::silent_payment::labeled_spend_key(&secp, &b_scan, b_spend_pub, 0)?;
        Some(crate::silent_payment::encode_sp_address(hrp, &b_scan_pub, &b_m0))
    } else {
        None
    };
    let change_address_warning: Option<&'static str> =
        change_address.as_ref().map(|_| CHANGE_WARNING);

    let scan_path = format!("m/352'/{coin}'/{}'/1'/0", args.account);
    let spend_path = format!("m/352'/{coin}'/{}'/0'/0", args.account);
    let scan_priv = hex::encode(b_scan.secret_bytes());
    let spend_priv = hex::encode(b_spend.secret_bytes());

    if args.json {
        let envelope = SilentPaymentJson {
            network: args.network.human_name().to_string(),
            account: args.account,
            address: base,
            labeled,
            change_address,
            change_address_warning,
            scan_pubkey: hex::encode(b_scan_pub.serialize()),
            spend_pubkey: hex::encode(b_spend_pub.serialize()),
            scan_path,
            spend_path,
            scan_priv,
            spend_priv,
        };
        serde_json::to_writer_pretty(&mut *stdout, &envelope)
            .map_err(|e| ToolkitError::SilentPayment(format!("json serialize: {e}")))?;
        writeln!(stdout).map_err(ToolkitError::Io)?;
    } else {
        writeln!(stdout, "silent-payment ({}, account {})", args.network.human_name(), args.account).map_err(ToolkitError::Io)?;
        writeln!(stdout, "  address:      {base}").map_err(ToolkitError::Io)?;
        for l in &labeled {
            writeln!(stdout, "  label {:<7} {}", l.m, l.address).map_err(ToolkitError::Io)?;
        }
        if let Some(ch) = &change_address {
            writeln!(stdout, "  change_addr:  {ch}   (BIP-352 m=0 CHANGE — internal change detection ONLY; never hand out as a receiving address)").map_err(ToolkitError::Io)?;
        }
        writeln!(stdout, "  scan_pubkey:  {}", hex::encode(b_scan_pub.serialize())).map_err(ToolkitError::Io)?;
        writeln!(stdout, "  spend_pubkey: {}", hex::encode(b_spend_pub.serialize())).map_err(ToolkitError::Io)?;
        writeln!(stdout, "  scan_path:    {scan_path}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  spend_path:   {spend_path}").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  scan_priv:    {scan_priv}   (online / hot key)").map_err(ToolkitError::Io)?;
        writeln!(stdout, "  spend_priv:   {spend_priv}   (COLD — full spending authority)").map_err(ToolkitError::Io)?;
    }
    secret_on_stdout_warning_unconditional(stderr);
    Ok(0)
}
