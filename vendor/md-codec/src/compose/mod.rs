//! Fixed, search-free lowering of an ORDERED spend-path list to a BIP-388
//! wallet-policy template (`design/SPEC_wallet_policy_composer.md` §4, §5 in
//! `mnemonic-engrave`).
//!
//! WHY A LOWERING AND NOT THE COMPILER. rust-miniscript's compiler picks
//! fragments by cost and its output moves between versions and contexts
//! (measured 2026-09-01: `andor` for a two-path wsh, `pk`/`pkh` flipped by
//! cost, taproot leaves reordered). Two implementations must agree byte for
//! byte on what a policy IS, so the rules here are fixed and the compiler is
//! only a cross-check of validity and meaning (spec §5b).
//!
//! WHY THE TREE AND NOT TEXT. The Go port has no template-text parser; it
//! decodes md1. Building `Descriptor`'s tree directly is what the port mirrors;
//! text comes from `render::descriptor_to_template` for humans and vectors.
//!
//! Errors here are [`ComposeError`], not [`crate::Error`]: the codec's error is
//! a pure wire/decode taxonomy and stays one.
//!
//! Layout: this file holds the types, the structural validator and the two
//! entry points; `lowering.rs` holds the path body, the wsh chain, numbering
//! and origins; `tr.rs` the taproot spine; `presets.rs` the archetypes.

use crate::encode::Descriptor;
use crate::origin_path::{OriginPath, PathComponent, PathDeclPaths};
use crate::render::{RenderError, descriptor_to_template};
use crate::tag::Tag;

mod lowering;
pub mod presets;
mod tr;

// THESE LIMITS ARE NECESSARY AND NOT SUFFICIENT (F-515).
//
// A policy inside every one of them can still be unmintable, because the md1
// wire also caps a card at 64 CHUNKS (`Error::TooManyChunks`, spec §9.8) and
// that ceiling is a function of the encoded SIZE -- the keys, their origins and
// every lock operand -- none of which the composer sees. `md compose` takes no
// keys at all, so it cannot predict the chunk count and does not pretend to.
//
// Measured over 1000 generated policies inside these limits: 10 of them needed
// 65 to 67 chunks and were refused at `md encode`, after the operator had
// already chosen the shape and the keys. That is the worst moment to learn it,
// and naming the second ceiling here is the cheapest place to say so.
//
// Do not "fix" this by lowering MAX_SLOTS. The two ceilings are independent and
// the wire one depends on inputs the composer does not have; a slot limit low
// enough to guarantee 64 chunks would refuse most policies that fit.

/// Spec §4: at most eight spend paths.
///
/// Necessary, not sufficient — see the note above: the 64-chunk wire cap is a
/// separate ceiling that depends on the keys.
pub const MAX_PATHS: usize = 8;
/// Spec §4b: at most nine keys in one path.
///
/// Necessary, not sufficient — see the note above.
pub const MAX_KEYS_PER_PATH: u8 = 9;
/// Spec §4b: the wire's 5-bit `path_decl.n` caps a policy at 32 slots.
///
/// Necessary, not sufficient — see the note above. This bounds the slot COUNT;
/// the 64-chunk cap bounds the encoded SIZE, and a policy can pass this and
/// fail that.
pub const MAX_SLOTS: u8 = 32;
/// BIP-68: bit 22 selects 512-second units.
pub const SEQUENCE_TYPE_FLAG: u32 = 1 << 22;
/// BIP-65: operands at or above this are Unix times, below are heights.
pub const LOCKTIME_THRESHOLD: u32 = 500_000_000;
/// BIP-379: miniscript admits absolute locktimes up to 2^31 - 1.
pub const MAX_ABSOLUTE_LOCKTIME: u32 = 0x7fff_ffff;

/// The script wrapper (spec §4a).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Wrapper {
    /// `tr(...)`; any path list.
    Tr,
    /// `wsh(...)`; any path list.
    Wsh,
    /// `sh(wsh(...))`; one unlocked, unhashed sorted multi-key path only.
    ShWsh,
    /// `sh(...)`; one unlocked, unhashed sorted multi-key path only.
    Sh,
}

