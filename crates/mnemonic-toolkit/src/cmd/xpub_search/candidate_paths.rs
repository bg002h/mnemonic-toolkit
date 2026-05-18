//! Candidate-path enumeration for `xpub-search` (P1/P2/P4).
//!
//! Synthesizes the candidate `(template_name, DerivationPath)` set per plan
//! §3.2 step 4:
//!
//! 1. **Single-sig templates** via `CliTemplate::derivation_path`:
//!    `bip44`, `bip49`, `bip84`, `bip86` × accounts in `[min, end)`.
//! 2. **BIP-48 multisig** via `MultisigPathFamily::Bip48.default_origin_path`:
//!    script_type `1'`/`2'`/`3'` × accounts in `[min, end)`.
//!    Template names: `bip48-sh-wsh` / `bip48-wsh` / `bip48-tr-multi-a`.
//! 3. **`--add-path`** user-supplied templates: for each, substitute the
//!    account index for the first `account'` token (then the first `account`
//!    token if no `account'` found); if neither token is present, search the
//!    path exactly once as-is.
//!
//! Determinism: fixed `Vec<&'static str>` template ordering with single-sig
//! first then multisig; accounts ascending; add-paths in user-supplied order.
//! No `HashMap`/`HashSet` for iteration (plan §9.6 / R0 R4 lock).

use crate::network::CliNetwork;
use crate::parse::MultisigPathFamily;
use crate::template::CliTemplate;
use bitcoin::bip32::DerivationPath;
use std::str::FromStr;

/// One element of the candidate set: (template-name, DerivationPath, account).
/// `account` is `Some(n)` when the template iterates over the account range,
/// `None` for `--add-path` templates that have no `account` token (those are
/// searched at exactly one path).
#[derive(Debug, Clone)]
pub struct CandidatePath {
    pub template_name: String,
    pub path: DerivationPath,
    pub account: Option<u32>,
}

