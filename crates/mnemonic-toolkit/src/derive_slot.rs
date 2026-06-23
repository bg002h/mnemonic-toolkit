//! BIP-39 + BIP-32 derivation helper shared by `cmd::bundle::resolve_slots`
//! (phrase / entropy slot branches) and `cmd::convert` (BIP-39-rooted edges).
//!
//! Locked v0.5.2: extracted from `bundle::resolve_slots` to remove the
//! duplicated derivation spine between phrase and entropy branches.

use crate::derive::DerivedAccount;
use crate::error::{BitcoinErrorKind, ToolkitError};
use crate::network::CliNetwork;
use crate::secret_string::SecretString;
use crate::template::CliTemplate;
use bip39::{Language as Bip39Language, Mnemonic};
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::{All, Secp256k1};
use zeroize::Zeroizing;

/// SPEC v0.9.0 §1 item 2 — consolidated BIP-39 → BIP-32 seed step.
/// Wraps the 64-byte PBKDF2-HMAC-SHA512 output in `Zeroizing` so it
/// scrubs on drop at every call site. Seven production BIP-39 →
/// BIP-32 spines in this crate share this helper:
///
/// - `derive_slot::derive_bip32_from_entropy` (this file)
/// - `derive_slot::derive_bip32_at_path` (this file)
/// - `synthesize::synthesize_multisig_full`
/// - `parse_descriptor::bind_full_mode`
/// - `cmd::bundle::bundle_run_unified_descriptor` (Phrase + Entropy arms)
/// - `cmd::derive_child::run` (Phrase master)
///
/// Per-site code remains site-specific (input type, network
/// handling, derivation path source, return shape); only the
/// `to_seed` step is consolidated here.
pub fn derive_master_seed(mnemonic: &Mnemonic, passphrase: &str) -> Zeroizing<[u8; 64]> {
    Zeroizing::new(mnemonic.to_seed(passphrase))
}

/// entropy → derive at the template's default path → `DerivedAccount`.
///
/// Thin wrapper over [`derive_bip32_from_entropy_at_path`]: it resolves the
/// path via `template.derivation_path` (the BIP-87 fallback for all multisig
/// templates) and delegates the derivation. `entropy.len()` must be a
/// BIP-39-valid length (16/20/24/28/32 bytes); invalid lengths are rejected as
/// `ToolkitError::Bip39` downstream.
pub(crate) fn derive_bip32_from_entropy(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    template: CliTemplate,
    account: u32,
) -> Result<DerivedAccount, ToolkitError> {
    // Single-sig + multisig-BIP-87 (default) path. `template.derivation_path`
    // returns the BIP-87 fallback `m/87'/coin'/account'` for ALL multisig
    // templates; a non-default `--multisig-path-family` (e.g. bip48) must route
    // through `derive_bip32_from_entropy_at_path` with the family path computed
    // by `resolve_slots` (F3 fix). For BIP-87 the two paths are identical, so
    // this wrapper is behaviourally unchanged for every pre-fix caller.
    let path = template.derivation_path(network, account);
    derive_bip32_from_entropy_at_path(entropy, passphrase, language, network, &path)
}

