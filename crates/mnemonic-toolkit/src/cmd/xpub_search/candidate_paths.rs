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

use crate::error::ToolkitError;
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

/// Refuse an account window that reaches 2^31 (BIP-32 hardened account indexes
/// are 31 bits). The window is `[min, end)` with
/// `end = max(min + number, max + 1)`; refused (never clamped) when
/// `min_account >= 2^31`, `max_account >= 2^31`, or
/// `min_account + number_of_accounts > 2^31` (sum in `u64`).
pub fn check_account_window(
    min_account: u32,
    number_of_accounts: u32,
    max_account: Option<u32>,
) -> Result<(), ToolkitError> {
    const LIMIT: u64 = 1 << 31;
    let too_big = u64::from(min_account) >= LIMIT
        || max_account.is_some_and(|m| u64::from(m) >= LIMIT)
        || u64::from(min_account) + u64::from(number_of_accounts) > LIMIT;
    if too_big {
        return Err(ToolkitError::BadInput(
            "--min-account / --max-account / --number-of-accounts: the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits"
                .into(),
        ));
    }
    Ok(())
}

/// The deterministic candidate set per plan §3.2 step 4, generated lazily by
/// [`CandidatePaths::iter`] (memory is constant in the window size).
#[derive(Debug, Clone)]
pub struct CandidatePaths {
    lo: u32,
    hi: u32,
    add_paths: Vec<String>,
    network: CliNetwork,
}

/// One `--add-path` template: where the account token sits, if any.
enum AddPathShape {
    /// `head` + account + `quoted?` + `tail` (account substituted).
    Token {
        head: String,
        tail: String,
        quote: bool,
    },
    /// Searched once, as given.
    Literal,
}

fn add_path_shape(tmpl: &str) -> AddPathShape {
    if let Some(prefix) = tmpl.find("account'") {
        let after = prefix + "account'".len();
        AddPathShape::Token {
            head: tmpl[..prefix].to_string(),
            tail: tmpl[after..].to_string(),
            quote: true,
        }
    } else if let Some(prefix) = tmpl.find("account") {
        let after = prefix + "account".len();
        AddPathShape::Token {
            head: tmpl[..prefix].to_string(),
            tail: tmpl[after..].to_string(),
            quote: false,
        }
    } else {
        AddPathShape::Literal
    }
}

fn substitute(head: &str, tail: &str, quote: bool, account: u32) -> String {
    if quote {
        format!("{head}{account}'{tail}")
    } else {
        format!("{head}{account}{tail}")
    }
}

const SINGLE_SIG: [(CliTemplate, &str); 4] = [
    (CliTemplate::Bip44, "bip44"),
    (CliTemplate::Bip49, "bip49"),
    (CliTemplate::Bip84, "bip84"),
    (CliTemplate::Bip86, "bip86"),
];
const MULTISIG: [(u32, &str); 3] = [
    (1, "bip48-sh-wsh"),
    (2, "bip48-wsh"),
    (3, "bip48-tr-multi-a"),
];

/// Template name of the BIP-87 multisig family `m/87'/<coin>'/<account>'`.
/// BIP-87: "m / purpose' / coin_type' / account' / change / address_index";
/// it is the toolkit's multisig template default and, since the 2026-11-25
/// BIP-defaults ruling, the descriptor-mode default for bare `sh(...)` and
/// `tr(...)` (review r2 I-A: the search must cover what `bundle` mints).
const BIP87_NAME: &str = "bip87";
/// Template name of BIP-45's purpose key `m/45'`. BIP-45: "m / 45' / *",
/// "Hardened derivation is used at this level", and "Only the hardened
/// purpose extended public key is shared". No coin or account level, so it
/// is searched once.
const BIP45_NAME: &str = "bip45";

