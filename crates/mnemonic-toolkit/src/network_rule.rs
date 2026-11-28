//! The network-contradiction rule (SPEC_network_rule.md, unit 0).
//!
//! One function, [`evaluate`], takes the run network (given or default), the
//! sources an input carries and the run's flags, and returns what to print:
//! the refusal lines (exit 8) or the warnings of a run that goes on. The texts
//! are SPEC §3's, verbatim, built in [`texts`]; the order, merging and drops
//! are N8's. Every verb that reads a source builds an [`Input`] and calls
//! [`enforce`] before its first stdout or file write.
//!
//! The module is `pub` (unlike most of the crate) so that the integration test
//! `tests/cli_network_rule_vectors.rs` can run the shared vector table through
//! the helper (Spec A unit 0: "the helper is pub in network.rs"); `network.rs`
//! re-exports it.

use std::cell::{Cell, RefCell};
use std::io::Write;

pub use crate::network::CliNetwork;

/// N7: the override flag, one name everywhere.
pub const ACCEPT_COIN_TYPE_FLAG: &str = "--accept-coin-type-mismatch";
/// N9: the exit code of every network refusal.
pub const EXIT_NETWORK_MISMATCH: u8 = 8;
/// N10: the environment variable that turns the machine line on.
pub const MACHINE_ENV: &str = "MNEMONIC_NETWORK_JSON";

/// N3 class grain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Main,
    Test,
}

impl Class {
    /// `mainnet` / `test-network`, as the texts write a class.
    pub fn adj(self) -> &'static str {
        match self {
            Class::Main => "mainnet",
            Class::Test => "test-network",
        }
    }
    /// The coin type of the class's paths (0 or 1).
    pub fn coin(self) -> u32 {
        match self {
            Class::Main => 0,
            Class::Test => 1,
        }
    }
    /// N10 network name at class grain.
    pub fn machine(self) -> &'static str {
        match self {
            Class::Main => "main",
            Class::Test => "test",
        }
    }
    pub fn of_coin(coin: u32) -> Class {
        if coin == 0 {
            Class::Main
        } else {
            Class::Test
        }
    }
    pub fn of_kind(kind: bitcoin::NetworkKind) -> Class {
        match kind {
            bitcoin::NetworkKind::Main => Class::Main,
            bitcoin::NetworkKind::Test => Class::Test,
        }
    }
    pub fn of_net(net: CliNetwork) -> Class {
        match net {
            CliNetwork::Mainnet => Class::Main,
            _ => Class::Test,
        }
    }
}

/// A bech32/bech32m HRP (N3 HRP grain).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hrp {
    Bc,
    Tb,
    Bcrt,
}

/// An address's network: its HRP, or a base58 version's class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrNet {
    Hrp(Hrp),
    B58(Class),
}

impl AddrNet {
    pub fn class(self) -> Class {
        match self {
            AddrNet::Hrp(Hrp::Bc) | AddrNet::B58(Class::Main) => Class::Main,
            _ => Class::Test,
        }
    }
    fn agrees(self, run: CliNetwork) -> bool {
        match self {
            AddrNet::Hrp(Hrp::Bc) => run == CliNetwork::Mainnet,
            AddrNet::Hrp(Hrp::Tb) => matches!(run, CliNetwork::Testnet | CliNetwork::Signet),
            AddrNet::Hrp(Hrp::Bcrt) => run == CliNetwork::Regtest,
            AddrNet::B58(c) => c == Class::of_net(run),
        }
    }
    fn machine(self) -> &'static str {
        match self {
            AddrNet::Hrp(Hrp::Bc) => "main",
            AddrNet::Hrp(Hrp::Tb) => "test",
            AddrNet::Hrp(Hrp::Bcrt) => "regtest",
            AddrNet::B58(c) => c.machine(),
        }
    }
    /// The run network at this source's grain (N10).
    fn run_machine(self, run: CliNetwork) -> &'static str {
        match self {
            AddrNet::Hrp(_) => match run {
                CliNetwork::Mainnet => "main",
                CliNetwork::Regtest => "regtest",
                _ => "test",
            },
            AddrNet::B58(_) => Class::of_net(run).machine(),
        }
    }
    /// The address's network from its text (HRP or base58 version), if it parses.
    pub fn of_address(address: &str) -> Option<AddrNet> {
        let lower = address.to_ascii_lowercase();
        if lower.starts_with("bcrt1") {
            return Some(AddrNet::Hrp(Hrp::Bcrt));
        }
        if lower.starts_with("bc1") {
            return Some(AddrNet::Hrp(Hrp::Bc));
        }
        if lower.starts_with("tb1") {
            return Some(AddrNet::Hrp(Hrp::Tb));
        }
        let data = bitcoin::base58::decode_check(address).ok()?;
        match data.first()? {
            0 | 5 => Some(AddrNet::B58(Class::Main)),
            111 | 196 => Some(AddrNet::B58(Class::Test)),
            _ => None,
        }
    }
}

/// A network an artifact states (N3: class grain or exact name).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatedNet {
    Class(Class),
    Exact(CliNetwork),
}

impl StatedNet {
    fn class(self) -> Class {
        match self {
            StatedNet::Class(c) => c,
            StatedNet::Exact(n) => Class::of_net(n),
        }
    }
    fn agrees(self, run: CliNetwork) -> bool {
        match self {
            StatedNet::Class(c) => c == Class::of_net(run),
            StatedNet::Exact(n) => n == run,
        }
    }
    fn machine(self) -> &'static str {
        match self {
            StatedNet::Class(c) => c.machine(),
            StatedNet::Exact(n) => exact_machine(n),
        }
    }
    fn run_machine(self, run: CliNetwork) -> &'static str {
        match self {
            StatedNet::Class(_) => Class::of_net(run).machine(),
            StatedNet::Exact(_) => exact_machine(run),
        }
    }
}

fn exact_machine(n: CliNetwork) -> &'static str {
    match n {
        CliNetwork::Mainnet => "main",
        CliNetwork::Testnet => "testnet",
        CliNetwork::Signet => "signet",
        CliNetwork::Regtest => "regtest",
    }
}

/// An indexed name with a plural form: `key @0` / `keys @0 and @1`,
/// `jade key @0` / `jade keys @0, @1 and @2`, `--cosigner @0's mk1 card` /
/// `--cosigner @0's and @1's mk1 cards` (Spec A §3 "<who> by arm").
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    pub pre: String,
    pub pre_plural: String,
    pub post: String,
    pub tail: String,
    pub tail_plural: String,
}

/// A source's name in the texts (`<who>`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Who {
    pub name: String,
    pub family: Option<(Family, usize)>,
}

impl Who {
    /// A name with no plural form of its own (`--target-xpub`, `--from mk1`).
    pub fn plain(name: impl Into<String>) -> Who {
        Who {
            name: name.into(),
            family: None,
        }
    }
    fn indexed(f: Family, i: usize) -> Who {
        Who {
            name: format!("{}@{i}{}{}", f.pre, f.post, f.tail),
            family: Some((f, i)),
        }
    }
    /// `<stem> @i` with plural `<stem>s @0 and @1` (`key`, `jade key`,
    /// `coldcard-multisig key`, `mk1 card`).
    pub fn stem(stem: &str, i: usize) -> Who {
        Who::indexed(
            Family {
                pre: format!("{stem} "),
                pre_plural: format!("{stem}s "),
                post: String::new(),
                tail: String::new(),
                tail_plural: String::new(),
            },
            i,
        )
    }
    /// `key @i`.
    pub fn key(i: usize) -> Who {
        Who::stem("key", i)
    }
    /// `--cosigner @i's <what>` with plural `--cosigner @0's and @1's <what>s`.
    pub fn cosigner(i: usize, what: &str) -> Who {
        Who::indexed(
            Family {
                pre: "--cosigner ".into(),
                pre_plural: "--cosigner ".into(),
                post: "'s".into(),
                tail: format!(" {what}"),
                tail_plural: format!(" {what}s"),
            },
            i,
        )
    }
}