/// Path-explicit sibling of [`derive_bip32_from_entropy`]: derive the account
/// key at an arbitrary BIP-32 `path` rather than the template default. Lets a
/// non-default `--multisig-path-family` (BIP-48: `m/48'/coin'/account'/script'`)
/// reach the actual derivation — not just the JSON metadata field — so the
/// emitted mk1/md1 origins match the derived key (F3 fix).
pub(crate) fn derive_bip32_from_entropy_at_path(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    path: &DerivationPath,
) -> Result<DerivedAccount, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` + `bitcoin::bip32::Xpriv`
    // have no Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
    // `rust-bitcoin-xpriv-zeroize-upstream`. Per-function lifetime is bounded
    // and the seed buffer is `Zeroizing<[u8; 64]>` via `derive_master_seed`.
    let mnemonic = Mnemonic::from_entropy_in(language, entropy).map_err(ToolkitError::Bip39)?;
    let seed = derive_master_seed(&mnemonic, passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    let account_xpriv = master
        .derive_priv(&secp, path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let account_xpub = Xpub::from_priv(&secp, &account_xpriv);

    if account_xpub.network != network.network_kind() {
        return Err(ToolkitError::BadInput(format!(
            "derived-xpub network {:?} does not match --network {}; this is a toolkit bug",
            account_xpub.network,
            network.human_name(),
        )));
    }

    let entropy_bytes = entropy.to_vec();
    // Cycle B Phase 3a Path B-lite Site 3 — pin BEFORE moving entropy_bytes
    // into the struct. Vec keeps its heap-data pointer stable across the
    // move, so the pin captured here remains valid for the lifetime of the
    // returned DerivedAccount.
    let entropy_pin = mnemonic_toolkit::mlock::pin_pages_for(&entropy_bytes[..]);
    Ok(DerivedAccount {
        entropy: Zeroizing::new(entropy_bytes),
        master_fingerprint,
        account_xpub,
        // wave2 T1: confine the account Xpriv in the move-only scrub-on-drop
        // newtype IMMEDIATELY (the `account_xpub`/network-guard reads above
        // happen on the bare local before this wrap).
        account_xpriv: ScrubbedXpriv::new(account_xpriv),
        account_path: path.clone(),
        _entropy_pin: entropy_pin,
    })
}

/// SPEC-A v0.6.1: path-driven sibling of `derive_bip32_from_entropy`.
///
/// entropy → mnemonic-in-language → seed → master xpriv → derive at the
/// supplied `--path` → return the leaf xpriv. Used by `cmd::convert`'s
/// `phrase`/`entropy` → `wif` edge (SPEC `§2`).
///
/// `path` may be at any BIP-32 depth; no normative depth assertion is made
/// (the caller is responsible for supplying a path that produces a leaf
/// privkey suitable for the downstream emission). Network-mismatch checks
/// in the parent `derive_bip32_from_entropy` are intentionally NOT
/// duplicated here — that helper guards the BIP-39 → account flow against a
/// toolkit-bug class; the path-driven flow inherits the same guarantees
/// from `Xpriv::new_master`.
pub(crate) fn derive_bip32_at_path(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    path: &DerivationPath,
) -> Result<Xpriv, ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` + `bitcoin::bip32::Xpriv`
    // have no Drop+Zeroize. FOLLOWUPS: `rust-bip39-mnemonic-zeroize-upstream`,
    // `rust-bitcoin-xpriv-zeroize-upstream`. Per-function lifetime is bounded.
    let mnemonic = Mnemonic::from_entropy_in(language, entropy).map_err(ToolkitError::Bip39)?;
    let seed = derive_master_seed(&mnemonic, passphrase);

    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    master
        .derive_priv(&secp, path)
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))
}

/// SPEC §4.5 (own-account subset-search, P0) — a MOVE-ONLY RAII scrub
/// wrapper around a `bitcoin::bip32::Xpriv` so an over-supply own-key
/// derivation loop never holds a bare, un-scrubbed `Xpriv` past its read.
///
/// `bitcoin::bip32::Xpriv` is `Copy` with no `Drop`/`Zeroize` upstream
/// (FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`); its `private_key:
/// secp256k1::SecretKey` and `chain_code: ChainCode` are `pub`. This
/// newtype adds a Drop-time scrub: `SecretKey::non_secure_erase()` on the
/// private key + a VOLATILE zero-write over the 32 `chain_code` bytes (a
/// plain `= ChainCode::from([0u8;32])` assignment is an elidable dead store
/// — `core::ptr::write_volatile` over `ChainCode::as_mut_ptr()` is not). The
/// non-secret metadata (`network`/`depth`/`parent_fingerprint`/
/// `child_number`) is left as-is.
///
/// **Move-only by construction.** Rust forbids `Copy` + `Drop` (E0184), so
/// the `Drop` impl makes a `#[derive(Copy)]` a hard compile error AT THE
/// `impl Drop` SITE — `Copy` is therefore structurally impossible while the
/// scrub exists (compiler-enforced, not test-enforced). `Clone` is
/// deliberately NOT derived. The only API is a by-value `new` + `&self`
/// accessors (`xpub`, `fingerprint`) — there is NO escape hatch for the
/// inner `Xpriv`.
///
/// `Clone`-absence is pinned at COMPILE TIME by the hand-rolled trait-based
/// static assertion in the `const _: fn()` block in the `scrub_tests` module
/// below (the same `AmbiguousIfImpl<_>` conflicting-blanket-impl pattern
/// `static_assertions::assert_not_impl_any!` uses): the qualified
/// `<ScrubbedXpriv as AmbiguousIfImpl<_>>::some_item` call only resolves
/// while the type is NOT `Clone`. (`static_assertions` is not a dependency,
/// and a
/// `compile_fail` doctest does NOT run here because `derive_slot` is a
/// BINARY-private module in normal builds — rustdoc collects doctests only
/// from the library target, and the lib mounts `derive_slot` solely under
/// `cfg(fuzzing)`. The trait-based assertion runs in the normal binary
/// build, so it is the load-bearing guard.)
///
/// SAFETY: third-party-blocked (tracked caveat
/// `rust-bitcoin-xpriv-zeroize-upstream`): the by-value `Xpriv` returned by
/// upstream `derive_priv` and secp internals are out of this newtype's reach;
/// `non_secure_erase` is best-effort.
//
// DO NOT add Clone/Copy/into_inner/Deref<Xpriv> — re-opens the Copy-escape.
//
// P0 (this phase) is ADDITIVE — `ScrubbedXpriv` + the xpub-only helpers have
// no production caller yet; P2 (own-only subset-search in `cmd/restore.rs`)
// consumes `derive_account_xpub_only`/`derive_accounts_xpub_only`. Until then
// they are exercised only by `scrub_tests`, so the binary build sees them as
// dead code; the `#[allow(dead_code)]` is removed when P2 wires the consumer.
#[allow(dead_code)]
pub struct ScrubbedXpriv(Xpriv);