impl CandidatePaths {
    /// Yields 4 single-sig templates × accounts, then 3 BIP-48 script types ×
    /// accounts, then BIP-87 × accounts, then BIP-45 `m/45'` once, then each
    /// `--add-path` in order (malformed substitutions skipped).
    pub fn iter(&self) -> impl Iterator<Item = CandidatePath> + '_ {
        let (lo, hi, network) = (self.lo, self.hi, self.network);
        let single = SINGLE_SIG.iter().flat_map(move |(template, name)| {
            (lo..hi).map(move |account| CandidatePath {
                template_name: (*name).to_string(),
                path: template.derivation_path(network, account),
                account: Some(account),
            })
        });
        let multi = MULTISIG.iter().flat_map(move |(script_type, name)| {
            (lo..hi).map(move |account| {
                let path_str =
                    MultisigPathFamily::Bip48.default_origin_path(network, account, *script_type);
                let path = DerivationPath::from_str(&path_str).expect(
                    "BIP-48 paths from default_origin_path are well-formed by construction",
                );
                CandidatePath {
                    template_name: (*name).to_string(),
                    path,
                    account: Some(account),
                }
            })
        });
        let bip87 = (lo..hi).map(move |account| CandidatePath {
            template_name: BIP87_NAME.to_string(),
            path: DerivationPath::from_str(
                &MultisigPathFamily::Bip87.default_origin_path(network, account, 0),
            )
            .expect("BIP-87 paths from default_origin_path are well-formed by construction"),
            account: Some(account),
        });
        let bip45 = std::iter::once(CandidatePath {
            template_name: BIP45_NAME.to_string(),
            path: DerivationPath::from_str("m/45'").expect("static path"),
            account: None,
        });
        let added = self.add_paths.iter().flat_map(move |tmpl| {
            let shape = add_path_shape(tmpl);
            let tmpl_name = tmpl.clone();
            let it: Box<dyn Iterator<Item = CandidatePath> + '_> = match shape {
                AddPathShape::Token { head, tail, quote } => {
                    Box::new((lo..hi).filter_map(move |account| {
                        // skip malformed; user will see no-match
                        DerivationPath::from_str(&substitute(&head, &tail, quote, account))
                            .ok()
                            .map(|path| CandidatePath {
                                template_name: tmpl_name.clone(),
                                path,
                                account: Some(account),
                            })
                    }))
                }
                AddPathShape::Literal => Box::new(
                    DerivationPath::from_str(tmpl)
                        .ok()
                        .map(|path| CandidatePath {
                            template_name: tmpl_name,
                            path,
                            account: None,
                        })
                        .into_iter(),
                ),
            };
            it
        });
        single.chain(multi).chain(bip87).chain(bip45).chain(added)
    }

    /// Number of candidates, equal to `self.iter().count()` without building
    /// them. A token `--add-path` counts the accounts in `lo..hi` whose
    /// substituted string parses (validity can depend on the account).
    pub fn len(&self) -> usize {
        let n = (self.hi.saturating_sub(self.lo)) as usize;
        // 4 single-sig + 3 BIP-48 + BIP-87 per account, plus BIP-45 once.
        let mut total = 8 * n + 1;
        for tmpl in &self.add_paths {
            total += match add_path_shape(tmpl) {
                AddPathShape::Literal => usize::from(DerivationPath::from_str(tmpl).is_ok()),
                AddPathShape::Token { head, tail, quote } => (self.lo..self.hi)
                    .filter(|&a| {
                        DerivationPath::from_str(&substitute(&head, &tail, quote, a)).is_ok()
                    })
                    .count(),
            };
        }
        total
    }
}