impl Wrapper {
    /// BIP-48 script-type component for a seed-derived slot (spec §4f).
    pub fn script_type(self) -> u32 {
        match self {
            Wrapper::Wsh | Wrapper::Sh => 2,
            Wrapper::ShWsh => 1,
            Wrapper::Tr => 3,
        }
    }

    fn is_legacy(self) -> bool {
        matches!(self, Wrapper::Sh | Wrapper::ShWsh)
    }
}

/// One timelock, in the operator's units (spec §4c).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lock {
    /// `older(n)`, n blocks, 1..=65535.
    OlderBlocks(u16),
    /// `older(0x400000 + u)`, u units of 512 seconds, 1..=65535.
    OlderUnits(u16),
    /// `after(h)`, a block height, 1..=499,999,999.
    AfterHeight(u32),
    /// `after(t)`, a Unix time, 500,000,000..=2,147,483,647.
    AfterTime(u32),
}

impl Lock {
    /// The tag and the consensus operand this lock encodes to, or the
    /// out-of-range reason.
    pub fn operand(self) -> Result<(Tag, u32), &'static str> {
        match self {
            Lock::OlderBlocks(0) => Err("older in blocks needs 1..=65535"),
            Lock::OlderBlocks(b) => Ok((Tag::Older, u32::from(b))),
            Lock::OlderUnits(0) => Err("older in 512-second units needs 1..=65535"),
            Lock::OlderUnits(u) => Ok((Tag::Older, SEQUENCE_TYPE_FLAG + u32::from(u))),
            Lock::AfterHeight(h) if h == 0 || h >= LOCKTIME_THRESHOLD => {
                Err("after height needs 1..=499999999")
            }
            Lock::AfterHeight(h) => Ok((Tag::After, h)),
            Lock::AfterTime(t) if !(LOCKTIME_THRESHOLD..=MAX_ABSOLUTE_LOCKTIME).contains(&t) => {
                Err("after time needs 500000000..=2147483647")
            }
            Lock::AfterTime(t) => Ok((Tag::After, t)),
        }
    }
}

/// k-of-n over FRESH slots (spec §4b, C5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeySet {
    /// Threshold.
    pub k: u8,
    /// Key count; each key is a new slot.
    pub n: u8,
    /// Sorted (`sortedmulti`/`sortedmulti_a`) where the position allows it;
    /// `false` asks for `multi`/`multi_a` there, which is EXPERIMENTAL.
    pub sorted: bool,
}

/// Which hash the SCRIPT commits to (SPEC_hashlock_kinds §5).
///
/// **Deliberately NOT the wire `Tag` set.** A hashlock field typed as `Tag`
/// can hold `Wpkh`; these four are the only fragments that take a preimage.
/// `tag()` is the total function from here into the wire set.
///
/// NOT the same axis as the preimage METHOD (how the preimage was derived from
/// a phrase, which lives in `ms-codec`). The two share the token `sha256` and
/// mean different things.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HashKind {
    /// `sha256(X)`.
    Sha256,
    /// `hash256(X)` = `sha256(sha256(X))`.
    Hash256,
    /// `ripemd160(X)`, the bare primitive.
    Ripemd160,
    /// `hash160(X)` = `ripemd160(sha256(X))`.
    Hash160,
}

impl HashKind {
    /// **THE ONLY PLACE A DIGEST LENGTH IS WRITTEN** (spec §5). The hex rule is
    /// `digest_len() * 2`, which is what makes the parser, the validator and
    /// the formatter agree by construction instead of by review.
    pub const fn digest_len(self) -> usize {
        match self {
            HashKind::Sha256 | HashKind::Hash256 => 32,
            HashKind::Ripemd160 | HashKind::Hash160 => 20,
        }
    }

    /// The wire tag. Total by construction — every kind has exactly one.
    pub fn tag(self) -> Tag {
        match self {
            HashKind::Sha256 => Tag::Sha256,
            HashKind::Hash256 => Tag::Hash256,
            HashKind::Ripemd160 => Tag::Ripemd160,
            HashKind::Hash160 => Tag::Hash160,
        }
    }