#[allow(dead_code)] // P0-additive; consumed by P2 (see struct note).
impl ScrubbedXpriv {
    /// Take ownership of `xpriv` by value, confining it inside the move-only
    /// wrapper. The scrub runs when the returned `ScrubbedXpriv` drops.
    pub fn new(xpriv: Xpriv) -> Self {
        ScrubbedXpriv(xpriv)
    }

    /// Read the PUBLIC account xpub. `&self`-borrowing — the inner `Xpriv`
    /// never escapes.
    pub fn xpub(&self, secp: &Secp256k1<All>) -> Xpub {
        Xpub::from_priv(secp, &self.0)
    }

    /// Read the master/account fingerprint. `&self`-borrowing.
    pub fn fingerprint(&self, secp: &Secp256k1<All>) -> Fingerprint {
        self.0.fingerprint(secp)
    }

    /// CONTROLLED escape hatch for the DELIBERATE `convert --to xprv` emission
    /// (`Xprv ∈ is_secret_bearing`). Returns the rendered xprv string wrapped in
    /// [`SecretString`] (length-only redacting Debug + scrub-on-drop) so the
    /// rendered secret never lingers un-scrubbed and never leaks via `{:?}`/
    /// panic. String-only — NO `Xpriv` handle escapes; byte-identical to the
    /// old `account_xpriv.to_string()` (`Xpriv::to_string`).
    // DO NOT widen to expose the Xpriv handle (would re-open the Copy-escape).
    pub fn expose_xprv_string(&self) -> SecretString {
        SecretString::new(self.0.to_string())
    }
}

// wave2 T1: REDACTING Debug — `DerivedAccount` derives `Debug` and now carries
// a `ScrubbedXpriv` field, so the type needs a `Debug` impl. It MUST NOT be a
// `#[derive(Debug)]` (that would render the inner `Xpriv`'s full secret into
// `{:?}`/panic — the exact leak class this newtype confines). Length/identity-
// free, like `SecretString`'s redacting Debug.
impl std::fmt::Debug for ScrubbedXpriv {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("ScrubbedXpriv(<redacted>)")
    }
}

