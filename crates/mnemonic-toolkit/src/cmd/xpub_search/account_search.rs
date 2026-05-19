//! Account-of-descriptor search primitive.
//!
//! For each cosigner xpub in the descriptor extract, runs the candidate-path
//! search via `match_xpub_against_paths` (reused from P1). Collects each
//! match; multi-match (same seed matches >1 cosigner) reports all per
//! plan §4.3 step 6.

use super::candidate_paths::CandidatePath;
use super::descriptor_intake::CosignerExtract;
use super::path_search::match_xpub_against_paths;
use bitcoin::bip32::{DerivationPath, Xpriv};

/// One matched cosigner with its place in the descriptor.
#[derive(Debug, Clone)]
pub struct CosignerMatch {
    pub cosigner_index: usize,
    pub template: String,
    pub path: DerivationPath,
    pub account: Option<u32>,
}

/// For each cosigner in `descriptor_extract`, search the candidate path set
/// for a child xpub byte-equal to the cosigner's 65-byte payload.
///
/// - Skipped: cosigners with `is_nums = true` (taproot NUMS internal key).
/// - Skipped: cosigners with `xpub_65 == None` (non-xpub keys, e.g. raw pubkeys).
pub fn match_descriptor_against_seed(
    master_xprv: &Xpriv,
    descriptor_extract: &[CosignerExtract],
    candidates: &[CandidatePath],
) -> Vec<CosignerMatch> {
    let mut out: Vec<CosignerMatch> = Vec::new();
    for cosigner in descriptor_extract {
        if cosigner.is_nums {
            continue;
        }
        let want = match cosigner.xpub_65 {
            Some(b) => b,
            None => continue,
        };
        if let Some(m) = match_xpub_against_paths(master_xprv, candidates, &want) {
            out.push(CosignerMatch {
                cosigner_index: cosigner.idx,
                template: m.template_name,
                path: m.path,
                account: m.account,
            });
        }
    }
    out
}
