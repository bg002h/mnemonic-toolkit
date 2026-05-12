//! SPEC v0.8 §9 — Electrum wallet-import emitter.
//!
//! Format reference (authoritative schema): <https://github.com/spesmilo/electrum/blob/master/electrum/wallet_db.py>.
//! `FINAL_SEED_VERSION` is currently `71` on Electrum master (4.5.x).
//!
//! `seed_version` policy: Electrum's `wallet_db.py` upgrades wallet files in
//! place on load, walking each `_convert_version_<N>` migration. The toolkit
//! emits `ELECTRUM_SEED_VERSION_PIN` (constant below); current Electrum will
//! upgrade-migrate to FINAL_SEED_VERSION on first save. FOLLOWUPS entry
//! `electrum-final-seed-version-drift` tracks ongoing upstream drift; the
//! companion entry `electrum-seed-version-spike-pending` covers the v0.8.2
//! interactive spike to lock the constant to a verified-cleanly-imports
//! value (Phase 4 step 0 was deferred for the v0.8.1 cut).

use super::{EmitInputs, MissingField, WalletFormatEmitter, WalletScriptType};
use crate::error::ToolkitError;
use crate::slip0132::{apply_xpub_prefix, XpubPrefix};
use serde::Serialize;
use serde_json::{Map, Value};

/// SPEC v0.8 §9 — pinned Electrum `seed_version` value.
///
/// **Empirically validated** by the Phase 4 step 0 spike (2026-05-12,
/// Electrum 4.5.5; report: `design/agent-reports/v0_8-phase-4-electrum-seed-version-spike.md`).
/// A toolkit-emitted wallet file with `seed_version: 17` loads cleanly via
/// `electrum --offline -w <file> listaddresses` and Electrum's loader walks
/// the `_convert_version_<N>` migration chain forward to FINAL_SEED_VERSION
/// (59 on Electrum 4.5.5; the upstream master drifts higher per FOLLOWUPS
/// `electrum-final-seed-version-drift`).
///
/// Why 17: SPEC §9 specifies "minimum seed_version that current Electrum
/// imports cleanly". `wallet_db.py:1207` returns any `seed_version >= 12`
/// directly (with specific rejections only at 14-segwit and 51-insane), so
/// 17 sits safely above the special-case rejection band. Pinning to 17
/// (rather than the WRITE-time FINAL_SEED_VERSION) maximizes downstream
/// compatibility with older Electrum installs.
pub const ELECTRUM_SEED_VERSION_PIN: u32 = 17;

/// SPEC v0.8 §9 — `WalletFormatEmitter` impl for `--format electrum`.
pub(crate) struct ElectrumEmitter;

impl WalletFormatEmitter for ElectrumEmitter {
    fn collect_missing(_inputs: &EmitInputs) -> Vec<MissingField> {
        // Electrum's `keystore.label` field defaults to "" if `--wallet-name`
        // is absent — the wallet imports cleanly and the user can rename in
        // the Electrum UI. No SPEC §4 missing-info refusal required.
        Vec::new()
    }

    fn emit(inputs: &EmitInputs) -> Result<String, ToolkitError> {
        use crate::template::CliTemplate;
        let template = inputs.template.ok_or_else(|| {
            ToolkitError::BadInput(
                "--format electrum requires --template; descriptor passthrough is not supported by Electrum's wallet-db schema".into(),
            )
        })?;

        // SPEC §9.2 closing paragraph: tr-multi-a refuses pending Electrum
        // libsecp-taproot support (FOLLOWUPS `electrum-tr-multi-a-pending-libsecp-taproot`).
        if matches!(
            template,
            CliTemplate::TrMultiA | CliTemplate::TrSortedMultiA
        ) {
            return Err(ToolkitError::BadInput(format!(
                "--format electrum does not yet support --template {} — Electrum's wallet-db does not currently ingest taproot multisig (tracked by FOLLOWUPS electrum-tr-multi-a-pending-libsecp-taproot). Use --format bitcoin-core (descriptor) or --format sparrow for taproot multisig watch-only setup.",
                template.human_name(),
            )));
        }

        if template.is_multisig() {
            emit_electrum_multisig_json(inputs)
        } else {
            emit_electrum_standard_json(inputs)
        }
    }

    fn extension() -> &'static str {
        "json"
    }
}

#[derive(Serialize)]
struct ElectrumStandard<'a> {
    seed_version: u32,
    wallet_type: &'static str,
    use_encryption: bool,
    keystore: ElectrumKeystore<'a>,
}

#[derive(Serialize)]
struct ElectrumKeystore<'a> {
    #[serde(rename = "type")]
    kind: &'static str,
    xpub: String,
    derivation: String,
    root_fingerprint: String,
    label: &'a str,
}