    /// The lowercase miniscript fragment name, which is also `md compose`'s
    /// option name (`ripemd160=<hex>`). Case is rejected, never folded.
    pub fn token(self) -> &'static str {
        match self {
            HashKind::Sha256 => "sha256",
            HashKind::Hash256 => "hash256",
            HashKind::Ripemd160 => "ripemd160",
            HashKind::Hash160 => "hash160",
        }
    }
}

/// A hashlock: which hash, and the digest it commits to (spec §5).
///
/// **The digest is a fixed `[u8; 32]` for the alloc gate**, so a 20-byte kind
/// carries twelve bytes of zero padding. That padding is unobservable through
/// `digest()`, and `digest()` is the only way callers should read it — a
/// lowering that reaches past it commits the padding into the script, which
/// still compiles, still round-trips, and produces a wallet nobody can spend.
#[derive(Debug, Clone, Copy)]
pub struct HashLock {
    kind: HashKind,
    digest: [u8; 32],
}

// Eq/Ord/Hash are HAND-WRITTEN over `(kind, digest())`, NOT derived.
//
// Deriving them compares the whole `[u8; 32]`, which makes the alloc-gate
// padding OBSERVABLE: two `Ripemd160` locks with identical 20-byte digests but
// different bytes at index 31 would be `!=`, hash differently, and both sit in
// one `HashSet` -- while `digest()` says they are the same lock. That
// contradicts spec §5's "unobservable", and §5 also mandates that eleven
// map/set/equality sites re-key on this exact type, so the trap would be
// load-bearing rather than theoretical. (R0 round 1, I-1.)
//
// The Go port inherits this: `==` on a struct holding a `[32]byte` compares all
// 32 bytes and cannot be overridden, so the port must compare the slice
// explicitly.
impl PartialEq for HashLock {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind && self.digest() == other.digest()
    }
}

impl Eq for HashLock {}

impl core::hash::Hash for HashLock {
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
        self.digest().hash(state);
    }
}

impl PartialOrd for HashLock {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for HashLock {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.kind
            .cmp(&other.kind)
            .then_with(|| self.digest().cmp(other.digest()))
    }
}

impl HashLock {
    /// Build one. `digest`'s tail beyond `kind.digest_len()` is padding and is
    /// never read back.
    pub const fn new(kind: HashKind, digest: [u8; 32]) -> Self {
        HashLock { kind, digest }
    }

    /// Which hash the script commits to.
    pub const fn kind(&self) -> HashKind {
        self.kind
    }

    /// The digest AT ITS KIND'S WIDTH — 20 bytes or 32, never the padding.
    pub fn digest(&self) -> &[u8] {
        &self.digest[..self.kind.digest_len()]
    }
}

/// One spend path: keys, optional hash, optional lock (spec §4b).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpendPath {
    /// `None` is a keyless path: wsh-only, needs `hash`, EXPERIMENTAL.
    pub keys: Option<KeySet>,
    /// A hashlock: which hash the script commits to, and the digest. H is the
    /// digest of a 32-byte preimage under `HashLock::kind` (spec §3 F1 — the
    /// preimage is 32 bytes for every kind; only the digest width moves).
    pub hash: Option<HashLock>,
    /// At most one timelock.
    pub lock: Option<Lock>,
}

impl SpendPath {
    fn is_bare_multi(&self) -> bool {
        matches!(self.keys, Some(KeySet { n, .. }) if n >= 2)
            && self.hash.is_none()
            && self.lock.is_none()
    }

    fn is_bare_single(&self) -> bool {
        matches!(self.keys, Some(KeySet { n: 1, .. })) && self.hash.is_none() && self.lock.is_none()
    }
}

/// The operator's ordered list under one wrapper (spec §4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PathList {
    /// The script wrapper every path sits under.
    pub wrapper: Wrapper,
    /// The spend paths, in the operator's listed order.
    pub paths: Vec<SpendPath>,
}