impl Drop for ScrubbedXpriv {
    fn drop(&mut self) {
        // 1) Scrub the spending secret. `SecretKey::non_secure_erase` is the
        //    upstream best-effort erase (secp256k1 0.29.x).
        self.0.private_key.non_secure_erase();
        // 2) VOLATILE zero-write over the 32 chain_code bytes. A plain
        //    assignment (`self.0.chain_code = ChainCode::from([0u8;32])`) is a
        //    dead store the optimizer may elide since `self` is dropping;
        //    `write_volatile` is guaranteed not elided. `ChainCode`'s
        //    `as_mut_ptr()` (bitcoin-internals `impl_array_newtype!`) yields a
        //    `*mut u8` to the backing `[u8; 32]`.
        let cc_ptr = self.0.chain_code.as_mut_ptr();
        // SAFETY: `cc_ptr` points at a live, 32-byte, properly-aligned `[u8;
        // 32]` owned by `self.0.chain_code` (we hold `&mut self`). Each
        // in-bounds byte is written exactly once; `u8` has no invalid
        // bit-patterns and no Drop, so volatile zero-writes are sound.
        for i in 0..32 {
            unsafe {
                core::ptr::write_volatile(cc_ptr.add(i), 0u8);
            }
        }
    }
}

/// SPEC §4.5 (P0) — PUBLIC-ONLY account derivation. Same parameter shape as
/// [`derive_bip32_from_entropy_at_path`], but the derived account `Xpriv` is
/// wrapped in [`ScrubbedXpriv`] IMMEDIATELY and only its PUBLIC projection
/// `(Xpub, Fingerprint)` is returned — the `Xpriv` NEVER escapes this fn and
/// is scrubbed on drop at fn exit. Used by the over-supply own-account
/// subset-search loop (which would otherwise hold `K_own` bare, un-scrubbed
/// `Xpriv`s). ADDITIVE — does not touch `DerivedAccount.account_xpriv` or the
/// existing `derive_bip32_from_entropy_at_path` callers (the broader 7-site
/// lift is FOLLOWUP `derive-slot-account-xpriv-scrub-confinement`).
#[allow(dead_code)] // P0-additive; consumed by P2 (see ScrubbedXpriv note).
pub fn derive_account_xpub_only(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    path: &DerivationPath,
) -> Result<(Xpub, Fingerprint), ToolkitError> {
    let secp = Secp256k1::new();
    let (master, master_fingerprint) =
        derive_master_for_xpub_only(entropy, passphrase, language, network, &secp)?;
    derive_one_xpub_only(&master, &secp, network, path, master_fingerprint)
    // `master` (ScrubbedXpriv) drops here → scrub.
}

/// Fan-out form (SPEC §4.5, encouraged) — derive the shared master ONCE for a
/// SLICE of `paths` (own candidates share the path family) and return per-path
/// `(Xpub, Fingerprint)`. Each account `Xpriv` is wrapped in [`ScrubbedXpriv`]
/// and scrubbed before the next iteration; the master is held in a single
/// `ScrubbedXpriv` for the loop and scrubbed at fn exit. Equivalent to calling
/// [`derive_account_xpub_only`] once per path (proven byte-identical by test),
/// but derives the seed/master a single time.
pub fn derive_accounts_xpub_only(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    paths: &[DerivationPath],
) -> Result<Vec<(Xpub, Fingerprint)>, ToolkitError> {
    let secp = Secp256k1::new();
    let (master, master_fingerprint) =
        derive_master_for_xpub_only(entropy, passphrase, language, network, &secp)?;
    let mut out = Vec::with_capacity(paths.len());
    for path in paths {
        out.push(derive_one_xpub_only(
            &master,
            &secp,
            network,
            path,
            master_fingerprint,
        )?);
    }
    Ok(out)
}

/// Shared seed→master step for the xpub-only helpers. Returns the master
/// wrapped in [`ScrubbedXpriv`] (so it scrubs at the caller's fn exit) plus
/// the master fingerprint. The seed buffer is `Zeroizing` via
/// [`derive_master_seed`].
#[allow(dead_code)] // P0-additive; consumed by P2 (see ScrubbedXpriv note).
fn derive_master_for_xpub_only(
    entropy: &[u8],
    passphrase: &str,
    language: Bip39Language,
    network: CliNetwork,
    secp: &Secp256k1<All>,
) -> Result<(ScrubbedXpriv, Fingerprint), ToolkitError> {
    // SAFETY: third-party-blocked — `bip39::Mnemonic` has no Drop+Zeroize
    // (FOLLOWUP `rust-bip39-mnemonic-zeroize-upstream`). The seed buffer is
    // `Zeroizing<[u8; 64]>`; the master `Xpriv` is confined to `ScrubbedXpriv`.
    let mnemonic = Mnemonic::from_entropy_in(language, entropy).map_err(ToolkitError::Bip39)?;
    let seed = derive_master_seed(&mnemonic, passphrase);

    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(secp);
    Ok((ScrubbedXpriv::new(master), master_fingerprint))
}