/// Build the deterministic candidate set per plan §3.2 step 4 (lazily; see
/// [`CandidatePaths`]). Infallible; precondition: the caller ran
/// [`check_account_window`].
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
) -> CandidatePaths {
    let lo = min_account;
    let hi = {
        let from_n = min_account.saturating_add(number_of_accounts);
        match max_account {
            Some(m) => from_n.max(m.saturating_add(1)),
            None => from_n,
        }
    };
    CandidatePaths {
        lo,
        hi,
        add_paths: add_paths.to_vec(),
        network,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_templates_at_account_0_default_range() {
        let candidates = build_candidate_paths(0, 20, None, &[], CliNetwork::Mainnet);
        // 8 families × 20 accounts + m/45' once = 161
        assert_eq!(candidates.len(), 161);
        let first = candidates.iter().next().unwrap();
        assert_eq!(first.template_name, "bip44");
        assert_eq!(first.path.to_string(), "44'/0'/0'");
    }

    #[test]
    fn min_account_5_n_3_yields_range_5_to_8() {
        let candidates = build_candidate_paths(5, 3, None, &[], CliNetwork::Mainnet);
        // 8 families × 3 accounts + m/45' once = 25
        assert_eq!(candidates.len(), 25);
        let bip84: Vec<CandidatePath> = candidates
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
        // max(0+5, 50+1) = 51; 8 × 51 + 1 = 409
        assert_eq!(candidates.len(), 8 * 51 + 1);
    }

    #[test]
    fn bip48_multisig_at_three_script_types() {
        let candidates = build_candidate_paths(0, 1, None, &[], CliNetwork::Mainnet);
        let mset: Vec<CandidatePath> = candidates
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
        let added: Vec<CandidatePath> = candidates
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
        let added: Vec<CandidatePath> = candidates
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
        let a_names: Vec<String> = a.iter().map(|c| c.template_name).collect();
        let b_names: Vec<String> = b.iter().map(|c| c.template_name).collect();
        assert_eq!(a_names, b_names);
        let a_paths: Vec<String> = a.iter().map(|c| c.path.to_string()).collect();
        let b_paths: Vec<String> = b.iter().map(|c| c.path.to_string()).collect();
        assert_eq!(a_paths, b_paths);
    }

    #[test]
    fn bip87_accounts_and_bip45_are_searched() {
        // Review r2 I-A: the descriptor-mode defaults (BIP-87) and the bare-P2SH
        // alternative (BIP-45 m/45') must be in the searched set.
        let c = build_candidate_paths(0, 2, None, &[], CliNetwork::Testnet);
        let got: Vec<(String, String, Option<u32>)> = c
            .iter()
            .filter(|p| p.template_name == "bip87" || p.template_name == "bip45")
            .map(|p| (p.template_name.clone(), p.path.to_string(), p.account))
            .collect();
        assert_eq!(
            got,
            vec![
                ("bip87".into(), "87'/1'/0'".into(), Some(0)),
                ("bip87".into(), "87'/1'/1'".into(), Some(1)),
                ("bip45".into(), "45'".into(), None),
            ]
        );
    }

    #[test]
    fn candidate_paths_len_matches_iter_count() {
        let garbage = ["m/1account'", "m/9'/account", "m/86'/0'/0'", "garbage"];
        let s = |v: &[&str]| v.iter().map(|x| x.to_string()).collect::<Vec<String>>();
        let cases: Vec<(u32, u32, Option<u32>, Vec<String>)> = vec![
            (0, 20, None, vec![]),
            (5, 3, None, vec![]),
            (0, 5, Some(50), vec![]),
            (0, 3, None, s(&garbage)),
            (2147483646, 2, None, s(&["m/1account'"])),
        ];
        for (min, n, max, add) in cases {
            let c = build_candidate_paths(min, n, max, &add, CliNetwork::Mainnet);
            assert_eq!(
                c.len(),
                c.iter().count(),
                "case ({min},{n},{max:?},{add:?})"
            );
        }
        let last = build_candidate_paths(
            2147483646,
            2,
            None,
            &s(&["m/1account'"]),
            CliNetwork::Mainnet,
        );
        assert_eq!(last.len(), 17);
    }

    #[test]
    fn candidate_paths_full_hardened_window_is_lazy() {
        let c = build_candidate_paths(0, 20, Some(2147483647), &[], CliNetwork::Mainnet);
        assert_eq!(c.len(), 8 * 2147483648usize + 1);
        let first: Vec<CandidatePath> = c.iter().take(3).collect();
        assert_eq!(first.len(), 3);
        for (i, cp) in first.iter().enumerate() {
            assert_eq!(cp.template_name, "bip44");
            assert_eq!(cp.account, Some(i as u32));
            assert_eq!(cp.path.to_string(), format!("44'/0'/{i}'"));
        }
    }

    #[test]
    fn check_account_window_boundaries() {
        for (min, n, max) in [
            (0u32, 20u32, None),
            (2147483646, 2, None),
            (2147483647, 1, None),
            (0, 0, Some(2147483647)),
            (0, u32::MAX >> 1, None),
        ] {
            assert!(
                check_account_window(min, n, max).is_ok(),
                "({min},{n},{max:?})"
            );
        }
        let w1 = "--min-account / --max-account / --number-of-accounts: the account window must stay below 2147483648 (2^31): BIP-32 hardened account indexes are 31 bits";
        for (min, n, max) in [
            (2147483648u32, 0u32, None),
            (2147483647, 2, None),
            (0, 20, Some(2147483648)),
            (1, 2147483648, None),
            (0, u32::MAX, None),
        ] {
            let e = check_account_window(min, n, max).unwrap_err();
            assert!(
                matches!(e, ToolkitError::BadInput(_)),
                "({min},{n},{max:?})"
            );
            assert_eq!(e.to_string(), format!("error: {w1}"), "({min},{n},{max:?})");
        }
    }
}
