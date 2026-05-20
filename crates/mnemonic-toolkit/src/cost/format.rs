//! Plaintext-table renderer + JSON envelope serializer. SPEC §5 + §11.

use std::io::{self, Write};

use serde::Serialize;

use super::enumerate::{Row, SEGWIT_INPUT_BASE_WU};
use super::KeypathSpend;

/// Per-row JSON shape — keys mirror SPEC §5 example.
#[derive(Serialize)]
pub struct RowJson {
    pub label: String,
    pub wsh_vbytes: i64,
    pub tr_vbytes: i64,
    pub delta_vbytes: i64,
    pub wsh_sats: i64,
    pub tr_sats: i64,
    pub delta_sats: i64,
}

#[derive(Serialize)]
pub struct InputJson<'a> {
    pub form: &'a str,
    pub value: &'a str,
}

/// SPEC §11 (v0.28.0) — keypath-spend cost surfaced when input is
/// `tr(IK, {M})` with a non-NUMS internal key.
#[derive(Serialize)]
pub struct KeypathSpendJson<'a> {
    pub internal_key_xonly_hex: &'a str,
    pub vbytes: i64,
    pub sats: i64,
}

#[derive(Serialize)]
pub struct Envelope<'a> {
    pub schema_version: u32,
    pub subcommand: &'a str,
    pub input: InputJson<'a>,
    pub extracted_miniscript: &'a str,
    pub feerate_sat_per_vb: f64,
    pub conditions: Vec<RowJson>,
    /// SPEC §11 — present only when input is `tr(IK, {M})` with
    /// non-NUMS IK. `None` is serialized as JSON `null` (preserved field
    /// for shape stability so downstream consumers can branch on presence
    /// via type-check rather than key-existence-check).
    pub keypath_spend: Option<KeypathSpendJson<'a>>,
    pub notes: &'a [String],
}

/// Convert a Row's raw `witness_bytes` to the full per-input cost (vbytes),
/// per SPEC §4: `vbytes = (164 + witness_bytes + 3) / 4`.
pub fn witness_bytes_to_vbytes(witness_bytes: usize) -> i64 {
    let total_wu = SEGWIT_INPUT_BASE_WU + witness_bytes;
    ((total_wu + 3) / 4) as i64
}

pub fn render_table<W: Write>(
    original_input: &str,
    extracted: &str,
    feerate: f64,
    rows: &[Row],
    notes: &[String],
    keypath_spend: Option<&KeypathSpend>,
    out: &mut W,
) -> io::Result<()> {
    // For --miniscript input the original equals the extracted M; for
    // --descriptor input the original is wsh(M) / sh(wsh(M)) etc. and we
    // surface both so the plaintext header doesn't silently drop the
    // wrapper the user typed. (JSON mode renders the same pair as
    // input.value + extracted_miniscript.)
    if original_input == extracted {
        writeln!(out, "Input: {extracted}")?;
    } else {
        writeln!(out, "Input:     {original_input}")?;
        writeln!(out, "Extracted: {extracted}")?;
    }
    writeln!(out, "Wrapper comparison: wsh(M)  vs  tr(NUMS, {{M}})")?;
    // Use {:.} formatting to avoid Rust's `f64` Display dropping the `.0`
    // suffix on integer-valued feerates — show at least one decimal.
    writeln!(out, "Feerate: {feerate:.1} sat/vB")?;
    writeln!(out)?;

    // Compute columns.
    let table_rows: Vec<RowJson> = rows
        .iter()
        .map(|r| build_row_json(r, feerate))
        .collect();

    // Column widths.
    let label_w = std::cmp::max(
        "Condition".len(),
        table_rows.iter().map(|r| r.label.len()).max().unwrap_or(0),
    );

    writeln!(
        out,
        "{:<lw$} | {:>6} | {:>5} | {:>5} | {:>8} | {:>7} | {:>6}",
        "Condition",
        "wsh vB",
        "tr vB",
        "Δ vB",
        "wsh sats",
        "tr sats",
        "Δ sats",
        lw = label_w
    )?;
    // Separator
    writeln!(
        out,
        "{:-<lw$}-+-{:-<6}-+-{:-<5}-+-{:-<5}-+-{:-<8}-+-{:-<7}-+-{:-<6}",
        "",
        "",
        "",
        "",
        "",
        "",
        "",
        lw = label_w
    )?;
    for r in &table_rows {
        writeln!(
            out,
            "{:<lw$} | {:>6} | {:>5} | {:>+5} | {:>8} | {:>7} | {:>+6}",
            r.label,
            r.wsh_vbytes,
            r.tr_vbytes,
            r.delta_vbytes,
            r.wsh_sats,
            r.tr_sats,
            r.delta_sats,
            lw = label_w
        )?;
    }

    // SPEC §11 — keypath-spend cost annotation. Lives below the table
    // (not as a vertical column) because the table is row-aligned by
    // script-path condition; the keyspend has no per-condition variance.
    if let Some(ks) = keypath_spend {
        writeln!(out)?;
        let sats = (ks.vbytes as f64 * feerate).round() as i64;
        writeln!(
            out,
            "Keypath-spend (via IK {ik}): {vb} vB | {sats} sats",
            ik = ks.internal_key_xonly_hex,
            vb = ks.vbytes,
            sats = sats,
        )?;
    }

    if !notes.is_empty() {
        writeln!(out)?;
        for n in notes {
            writeln!(out, "note: {n}")?;
        }
    }

    Ok(())
}