/// Build the full deterministic candidate set per plan §3.2 step 4.
///
/// - `min_account`: lower bound (inclusive).
/// - `number_of_accounts`: window size (default 20).
/// - `max_account`: optional upper bound. Effective end is
///   `max(min_account + number_of_accounts, max_account.unwrap_or(...))`.
/// - `add_paths`: user-supplied `--add-path` strings in invocation order.
/// - `network`: drives the `coin'` component in standard templates.
pub fn build_candidate_paths(
    min_account: u32,
    number_of_accounts: u32,
    max_account: Option<u32>,
    add_paths: &[String],
    network: CliNetwork,
) -> Vec<CandidatePath> {
    let lo = min_account;
    let hi = {
        let from_n = min_account.saturating_add(number_of_accounts);
        match max_account {
            Some(m) => from_n.max(m.saturating_add(1)),
            None => from_n,
        }
    };
    let n = hi.saturating_sub(lo);

    let mut out: Vec<CandidatePath> = Vec::with_capacity((n as usize) * 7 + add_paths.len());

    // 1) Single-sig templates: bip44, bip49, bip84, bip86 — fixed order.
    let single_sig: &[(CliTemplate, &str)] = &[
        (CliTemplate::Bip44, "bip44"),
        (CliTemplate::Bip49, "bip49"),
        (CliTemplate::Bip84, "bip84"),
        (CliTemplate::Bip86, "bip86"),
    ];
    for (template, name) in single_sig {
        for account in lo..hi {
            out.push(CandidatePath {
                template_name: (*name).to_string(),
                path: template.derivation_path(network, account),
                account: Some(account),
            });
        }
    }

    // 2) BIP-48 multisig at script_type 1' / 2' / 3'.
    let multisig: &[(u32, &str)] = &[
        (1, "bip48-sh-wsh"),
        (2, "bip48-wsh"),
        (3, "bip48-tr-multi-a"),
    ];
    for (script_type, name) in multisig {
        for account in lo..hi {
            let path_str =
                MultisigPathFamily::Bip48.default_origin_path(network, account, *script_type);
            let path = DerivationPath::from_str(&path_str)
                .expect("BIP-48 paths from default_origin_path are well-formed by construction");
            out.push(CandidatePath {
                template_name: (*name).to_string(),
                path,
                account: Some(account),
            });
        }
    }

    // 3) --add-path user-supplied templates, in user-supplied order.
    for tmpl in add_paths {
        let (has_token, replacer): (bool, Box<dyn Fn(u32) -> Option<String>>) =
            if tmpl.contains("account'") {
                let prefix = tmpl
                    .find("account'")
                    .expect("contains() succeeded; find() must succeed");
                let after = prefix + "account'".len();
                let head = tmpl[..prefix].to_string();
                let tail = tmpl[after..].to_string();
                (
                    true,
                    Box::new(move |a: u32| Some(format!("{head}{a}'{tail}"))),
                )
            } else if tmpl.contains("account") {
                let prefix = tmpl
                    .find("account")
                    .expect("contains() succeeded; find() must succeed");
                let after = prefix + "account".len();
                let head = tmpl[..prefix].to_string();
                let tail = tmpl[after..].to_string();
                (
                    true,
                    Box::new(move |a: u32| Some(format!("{head}{a}{tail}"))),
                )
            } else {
                let lit = tmpl.clone();
                (false, Box::new(move |_a: u32| Some(lit.clone())))
            };

        if has_token {
            for account in lo..hi {
                let path_str = replacer(account).unwrap();
                match DerivationPath::from_str(&path_str) {
                    Ok(path) => out.push(CandidatePath {
                        template_name: tmpl.clone(),
                        path,
                        account: Some(account),
                    }),
                    Err(_) => continue, // skip malformed; user will see no-match
                }
            }
        } else {
            // Search once at the literal path (no account token).
            match DerivationPath::from_str(tmpl) {
                Ok(path) => out.push(CandidatePath {
                    template_name: tmpl.clone(),
                    path,
                    account: None,
                }),
                Err(_) => continue,
            }
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_templates_at_account_0_default_range() {
        let candidates = build_candidate_paths(0, 20, None, &[], CliNetwork::Mainnet);
        // 7 templates × 20 accounts = 140
        assert_eq!(candidates.len(), 140);
        assert_eq!(candidates[0].template_name, "bip44");
        assert_eq!(candidates[0].path.to_string(), "44'/0'/0'");
    }

    #[test]
    fn min_account_5_n_3_yields_range_5_to_8() {
        let candidates = build_candidate_paths(5, 3, None, &[], CliNetwork::Mainnet);
        // 7 templates × 3 accounts = 21
        assert_eq!(candidates.len(), 21);
        let bip84: Vec<&CandidatePath> = candidates
            .iter()
            .filter(|c| c.template_name == "bip84")
            .collect();
        assert_eq!(bip84.len(), 3);
        assert_eq!(bip84[0].account, Some(5));
        assert_eq!(bip84[2].account, Some(7));
    }

    #[test]
    fn max_account_50_widens_range() {
        let candidates = build_candidate_paths(0, 5, Some(50), &[], CliNetwork::Mainnet);
        // max(0+5, 50+1) = 51; 7 × 51 = 357
        assert_eq!(candidates.len(), 7 * 51);
    }

    #[test]
    fn bip48_multisig_at_three_script_types() {
        let candidates = build_candidate_paths(0, 1, None, &[], CliNetwork::Mainnet);
        let mset: Vec<&CandidatePath> = candidates
            .iter()
            .filter(|c| c.template_name.starts_with("bip48-"))
            .collect();
        assert_eq!(mset.len(), 3);
        assert_eq!(mset[0].path.to_string(), "48'/0'/0'/1'");
        assert_eq!(mset[1].path.to_string(), "48'/0'/0'/2'");
        assert_eq!(mset[2].path.to_string(), "48'/0'/0'/3'");
    }

    #[test]
    fn add_path_with_account_quote_substitutes() {
        let add_paths = vec!["m/87'/0'/account'".to_string()];
        let candidates = build_candidate_paths(0, 3, None, &add_paths, CliNetwork::Mainnet);
        let added: Vec<&CandidatePath> = candidates
            .iter()
            .filter(|c| c.template_name == "m/87'/0'/account'")
            .collect();
        assert_eq!(added.len(), 3);
        assert_eq!(added[0].path.to_string(), "87'/0'/0'");
        assert_eq!(added[1].path.to_string(), "87'/0'/1'");
    }

    #[test]
    fn add_path_no_account_token_searched_once() {
        let add_paths = vec!["m/9999'/0'/0'".to_string()];
        let candidates = build_candidate_paths(0, 5, None, &add_paths, CliNetwork::Mainnet);
        let added: Vec<&CandidatePath> = candidates
            .iter()
            .filter(|c| c.template_name == "m/9999'/0'/0'")
            .collect();
        assert_eq!(added.len(), 1);
        assert_eq!(added[0].path.to_string(), "9999'/0'/0'");
        assert_eq!(added[0].account, None);
    }

    #[test]
    fn determinism_two_consecutive_calls_byte_equal() {
        let a = build_candidate_paths(0, 5, None, &[], CliNetwork::Mainnet);
        let b = build_candidate_paths(0, 5, None, &[], CliNetwork::Mainnet);
        let a_names: Vec<&str> = a.iter().map(|c| c.template_name.as_str()).collect();
        let b_names: Vec<&str> = b.iter().map(|c| c.template_name.as_str()).collect();
        assert_eq!(a_names, b_names);
        let a_paths: Vec<String> = a.iter().map(|c| c.path.to_string()).collect();
        let b_paths: Vec<String> = b.iter().map(|c| c.path.to_string()).collect();
        assert_eq!(a_paths, b_paths);
    }
}