/// `a`, `a and b`, `a, b and c`.
fn join_and(items: &[String]) -> String {
    match items.len() {
        0 => String::new(),
        1 => items[0].clone(),
        n => format!("{} and {}", items[..n - 1].join(", "), items[n - 1]),
    }
}

/// The plural `<who>` of several sources of one contradiction (N8).
fn plural(whos: &[&Who]) -> String {
    if whos.len() == 1 {
        return whos[0].name.clone();
    }
    if let Some((f0, _)) = &whos[0].family {
        if whos
            .iter()
            .all(|w| matches!(&w.family, Some((f, _)) if f == f0))
        {
            let items: Vec<String> = whos
                .iter()
                .map(|w| format!("@{}{}", w.family.as_ref().expect("checked").1, f0.post))
                .collect();
            return format!("{}{}{}", f0.pre_plural, join_and(&items), f0.tail_plural);
        }
    }
    let names: Vec<String> = whos.iter().map(|w| w.name.clone()).collect();
    join_and(&names)
}

/// A coin type read from a path (N1 SOFT shape only), with how its `<who>`
/// renders.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyCoin {
    pub path_who: PathWho,
    /// `m/84h/1h/0h`: hardened steps written with `h`.
    pub path: String,
    pub coin: u32,
}

/// How "<who>'s path <path>" renders for a coin.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathWho {
    /// The origin of the key that carries the coin: "<key who>'s path <path>".
    OwnKey,
    /// Another name (an md1 slot): "key @0's path <path>".
    Named(Who),
    /// A path flag: "the path <path> given with --path".
    Flag(String),
}

/// An extended key (or anything with key version bytes).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeySource {
    pub who: Who,
    /// The prefix as typed (`xpub`, `tpub`, `zpub`, `vpub`, ...).
    pub pfx: String,
    pub class: Class,
    /// Coins compared with THIS key's class (B2).
    pub coins: Vec<KeyCoin>,
}

impl KeySource {
    /// A key from its text as typed; `None` when the prefix names no class.
    pub fn from_text(who: Who, text: &str, coins: Vec<KeyCoin>) -> Option<KeySource> {
        let pfx: String = text.chars().take(4).collect();
        let class = class_of_prefix(&pfx)?;
        Some(KeySource {
            who,
            pfx,
            class,
            coins,
        })
    }
}

/// The class an extended-key prefix names (SLIP-132 included), if any.
pub fn class_of_prefix(pfx: &str) -> Option<Class> {
    match pfx {
        "xpub" | "ypub" | "zpub" | "Ypub" | "Zpub" | "xprv" | "yprv" | "zprv" | "Yprv" | "Zprv" => {
            Some(Class::Main)
        }
        "tpub" | "upub" | "vpub" | "Upub" | "Vpub" | "tprv" | "uprv" | "vprv" | "Uprv" | "Vprv" => {
            Some(Class::Test)
        }
        _ => None,
    }
}

/// A key of a file that states a network (B3: it appears only here).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SectionKey {
    /// TX-H4C `<section>` (`bip84`, `bundle.descriptor`).
    pub section: String,
    pub key: KeySource,
}

/// TX-H4C `<device>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Device {
    Coldcard,
    Signer,
}

/// One source of a run's input (N1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    Key(KeySource),
    /// A coin with no key version: compared with the run only (B2). Its
    /// `path_who` is `Named` or `Flag`.
    Coin(KeyCoin),
    Address {
        who: String,
        address: String,
        net: AddrNet,
    },
    Wif {
        class: Class,
        version: u8,
    },
    Stated {
        /// `<file>`: the path as given.
        file: String,
        /// `<stated>`: `testnet`, `chain BTC (mainnet)`, ...
        text: String,
        net: StatedNet,
        device: Device,
        keys: Vec<SectionKey>,
    },
}

/// TX-T `<thing>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Thing {
    Card,
    Key,
    Descriptor,
}

impl Thing {
    fn word(self) -> &'static str {
        match self {
            Thing::Card => "card",
            Thing::Key => "key",
            Thing::Descriptor => "descriptor",
        }
    }
}

/// The run network (N2): the flag's, or mainnet; `given` drives `<run>`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunNet {
    pub net: CliNetwork,
    pub given: bool,
}

impl RunNet {
    pub fn new(net: CliNetwork, given: bool) -> RunNet {
        RunNet { net, given }
    }
    /// From an `Option<CliNetwork>` flag (None = mainnet, not given).
    pub fn from_opt(net: Option<CliNetwork>) -> RunNet {
        RunNet {
            net: net.unwrap_or(CliNetwork::Mainnet),
            given: net.is_some(),
        }
    }
    /// From a flag with a clap default: `given` is what the command line said.
    pub fn from_defaulted(net: CliNetwork) -> RunNet {
        RunNet {
            net,
            given: network_given_or(net != CliNetwork::Mainnet),
        }
    }
    fn class(self) -> Class {
        Class::of_net(self.net)
    }
    fn name(self) -> &'static str {
        self.net.human_name()
    }
    /// `<run>`.
    fn phrase(self) -> String {
        if self.given {
            format!("--network is {}", self.name())
        } else {
            format!("this run is on {} (no --network given)", self.name())
        }
    }
}

/// What a run read and may print.
#[derive(Debug, Clone)]
pub struct Input {
    pub run: RunNet,
    /// In input order (N8 "by input order").
    pub sources: Vec<Source>,
    /// N6: the run read a kept artifact (or a key with no network).
    pub reads_kept: bool,
    /// N6 exemption: the run only derives from a seed typed now.
    pub seed_only: bool,
    /// N12: secret input and `PrivateKeyMaterial` on stdout.
    pub secret_out: bool,
    /// N7: `--accept-coin-type-mismatch` given.
    pub accept_coin_type: bool,
    pub thing: Thing,
}