#[allow(clippy::too_many_arguments)] // §11 added keypath_spend; lifting to a
                                     // builder struct is gold-plating for a
                                     // thin two-call renderer.
pub fn render_json<W: Write>(
    original_input: &str,
    input_form_label: &str,
    extracted: &str,
    feerate: f64,
    rows: &[Row],
    notes: &[String],
    keypath_spend: Option<&KeypathSpend>,
    out: &mut W,
) -> io::Result<()> {
    let conditions: Vec<RowJson> = rows.iter().map(|r| build_row_json(r, feerate)).collect();
    let keypath_spend_json = keypath_spend.map(|ks| KeypathSpendJson {
        internal_key_xonly_hex: ks.internal_key_xonly_hex.as_str(),
        vbytes: ks.vbytes,
        sats: (ks.vbytes as f64 * feerate).round() as i64,
    });
    let envelope = Envelope {
        schema_version: 1,
        subcommand: "compare-cost",
        input: InputJson {
            form: input_form_label,
            value: original_input,
        },
        extracted_miniscript: extracted,
        feerate_sat_per_vb: feerate,
        conditions,
        keypath_spend: keypath_spend_json,
        notes,
    };
    serde_json::to_writer_pretty(&mut *out, &envelope)
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
    writeln!(out)?;
    Ok(())
}

fn build_row_json(r: &Row, feerate: f64) -> RowJson {
    let wsh_vb = witness_bytes_to_vbytes(r.wsh_witness_bytes);
    let tr_vb = witness_bytes_to_vbytes(r.tr_witness_bytes);
    let delta_vb = tr_vb - wsh_vb;
    let wsh_sats = (wsh_vb as f64 * feerate).round() as i64;
    let tr_sats = (tr_vb as f64 * feerate).round() as i64;
    let delta_sats = tr_sats - wsh_sats;
    RowJson {
        label: r.label.clone(),
        wsh_vbytes: wsh_vb,
        tr_vbytes: tr_vb,
        delta_vbytes: delta_vb,
        wsh_sats,
        tr_sats,
        delta_sats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn witness_bytes_to_vbytes_known_values() {
        // SegWit input overhead = 41 vB.
        assert_eq!(witness_bytes_to_vbytes(0), 41);
        // Schnorr keyspend witness = 66 bytes; (164+66+3)/4 = 233/4 = 58.
        assert_eq!(witness_bytes_to_vbytes(66), 58);
    }
}
