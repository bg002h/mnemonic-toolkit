//! Spec A unit 0: the shared vector table (`design/network-rule/vectors.tsv`,
//! vendored as `tests/data/network_rule_vectors.tsv`) run through the rule
//! helper, the hash pin, exit 8 and the N10 machine line.
//!
//! Source model of a row (plan §2.5, reading A7): `key=c` is a key `key @i`
//! (prefix xpub/tpub); `coin=c` right after a `key=` is that key's own origin
//! coin; a `coin=` with no key before it is a coin with no key (`key @j`'s
//! path); `path=other` is a key path of no N1 shape (no source); `hrp=`/`b58=`
//! an address; `wif=` a WIF; `stated=<n>` a stated exact name with no keys;
//! `stated2=c` a two-valued statement whose keys are every `key=`/`coin=` after
//! it; `reads=kept` / `reads=seed` / `out=secret` the run's flags.

use mnemonic_toolkit::network_rule::{
    evaluate, machine_line, AddrNet, Class, CliNetwork, Device, Hrp, Input, KeyCoin, KeySource,
    LineClass, PathWho, RunNet, SectionKey, Source, StatedNet, Thing, Verdict, Who,
    EXIT_NETWORK_MISMATCH,
};
use sha2::{Digest, Sha256};

const VENDORED: &str = include_str!("data/network_rule_vectors.tsv");
const DESIGN: &str = include_str!("../../../design/network-rule/vectors.tsv");
/// sha256 of vectors.tsv at d72a18b1 (identical at the spec's rev 36c1f155).
const PINNED_SHA256: &str = "7c076027dc73e8a31f794f9c111cc5232f619a3893dae582996179eabb9f9c30";

fn sha256_hex(s: &str) -> String {
    hex::encode(Sha256::digest(s.as_bytes()))
}

#[test]
fn nr0_vectors_tsv_hash_pinned() {
    assert_eq!(sha256_hex(VENDORED), PINNED_SHA256);
    assert_eq!(sha256_hex(DESIGN), PINNED_SHA256);
}

fn class(c: &str) -> Class {
    match c {
        "main" => Class::Main,
        "test" => Class::Test,
        o => panic!("class {o}"),
    }
}

fn pfx(c: Class) -> &'static str {
    match c {
        Class::Main => "xpub",
        Class::Test => "tpub",
    }
}

fn exact(n: &str) -> CliNetwork {
    match n {
        "main" | "mainnet" => CliNetwork::Mainnet,
        "test" | "testnet" | "testnet4" => CliNetwork::Testnet,
        "signet" => CliNetwork::Signet,
        "regtest" => CliNetwork::Regtest,
        o => panic!("network {o}"),
    }
}

fn coin(c: &str, i: usize, path_who: PathWho) -> KeyCoin {
    let coin: u32 = c.parse().expect("coin");
    KeyCoin {
        path_who,
        path: format!("m/84h/{coin}h/{i}h"),
        coin,
    }
}

/// One row's sources and flags as an `Input`.
fn row_input(run: &str, sources: &str, accept: bool) -> Input {
    let run = if run == "none" {
        RunNet::new(CliNetwork::Mainnet, false)
    } else {
        RunNet::new(exact(run), true)
    };
    let mut input = Input::new(run, vec![], Thing::Descriptor);
    input.reads_kept = false;
    input.accept_coin_type = accept;
    let mut out: Vec<Source> = Vec::new();
    // Index of the open stated2 source whose keys follow it.
    let mut stated2: Option<usize> = None;
    let mut last_was_key = false;
    let mut nkey = 0usize;
    for tok in sources.split(';').filter(|t| *t != "-") {
        let (k, v) = tok.split_once('=').expect("k=v");
        let was_key = last_was_key;
        last_was_key = false;
        match k {
            "key" => {
                let c = class(v);
                let key = KeySource {
                    who: Who::key(nkey),
                    pfx: pfx(c).into(),
                    class: c,
                    coins: vec![],
                };
                nkey += 1;
                match stated2 {
                    Some(si) => {
                        if let Source::Stated { keys, .. } = &mut out[si] {
                            keys.push(SectionKey {
                                section: "bip84".into(),
                                key: KeySource {
                                    who: Who::plain("bip84"),
                                    ..key
                                },
                            });
                        }
                    }
                    None => out.push(Source::Key(key)),
                }
                last_was_key = true;
            }
            "coin" => {
                if was_key {
                    let target = match stated2 {
                        Some(si) => match &mut out[si] {
                            Source::Stated { keys, .. } => &mut keys.last_mut().unwrap().key,
                            _ => unreachable!(),
                        },
                        None => match out.last_mut() {
                            Some(Source::Key(k)) => k,
                            _ => unreachable!(),
                        },
                    };
                    target.coins.push(coin(v, nkey - 1, PathWho::OwnKey));
                    last_was_key = true;
                } else {
                    let j = nkey;
                    nkey += 1;
                    out.push(Source::Coin(coin(v, j, PathWho::Named(Who::key(j)))));
                }
            }
            "path" => {
                // A path of no N1 shape: no source (and no key).
                assert_eq!(v, "other");
            }
            "hrp" => out.push(Source::Address {
                who: "the address".into(),
                address: format!("{v}1qexample"),
                net: AddrNet::Hrp(match v {
                    "bc" => Hrp::Bc,
                    "tb" => Hrp::Tb,
                    "bcrt" => Hrp::Bcrt,
                    o => panic!("hrp {o}"),
                }),
            }),
            "b58" => out.push(Source::Address {
                who: "the address".into(),
                address: "1example".into(),
                net: AddrNet::B58(class(v)),
            }),
            "wif" => {
                let c = class(v);
                out.push(Source::Wif {
                    class: c,
                    version: if c == Class::Main { 128 } else { 239 },
                })
            }
            "stated" => out.push(Source::Stated {
                file: "f.json".into(),
                text: v.into(),
                net: StatedNet::Exact(exact(v)),
                device: Device::Signer,
                keys: vec![],
            }),
            "stated2" => {
                let c = class(v);
                out.push(Source::Stated {
                    file: "cc.json".into(),
                    text: match c {
                        Class::Main => "chain BTC (mainnet)".into(),
                        Class::Test => "chain XTN (a test network)".into(),
                    },
                    net: StatedNet::Class(c),
                    device: Device::Coldcard,
                    keys: vec![],
                });
                stated2 = Some(out.len() - 1);
            }
            "reads" => match v {
                "kept" => input.reads_kept = true,
                "seed" => input.seed_only = true,
                o => panic!("reads {o}"),
            },
            "out" => {
                assert_eq!(v, "secret");
                input.secret_out = true;
            }
            o => panic!("source kind {o} in a toolkit row"),
        }
    }
    input.sources = out;
    input
}