impl Input {
    pub fn new(run: RunNet, sources: Vec<Source>, thing: Thing) -> Input {
        Input {
            run,
            sources,
            reads_kept: true,
            seed_only: false,
            secret_out: false,
            accept_coin_type: false,
            thing,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineClass {
    Hard,
    Soft,
    Notice,
}

/// N10 `a` / `b`.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct End {
    pub what: String,
    pub network: &'static str,
}

fn end(what: impl Into<String>, network: &'static str) -> End {
    End {
        what: what.into(),
        network,
    }
}

/// One printed line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub class: LineClass,
    pub text_id: &'static str,
    pub text: String,
    pub a: End,
    pub b: End,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    /// Exit 8: the lines, in print order.
    Refuse(Vec<Line>),
    /// The run goes on: warnings to print on stderr (TX-S-OK, TX-N, TX-T, TX-R).
    Proceed(Vec<Line>),
}

impl Verdict {
    pub fn lines(&self) -> &[Line] {
        match self {
            Verdict::Refuse(l) | Verdict::Proceed(l) => l,
        }
    }
}

/// The coin type of an origin path, read only from the N1 SOFT shape:
/// `purpose'/coin'/...` with purpose in {44, 48, 49, 84, 86, 87, 352, 270028}
/// and coin in {0, 1}, both hardened. Returns the path text and the coin.
pub fn coin_of_path(p: &bitcoin::bip32::DerivationPath) -> Option<(String, u32)> {
    use bitcoin::bip32::ChildNumber;
    let steps: Vec<ChildNumber> = p.into_iter().copied().collect();
    let (purpose, coin) = match (steps.first(), steps.get(1)) {
        (
            Some(ChildNumber::Hardened { index: purpose }),
            Some(ChildNumber::Hardened { index: coin }),
        ) => (*purpose, *coin),
        _ => return None,
    };
    if !matches!(purpose, 44 | 48 | 49 | 84 | 86 | 87 | 352 | 270028) || coin > 1 {
        return None;
    }
    Some((path_text(p), coin))
}

/// `m/` and the steps, hardened steps written with `h` (Spec A §3 `<path>`).
pub fn path_text(p: &bitcoin::bip32::DerivationPath) -> String {
    use bitcoin::bip32::ChildNumber;
    let mut s = String::from("m");
    for c in p {
        match c {
            ChildNumber::Hardened { index } => s.push_str(&format!("/{index}h")),
            ChildNumber::Normal { index } => s.push_str(&format!("/{index}")),
        }
    }
    s
}

/// A `KeyCoin` for a path, when it has an N1 coin type.
pub fn key_coin(path_who: PathWho, p: &bitcoin::bip32::DerivationPath) -> Option<KeyCoin> {
    coin_of_path(p).map(|(path, coin)| KeyCoin {
        path_who,
        path,
        coin,
    })
}

/// The sources of a parsed descriptor: one per key in BIP-388 keys_info order
/// (first appearance, deduplicated), `who` = `key @i`. An extended key is a
/// `Key` (prefix as typed in `raw`), carrying its origin coin; a single key
/// yields only a `Coin` (its origin's coin, if any). Paths the input does not
/// carry are never a source (B1).
pub fn descriptor_sources(
    raw: &str,
    d: &miniscript::Descriptor<miniscript::DescriptorPublicKey>,
) -> Vec<Source> {
    descriptor_sources_named(raw, d, Who::key)
}

/// As [`descriptor_sources`], with the caller's `<who>` per key index.
pub fn descriptor_sources_named(
    raw: &str,
    d: &miniscript::Descriptor<miniscript::DescriptorPublicKey>,
    who: impl Fn(usize) -> Who,
) -> Vec<Source> {
    descriptor_keys(raw, d)
        .into_iter()
        .enumerate()
        .filter_map(|(i, k)| desc_key_source(k, who(i)))
        .collect()
}

/// One key of a descriptor, as the rule reads it.
#[derive(Debug, Clone)]
pub struct DescKey {
    /// `Some((pfx, class))` for an extended key.
    pub xkey: Option<(String, Class)>,
    pub origin: Option<bitcoin::bip32::DerivationPath>,
}

/// The keys of a descriptor in keys_info order.
pub fn descriptor_keys(
    raw: &str,
    d: &miniscript::Descriptor<miniscript::DescriptorPublicKey>,
) -> Vec<DescKey> {
    use miniscript::descriptor::DescriptorPublicKey as K;
    use miniscript::ForEachKey;
    let prefixes = raw_prefixes(raw);
    let mut seen: Vec<String> = Vec::new();
    let mut out = Vec::new();
    d.for_each_key(|k| {
        let (id, xkey, origin) = match k {
            K::Single(s) => (
                format!("{:?}{:?}", s.origin, s.key),
                None,
                s.origin.as_ref().map(|o| o.1.clone()),
            ),
            K::XPub(x) => (
                format!("{:?}{}", x.origin, x.xkey),
                Some(x.xkey),
                x.origin.as_ref().map(|o| o.1.clone()),
            ),
            K::MultiXPub(x) => (
                format!("{:?}{}", x.origin, x.xkey),
                Some(x.xkey),
                x.origin.as_ref().map(|o| o.1.clone()),
            ),
        };
        if !seen.contains(&id) {
            seen.push(id);
            let xkey = xkey.map(|x| {
                let class = Class::of_kind(x.network);
                let pfx = prefixes
                    .iter()
                    .find(|(data, _)| *data == key_data(&x))
                    .map(|(_, p)| p.clone())
                    .unwrap_or_else(|| default_pfx(class).to_string());
                (pfx, class)
            });
            out.push(DescKey { xkey, origin });
        }
        true
    });
    out
}

fn desc_key_source(k: DescKey, who: Who) -> Option<Source> {
    match k.xkey {
        Some((pfx, class)) => {
            let coins = k
                .origin
                .as_ref()
                .and_then(|p| key_coin(PathWho::OwnKey, p))
                .into_iter()
                .collect();
            Some(Source::Key(KeySource {
                who,
                pfx,
                class,
                coins,
            }))
        }
        None => k
            .origin
            .as_ref()
            .and_then(|p| key_coin(PathWho::Named(who), p))
            .map(Source::Coin),
    }
}

fn default_pfx(class: Class) -> &'static str {
    match class {
        Class::Main => "xpub",
        Class::Test => "tpub",
    }
}

fn key_data(x: &bitcoin::bip32::Xpub) -> Vec<u8> {
    let mut v = x.public_key.serialize().to_vec();
    v.extend_from_slice(x.chain_code.as_bytes());
    v
}

/// Every extended public key token in `raw`, with its key data and prefix as typed.
fn raw_prefixes(raw: &str) -> Vec<(Vec<u8>, String)> {
    let mut out = Vec::new();
    let bytes: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i + 4 <= bytes.len() {
        let pfx: String = bytes[i..i + 4].iter().collect();
        if class_of_prefix(&pfx).is_some() && pfx.ends_with("pub") {
            let mut j = i;
            while j < bytes.len() && bytes[j].is_ascii_alphanumeric() {
                j += 1;
            }
            let tok: String = bytes[i..j].iter().collect();
            if let Ok(data) = bitcoin::base58::decode_check(&tok) {
                if data.len() == 78 {
                    let mut v = data[45..78].to_vec();
                    v.extend_from_slice(&data[13..45]);
                    out.push((v, pfx));
                }
            }
            i = j.max(i + 1);
        } else {
            i += 1;
        }
    }
    out
}

/// The texts of Spec A §3 (verbatim). `<P>` is `error:`; warnings start `warning:`.
pub mod texts {
    use super::{Class, RunNet};

    pub const P: &str = "error:";

    /// TX-H1's two remedy sentences, the first one dropped when `short`.
    pub fn h1_remedy(key: Class, run: RunNet, short: bool) -> String {
        let (first, second) = match key {
            Class::Test => (
                "For a test wallet, rerun with --network signet, --network testnet or --network regtest.".to_string(),
                "For a mainnet wallet, export the key again with the signer set to mainnet.".to_string(),
            ),
            Class::Main => (
                "For a mainnet wallet, rerun with --network mainnet.".to_string(),
                format!(
                    "For a {} wallet, export the key again with the signer set to a test network.",
                    run.name()
                ),
            ),
        };
        if short {
            second
        } else {
            format!("{first} {second}")
        }
    }

    /// TX-H1. `who` is already plural when several keys merge (then `plural`).
    pub fn h1(who: &str, plural: bool, pfx: &str, key: Class, run: RunNet, short: bool) -> String {
        let (verb, noun) = if plural {
            ("are", "keys")
        } else {
            ("is", "key")
        };
        let article = if plural { "" } else { "a " };
        format!(
            "{P} network mismatch: {who} {verb} {article}{} {noun} ({pfx}), but {}. Nothing was written. {}",
            key.adj(),
            run.phrase(),
            h1_remedy(key, run, short)
        )
    }

    /// TX-H2.
    pub fn h2(who1: &str, pfx1: &str, who2: &str, pfx2: &str) -> String {
        format!(
            "{P} network mismatch: {who1} is a mainnet key ({pfx1}), but {who2} is a test-network key ({pfx2}). Nothing was written. Every key of one wallet is for one network: export the odd key again."
        )
    }