/// A slot's declared origin and, when known, its master fingerprint (spec §4f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SlotOrigin {
    /// The declared derivation origin (spec §4f).
    pub origin: OriginPath,
    /// The master fingerprint, when the seating knows it.
    pub fingerprint: Option<[u8; 4]>,
}

/// Where an emitted slot came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Slot {
    /// The emitted `@index` (first appearance in the template text).
    pub index: u8,
    /// Index into `PathList::paths`.
    pub path: usize,
    /// Position within that path's key set, 0-based.
    pub ordinal: u8,
}

/// The EXPERIMENTAL conditions a list triggered (spec §4b, §5; C16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Experimental {
    /// Path `.0` has no key.
    KeylessPath(usize),
    /// Path `.0` asked for unsorted keys where sorted was legal.
    UnsortedKeys(usize),
}

/// A lowered policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Composed {
    /// The template tree with every slot's origin (declared or §4f default),
    /// ready for `encode_payload` / `chunk::split` /
    /// `render::descriptor_to_template`. Keys are bound by the caller.
    pub descriptor: Descriptor,
    /// Every emitted slot, in emitted order.
    pub slots: Vec<Slot>,
    /// The path extracted as the taproot internal key, if any.
    pub internal_key_path: Option<usize>,
    /// Every EXPERIMENTAL condition the list triggered.
    pub experimental: Vec<Experimental>,
}

/// Why a list cannot be lowered (spec §4e, §4c, §4f).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComposeError {
    /// The list has no paths.
    NoPaths,
    /// More than [`MAX_PATHS`] paths.
    TooManyPaths {
        /// The number of paths given.
        got: usize,
    },
    /// No path carries a key (BIP-388 l.191).
    NoKeyedPath,
    /// Path `path` has neither keys nor hash: anyone could spend after its lock.
    LockOnlyPath {
        /// 0-based index into `PathList::paths`.
        path: usize,
    },
    /// Path `path` is keyless under `tr`.
    KeylessUnderTr {
        /// 0-based index into `PathList::paths`.
        path: usize,
    },
    /// Two or more paths have no key (spec §4e).
    ///
    /// Paths chain right-leaning as `or_i(P, rest)` (`or_d` under a bare-multi
    /// head) and `or_i(l, r)` is non-malleable only if `l.safe || r.safe`
    /// (`vendor/miniscript/.../types/malleability.rs`, `safe` = every
    /// satisfaction needs a signature). A key-less path is never safe, so with
    /// two of them ANYWHERE in the list some `or_i` on the spine has two unsafe
    /// arms and the whole script is malleable: `md encode` refuses it and
    /// Bitcoin Core will not import it. A timelock does not change this --
    /// `older`/`after` need no signature either.
    TooManyKeylessPaths {
        /// 0-based index into `PathList::paths` of the first key-less path.
        first: usize,
        /// 0-based index into `PathList::paths` of the second one.
        second: usize,
    },
    /// `k`/`n` outside 1 ≤ k ≤ n ≤ 9.
    BadThreshold {
        /// 0-based index into `PathList::paths`.
        path: usize,
        /// The threshold given.
        k: u8,
        /// The key count given.
        n: u8,
    },
    /// More than [`MAX_SLOTS`] slots in total.
    TooManySlots {
        /// The slot count the list would need.
        got: usize,
        /// The wire's cap.
        max: u8,
    },
    /// `sh`/`sh(wsh)` with anything but one unlocked, unhashed sorted multi-key path.
    LegacyWrapperShape,
    /// A lock operand outside spec §4c.
    LockOutOfRange {
        /// 0-based index into `PathList::paths`.
        path: usize,
        /// The band that was missed, in the words the operator sees.
        why: &'static str,
    },
    /// `compose_with` was given a declaration slice of the wrong length.
    WrongSlotCount {
        /// Declarations given.
        got: usize,
        /// Slots the policy has.
        want: usize,
    },
    /// Two slots would declare the same origin without two distinct fingerprints.
    IndistinguishableSlots {
        /// The lower emitted slot index.
        a: u8,
        /// The higher emitted slot index.
        b: u8,
    },
    /// A preset's parameters do not form the archetype it is named for
    /// (`presets`, spec §4d): e.g. decaying tiers that do not unlock later.
    PresetShape {
        /// What the archetype needs, in the words the operator sees.
        why: &'static str,
    },
}