/// Derive ONE account from an already-scrubbed master, returning only the
/// PUBLIC `(account_xpub, master_fingerprint)`. The account `Xpriv` is wrapped
/// in [`ScrubbedXpriv`] for its read and scrubbed before return. Keeps the
/// network-mismatch guard from [`derive_bip32_from_entropy_at_path`].
#[allow(dead_code)] // P0-additive; consumed by P2 (see ScrubbedXpriv note).
fn derive_one_xpub_only(
    master: &ScrubbedXpriv,
    secp: &Secp256k1<All>,
    network: CliNetwork,
    path: &DerivationPath,
    master_fingerprint: Fingerprint,
) -> Result<(Xpub, Fingerprint), ToolkitError> {
    // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` has no Drop+Zeroize
    // (FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`). `derive_priv` returns a
    // by-value `Xpriv` temp; we wrap it IMMEDIATELY in `ScrubbedXpriv` so it is
    // scrubbed (private_key.non_secure_erase + volatile chain_code zero) at the
    // end of this expression's scope.
    let account = ScrubbedXpriv::new(
        master
            .0
            .derive_priv(secp, path)
            .map_err(|e| ToolkitError::Bitcoin(BitcoinErrorKind::Bip32(e)))?,
    );
    let account_xpub = account.xpub(secp);
    if account_xpub.network != network.network_kind() {
        return Err(ToolkitError::BadInput(format!(
            "derived-xpub network {:?} does not match --network {}; this is a toolkit bug",
            account_xpub.network,
            network.human_name(),
        )));
    }
    Ok((account_xpub, master_fingerprint))
    // `account` (ScrubbedXpriv) drops here → scrub.
}

#[cfg(test)]
mod scrub_tests {
    use super::*;
    use bip39::Language as Bip39Language;
    use bitcoin::bip32::DerivationPath;
    use std::str::FromStr;

    // ========================================================================
    // TDD-1 — COMPILE-TIME move-only / non-clonable assertion.
    //
    // `Copy` is already structurally impossible: `impl Drop for ScrubbedXpriv`
    // makes `#[derive(Copy)]` a hard E0184 compile error at the Drop site. The
    // only remaining axis to pin is `Clone` (a manual `impl Clone` would NOT
    // trip E0184). We pin `!Clone` at compile time with the SAME pattern
    // `static_assertions::assert_not_impl_any!` uses internally — TWO
    // overlapping TRAIT impls of `AmbiguousIfImpl<A>` (one blanket, one gated
    // on `Clone`). Both are trait methods (no inherent-method shadowing), so if
    // `ScrubbedXpriv` ever becomes `Clone`, BOTH impls apply and the
    // `T::some_item()` call is AMBIGUOUS (E0034) ⇒ this module FAILS TO COMPILE
    // and the whole binary-test target goes RED. We cannot add
    // `static_assertions` as a dep, and a `compile_fail` doctest does NOT run
    // for this binary-private module (rustdoc collects doctests only from the
    // lib target; the lib mounts `derive_slot` solely under `cfg(fuzzing)`).
    // This `const _: fn()` block is the load-bearing move-only guard.
    // ========================================================================
    const _: fn() = || {
        trait AmbiguousIfImpl<A> {
            fn some_item() {}
        }
        // Blanket impl — ALWAYS applies (the `()` anchor arm). `ScrubbedXpriv`
        // is `Sized`, so no `?Sized` is needed (and clippy's
        // `needless_maybe_sized` would reject it next to the `Clone` bound).
        impl<T> AmbiguousIfImpl<()> for T {}
        // Extra impl — applies ONLY when `T: Clone`, parameterised on a
        // DISTINCT marker `Invalid`. When both apply, the inferred `<_>` in
        // the qualified call below cannot be uniquely resolved (E0283:
        // "multiple impls satisfying ... AmbiguousIfImpl<_>").
        struct Invalid;
        impl<T: Clone> AmbiguousIfImpl<Invalid> for T {}

        // Qualified call with an INFERRED type-arg `<_>` — this is the form
        // `static_assertions::assert_not_impl_any!` uses. It forces the
        // compiler to UNIFY `A`; ambiguous ⇒ compile error IFF
        // `ScrubbedXpriv: Clone`. Resolves cleanly when it is NOT `Clone`.
        let _ = <ScrubbedXpriv as AmbiguousIfImpl<_>>::some_item;
    };