    /// TX-H3 `<kind>` and `<remedy>`.
    pub fn h3_kind(net: super::AddrNet) -> (&'static str, &'static str) {
        use super::{AddrNet, Hrp};
        match net {
            AddrNet::Hrp(Hrp::Bc) | AddrNet::B58(Class::Main) => ("mainnet", "--network mainnet"),
            AddrNet::Hrp(Hrp::Bcrt) => ("regtest", "--network regtest"),
            AddrNet::Hrp(Hrp::Tb) => ("signet or testnet", "--network signet or --network testnet"),
            AddrNet::B58(Class::Test) => (
                "test-network",
                "--network signet, --network testnet or --network regtest",
            ),
        }
    }

    /// TX-H3; `short` drops `Rerun with <remedy>, or ` (N8: the address and a key disagree).
    pub fn h3(who: &str, address: &str, net: super::AddrNet, run: RunNet, short: bool) -> String {
        let (kind, remedy) = h3_kind(net);
        let tail = if short {
            format!("Give an address of {}.", run.name())
        } else {
            format!("Rerun with {remedy}, or give an address of {}.", run.name())
        };
        format!(
            "{P} network mismatch: {who} {address} is a {kind} address, but {}. Nothing was written. {tail}",
            run.phrase()
        )
    }

    /// TX-H4 `<remedy>`.
    pub fn h4_remedy(net: super::StatedNet) -> String {
        match net {
            super::StatedNet::Exact(n) => format!("--network {}", n.human_name()),
            super::StatedNet::Class(Class::Main) => "--network mainnet".into(),
            super::StatedNet::Class(Class::Test) => {
                "--network signet, --network testnet or --network regtest".into()
            }
        }
    }

    /// TX-H4.
    pub fn h4(file: &str, stated: &str, net: super::StatedNet, run: RunNet) -> String {
        format!(
            "{P} network mismatch: {file} says network {stated}, but {}. Nothing was written. Rerun with {}, or use the file made for {}.",
            run.phrase(),
            h4_remedy(net),
            run.name()
        )
    }

    /// TX-H4C; `sections` already joined, `plural` when several keys.
    pub fn h4c(
        file: &str,
        stated: &str,
        sections: &str,
        plural: bool,
        key: Class,
        pfx: &str,
        device: &str,
    ) -> String {
        let keyp = if plural {
            format!("{sections} keys are {} keys", key.adj())
        } else {
            format!("{sections} key is a {} key", key.adj())
        };
        format!(
            "{P} network mismatch: {file} says {stated}, but its {keyp} ({pfx}). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the {device} with its network set for the wallet's network."
        )
    }

    /// TX-H5.
    pub fn h5(wif: Class, version: u8, run: RunNet) -> String {
        format!(
            "{P} network mismatch: the WIF key is a {} key (version byte {version}), but {}. Nothing was written. {}",
            wif.adj(),
            run.phrase(),
            h1_remedy(wif, run, false)
        )
    }

    /// The flag sentence of a SOFT line (N8 drops it behind a HARD line).
    pub fn s_flag(subject: &str, run: RunNet) -> String {
        format!(
            " To use {subject} as it is, add --accept-coin-type-mismatch: the output is then for {} and this warning is printed every time.",
            run.name()
        )
    }

    /// TX-S key form. `path_phrase` is "<who>'s path <path> is a <pclass> path"
    /// (or its plural); `key_phrase` "<who> is a <kclass> key" (or plural).
    pub fn s_key(
        path_phrase: &str,
        pclass: Class,
        key_phrase: &str,
        kclass: Class,
        pfx: &str,
        card_path: bool,
        run: RunNet,
        flag: bool,
    ) -> String {
        let remedy = if card_path {
            format!(
                "If the path is a mistake, make the md1 card again at the {} path (coin type {}h).",
                kclass.adj(),
                kclass.coin()
            )
        } else {
            format!(
                "If the path is a mistake, export the key again at the {} path (coin type {}h).",
                kclass.adj(),
                kclass.coin()
            )
        };
        let flag = if flag {
            s_flag("this key", run)
        } else {
            String::new()
        };
        format!(
            "{P} network mismatch (coin type): {path_phrase} (coin type {}h), but {key_phrase} ({pfx}). Nothing was written. {remedy}{flag}",
            pclass.coin()
        )
    }

    /// TX-S run form.
    pub fn s_run(path_phrase: &str, pclass: Class, run: RunNet, flag: bool) -> String {
        let remedy = match pclass {
            Class::Main => "--network mainnet",
            Class::Test => "--network signet, --network testnet or --network regtest",
        };
        let flag = if flag {
            s_flag(&format!("this path on {}", run.name()), run)
        } else {
            String::new()
        };
        format!(
            "{P} network mismatch (coin type): {path_phrase} (coin type {}h), but {}. Nothing was written. For a {} wallet, rerun with {remedy}.{flag}",
            pclass.coin(),
            run.phrase(),
            pclass.adj()
        )
    }

    /// TX-S-OK; `path_phrase` as in TX-S.
    pub fn s_ok(path_phrase: &str, run: RunNet) -> String {
        format!(
            "warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: {path_phrase}; the output is for {0}. A signer or recovery tool that looks only at {0} paths will not find coins sent here.",
            run.name()
        )
    }

    /// TX-N.
    pub const N: &str = "warning: nothing here names a network and no --network was given: read as mainnet. For a signet, testnet or regtest wallet, rerun with --network signet, --network testnet or --network regtest.";

    /// TX-T.
    pub fn t(run: RunNet, thing: &str) -> String {
        format!(
            "warning: this run is on {}, a test network, because --network says so; this {thing} names no network. A wallet holding real bitcoin is read with --network mainnet.",
            run.name()
        )
    }

    /// TX-R.
    pub const R: &str = "warning: This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.";

    /// TX-V (verify-bundle cause line).
    pub fn v(run: RunNet) -> String {
        format!(
            "note: every mismatch above is a coin-type or key-version row: the bundle may be for another network than --network {}; rerun with the network the bundle was made for.",
            run.name()
        )
    }
}

/// A coin with the key it is compared with (key form) or none (run form).
struct SoftEntry<'a> {
    idx: usize,
    coin: &'a KeyCoin,
    key: Option<&'a KeySource>,
}

fn pfx_join(keys: &[&KeySource]) -> String {
    let mut seen: Vec<&str> = Vec::new();
    for k in keys {
        if !seen.contains(&k.pfx.as_str()) {
            seen.push(&k.pfx);
        }
    }
    seen.join(", ")
}

/// "<who>'s path <path>" for one coin, or its plural; returns (phrase prefix
/// for the text, machine `a.what`), the text ending before " is a"/" are".
fn path_whats(entries: &[&SoftEntry]) -> (String, String) {
    let who_of = |e: &SoftEntry| -> Option<Who> {
        match &e.coin.path_who {
            PathWho::OwnKey => e.key.map(|k| k.who.clone()),
            PathWho::Named(w) => Some(w.clone()),
            PathWho::Flag(_) => None,
        }
    };
    let paths: Vec<String> = entries.iter().map(|e| e.coin.path.clone()).collect();
    if entries.len() == 1 {
        let e = entries[0];
        let s = match (&e.coin.path_who, who_of(e)) {
            (PathWho::Flag(f), _) => format!("the path {} given with {f}", e.coin.path),
            (_, Some(w)) => format!("{}'s path {}", w.name, e.coin.path),
            (_, None) => format!("the path {}", e.coin.path),
        };
        return (s.clone(), s);
    }
    if entries
        .iter()
        .all(|e| matches!(e.coin.path_who, PathWho::Flag(_)))
    {
        let flags: Vec<String> = entries
            .iter()
            .map(|e| match &e.coin.path_who {
                PathWho::Flag(f) => f.clone(),
                _ => unreachable!(),
            })
            .collect();
        let s = format!(
            "the paths {} given with {}",
            join_and(&paths),
            join_and(&flags)
        );
        return (s.clone(), s);
    }
    let whos: Vec<Who> = entries
        .iter()
        .map(|e| who_of(e).unwrap_or_else(|| Who::plain("the path")))
        .collect();
    let refs: Vec<&Who> = whos.iter().collect();
    let p = plural(&refs);
    (
        format!("{p}'s paths {}", join_and(&paths)),
        format!("{p}'s paths"),
    )
}