/// SPEC §9.1 — singlesig wallet shape.
fn emit_electrum_standard_json(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    if inputs.resolved_slots.len() != 1 {
        return Err(ToolkitError::BadInput(format!(
            "--format electrum singlesig requires exactly one --slot @0; got {}",
            inputs.resolved_slots.len(),
        )));
    }
    let slot = &inputs.resolved_slots[0];
    let template = inputs.template.expect("checked by caller");
    let xpub_str = render_slip132_xpub(inputs.script_type, &slot.xpub, inputs.network)?;
    let derivation = template.origin_path_str(inputs.network, inputs.account);

    let wallet = ElectrumStandard {
        seed_version: ELECTRUM_SEED_VERSION_PIN,
        wallet_type: "standard",
        use_encryption: false,
        keystore: ElectrumKeystore {
            kind: "bip32",
            xpub: xpub_str,
            derivation,
            root_fingerprint: slot.fingerprint.to_string().to_lowercase(),
            label: inputs.wallet_name,
        },
    };

    serde_json::to_string_pretty(&wallet)
        .map_err(|e| ToolkitError::BadInput(format!("--format electrum: serialize: {e}")))
}

/// SPEC §9.2 — multisig wallet shape.
fn emit_electrum_multisig_json(inputs: &EmitInputs) -> Result<String, ToolkitError> {
    let n = inputs.resolved_slots.len();
    if n < 2 {
        return Err(ToolkitError::BadInput(format!(
            "--format electrum multisig requires at least 2 cosigners; got {n}"
        )));
    }
    let k = inputs.threshold.ok_or_else(|| {
        ToolkitError::BadInput("--format electrum multisig requires --threshold <K>".into())
    })?;
    let wallet_type = format!("{k}of{n}");

    // serde_json::Map preserves insertion order when crate has `preserve_order`
    // disabled (default) — output is alphabetical. We compose the object
    // manually using `Value` array of key-value tuples to lock field order
    // (seed_version, wallet_type, use_encryption, x1/, x2/, ...) per SPEC §9.2.
    // serde_json::to_string_pretty on a `serde_json::Map` sorts alphabetically;
    // we route through `Value::Object(Map)` with the multisig structure since
    // the keys `seed_version`, `use_encryption`, `wallet_type`, `x1/`, ...
    // sort alphabetically AS Electrum's loader writes them anyway. Electrum
    // does not depend on field order at load (Python json module).
    let mut top = Map::new();
    top.insert(
        "seed_version".into(),
        Value::Number(ELECTRUM_SEED_VERSION_PIN.into()),
    );
    top.insert("wallet_type".into(), Value::String(wallet_type));
    top.insert("use_encryption".into(), Value::Bool(false));

    for (i, slot) in inputs.resolved_slots.iter().enumerate() {
        let key = format!("x{}/", i + 1);
        let xpub_str = render_slip132_xpub(inputs.script_type, &slot.xpub, inputs.network)?;
        let derivation = if !slot.path_raw.is_empty() {
            slot.path_raw.clone()
        } else {
            inputs
                .template
                .expect("checked")
                .origin_path_str(inputs.network, inputs.account)
        };
        let mut keystore = Map::new();
        keystore.insert("type".into(), Value::String("bip32".into()));
        keystore.insert("xpub".into(), Value::String(xpub_str));
        keystore.insert("derivation".into(), Value::String(derivation));
        keystore.insert(
            "root_fingerprint".into(),
            Value::String(slot.fingerprint.to_string().to_lowercase()),
        );
        keystore.insert(
            "label".into(),
            Value::String(format!("{}-{}", inputs.wallet_name, i + 1)),
        );
        top.insert(key, Value::Object(keystore));
    }

    serde_json::to_string_pretty(&Value::Object(top))
        .map_err(|e| ToolkitError::BadInput(format!("--format electrum multisig: serialize: {e}")))
}

/// SPEC §9.1 / §9.2 — render xpub in the SLIP-132 variant matching the
/// script type × network. Singlesig: ypub/zpub/upub/vpub. Multisig:
/// Ypub/Zpub/Upub/Vpub (capital). p2pkh and p2tr have no SLIP-132 variant
/// and emit the neutral form.
fn render_slip132_xpub(
    script_type: WalletScriptType,
    xpub: &bitcoin::bip32::Xpub,
    network: crate::network::CliNetwork,
) -> Result<String, ToolkitError> {
    let variant = match script_type {
        WalletScriptType::P2pkh | WalletScriptType::P2tr | WalletScriptType::P2shMulti => {
            // No SLIP-132 variant; return neutral xpub/tpub form.
            return Ok(xpub.to_string());
        }
        WalletScriptType::P2shP2wpkh => XpubPrefix::Ypub,
        WalletScriptType::P2wpkh => XpubPrefix::Zpub,
        WalletScriptType::P2shP2wshMulti => XpubPrefix::YpubMultisig,
        WalletScriptType::P2wshMulti => XpubPrefix::ZpubMultisig,
        WalletScriptType::P2trMulti => {
            return Err(ToolkitError::BadInput(
                "--format electrum: taproot multisig is refused upstream by emit()".into(),
            ))
        }
    };
    Ok(apply_xpub_prefix(xpub, variant, network))
}