impl core::fmt::Display for ComposeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            ComposeError::NoPaths => write!(f, "a policy needs at least one spend path"),
            ComposeError::TooManyPaths { got } => {
                write!(
                    f,
                    "a policy holds at most {MAX_PATHS} spend paths; got {got}"
                )
            }
            ComposeError::NoKeyedPath => {
                write!(f, "every wallet needs at least one path with a key")
            }
            ComposeError::LockOnlyPath { path } => write!(
                f,
                "path {} has only a time lock, so anyone could spend after it; add a key or a hash",
                path + 1
            ),
            ComposeError::KeylessUnderTr { path } => write!(
                f,
                "path {} has no key; this build will not put a key-less path in taproot (use wsh, or add a key)",
                path + 1
            ),
            ComposeError::TooManyKeylessPaths { first, second } => write!(
                f,
                "paths {} and {} both have no key; side by side they lower to a malleable `or_i` that `md encode` refuses and no wallet will import. Give one of them a key, a timelock does not help, or fold them into one path",
                first + 1,
                second + 1
            ),
            ComposeError::BadThreshold { path, k, n } => write!(
                f,
                "path {}: {k}-of-{n} is not admitted (1 <= k <= n <= {MAX_KEYS_PER_PATH})",
                path + 1
            ),
            ComposeError::TooManySlots { got, max } => {
                write!(
                    f,
                    "this wallet would have {got} key slots; the wire holds at most {max}"
                )
            }
            ComposeError::LegacyWrapperShape => write!(
                f,
                "legacy wrappers hold one plain sorted multisig only (n >= 2, no lock, no hash); use wsh or tr"
            ),
            ComposeError::LockOutOfRange { path, why } => {
                write!(f, "path {}: {why}", path + 1)
            }
            ComposeError::WrongSlotCount { got, want } => {
                write!(
                    f,
                    "declarations for {got} slots given, but the policy has {want}"
                )
            }
            ComposeError::IndistinguishableSlots { a, b } => write!(
                f,
                "slots @{a} and @{b} declare the same origin without two distinct fingerprints; a template like that cannot be restored"
            ),
            ComposeError::PresetShape { why } => write!(f, "preset: {why}"),
        }
    }
}

impl std::error::Error for ComposeError {}

/// Structural validation of a list (spec §4e), before any lowering.
///
/// Returns the total slot count on success.
pub fn validate(list: &PathList) -> Result<usize, ComposeError> {
    if list.paths.is_empty() {
        return Err(ComposeError::NoPaths);
    }
    if list.paths.len() > MAX_PATHS {
        return Err(ComposeError::TooManyPaths {
            got: list.paths.len(),
        });
    }
    let mut slots = 0usize;
    let mut any_keyed = false;
    let mut keyless: Vec<usize> = Vec::new();
    for (i, p) in list.paths.iter().enumerate() {
        if let Some(ks) = p.keys {
            if ks.k == 0 || ks.n == 0 || ks.k > ks.n || ks.n > MAX_KEYS_PER_PATH {
                return Err(ComposeError::BadThreshold {
                    path: i,
                    k: ks.k,
                    n: ks.n,
                });
            }
            slots += usize::from(ks.n);
            any_keyed = true;
        } else if p.hash.is_none() {
            return Err(ComposeError::LockOnlyPath { path: i });
        } else if list.wrapper == Wrapper::Tr {
            return Err(ComposeError::KeylessUnderTr { path: i });
        } else {
            keyless.push(i);
        }
        if let Some(lock) = p.lock {
            if let Err(why) = lock.operand() {
                return Err(ComposeError::LockOutOfRange { path: i, why });
            }
        }
    }
    if !any_keyed {
        return Err(ComposeError::NoKeyedPath);
    }
    if slots > usize::from(MAX_SLOTS) {
        return Err(ComposeError::TooManySlots {
            got: slots,
            max: MAX_SLOTS,
        });
    }
    if list.wrapper.is_legacy() {
        let sole = list.paths.len() == 1 && list.paths[0].is_bare_multi();
        let sorted = matches!(
            list.paths.first().and_then(|p| p.keys),
            Some(KeySet { sorted: true, .. })
        );
        if !(sole && sorted) {
            return Err(ComposeError::LegacyWrapperShape);
        }
    }
    // AT MOST ONE KEY-LESS PATH (spec §4e; composer fable review r0, lens 1
    // C-1). Two of them make the spine malleable wherever they sit -- see
    // `ComposeError::TooManyKeylessPaths` for the type-system reason and
    // `tests/compose_keyless_cap.rs` for the 859-shape measurement. The rule
    // is STATED here rather than discovered by re-parsing the output, because
    // the device's Go port has no miniscript library to re-parse with: before
    // this, `md compose`'s read-back (md-cli F-600) was the only thing that
    // knew, and the port cut such a policy into steel.
    //
    // PRECEDENCE (fold-A review, M-1): this runs AFTER `TooManySlots` and
    // `LegacyWrapperShape`, because its remedy -- "fold them into one path" --
    // does not cure either of those: 36 slots folded are still 36, and `sh`
    // with two paths is still not one sorted multisig. A remedy that does not
    // work is worse than no remedy. The two rules above (`KeylessUnderTr`,
    // `NoKeyedPath`) also precede it, by the same argument.
    if let (Some(&first), Some(&second)) = (keyless.first(), keyless.get(1)) {
        return Err(ComposeError::TooManyKeylessPaths { first, second });
    }
    Ok(slots)
}