struct SoftGroup<'a> {
    first: usize,
    entries: Vec<&'a SoftEntry<'a>>,
}

impl SoftGroup<'_> {
    fn pclass(&self) -> Class {
        Class::of_coin(self.entries[0].coin.coin)
    }
    fn is_key_form(&self) -> bool {
        self.entries[0].key.is_some()
    }
    /// (TX-S text, TX-S-OK text, a, b).
    fn render(&self, run: RunNet, flag: bool) -> (String, String, End, End) {
        let pclass = self.pclass();
        let plural_n = self.entries.len() > 1;
        let (pp, a_what) = path_whats(&self.entries);
        let path_phrase = if plural_n {
            format!("{pp} are {} paths", pclass.adj())
        } else {
            format!("{pp} is a {} path", pclass.adj())
        };
        let ok = texts::s_ok(&path_phrase, run);
        let a = end(a_what, pclass.machine());
        if self.is_key_form() {
            let keys: Vec<&KeySource> = self.entries.iter().filter_map(|e| e.key).collect();
            let kclass = keys[0].class;
            let whos: Vec<&Who> = keys.iter().map(|k| &k.who).collect();
            let kw = plural(&whos);
            let key_phrase = if plural_n {
                format!("{kw} are {} keys", kclass.adj())
            } else {
                format!("{kw} is a {} key", kclass.adj())
            };
            let card_path = self
                .entries
                .iter()
                .any(|e| matches!(e.coin.path_who, PathWho::Named(_)));
            let text = texts::s_key(
                &path_phrase,
                pclass,
                &key_phrase,
                kclass,
                &pfx_join(&keys),
                card_path,
                run,
                flag,
            );
            (text, ok, a, end(kw, kclass.machine()))
        } else {
            let text = texts::s_run(&path_phrase, pclass, run, flag);
            (text, ok, a, end("this run", run.class().machine()))
        }
    }
}

/// The rule (N1-N8, N12): what a run prints and whether it refuses.
pub fn evaluate(input: &Input) -> Verdict {
    let run = input.run;
    let rc = run.class();
    // (order, input index, line)
    let mut hard: Vec<(u8, usize, Line)> = Vec::new();
    let mut soft_entries: Vec<SoftEntry> = Vec::new();

    let top_keys: Vec<(usize, &KeySource)> = input
        .sources
        .iter()
        .enumerate()
        .filter_map(|(i, s)| match s {
            Source::Key(k) => Some((i, k)),
            _ => None,
        })
        .collect();

    // Stated files (B3).
    for (i, s) in input.sources.iter().enumerate() {
        let Source::Stated {
            file,
            text,
            net,
            device,
            keys,
        } = s
        else {
            continue;
        };
        let sc = net.class();
        let bad: Vec<&SectionKey> = keys.iter().filter(|k| k.key.class != sc).collect();
        if !bad.is_empty() {
            let mut sections: Vec<String> = Vec::new();
            for b in &bad {
                if !sections.contains(&b.section) {
                    sections.push(b.section.clone());
                }
            }
            let bk: Vec<&KeySource> = bad.iter().map(|b| &b.key).collect();
            let kclass = bk[0].class;
            let dev = match device {
                Device::Coldcard => "Coldcard",
                Device::Signer => "signer",
            };
            let line = Line {
                class: LineClass::Hard,
                text_id: "TX-H4C",
                text: texts::h4c(
                    file,
                    text,
                    &join_and(&sections),
                    bad.len() > 1,
                    kclass,
                    &pfx_join(&bk),
                    dev,
                ),
                a: end(file.clone(), net.machine()),
                b: end(
                    format!(
                        "its {} {}",
                        join_and(&sections),
                        if bad.len() > 1 { "keys" } else { "key" }
                    ),
                    kclass.machine(),
                ),
            };
            hard.push((0, i, line));
            continue;
        }
        if !net.agrees(run.net) {
            hard.push((
                1,
                i,
                Line {
                    class: LineClass::Hard,
                    text_id: "TX-H4",
                    text: texts::h4(file, text, *net, run),
                    a: end(file.clone(), net.machine()),
                    b: end("this run", net.run_machine(run.net)),
                },
            ));
        }
        for k in keys {
            for c in &k.key.coins {
                if Class::of_coin(c.coin) != k.key.class {
                    soft_entries.push(SoftEntry {
                        idx: i,
                        coin: c,
                        key: Some(&k.key),
                    });
                }
            }
        }
    }

    // TX-H2: a mainnet key and a test key.
    let first_main = top_keys.iter().find(|(_, k)| k.class == Class::Main);
    let first_test = top_keys.iter().find(|(_, k)| k.class == Class::Test);
    let h2 = if let (Some((im, m)), Some((it, t))) = (first_main, first_test) {
        hard.push((
            3,
            (*im).min(*it),
            Line {
                class: LineClass::Hard,
                text_id: "TX-H2",
                text: texts::h2(&m.who.name, &m.pfx, &t.who.name, &t.pfx),
                a: end(m.who.name.clone(), "main"),
                b: end(t.who.name.clone(), "test"),
            },
        ));
        true
    } else {
        false
    };

    let addresses: Vec<(usize, &String, &String, AddrNet)> = input
        .sources
        .iter()
        .enumerate()
        .filter_map(|(i, s)| match s {
            Source::Address { who, address, net } => Some((i, who, address, *net)),
            _ => None,
        })
        .collect();

    // TX-H1: keys that disagree with the run, one merged line.
    let bad_keys: Vec<&(usize, &KeySource)> =
        top_keys.iter().filter(|(_, k)| k.class != rc).collect();
    if let Some((i0, k0)) = bad_keys.first() {
        let kclass = k0.class;
        let addr_disagrees = addresses.iter().any(|(_, _, _, n)| n.class() != kclass);
        let whos: Vec<&Who> = bad_keys.iter().map(|(_, k)| &k.who).collect();
        let ks: Vec<&KeySource> = bad_keys.iter().map(|(_, k)| *k).collect();
        let who = plural(&whos);
        hard.push((
            4,
            *i0,
            Line {
                class: LineClass::Hard,
                text_id: "TX-H1",
                text: texts::h1(
                    &who,
                    bad_keys.len() > 1,
                    &pfx_join(&ks),
                    kclass,
                    run,
                    h2 || addr_disagrees,
                ),
                a: end(who.clone(), kclass.machine()),
                b: end("this run", rc.machine()),
            },
        ));
    }

    // TX-H3: each address that disagrees with the run.
    for (i, who, address, net) in &addresses {
        if net.agrees(run.net) {
            continue;
        }
        let short = top_keys.iter().any(|(_, k)| k.class != net.class());
        hard.push((
            5,
            *i,
            Line {
                class: LineClass::Hard,
                text_id: "TX-H3",
                text: texts::h3(who, address, *net, run, short),
                a: end((*who).clone(), net.machine()),
                b: end("this run", net.run_machine(run.net)),
            },
        ));
    }

    // TX-H5: a WIF of the other class.
    for (i, s) in input.sources.iter().enumerate() {
        if let Source::Wif { class, version } = s {
            if *class != rc {
                hard.push((
                    6,
                    i,
                    Line {
                        class: LineClass::Hard,
                        text_id: "TX-H5",
                        text: texts::h5(*class, *version, run),
                        a: end("the WIF key", class.machine()),
                        b: end("this run", rc.machine()),
                    },
                ));
            }
        }
    }

    // SOFT: coins of top-level keys (against their own key, B2), coins with
    // no key (against the run).
    for (i, s) in input.sources.iter().enumerate() {
        match s {
            Source::Key(k) => {
                for c in &k.coins {
                    if Class::of_coin(c.coin) != k.class {
                        soft_entries.push(SoftEntry {
                            idx: i,
                            coin: c,
                            key: Some(k),
                        });
                    }
                }
            }
            Source::Coin(c) => {
                if Class::of_coin(c.coin) != rc {
                    soft_entries.push(SoftEntry {
                        idx: i,
                        coin: c,
                        key: None,
                    });
                }
            }
            _ => {}
        }
    }
    soft_entries.sort_by_key(|e| e.idx);

    // B5: key forms merge by (pclass, kclass); run forms by pclass.
    let mut groups: Vec<SoftGroup> = Vec::new();
    for e in &soft_entries {
        let found = groups.iter_mut().find(|g| {
            let g0 = g.entries[0];
            g0.key.is_some() == e.key.is_some()
                && g0.coin.coin == e.coin.coin
                && g0.key.map(|k| k.class) == e.key.map(|k| k.class)
        });
        match found {
            Some(g) => g.entries.push(e),
            None => groups.push(SoftGroup {
                first: e.idx,
                entries: vec![e],
            }),
        }
    }
    groups.sort_by_key(|g| g.first);

    hard.sort_by_key(|(o, i, _)| (*o, *i));
    let any_hard = !hard.is_empty();
    let mut lines: Vec<Line> = hard.into_iter().map(|(_, _, l)| l).collect();

    if any_hard || (!groups.is_empty() && !input.accept_coin_type) {
        for g in &groups {
            let (text, _, a, b) = g.render(run, !any_hard);
            lines.push(Line {
                class: LineClass::Soft,
                text_id: "TX-S",
                text,
                a,
                b,
            });
        }
        return Verdict::Refuse(lines);
    }

    let mut warnings = Vec::new();
    for g in &groups {
        let (_, ok, a, b) = g.render(run, false);
        warnings.push(Line {
            class: LineClass::Soft,
            text_id: "TX-S-OK",
            text: ok,
            a,
            b,
        });
    }
    if input.sources.is_empty() && input.reads_kept && !input.seed_only {
        if !run.given {
            warnings.push(notice("TX-N", texts::N.to_string()));
        } else if rc == Class::Test {
            warnings.push(notice("TX-T", texts::t(run, input.thing.word())));
        }
    }
    if input.secret_out && rc == Class::Test {
        warnings.push(notice("TX-R", texts::R.to_string()));
    }
    Verdict::Proceed(warnings)
}

