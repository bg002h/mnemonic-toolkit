//! Plaintext-table renderer + JSON envelope serializer. SPEC §5.

use std::io::{self, Write};

use serde::Serialize;

use super::enumerate::{Row, SEGWIT_INPUT_BASE_WU};

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

#[derive(Serialize)]
pub struct Envelope<'a> {
    pub schema_version: u32,
    pub subcommand: &'a str,
    pub input: InputJson<'a>,
    pub extracted_miniscript: &'a str,
    pub feerate_sat_per_vb: f64,
    pub conditions: Vec<RowJson>,
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

    if !notes.is_empty() {
        writeln!(out)?;
        for n in notes {
            writeln!(out, "note: {n}")?;
        }
    }

    Ok(())
}

pub fn render_json<W: Write>(
    original_input: &str,
    input_form_label: &str,
    extracted: &str,
    feerate: f64,
    rows: &[Row],
    notes: &[String],
    out: &mut W,
) -> io::Result<()> {
    let conditions: Vec<RowJson> = rows.iter().map(|r| build_row_json(r, feerate)).collect();
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