/// The §4f default origin for a slot: `m/48'/0'/account'/T'`.
pub fn default_origin(wrapper: Wrapper, account: u32) -> OriginPath {
    OriginPath {
        components: vec![
            PathComponent {
                hardened: true,
                value: 48,
            },
            PathComponent {
                hardened: true,
                value: 0,
            },
            PathComponent {
                hardened: true,
                value: account,
            },
            PathComponent {
                hardened: true,
                value: wrapper.script_type(),
            },
        ],
    }
}

/// Lower a list with every slot UNSEATED: each slot takes the §4f default
/// origin at the lowest account not yet declared (so slot `i` gets account
/// `i`), and no fingerprint.
pub fn compose(list: &PathList) -> Result<Composed, ComposeError> {
    let n = validate(list)?;
    let none: Vec<Option<SlotOrigin>> = vec![None; n];
    compose_with(list, &none)
}

/// Lower a list with per-slot declarations, indexed by EMITTED slot index
/// (call [`compose`] first to learn the slot map). `None` means unseated.
pub fn compose_with(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
) -> Result<Composed, ComposeError> {
    let n = validate(list)?;
    if declared.len() != n {
        return Err(ComposeError::WrongSlotCount {
            got: declared.len(),
            want: n,
        });
    }
    lowering::lower(list, declared)
}

/// The rendered template with each slot's origin written inline
/// (`@0/48'/0'/0'/2'/<0;1>/*`): the form `md encode` reads back to the same
/// card, the form `md compose` prints, and the form the vector corpus stores.
/// The plain renderer omits origins by design (descriptor-mnemonic F-219).
pub fn template_with_origins(c: &Composed) -> Result<String, RenderError> {
    let mut out = descriptor_to_template(&c.descriptor)?;
    let n = usize::from(c.descriptor.n);
    let origins: Vec<&OriginPath> = match &c.descriptor.path_decl.paths {
        PathDeclPaths::Shared(o) => vec![o; n],
        PathDeclPaths::Divergent(v) => v.iter().collect(),
    };
    for (i, o) in origins.iter().enumerate() {
        let mut rendered = String::new();
        for comp in &o.components {
            rendered.push('/');
            rendered.push_str(&comp.value.to_string());
            if comp.hardened {
                rendered.push('\'');
            }
        }
        // `@1/` never occurs inside `@10/` or `@11/`, and a slot appears once.
        out = out.replace(&format!("@{i}/"), &format!("@{i}{rendered}/"));
    }
    Ok(out)
}