fn notice(id: &'static str, text: String) -> Line {
    Line {
        class: LineClass::Notice,
        text_id: id,
        text,
        a: end("", ""),
        b: end("", ""),
    }
}

#[derive(serde::Serialize)]
struct MachineC<'a> {
    class: &'static str,
    text: &'static str,
    a: &'a End,
    b: &'a End,
}

#[derive(serde::Serialize)]
struct Machine<'a> {
    kind: &'static str,
    class: &'static str,
    contradictions: Vec<MachineC<'a>>,
}

fn class_word(c: LineClass) -> &'static str {
    match c {
        LineClass::Hard => "hard",
        _ => "soft",
    }
}

/// N10: the machine line of a refusal (JSON, no spaces outside strings).
pub fn machine_line(lines: &[Line]) -> String {
    let cs: Vec<MachineC> = lines
        .iter()
        .filter(|l| l.class != LineClass::Notice)
        .map(|l| MachineC {
            class: class_word(l.class),
            text: l.text_id,
            a: &l.a,
            b: &l.b,
        })
        .collect();
    let top = if cs.iter().all(|c| c.class == "soft") {
        "soft"
    } else {
        "hard"
    };
    serde_json::to_string(&Machine {
        kind: "network_mismatch",
        class: top,
        contradictions: cs,
    })
    .expect("serializes")
}

/// What a refused run prints last on stderr: its lines, then the machine line
/// when `MNEMONIC_NETWORK_JSON` is `1`.
pub fn refusal_trailer(lines: &[Line]) -> Vec<String> {
    let mut out: Vec<String> = lines.iter().map(|l| l.text.clone()).collect();
    if std::env::var(MACHINE_ENV).as_deref() == Ok("1") {
        out.push(machine_line(lines));
    }
    out
}

/// Run the rule for a verb: a refusal becomes `ToolkitError::NetworkRule`
/// (exit 8; `cli.rs` prints its lines last); the warnings of a run that goes
/// on print on stderr now.
pub(crate) fn enforce<E: Write + ?Sized>(
    input: &Input,
    stderr: &mut E,
) -> Result<(), crate::error::ToolkitError> {
    match evaluate(input) {
        Verdict::Refuse(lines) => Err(crate::error::ToolkitError::NetworkRule { lines }),
        Verdict::Proceed(warnings) => {
            for w in warnings {
                let _ = writeln!(stderr, "{}", w.text);
            }
            Ok(())
        }
    }
}

thread_local! {
    static NETWORK_GIVEN: Cell<Option<bool>> = const { Cell::new(None) };
    static SECRET_RUN: RefCell<Option<(RunNet, bool, bool)>> = const { RefCell::new(None) };
}

/// Set by `cli.rs` from the parsed command line: whether `--network` was typed.
pub fn set_network_given(given: bool) {
    NETWORK_GIVEN.with(|c| c.set(Some(given)));
}

/// Whether `--network` was typed; `fallback` when the CLI did not say (a
/// direct call of a verb's `run`).
pub fn network_given_or(fallback: bool) -> bool {
    NETWORK_GIVEN.with(|c| c.get()).unwrap_or(fallback)
}

/// N12 bookkeeping: a verb that can print `PrivateKeyMaterial` records its run
/// network, whether its input held seed material and whether its stdout class
/// was `PrivateKeyMaterial`. `cli.rs` prints TX-R after the run.
pub fn record_secret_run(run: RunNet, secret_input: bool, private_out: bool) {
    SECRET_RUN.with(|c| {
        let mut c = c.borrow_mut();
        let (si, po) = match *c {
            Some((_, si, po)) => (si || secret_input, po || private_out),
            None => (secret_input, private_out),
        };
        *c = Some((run, si, po));
    });
}

/// Mark the recorded run's stdout as `PrivateKeyMaterial` (N12), if a run was recorded.
pub fn record_private_out() {
    SECRET_RUN.with(|c| {
        if let Some((_, _, po)) = c.borrow_mut().as_mut() {
            *po = true;
        }
    });
}