    // Trezor canonical 24-word vector → 32-zero-byte entropy.
    const TREZOR_24_ENTROPY: [u8; 32] = [0u8; 32];
    // "abandon × 11 about" → 16-zero-byte entropy (12-word canonical).
    const ZERO_16: [u8; 16] = [0u8; 16];

    fn paths() -> Vec<DerivationPath> {
        vec![
            DerivationPath::from_str("m/84'/0'/0'").unwrap(),
            DerivationPath::from_str("m/48'/0'/0'/2'").unwrap(),
            DerivationPath::from_str("m/87'/0'/0'").unwrap(),
            DerivationPath::from_str("m/49'/0'/3'").unwrap(),
        ]
    }

    /// TDD-2 (no-`Xpriv`-escape, type-level). `derive_account_xpub_only`
    /// compiles and returns ONLY public material `(Xpub, Fingerprint)`. The
    /// `Xpriv` is confined to the fn body (it is wrapped in `ScrubbedXpriv`
    /// immediately and never escapes — see the impl). This test pins the
    /// public return type; the confinement is structural (no `Xpriv` in the
    /// signature).
    #[test]
    fn derive_account_xpub_only_returns_public_types() {
        let (xpub, fp): (Xpub, bitcoin::bip32::Fingerprint) = derive_account_xpub_only(
            &TREZOR_24_ENTROPY,
            "",
            Bip39Language::English,
            CliNetwork::Mainnet,
            &DerivationPath::from_str("m/84'/0'/0'").unwrap(),
        )
        .unwrap();
        assert!(xpub.to_string().starts_with("xpub6"));
        // Fingerprint is the 4-byte master fingerprint; non-empty string form.
        assert_eq!(fp.to_string().len(), 8);
    }

    /// TDD-3 (byte-identical golden — the funds-safety regression guard).
    /// `derive_account_xpub_only`'s `(xpub, fingerprint)` MUST EQUAL the
    /// existing bare-`Xpriv` `derive_bip32_from_entropy_at_path`'s
    /// `account_xpub` + `master_fingerprint` for the SAME inputs, over
    /// several public BIP-39 vectors + paths (incl. a hardened multisig path
    /// `m/48'/0'/0'/2'` and a single-sig `m/84'/0'/0'`). Proves the
    /// scrub-confining helper did NOT perturb the derived keys. This test is
    /// genuinely RED if the keys differ (it asserts equality of both the xpub
    /// and the fingerprint against the golden path).
    #[test]
    fn derive_account_xpub_only_byte_identical_to_bare_helper() {
        for entropy in [&TREZOR_24_ENTROPY[..], &ZERO_16[..]] {
            for passphrase in ["", "TREZOR"] {
                for network in [CliNetwork::Mainnet, CliNetwork::Testnet] {
                    for path in paths() {
                        let golden = derive_bip32_from_entropy_at_path(
                            entropy,
                            passphrase,
                            Bip39Language::English,
                            network,
                            &path,
                        )
                        .unwrap();
                        let (xpub, fp) = derive_account_xpub_only(
                            entropy,
                            passphrase,
                            Bip39Language::English,
                            network,
                            &path,
                        )
                        .unwrap();
                        assert_eq!(
                            xpub, golden.account_xpub,
                            "xpub differs: entropy={:?} pass={:?} net={:?} path={}",
                            entropy, passphrase, network, path
                        );
                        assert_eq!(
                            fp, golden.master_fingerprint,
                            "fingerprint differs: entropy={:?} pass={:?} net={:?} path={}",
                            entropy, passphrase, network, path
                        );
                    }
                }
            }
        }
    }