#[test]
fn nr0_vectors_tsv_every_row() {
    let mut lines = VENDORED.lines();
    assert_eq!(
        lines.next(),
        Some("id\ttool\thas_network_flag\trun_network\tsources\toverride\tclass\ttext\texit")
    );
    let mut ran = Vec::new();
    for row in lines {
        let f: Vec<&str> = row.split('\t').collect();
        assert_eq!(f.len(), 9, "{row}");
        let (id, tool, run, sources, ovr, want_class, want_text, want_exit) =
            (f[0], f[1], f[3], f[4], f[5], f[6], f[7], f[8]);
        if tool != "any" && tool != "mnemonic" {
            continue;
        }
        let input = row_input(run, sources, ovr == "1");
        let verdict = evaluate(&input);
        let ids: Vec<&str> = verdict.lines().iter().map(|l| l.text_id).collect();
        let got_text = if ids.is_empty() {
            "-".to_string()
        } else {
            ids.join(",")
        };
        assert_eq!(got_text, want_text, "{id}: text ids");
        let exit = match verdict {
            Verdict::Refuse(_) => EXIT_NETWORK_MISMATCH,
            Verdict::Proceed(_) => 0,
        };
        assert_eq!(exit.to_string(), want_exit, "{id}: exit");
        let has = |c: LineClass| verdict.lines().iter().any(|l| l.class == c);
        let refusal = matches!(verdict, Verdict::Refuse(_));
        let got_class = if refusal && has(LineClass::Hard) && has(LineClass::Soft) {
            "hard+soft"
        } else if refusal && has(LineClass::Hard) {
            "hard"
        } else if has(LineClass::Soft) {
            "soft"
        } else if ids.iter().any(|t| *t == "TX-N" || *t == "TX-T") {
            "no-source"
        } else {
            "none"
        };
        assert_eq!(got_class, want_class, "{id}: class");
        ran.push(id.to_string());
    }
    // V01-V25, V28-V33, V42-V46, V60.
    assert_eq!(ran.len(), 37, "rows run: {ran:?}");
}

#[test]
fn nr0_exit_code_8_is_network_mismatch() {
    // N9: the one exit code of a network refusal. (The variant's own
    // `exit_code()` is pinned by the same-named unit test in `error.rs`; the
    // arm units pin exit 8 through the CLI.)
    assert_eq!(EXIT_NETWORK_MISMATCH, 8);
    assert!(matches!(
        evaluate(&row_input("none", "key=test", false)),
        Verdict::Refuse(_)
    ));
}

/// No space outside a JSON string.
fn no_space_outside_strings(s: &str) -> bool {
    let mut in_str = false;
    let mut esc = false;
    for ch in s.chars() {
        if in_str {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == '"' {
                in_str = false;
            }
        } else if ch == '"' {
            in_str = true;
        } else if ch.is_whitespace() {
            return false;
        }
    }
    true
}

#[test]
fn nr0_machine_line_shape() {
    // Spec A §5: R1.4's line, byte for byte.
    let r14 = row_input("main", "key=main;coin=1", false);
    assert_eq!(
        machine_line(evaluate(&r14).lines()),
        r#"{"kind":"network_mismatch","class":"soft","contradictions":[{"class":"soft","text":"TX-S","a":{"what":"key @0's path m/84h/1h/0h","network":"test"},"b":{"what":"key @0","network":"main"}}]}"#
    );
    // R1.14: two contradictions, TX-H2 then TX-H1, top class hard.
    let r114 = row_input("none", "key=main;key=test", false);
    let m = machine_line(evaluate(&r114).lines());
    assert!(
        m.starts_with(r#"{"kind":"network_mismatch","class":"hard","contradictions":[{"class":"hard","text":"TX-H2","#),
        "{m}"
    );
    assert!(m.contains(r#"},{"class":"hard","text":"TX-H1","#), "{m}");
    assert!(no_space_outside_strings(&m), "{m}");
}