/// TX-R, when the recorded run meets N12 (the post-run hook, `cli.rs`).
pub fn take_tx_r() -> Option<Line> {
    let rec = SECRET_RUN.with(|c| c.borrow_mut().take())?;
    let (run, si, po) = rec;
    let input = Input {
        run,
        sources: vec![],
        reads_kept: false,
        seed_only: true,
        secret_out: si && po,
        accept_coin_type: false,
        thing: Thing::Key,
    };
    evaluate(&input)
        .lines()
        .iter()
        .find(|l| l.text_id == "TX-R")
        .cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    const MAINNET: RunNet = RunNet {
        net: CliNetwork::Mainnet,
        given: true,
    };
    const NOFLAG: RunNet = RunNet {
        net: CliNetwork::Mainnet,
        given: false,
    };
    const SIGNET: RunNet = RunNet {
        net: CliNetwork::Signet,
        given: true,
    };
    const TESTNET: RunNet = RunNet {
        net: CliNetwork::Testnet,
        given: true,
    };

    fn key(who: Who, pfx: &str, coins: Vec<KeyCoin>) -> Source {
        Source::Key(KeySource::from_text(who, pfx, coins).expect("prefix"))
    }
    fn own(path: &str, coin: u32) -> KeyCoin {
        KeyCoin {
            path_who: PathWho::OwnKey,
            path: path.into(),
            coin,
        }
    }
    fn texts_of(v: &Verdict) -> Vec<String> {
        v.lines().iter().map(|l| l.text.clone()).collect()
    }

    #[test]
    fn tx_h1_test_key_no_flag_literal() {
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![key(Who::plain("--target-xpub"), "tpub", vec![])],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: --target-xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    }

    #[test]
    fn tx_h1_main_key_test_run_literal() {
        let v = evaluate(&Input::new(
            TESTNET,
            vec![key(Who::key(0), "xpub", vec![])],
            Thing::Descriptor,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: key @0 is a mainnet key (xpub), but --network is testnet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. For a testnet wallet, export the key again with the signer set to a test network."]);
    }

    #[test]
    fn tx_h1_plural_cosigner_cards_literal() {
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![
                key(Who::cosigner(0, "mk1 card"), "tpub", vec![]),
                key(Who::cosigner(1, "mk1 card"), "tpub", vec![]),
            ],
            Thing::Card,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: --cosigner @0's and @1's mk1 cards are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    }

    #[test]
    fn tx_h2_then_h1_short_literal() {
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![
                key(Who::plain("--slot @0.xpub"), "xpub", vec![]),
                key(Who::plain("--slot @0.master_xpub"), "tpub", vec![]),
            ],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), [
            "error: network mismatch: --slot @0.xpub is a mainnet key (xpub), but --slot @0.master_xpub is a test-network key (tpub). Nothing was written. Every key of one wallet is for one network: export the odd key again.",
            "error: network mismatch: --slot @0.master_xpub is a test-network key (tpub), but this run is on mainnet (no --network given). Nothing was written. For a mainnet wallet, export the key again with the signer set to mainnet.",
        ]);
    }

    #[test]
    fn tx_h3_short_literal() {
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![
                key(Who::plain("--xpub"), "xpub", vec![]),
                Source::Address {
                    who: "--target-address".into(),
                    address: "tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl".into(),
                    net: AddrNet::Hrp(Hrp::Tb),
                },
            ],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: --target-address tb1q6rz28mcfaxtmd6v789l9rrlrusdprr9pqcpvkl is a signet or testnet address, but this run is on mainnet (no --network given). Nothing was written. Give an address of mainnet."]);
    }

    #[test]
    fn tx_h3_full_literal() {
        let v = evaluate(&Input::new(
            TESTNET,
            vec![Source::Address {
                who: "--search-address".into(),
                address: "bcrt1qxyz".into(),
                net: AddrNet::Hrp(Hrp::Bcrt),
            }],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: --search-address bcrt1qxyz is a regtest address, but --network is testnet. Nothing was written. Rerun with --network regtest, or give an address of testnet."]);
    }

    #[test]
    fn tx_h4_literal() {
        let v = evaluate(&Input::new(
            MAINNET,
            vec![Source::Stated {
                file: "env_t.json".into(),
                text: "testnet".into(),
                net: StatedNet::Exact(CliNetwork::Testnet),
                device: Device::Signer,
                keys: vec![SectionKey {
                    section: "bundle.descriptor".into(),
                    key: KeySource::from_text(Who::key(0), "tpub", vec![own("m/84h/1h/0h", 1)])
                        .unwrap(),
                }],
            }],
            Thing::Descriptor,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: env_t.json says network testnet, but --network is mainnet. Nothing was written. Rerun with --network testnet, or use the file made for mainnet."]);
    }

    #[test]
    fn tx_h4c_literal_and_no_tx_s_inside() {
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![Source::Stated {
                file: "cc_BTC_T_0.json".into(),
                text: "chain BTC (mainnet)".into(),
                net: StatedNet::Class(Class::Main),
                device: Device::Coldcard,
                keys: vec![SectionKey {
                    section: "bip84".into(),
                    key: KeySource::from_text(
                        Who::plain("bip84"),
                        "tpub",
                        vec![own("m/84h/0h/0h", 0)],
                    )
                    .unwrap(),
                }],
            }],
            Thing::Descriptor,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: cc_BTC_T_0.json says chain BTC (mainnet), but its bip84 key is a test-network key (tpub). Nothing was written. No --network repairs a file that contradicts itself: export the file again from the Coldcard with its network set for the wallet's network."]);
    }

    #[test]
    fn tx_h4c_merged_sections_literal() {
        let sk = |s: &str| SectionKey {
            section: s.into(),
            key: KeySource::from_text(Who::plain(s), "tpub", vec![]).unwrap(),
        };
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![Source::Stated {
                file: "cc.json".into(),
                text: "chain BTC (mainnet)".into(),
                net: StatedNet::Class(Class::Main),
                device: Device::Coldcard,
                keys: vec![sk("bip44"), sk("bip49"), sk("bip84")],
            }],
            Thing::Descriptor,
        ));
        assert!(texts_of(&v)[0]
            .contains("but its bip44, bip49 and bip84 keys are test-network keys (tpub)."));
    }

    #[test]
    fn tx_h5_literal() {
        let v = evaluate(&Input::new(
            MAINNET,
            vec![Source::Wif {
                class: Class::Test,
                version: 239,
            }],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch: the WIF key is a test-network key (version byte 239), but --network is mainnet. Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet."]);
    }

    #[test]
    fn tx_s_key_form_and_ok_literal() {
        let src = vec![key(Who::key(0), "xpub", vec![own("m/84h/1h/0h", 1)])];
        let v = evaluate(&Input::new(MAINNET, src.clone(), Thing::Descriptor));
        assert_eq!(texts_of(&v), ["error: network mismatch (coin type): key @0's path m/84h/1h/0h is a test-network path (coin type 1h), but key @0 is a mainnet key (xpub). Nothing was written. If the path is a mistake, export the key again at the mainnet path (coin type 0h). To use this key as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
        let mut i = Input::new(MAINNET, src, Thing::Descriptor);
        i.accept_coin_type = true;
        let v = evaluate(&i);
        assert!(matches!(v, Verdict::Proceed(_)));
        assert_eq!(texts_of(&v), ["warning: network mismatch (coin type) accepted with --accept-coin-type-mismatch: key @0's path m/84h/1h/0h is a test-network path; the output is for mainnet. A signer or recovery tool that looks only at mainnet paths will not find coins sent here."]);
    }

    #[test]
    fn tx_s_run_form_flag_path_literal() {
        let v = evaluate(&Input::new(
            SIGNET,
            vec![Source::Coin(KeyCoin {
                path_who: PathWho::Flag("--slot @0.path".into()),
                path: "m/84h/0h/0h".into(),
                coin: 0,
            })],
            Thing::Key,
        ));
        assert_eq!(texts_of(&v), ["error: network mismatch (coin type): the path m/84h/0h/0h given with --slot @0.path is a mainnet path (coin type 0h), but --network is signet. Nothing was written. For a mainnet wallet, rerun with --network mainnet. To use this path on signet as it is, add --accept-coin-type-mismatch: the output is then for signet and this warning is printed every time."]);
    }

    #[test]
    fn tx_s_run_form_plural_literal() {
        let c = |i| {
            Source::Coin(KeyCoin {
                path_who: PathWho::Named(Who::key(i)),
                path: "m/87h/1h/0h".into(),
                coin: 1,
            })
        };
        let v = evaluate(&Input::new(NOFLAG, vec![c(0), c(1)], Thing::Card));
        assert_eq!(texts_of(&v), ["error: network mismatch (coin type): keys @0 and @1's paths m/87h/1h/0h and m/87h/1h/0h are test-network paths (coin type 1h), but this run is on mainnet (no --network given). Nothing was written. For a test-network wallet, rerun with --network signet, --network testnet or --network regtest. To use this path on mainnet as it is, add --accept-coin-type-mismatch: the output is then for mainnet and this warning is printed every time."]);
        let m = machine_line(v.lines());
        assert!(
            m.contains(r#""a":{"what":"keys @0 and @1's paths","network":"test"}"#),
            "{m}"
        );
    }

    #[test]
    fn tx_h1_then_tx_s_plural_jade_literal() {
        let j = |i| {
            key(
                Who::stem("jade key", i),
                "tpub",
                vec![own("m/48h/0h/0h/2h", 0)],
            )
        };
        let v = evaluate(&Input::new(
            NOFLAG,
            vec![j(0), j(1), j(2)],
            Thing::Descriptor,
        ));
        assert_eq!(texts_of(&v), [
            "error: network mismatch: jade keys @0, @1 and @2 are test-network keys (tpub), but this run is on mainnet (no --network given). Nothing was written. For a test wallet, rerun with --network signet, --network testnet or --network regtest. For a mainnet wallet, export the key again with the signer set to mainnet.",
            "error: network mismatch (coin type): jade keys @0, @1 and @2's paths m/48h/0h/0h/2h, m/48h/0h/0h/2h and m/48h/0h/0h/2h are mainnet paths (coin type 0h), but jade keys @0, @1 and @2 are test-network keys (tpub). Nothing was written. If the path is a mistake, export the key again at the test-network path (coin type 1h).",
        ]);
    }

    #[test]
    fn tx_n_t_r_literal() {
        assert_eq!(texts::N, "warning: nothing here names a network and no --network was given: read as mainnet. For a signet, testnet or regtest wallet, rerun with --network signet, --network testnet or --network regtest.");
        assert_eq!(texts::t(SIGNET, "descriptor"), "warning: this run is on signet, a test network, because --network says so; this descriptor names no network. A wallet holding real bitcoin is read with --network mainnet.");
        assert_eq!(texts::R, "warning: This rehearsal used your real seeds. Everything it engraved or printed from them is as secret as your real kit: destroy it, do not discard it.");
        assert_eq!(texts::v(MAINNET), "note: every mismatch above is a coin-type or key-version row: the bundle may be for another network than --network mainnet; rerun with the network the bundle was made for.");
    }

    #[test]
    fn machine_line_r1_4_byte_exact() {
        let v = evaluate(&Input::new(
            MAINNET,
            vec![key(Who::key(0), "xpub", vec![own("m/84h/1h/0h", 1)])],
            Thing::Descriptor,
        ));
        assert_eq!(
            machine_line(v.lines()),
            r#"{"kind":"network_mismatch","class":"soft","contradictions":[{"class":"soft","text":"TX-S","a":{"what":"key @0's path m/84h/1h/0h","network":"test"},"b":{"what":"key @0","network":"main"}}]}"#
        );
    }

    #[test]
    fn coin_of_path_shapes() {
        use std::str::FromStr;
        let p = |s: &str| bitcoin::bip32::DerivationPath::from_str(s).unwrap();
        assert_eq!(
            coin_of_path(&p("m/84'/1'/0'")),
            Some(("m/84h/1h/0h".into(), 1))
        );
        assert_eq!(coin_of_path(&p("m/270028'/0'/3'")).map(|c| c.1), Some(0));
        assert_eq!(coin_of_path(&p("m/45'/0'")), None);
        assert_eq!(coin_of_path(&p("m/84'/5'/0'")), None);
        assert_eq!(coin_of_path(&p("m/84/1'/0'")), None);
        assert_eq!(coin_of_path(&p("m/0'")), None);
        assert_eq!(coin_of_path(&p("m")), None);
    }

    #[test]
    fn address_network_of_text() {
        assert_eq!(
            AddrNet::of_address("bc1qcr8te4kr609gcawutmrza0j4xv80jy8z306fyu"),
            Some(AddrNet::Hrp(Hrp::Bc))
        );
        assert_eq!(
            AddrNet::of_address("bcrt1qxyz"),
            Some(AddrNet::Hrp(Hrp::Bcrt))
        );
        assert_eq!(AddrNet::of_address("tb1qxyz"), Some(AddrNet::Hrp(Hrp::Tb)));
        assert_eq!(
            AddrNet::of_address("1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2"),
            Some(AddrNet::B58(Class::Main))
        );
        assert_eq!(
            AddrNet::of_address("mipcBbFg9gMiCh81Kj8tqqdgoZub1ZJRfn"),
            Some(AddrNet::B58(Class::Test))
        );
    }
}

/// The sources of `--slot @N.<subkey>=` inputs (export-wallet, bundle), in
/// slot order: `@N.xpub` and `@N.master_xpub` are keys (prefix as typed), a
/// typed `@N.path` is a coin on the slot's xpub (key form) or, for a seed
/// slot, a coin with no key (run form); `@N.wif` is a WIF. A path the slot
/// does not type is no source (B1). Values read from stdin (`-`) and values
/// that do not parse are left to the verb's own checks.
pub(crate) fn slot_sources(slots: &[crate::slot_input::SlotInput]) -> Vec<Source> {
    use crate::slot_input::SlotSubkey as K;
    use std::str::FromStr;
    let mut indices: Vec<u8> = slots.iter().map(|s| s.index).collect();
    indices.sort_unstable();
    indices.dedup();
    let mut out = Vec::new();
    for idx in indices {
        let get = |k: K| -> Option<&str> {
            slots
                .iter()
                .find(|s| s.index == idx && s.subkey == k)
                .map(|s| &*s.value)
                .filter(|v| *v != "-")
        };
        let flag = format!("--slot @{idx}.path");
        let coin = get(K::Path)
            .and_then(|p| bitcoin::bip32::DerivationPath::from_str(p).ok())
            .and_then(|p| key_coin(PathWho::Flag(flag.clone()), &p));
        let mut coin_used = false;
        for (k, name) in [(K::Xpub, "xpub"), (K::Xprv, "xprv")] {
            if let Some(v) = get(k) {
                let coins: Vec<KeyCoin> = coin.clone().into_iter().collect();
                coin_used = true;
                if let Some(ks) =
                    KeySource::from_text(Who::plain(format!("--slot @{idx}.{name}")), v, coins)
                {
                    out.push(Source::Key(ks));
                }
            }
        }
        if let Some(v) = get(K::MasterXpub) {
            if let Some(ks) =
                KeySource::from_text(Who::plain(format!("--slot @{idx}.master_xpub")), v, vec![])
            {
                out.push(Source::Key(ks));
            }
        }
        if let Some(v) = get(K::Wif) {
            // Only the version byte is kept; the decoded buffer is scrubbed.
            if let Ok(data) = bitcoin::base58::decode_check(v).map(zeroize::Zeroizing::new) {
                match data.first() {
                    Some(128) => out.push(Source::Wif {
                        class: Class::Main,
                        version: 128,
                    }),
                    Some(239) => out.push(Source::Wif {
                        class: Class::Test,
                        version: 239,
                    }),
                    _ => {}
                }
            }
        }
        if !coin_used {
            if let Some(c) = coin {
                out.push(Source::Coin(c));
            }
        }
    }
    out
}