    /// TDD-4 (fan-out parity). The shared-master fan-out form
    /// `derive_accounts_xpub_only` derives the master ONCE for a slice of
    /// paths and returns per-path `(Xpub, Fingerprint)` == the single-account
    /// form for each path.
    #[test]
    fn derive_accounts_xpub_only_matches_single_account_form() {
        let ps = paths();
        let fanned = derive_accounts_xpub_only(
            &TREZOR_24_ENTROPY,
            "",
            Bip39Language::English,
            CliNetwork::Mainnet,
            &ps,
        )
        .unwrap();
        assert_eq!(fanned.len(), ps.len());
        for (i, path) in ps.iter().enumerate() {
            let single = derive_account_xpub_only(
                &TREZOR_24_ENTROPY,
                "",
                Bip39Language::English,
                CliNetwork::Mainnet,
                path,
            )
            .unwrap();
            assert_eq!(fanned[i], single, "fan-out[{}] != single for {}", i, path);
        }
    }

    /// TDD — the network-mismatch guard is preserved (the helper keeps the
    /// "derived-xpub network does not match" check; this is a structural
    /// inheritance — both helpers derive against the same `network`).
    /// We assert a successful derivation produces a network-consistent xpub.
    #[test]
    fn derive_account_xpub_only_testnet_yields_tpub() {
        let (xpub, _fp) = derive_account_xpub_only(
            &TREZOR_24_ENTROPY,
            "",
            Bip39Language::English,
            CliNetwork::Testnet,
            &DerivationPath::from_str("m/84'/0'/0'").unwrap(),
        )
        .unwrap();
        assert!(xpub.to_string().starts_with("tpub"));
    }

    /// TDD-1 (move-only, runtime witness). `ScrubbedXpriv::new` takes the
    /// `Xpriv` BY VALUE and exposes only `&self` accessors. The compile-time
    /// proof that it is NOT `Copy`/`Clone` lives in the `const _: fn()` block
    /// above (`Copy` is E0184-blocked by `impl Drop`; `Clone`-absence is the
    /// `AmbiguousIfImpl<_>` assertion). This runtime test exercises the
    /// `&self` accessor surface + the scrub-on-drop path (Drop runs at end of
    /// scope).
    #[test]
    fn scrubbed_xpriv_self_accessors_and_drop() {
        let secp = Secp256k1::new();
        let seed = [7u8; 32];
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let scrubbed = ScrubbedXpriv::new(master);
        let xpub = scrubbed.xpub(&secp);
        let fp = scrubbed.fingerprint(&secp);
        // Public outputs match the bare upstream derivation of the same key.
        assert_eq!(xpub, Xpub::from_priv(&secp, &master));
        assert_eq!(fp, master.fingerprint(&secp));
        // `scrubbed` drops here → private_key.non_secure_erase() + volatile
        // chain_code zero-write run. (Best-effort; see the impl SAFETY note.)
    }

    /// wave2 T1 — `ScrubbedXpriv::expose_xprv_string` (the CONTROLLED escape
    /// hatch for the DELIBERATE `convert --to xprv` emission). The returned
    /// `SecretString`:
    ///  (a) renders VERBATIM via Display — byte-identical to the old
    ///      `account_xpriv.to_string()` (`Xpriv::to_string`); this is the
    ///      funds-fidelity guard for the convert.rs:1314 reader migration.
    ///  (b) has a length-only REDACTING Debug — `{:?}` never leaks the xprv
    ///      substring (so it cannot linger via a panic/log).
    /// No `Xpriv` handle escapes — string only.
    #[test]
    fn expose_xprv_string_debug_is_redacting_and_display_is_verbatim() {
        let seed = [9u8; 32];
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        // The canonical string the OLD bare-field reader produced.
        let canonical = master.to_string();
        assert!(canonical.starts_with("xprv"));

        let scrubbed = ScrubbedXpriv::new(master);
        let exposed = scrubbed.expose_xprv_string();

        // (a) Display / to_string is BYTE-IDENTICAL to the bare Xpriv string.
        assert_eq!(exposed.to_string(), canonical);
        assert_eq!(&*exposed, canonical.as_str());

        // (b) Debug is length-only redacted — never the xprv substring.
        let dbg = format!("{exposed:?}");
        assert!(
            !dbg.contains(&canonical),
            "expose_xprv_string Debug leaked the xprv: {dbg}"
        );
        assert!(
            !dbg.contains("xprv"),
            "expose_xprv_string Debug leaked an xprv prefix: {dbg}"
        );
        assert!(
            dbg.contains("redacted"),
            "Debug should mark redaction: {dbg}"
        );
    }
}
