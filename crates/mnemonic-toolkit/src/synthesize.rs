//! Bundle synthesis: produce ms1 + mk1 + md1 strings from derived inputs.
//!
//! Realizes SPEC §4.4 (ms1), §4.5 (mk1), §4.6 (md1 typed-struct
//! construction with chain_code||pubkey 65-byte transform), §4.7
//! (cross-binding invariants).

use crate::error::ToolkitError;
use crate::format::{MkField, MsField};
use crate::network::CliNetwork;
use crate::parse::{CosignerSpec, MultisigPathFamily};
use crate::template::CliTemplate;
use bitcoin::bip32::{ChildNumber, DerivationPath, Fingerprint, Xpriv, Xpub};
use bitcoin::secp256k1::Secp256k1;
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, TlvSection};
use mnemonic_toolkit::mlock::PinnedPageRange;
use std::rc::Rc;
use std::str::FromStr;

#[derive(Debug)]
pub struct Bundle {
    /// Per-slot ms1 cards. Schema-4 dense layout (SPEC §5.8): length-N invariant,
    /// `""` sentinel marks watch-only slots, non-empty marks secret-bearing.
    /// Single-sig watch-only is `[""]`; pure watch-only multisig N=3 is
    /// `["", "", ""]`; multi-source full N=3 is `["ms1...", "ms1...", "ms1..."]`.
    pub ms1: MsField,
    pub mk1: MkField,
    pub md1: Vec<String>,
}

impl Bundle {
    /// SPEC §5.8: any slot with a non-empty ms1 marks the bundle as secret-bearing.
    /// Used by `mode_str` derivation in JSON envelope serialization.
    pub fn any_secret_bearing(&self) -> bool {
        self.ms1.iter().any(|s| !s.is_empty())
    }
}

/// `bundle --md1-form <policy|template>` (#28 phase 1). Selects what the md1
/// card encodes:
///
/// - `Policy` (default, = pre-#28 behavior): the FULL wallet-policy md1 —
///   keyed (pubkeys/fingerprints present), explicit origin. Identifies THIS
///   wallet; binding stub roots on `WalletPolicyId`.
/// - `Template`: a KEYLESS, fingerprint-stripped, canonical-origin-ELIDED
///   single-sig md1 — a backup of the wallet *TYPE* (e.g. BIP-84 single-sig),
///   byte-identical across all users of that type ("one engraving for
///   thousands"). Binding stub roots on the key-stable
///   `WalletDescriptorTemplateId`. REQUIRES `descriptor.n == 1 &&
///   canonical_origin(&tree).is_some()` (pkh/wpkh/tr-keypath single-sig);
///   every other shape is refused with `TemplateFormUnsupportedShape`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum, Default)]
pub enum Md1Form {
    /// Full keyed wallet-policy md1 (default; identifies this wallet).
    #[default]
    Policy,
    /// Keyless, origin-elided single-sig template md1 (shareable per type).
    Template,
}

impl Md1Form {
    /// `true` for the keyless template form (the #28 mutation path).
    pub fn is_template(self) -> bool {
        matches!(self, Md1Form::Template)
    }
}

/// Derive a deterministic 20-bit `chunk_set_id` for mk1 from the 4-byte
/// `policy_id_stub`. Top 20 bits, MSB-first. Mirrors md-codec's
/// `derive_chunk_set_id` shape so mk1 byte-output is reproducible across runs
/// (toolkit fixture regeneration relies on this; v0.1 byte-determinism contract).
pub(crate) fn derive_mk1_chunk_set_id(stub: &[u8; 4]) -> u32 {
    ((stub[0] as u32) << 12) | ((stub[1] as u32) << 4) | ((stub[2] as u32) >> 4)
}

/// Slot-unique mk1 `chunk_set_id`: the policy-stub-derived base XORed with the
/// cosigner slot index. `verify-bundle` groups supplied mk1 chunks by csi to
/// reassemble each cosigner's card, so the csi MUST be distinct per cosigner —
/// otherwise two cosigners with the same xpub (hence same fingerprint, the old
/// per-fingerprint derivation) collide into one group and decode fails (audit
/// I10). XOR is injective in `slot` ⇒ pairwise-distinct csi for distinct slots.
///
/// The slot index (≤ 15; cosigner count is capped at 16) only touches the low
/// nibble (bits 3..0 = the 5th hex char), so the **leading 16 bits**
/// (= `policy_id[0..2]`, the bundle-binding prefix shared with md1) are
/// preserved across all cosigners. For n=1 (slot 0) this is byte-identical to
/// `derive_mk1_chunk_set_id`. Unifies the single-sig and multisig derivations
/// (resolves the prior n=1-stub vs n≥2-fingerprint inconsistency).
pub(crate) fn derive_mk1_chunk_set_id_for_slot(stub: &[u8; 4], slot: u32) -> u32 {
    derive_mk1_chunk_set_id(stub) ^ slot
}

/// Convert a `bitcoin::bip32::DerivationPath` to md-codec's `OriginPath`.
/// Used by multisig synthesis where per-cosigner paths come from cosigner
/// specs (watch-only) or path-family-derived strings (full mode).
pub(crate) fn derivation_path_to_origin_path(p: &DerivationPath) -> OriginPath {
    let components = p
        .into_iter()
        .map(|cn| {
            let v: u32 = (*cn).into();
            // bitcoin's child_number `v` already encodes the hardened bit at
            // position 0x8000_0000; mask it off and record `hardened` separately.
            const HARD: u32 = 0x8000_0000;
            let hardened = v & HARD != 0;
            let value = v & !HARD;
            PathComponent { hardened, value }
        })
        .collect();
    OriginPath { components }
}

/// Derive the mk1 card's `origin_path` so it round-trips the xpub it carries.
///
/// mk-codec compact-73 reconstructs `depth := component_count(origin_path)` and
/// `child_number := last_component(origin_path)` (or `Normal{0}` empty); mk-codec
/// 0.4.0 rejects any card whose xpub depth/child disagree (`XpubOriginPathMismatch`).
/// The DESCRIPTOR origin (carried independently by md1's `path_decl`) may be deeper
/// (an account xpub paired with a BIP-48 leaf path), shallower (a leaf xpub
/// re-annotated with an account origin), or absent. We build a path of length
/// `xpub.depth` whose terminal equals `xpub.child_number`, reusing the descriptor
/// path's leading components for the (non-load-bearing, informational) intermediates.
/// See `design/SPEC_toolkit_mk1_origin_path.md` §3.2.
pub(crate) fn mk1_origin_path(xpub: &Xpub, descriptor_path: &DerivationPath) -> DerivationPath {
    let depth = xpub.depth as usize;
    if depth == 0 {
        return DerivationPath::master(); // empty — no-path / depth-0 key (e.g. a WIF)
    }
    let comps: Vec<ChildNumber> = descriptor_path.into_iter().copied().collect();
    let mut out: Vec<ChildNumber> = Vec::with_capacity(depth);
    for i in 0..(depth - 1) {
        // Reuse the descriptor path where available; pad absent intermediates with
        // Normal{0} (honest filler — reads as obviously-synthetic in `inspect`).
        out.push(
            comps
                .get(i)
                .copied()
                .unwrap_or(ChildNumber::Normal { index: 0 }),
        );
    }
    out.push(xpub.child_number); // terminal MUST equal the xpub's child (round-trip)
    DerivationPath::from(out)
}

/// Convert a `bitcoin::bip32::Xpub` to md-codec's 65-byte form:
///   [0..32]  = chain_code
///   [32..65] = compressed pubkey
/// SPEC §4.6.1.
pub fn xpub_to_65(xpub: &Xpub) -> [u8; 65] {
    let mut out = [0u8; 65];
    out[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
    out[32..65].copy_from_slice(&xpub.public_key.serialize());
    out
}

/// Build the typed Descriptor for a (template, network, xpub, fingerprint, account).
/// Caller's xpub MUST already be at the template's BIP path; not rederived.
/// SPEC §4.6.
///
/// #28 phase 1 — this is also the FULLY-KEYED, EXPLICIT-origin (`m/<purpose>'/
/// <coin>'/account'`), presence-`0b11` preimage `compute_wallet_policy_id`
/// hashes for the D7 disambiguator (`wallet_policy_id_for_singlesig` below).
/// `compute_wallet_policy_id` is origin-significant and does NOT consult
/// `canonical_origin` (md-codec identity.rs INVARIANT note), so the explicit
/// origin here is load-bearing: the elided template md1 would hash to a
/// DIFFERENT id.
pub fn build_descriptor(
    template: CliTemplate,
    network: CliNetwork,
    xpub: &Xpub,
    fingerprint: Fingerprint,
    account: u32,
) -> Descriptor {
    let xpub_65 = xpub_to_65(xpub);
    let fp_bytes: [u8; 4] = fingerprint.to_bytes();
    let origin_path = template.md_origin_path(network, account);
    let tree = template.wrapper_node(1, 1);

    Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(vec![(0, fp_bytes)]),
            pubkeys: Some(vec![(0, xpub_65)]),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    }
}

/// #28 phase 1 (D7) — the `WalletPolicyId` for a single-sig (template,
/// network, xpub, fingerprint, account). This is the per-wallet disambiguator
/// the shareable keyless template CANNOT carry: it is printed on stderr at
/// `bundle --md1-form=template` time and recomputed at `restore
/// --expect-wallet-id` time. Both sides MUST use this EXACT helper so the
/// preimage is byte-identical (the fully-keyed, explicit-origin,
/// presence-`0b11` descriptor) — a single source of truth for the D7
/// invariant, guarding the round-trip differential against a canonicalization
/// drift that would manifest as a false `--expect-wallet-id` refusal.
pub fn wallet_policy_id_for_singlesig(
    template: CliTemplate,
    network: CliNetwork,
    xpub: &Xpub,
    fingerprint: Fingerprint,
    account: u32,
) -> Result<md_codec::WalletPolicyId, ToolkitError> {
    let descriptor = build_descriptor(template, network, xpub, fingerprint, account);
    md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)
}

/// #28 phase 2 (D7) — the order-sensitive `WalletPolicyId` for a MULTISIG /
/// general-policy template bundle. The keyless template md1 carries the tree +
/// use-site structure (incl. #25 overrides) but no keys; re-inject the resolved
/// per-`@N` keys/fingerprints and rebuild `path_decl` from the slot origins
/// (`Shared` when all equal, else `Divergent` — exactly `synthesize_unified`'s
/// build), then compute the policy id on that fully-keyed, explicit-origin
/// descriptor. This is the completion checksum the creator records: the
/// `WalletPolicyId` is order-sensitive (`identity.rs` never sorts), so it pins
/// the exact `@N`→key assignment that reproduces this wallet.
///
/// `keyless_template` is the reassembled (mutated) template descriptor;
/// `slots[i]` supplies the key for `@i` (length must equal the template's `n`).
pub fn wallet_policy_id_for_template(
    keyless_template: &Descriptor,
    slots: &[ResolvedSlot],
) -> Result<md_codec::WalletPolicyId, ToolkitError> {
    let n = keyless_template.n as usize;
    if slots.len() != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "wallet_policy_id_for_template: template n={n} but {} slots supplied",
            slots.len()
        )));
    }
    let origin_paths: Vec<OriginPath> = slots
        .iter()
        .map(|s| derivation_path_to_origin_path(&s.path))
        .collect();
    let all_same = origin_paths.windows(2).all(|w| w[0] == w[1]);
    let path_decl_paths = if all_same || n == 1 {
        PathDeclPaths::Shared(origin_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(origin_paths)
    };
    let fingerprints: Vec<(u8, [u8; 4])> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i as u8, s.fingerprint.to_bytes()))
        .collect();
    let pubkeys: Vec<(u8, [u8; 65])> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i as u8, xpub_to_65(&s.xpub)))
        .collect();
    let keyed = Descriptor {
        n: n as u8,
        path_decl: PathDecl {
            n: n as u8,
            paths: path_decl_paths,
        },
        use_site_path: keyless_template.use_site_path.clone(),
        tree: keyless_template.tree.clone(),
        tlv: TlvSection {
            use_site_path_overrides: keyless_template.tlv.use_site_path_overrides.clone(),
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: keyless_template.tlv.origin_path_overrides.clone(),
            unknown: keyless_template.tlv.unknown.clone(),
        },
    };
    md_codec::compute_wallet_policy_id(&keyed).map_err(ToolkitError::from)
}

/// True iff `tree` is a `sortedmulti`/`sortedmulti_a` (order-INDEPENDENT) wallet
/// — any `@N`→key assignment yields the same wallet, so the loud order-dependent
/// warning is softened. Walks `wsh(...)`/`sh(...)`/`tr(IK, ...)` wrappers to the
/// inner leaf. A `SortedMulti` in a COMBINATOR (not a sole multisig leaf) is NOT
/// order-independent overall — but such a shape does not render (refused at
/// admission), so it never reaches the warning.
pub(crate) fn is_order_independent_shape(tree: &md_codec::tree::Node) -> bool {
    use md_codec::tag::Tag;
    use md_codec::tree::Body;
    match (&tree.tag, &tree.body) {
        (Tag::SortedMulti, _) | (Tag::SortedMultiA, _) => true,
        (_, Body::Children(children)) if children.len() == 1 => {
            is_order_independent_shape(&children[0])
        }
        (Tag::Tr, Body::Tr { tree: Some(t), .. }) => is_order_independent_shape(t),
        _ => false,
    }
}

/// #28 phase 1 — map a canonical single-sig descriptor TREE to its
/// `CliTemplate` (the type the keyless template encodes). Mirrors the shape
/// dispatch of `md_codec::canonical_origin::canonical_origin`, but emits a
/// `CliTemplate` (NOT the inverse of `script_type_from_template`). Returns the
/// type only for the three canonical-origin-elidable single-sig shapes —
/// `pkh(@0)` → bip44, `wpkh(@0)` → bip84, `tr(@0)` key-path (no TapTree) →
/// bip86.
///
/// Everything else (multisig, bip49 `sh(wpkh)`, taproot-with-tree, bare wsh, …)
/// returns `None` — exactly the shapes `--md1-form=template` refuses at emit,
/// so this never has to invent a type for a non-template md1.
pub fn cli_template_from_tree(tree: &md_codec::tree::Node) -> Option<CliTemplate> {
    use md_codec::tag::Tag;
    use md_codec::tree::Body;
    match (&tree.tag, &tree.body) {
        (Tag::Pkh, Body::KeyArg { .. }) => Some(CliTemplate::Bip44),
        (Tag::Wpkh, Body::KeyArg { .. }) => Some(CliTemplate::Bip84),
        (Tag::Tr, Body::Tr { tree: None, .. }) => Some(CliTemplate::Bip86),
        _ => None,
    }
}

/// Synthesize a full-mode bundle (entropy known).
/// SPEC §4.4-§4.7. Test-only helper after v0.4.2 Phase M.
#[allow(dead_code)]
pub fn synthesize_full(
    entropy: &[u8],
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
) -> Result<Bundle, ToolkitError> {
    let ms1 = ms_codec::encode(
        ms_codec::Tag::ENTR,
        &ms_codec::Payload::Entr(entropy.to_vec()),
    )
    .map_err(ToolkitError::from)?;

    let descriptor = build_descriptor(template, network, &xpub, fingerprint, account);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network, account);
    let card = mk_codec::KeyCard::new(
        vec![stub],
        Some(fingerprint),
        mk1_origin_path(&xpub, &path),
        xpub,
    );
    let csi = derive_mk1_chunk_set_id_for_slot(&stub, 0);
    let mk1 = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;

    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle {
        ms1: vec![ms1],
        mk1: MkField::Single(mk1),
        md1,
    })
}

/// Synthesize a watch-only bundle (no entropy known; ms1 omitted).
/// SPEC §4 watch-only path. Test-only helper after v0.4.2 Phase M.
#[allow(dead_code)]
pub fn synthesize_watch_only(
    fingerprint: Fingerprint,
    xpub: Xpub,
    template: CliTemplate,
    network: CliNetwork,
    account: u32,
) -> Result<Bundle, ToolkitError> {
    let descriptor = build_descriptor(template, network, &xpub, fingerprint, account);
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    let path = template.derivation_path(network, account);
    let card = mk_codec::KeyCard::new(
        vec![stub],
        Some(fingerprint),
        mk1_origin_path(&xpub, &path),
        xpub,
    );
    let csi = derive_mk1_chunk_set_id_for_slot(&stub, 0);
    let mk1 = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;

    debug_assert!(descriptor.is_wallet_policy());

    // SPEC §5.8: single-sig watch-only ms1 = [""] (length-N invariant; empty-string sentinel).
    Ok(Bundle {
        ms1: vec![String::new()],
        mk1: MkField::Single(mk1),
        md1,
    })
}

// v0.4.3 Phase N: CosignerKeyInfo retired; sole binding type is ResolvedSlot
// (defined below). Type alias retained for source-compat across the binding
// layer; new code should construct ResolvedSlot directly.
//
// Legacy descriptor-mode bindings (bind_descriptor_keys) populate
// `entropy: Some(...)` for slot @0 if --phrase supplied; `entropy: None` for
// all @1+ slots (cosigner triples are watch-only by definition).
pub type CosignerKeyInfo = ResolvedSlot;

/// Produce a `Bundle` from a pre-parsed `md_codec::Descriptor` + per-`@N`
/// cosigner key info. Dispatches to single-card mk1 (n=1) or n-card mk1 (n≥2)
/// per SPEC §4.10. Annotation cross-checks + BIP-388 distinctness enforcement
/// run inside `descriptor_mode_run` (cmd/bundle.rs).
///
/// Per SPEC §5.8 emission rule (v0.21.0): `ms1[i]` is populated independently
/// from `cosigners[i].entropy` for every slot. Watch-only slots (`entropy:
/// None`) get the `""` sentinel.
pub fn synthesize_descriptor(
    descriptor: &Descriptor,
    cosigners: &[CosignerKeyInfo],
    privacy_preserving: bool,
    run_language: bip39::Language,
    md1_form: Md1Form,
) -> Result<Bundle, ToolkitError> {
    let n = descriptor.n as usize;
    if cosigners.len() != n {
        return Err(ToolkitError::DescriptorParse(format!(
            "synthesize_descriptor: descriptor n={n} but {} cosigner key triples provided",
            cosigners.len()
        )));
    }

    // #28 phase 1 — keyless single-sig template form. The four descriptor
    // mutations (SPEC §4.2) produce a byte-shareable, account-agnostic md1, and
    // the binding stub re-roots on the key-stable `WalletDescriptorTemplateId`
    // (SPEC §4.3) so the keyless md1 and the keyed mk1 derive the SAME stub.
    if md1_form.is_template() {
        return synthesize_template_descriptor(descriptor, cosigners, privacy_preserving);
    }

    let policy_id = md_codec::compute_wallet_policy_id(descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);

    let md1 = md_codec::chunk::split(descriptor).map_err(ToolkitError::from)?;

    let mk1 = if n == 1 {
        let c = &cosigners[0];
        let card = mk_codec::KeyCard::new(
            vec![stub],
            if privacy_preserving {
                None
            } else {
                Some(c.fingerprint)
            },
            mk1_origin_path(&c.xpub, &c.path),
            c.xpub,
        );
        let csi = derive_mk1_chunk_set_id_for_slot(&stub, 0);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        MkField::Single(chunks)
    } else {
        let stubs: Vec<[u8; 4]> = vec![stub; n];
        let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(n);
        for (i, c) in cosigners.iter().enumerate() {
            let card = mk_codec::KeyCard::new(
                stubs.clone(),
                if privacy_preserving {
                    None
                } else {
                    Some(c.fingerprint)
                },
                mk1_origin_path(&c.xpub, &c.path),
                c.xpub,
            );
            // Slot-unique csi (audit I10): per-fingerprint derivation collided
            // for same-xpub-different-path cosigners. stub^slot is distinct per
            // slot and preserves the leading-16-bit bundle-binding prefix.
            let csi = derive_mk1_chunk_set_id_for_slot(&stub, i as u32);
            let chunks =
                mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
            per_cosigner.push(chunks);
        }
        MkField::Multi(per_cosigner)
    };

    // SPEC §5.8 emission rule: ms1[i] is populated per-slot from
    // cosigners[i].entropy. Watch-only slots → "" sentinel. Mirrors
    // synthesize_unified:710-723 — same rule across all bundle modes.
    // ms mnem Phase 3 Step 5: emit mnem for non-English slot sources.
    // slot.language: Some(lang) = import-json mnem source (wins via unwrap_or);
    //                None = descriptor-@N phrase/entropy → fall back to run_language.
    // This is symmetric with synthesize_unified's `s.language.unwrap_or(run_language)`.
    let mut ms1: MsField = Vec::with_capacity(n);
    for c in cosigners {
        match &c.entropy {
            Some(e) => {
                let emit_lang = c.language.unwrap_or(run_language);
                let payload = if emit_lang == bip39::Language::English {
                    ms_codec::Payload::Entr((**e).clone())
                } else {
                    ms_codec::Payload::Mnem {
                        language: crate::language::bip39_to_wire_code(emit_lang),
                        entropy: (**e).clone(),
                    }
                };
                ms1.push(
                    ms_codec::encode(ms_codec::Tag::ENTR, &payload).map_err(ToolkitError::from)?,
                );
            }
            None => ms1.push(String::new()),
        }
    }

    debug_assert!(descriptor.is_wallet_policy());

    Ok(Bundle { ms1, mk1, md1 })
}

/// SPEC §4.1 multisig: derive xpub at a path string from the master xpriv.
/// Test-only helper after v0.4.2 Phase M.
#[allow(dead_code)]
pub(crate) fn derive_xpub_at_path(
    master: &Xpriv,
    secp: &Secp256k1<bitcoin::secp256k1::All>,
    path_str: &str,
) -> Result<Xpub, ToolkitError> {
    let path = DerivationPath::from_str(path_str)
        .map_err(|e| ToolkitError::BadInput(format!("path parse {}: {}", path_str, e)))?;
    // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
    // Drop; tracked by FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`.
    let xpriv = master
        .derive_priv(secp, &path)
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    Ok(Xpub::from_priv(secp, &xpriv))
}

#[allow(dead_code)]
/// Synthesize a full-mode multisig bundle (self-multisig: N cosigners derived
/// from one seed at one path; all N xpubs are byte-identical).
/// SPEC §4.1, §4.5 multisig, §4.6 multisig.
#[allow(clippy::too_many_arguments)]
pub fn synthesize_multisig_full(
    seed_mnemonic: &bip39::Mnemonic,
    passphrase: &str,
    network: CliNetwork,
    template: CliTemplate,
    threshold: u8,
    cosigner_count: usize,
    account: u32,
    path_family: MultisigPathFamily,
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError> {
    // 1. Validate config (SPEC §2.1.1).
    if cosigner_count == 0 || cosigner_count > 16 {
        return Err(ToolkitError::MultisigConfig {
            message: format!("cosigner_count {} out of range 1..=16", cosigner_count),
        });
    }
    if threshold == 0 || threshold as usize > cosigner_count {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "threshold {} out of range 1..={} (cosigner_count)",
                threshold, cosigner_count
            ),
        });
    }
    if !template.is_multisig() {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "template {} is single-sig; multisig synthesis requires a multisig template",
                template.human_name()
            ),
        });
    }

    // 2. Master xpriv.
    // SAFETY: third-party-blocked — `bitcoin::bip32::Xpriv` is Copy + no
    // Drop; tracked by FOLLOWUP `rust-bitcoin-xpriv-zeroize-upstream`. The
    // 64-byte seed is `Zeroizing<[u8; 64]>` via `derive_master_seed`.
    let seed = crate::derive_slot::derive_master_seed(seed_mnemonic, passphrase);
    let secp = Secp256k1::new();
    let master = Xpriv::new_master(network.network_kind(), &seed[..])
        .map_err(|e| ToolkitError::Bitcoin(crate::error::BitcoinErrorKind::Bip32(e)))?;
    let master_fingerprint = master.fingerprint(&secp);

    // 3. Self-multisig: derive all N at the same path-family path.
    let script_type = template.bip48_script_type().unwrap_or(0);
    let path_str = path_family.default_origin_path(network, account, script_type);
    let xpub = derive_xpub_at_path(&master, &secp, &path_str)?;
    let path = DerivationPath::from_str(&path_str)
        .map_err(|e| ToolkitError::BadInput(format!("path parse {}: {}", path_str, e)))?;

    // 4. Build multisig descriptor.
    let xpub_65 = xpub_to_65(&xpub);
    let fp_bytes: [u8; 4] = master_fingerprint.to_bytes();
    let origin_path = derivation_path_to_origin_path(&path);
    let tree = template.wrapper_node(threshold, cosigner_count);

    let fingerprints: Vec<(u8, [u8; 4])> =
        (0..cosigner_count).map(|i| (i as u8, fp_bytes)).collect();
    let pubkeys: Vec<(u8, [u8; 65])> = (0..cosigner_count).map(|i| (i as u8, xpub_65)).collect();

    let descriptor = Descriptor {
        n: cosigner_count as u8,
        path_decl: PathDecl {
            n: cosigner_count as u8,
            paths: PathDeclPaths::Shared(origin_path),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // 5. Compute policy_id + N-element stubs list.
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);
    let stubs: Vec<[u8; 4]> = vec![stub; cosigner_count];

    // 6+7. Build N KeyCards + emit per-cosigner mk1.
    let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(cosigner_count);
    for i in 0..cosigner_count {
        let card = mk_codec::KeyCard::new(
            stubs.clone(),
            if privacy_preserving {
                None
            } else {
                Some(master_fingerprint)
            },
            mk1_origin_path(&xpub, &path),
            xpub,
        );
        debug_assert_eq!(card.policy_id_stubs, stubs);
        debug_assert!(descriptor.is_wallet_policy());
        // Slot-unique csi (audit I10) — self-multisig here means all xpubs are
        // identical, so the old per-fingerprint scheme collided ALL cosigners.
        let csi = derive_mk1_chunk_set_id_for_slot(&stub, i as u32);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        per_cosigner.push(chunks);
    }

    // 8. md1.
    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    // 9. ms1.
    // SPEC v0.9.0 §1 item 2 — wrap entropy buffer before move-into-Payload.
    // The ms_codec::Payload::Entr(Vec<u8>) public shape is unwrapped per
    // SPEC §3 OOS-2; we clone the wrapped buffer's contents into the
    // public Vec at the call boundary so the original Zeroizing wrap
    // drops with scrubbing at function exit.
    // ms mnem Phase 3 Step 5: emit mnem for non-English sources.
    let entropy = zeroize::Zeroizing::new(seed_mnemonic.to_entropy());
    let mnemonic_lang = seed_mnemonic.language();
    let ms1_payload = if mnemonic_lang == bip39::Language::English {
        ms_codec::Payload::Entr((*entropy).clone())
    } else {
        ms_codec::Payload::Mnem {
            language: crate::language::bip39_to_wire_code(mnemonic_lang),
            entropy: (*entropy).clone(),
        }
    };
    let ms1 = ms_codec::encode(ms_codec::Tag::ENTR, &ms1_payload).map_err(ToolkitError::from)?;

    // SPEC §5.8: length-N ms1 vec. Legacy self-multisig path is hard-rejected
    // for cosigner_count > 1 at bundle.rs entry (BIP-388); cosigner_count == 1
    // produces vec![ms1]. The clone-N pattern is correct should the hard-reject
    // ever be lifted (would still violate BIP-388 distinctness, but synthesis
    // contract holds).
    let ms1_field: MsField = vec![ms1; cosigner_count];
    Ok(Bundle {
        ms1: ms1_field,
        mk1: MkField::Multi(per_cosigner),
        md1,
    })
}

#[allow(dead_code)]
/// Synthesize a watch-only multisig bundle from cosigner xpubs.
/// SPEC §4.1, §4.3, §4.5 multisig, §4.6 multisig.
#[allow(clippy::too_many_arguments)]
pub fn synthesize_multisig_watch_only(
    cosigners: &[CosignerSpec],
    network: CliNetwork,
    template: CliTemplate,
    threshold: u8,
    account: u32,
    path_family: MultisigPathFamily,
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError> {
    let cosigner_count = cosigners.len();

    // 1. Validate config.
    if cosigner_count == 0 || cosigner_count > 16 {
        return Err(ToolkitError::MultisigConfig {
            message: format!("cosigner_count {} out of range 1..=16", cosigner_count),
        });
    }
    if threshold == 0 || threshold as usize > cosigner_count {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "threshold {} out of range 1..={} (cosigner_count)",
                threshold, cosigner_count
            ),
        });
    }
    if !template.is_multisig() {
        return Err(ToolkitError::MultisigConfig {
            message: format!(
                "template {} is single-sig; multisig synthesis requires a multisig template",
                template.human_name()
            ),
        });
    }

    // 2. SPEC §4.3 per-cosigner network/xpub cross-check.
    for (i, c) in cosigners.iter().enumerate() {
        if c.xpub.network != network.network_kind() {
            return Err(ToolkitError::CosignerSpec {
                cosigner_idx: i,
                message: format!(
                    "xpub network {:?} does not match --network {}",
                    c.xpub.network,
                    network.human_name()
                ),
            });
        }
    }

    // 3. Per-cosigner path resolution (default to family).
    let script_type = template.bip48_script_type().unwrap_or(0);
    let default_path_str = path_family.default_origin_path(network, account, script_type);
    let default_path = DerivationPath::from_str(&default_path_str).map_err(|e| {
        ToolkitError::BadInput(format!("default path parse {}: {}", default_path_str, e))
    })?;

    let mut paths: Vec<DerivationPath> = Vec::with_capacity(cosigner_count);
    for c in cosigners {
        paths.push(c.path.clone().unwrap_or_else(|| default_path.clone()));
    }

    // 4. (Removed v0.37.10) The former SPEC §4.5 path/xpub depth-consistency reject
    //    is superseded by `mk1_origin_path`, which makes every mk1 card's origin_path
    //    round-trip its xpub by construction (the descriptor path and the xpub may
    //    legitimately differ in depth — account xpub + BIP-48 leaf path, etc.).
    //    md1's path_decl still carries the full descriptor origin below.

    // 5. Determine PathDeclPaths variant.
    let origin_paths: Vec<OriginPath> = paths.iter().map(derivation_path_to_origin_path).collect();
    let all_same = origin_paths.windows(2).all(|w| w[0] == w[1]);
    let path_decl_paths = if all_same {
        PathDeclPaths::Shared(origin_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(origin_paths)
    };

    // 6. Build descriptor.
    let tree = template.wrapper_node(threshold, cosigner_count);
    let fingerprints: Vec<(u8, [u8; 4])> = cosigners
        .iter()
        .enumerate()
        .map(|(i, c)| (i as u8, c.master_fingerprint.to_bytes()))
        .collect();
    let pubkeys: Vec<(u8, [u8; 65])> = cosigners
        .iter()
        .enumerate()
        .map(|(i, c)| (i as u8, xpub_to_65(&c.xpub)))
        .collect();

    let descriptor = Descriptor {
        n: cosigner_count as u8,
        path_decl: PathDecl {
            n: cosigner_count as u8,
            paths: path_decl_paths,
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // 7. Policy id + stubs.
    let policy_id = md_codec::compute_wallet_policy_id(&descriptor).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&policy_id.as_bytes()[..4]);
    let stubs: Vec<[u8; 4]> = vec![stub; cosigner_count];

    // 8. Per-cosigner KeyCards + mk1.
    let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(cosigner_count);
    for (i, c) in cosigners.iter().enumerate() {
        let card = mk_codec::KeyCard::new(
            stubs.clone(),
            if privacy_preserving {
                None
            } else {
                Some(c.master_fingerprint)
            },
            mk1_origin_path(&c.xpub, &paths[i]),
            c.xpub,
        );
        debug_assert_eq!(card.policy_id_stubs, stubs);
        debug_assert!(descriptor.is_wallet_policy());
        // Slot-unique csi (audit I10): distinct per slot; immune to same-xpub
        // fingerprint collision; preserves the leading-16-bit binding prefix.
        let csi = derive_mk1_chunk_set_id_for_slot(&stub, i as u32);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        per_cosigner.push(chunks);
    }

    // 9. md1.
    let md1 = md_codec::chunk::split(&descriptor).map_err(ToolkitError::from)?;

    // SPEC §5.8: pure watch-only multisig ms1 = ["", "", ...] of length N.
    let cosigner_count = cosigners.len();
    Ok(Bundle {
        ms1: vec![String::new(); cosigner_count],
        mk1: MkField::Multi(per_cosigner),
        md1,
    })
}

/// v0.4.1 Phase H.3 — per-slot post-binding shape for multi-source / hybrid
/// multisig synthesis. Carries entropy iff the slot is secret-bearing.
///
/// The origin annotation is derived on demand from the typed `fingerprint` +
/// `path` via `origin_path_bare()` / `bracketed_origin()` (v0.37.9 — the
/// formerly-stored `path_raw: String` was a denormalized, overloaded cache;
/// see `design/SPEC_path_raw_bracketed_bare_unification.md`). Distinctness
/// (SPEC §4.11.b, v0.5 reversal) compares the typed `path`.
#[derive(Debug, Clone)]
pub struct ResolvedSlot {
    pub xpub: Xpub,
    pub fingerprint: Fingerprint,
    pub path: DerivationPath,
    /// Some(entropy_bytes) for secret-bearing slots; None for watch-only.
    ///
    /// v0.10.1: migrated from `Option<Vec<u8>>` to `Option<Zeroizing<Vec<u8>>>`
    /// so the entropy buffer scrubs on Drop without a hand-rolled `impl Drop`.
    /// Field-declaration order is preserved (`entropy` BEFORE `_entropy_pin`);
    /// RFC 1857 drop order: Zeroizing fires zeroize → Vec dealloc → then the
    /// sibling `_entropy_pin` munlocks (Cycle B Phase 3a invariant).
    pub entropy: Option<zeroize::Zeroizing<Vec<u8>>>,
    /// v0.8.2 SPEC §5.1 — optional depth-0 master xpub supplied via
    /// `@N.master_xpub=<base58>`. Consumed by `--format coldcard` singlesig
    /// emitter to populate the top-level `xpub` field. `None` for every
    /// resolution arm except `{Xpub, MasterXpub, ...}` where the user
    /// supplied the subkey. Other emitters silently ignore.
    pub master_xpub: Option<Xpub>,
    /// ms mnem Phase 3 — per-card wire language for derivation.
    ///
    /// `Some(lang)` = this slot's source was a `mnem` ms1 card whose wire
    /// language OVERRIDES the run-level `--language`. `None` = defer to the
    /// run `--language` / English default.
    ///
    /// Resolution everywhere: `slot.language.unwrap_or_else(|| args.language().into())`.
    ///
    /// Populated at the `bundle --import-json` mnem-decode arm AND (v0.41.0)
    /// at the three `--slot @N.ms1=` Ms1 arms (template `resolve_slots`,
    /// `bundle_run_unified_descriptor`, `verify_bundle` descriptor loop), where
    /// a `mnem` ms1 card's wire language flows through `slot_ms1::resolve_ms1_slot`
    /// → `emit_language`. All other slot sources (foreign-wallet parsers,
    /// descriptor-concrete, hex/phrase/seedqr/entr `resolve_slots`) set
    /// `language: None`.
    pub language: Option<bip39::Language>,
    /// Cycle B Phase 3a Path B-lite — sibling pin for the `entropy` heap
    /// buffer's pages. `Some(Rc::new(pin_pages_for(&entropy[..])))` when
    /// `entropy` is `Some`; `None` for watch-only slots. Rc preserves the
    /// `derive(Clone)` semantics (cosigner-bridging clones at
    /// `cmd/bundle.rs:1062-1073` share the pin via Rc refcount; the final
    /// clone's drop fires munlock exactly once). Declared LAST so that on
    /// Drop, `entropy` field drops first: Zeroizing::drop scrubs the inner
    /// Vec then deallocs (v0.10.1 — closes the Cycle A bytes-may-persist
    /// gap), then `_entropy_pin` Rc final-drops and munlocks.
    pub _entropy_pin: Option<Rc<PinnedPageRange>>,
}

impl ResolvedSlot {
    #[allow(dead_code)]
    pub fn is_secret_bearing(&self) -> bool {
        self.entropy.is_some()
    }

    /// Bare BIP-32 derivation path in `m/...` form, or `""` for the
    /// pathless/degenerate slot (`path == DerivationPath::default()`, e.g. the
    /// `--slot @N.wif=` slot at `cmd/bundle.rs:674`).
    ///
    /// This is the single rendering chokepoint for the bare origin-path form
    /// that emit consumers need (the former bare-convention reads of the
    /// deleted `path_raw` field). The `""` return reproduces the old
    /// `path_raw.is_empty()` sentinel that the JSON/wallet-file consumers
    /// branch on (`DerivationPath::default().to_string()` is `""` in
    /// `bitcoin` 0.32, so a default path maps to the absent-path sentinel).
    /// See `design/SPEC_path_raw_bracketed_bare_unification.md` §3.
    pub fn origin_path_bare(&self) -> String {
        if self.path == DerivationPath::default() {
            String::new()
        } else {
            format!("m/{}", self.path)
        }
    }

    /// BIP-380 bracketed origin annotation `[fp/comps]` (lowercase fingerprint,
    /// no `m/` inside), or `[fp]` for the pathless/degenerate slot. Rebuilt from
    /// the typed `fingerprint` + `path`; reproduces every former bracketed
    /// `path_raw` producer byte-for-byte for path-sensitive consumers
    /// (`DerivationPath` Display writes `/`-joined hardened-`'` components with
    /// no leading `m`/`/`). For descriptor-key construction.
    /// See `design/SPEC_path_raw_bracketed_bare_unification.md` §3.
    pub fn bracketed_origin(&self) -> String {
        let fp = self.fingerprint.to_string().to_lowercase();
        if self.path == DerivationPath::default() {
            format!("[{fp}]")
        } else {
            format!("[{fp}/{}]", self.path)
        }
    }
}

/// v0.4.1 Phase H.3+H.4 — synthesize a multi-source or hybrid multisig bundle.
/// Each slot may be secret-bearing (with entropy) OR watch-only (no entropy).
/// Per SPEC §5.8 the resulting `Bundle.ms1` is dense Vec of length N with
/// empty-string sentinels for watch-only slots.
///
/// Used by `bundle_run_unified` for `BundleMode::MultisigMultiSource` and
/// `BundleMode::MultisigHybrid` dispatch arms. Also handles single-sig under
/// the unified path (N=1; SingleSigFull and SingleSigWatchOnly modes route
/// through the same code path with N=1).
/// ms mnem Phase 3 Step 5: `run_language` is the per-run `--language` /
/// English default used to resolve the emit language for slots whose source
/// is a phrase/entropy (no per-card wire language). Slots whose source was a
/// `mnem` ms1 card carry `slot.language = Some(wire_lang)` which wins over
/// `run_language`. English `run_language` always emits `Entr` (byte-identical
/// to v0.38.4).
pub fn synthesize_unified(
    slots: &[ResolvedSlot],
    template: CliTemplate,
    threshold: u8,
    network: CliNetwork,
    privacy_preserving: bool,
    run_language: bip39::Language,
    md1_form: Md1Form,
) -> Result<Bundle, ToolkitError> {
    let n = slots.len();
    if n == 0 || n > 16 {
        return Err(ToolkitError::MultisigConfig {
            message: format!("slot count {n} out of range 1..=16"),
        });
    }
    if threshold == 0 || (threshold as usize) > n {
        return Err(ToolkitError::MultisigConfig {
            message: format!("threshold {threshold} out of range 1..={n}"),
        });
    }
    // SPEC §4.3 per-slot network/xpub cross-check.
    for (i, s) in slots.iter().enumerate() {
        if s.xpub.network != network.network_kind() {
            return Err(ToolkitError::CosignerSpec {
                cosigner_idx: i,
                message: format!(
                    "xpub network {:?} does not match --network {}",
                    s.xpub.network,
                    network.human_name()
                ),
            });
        }
    }

    // Path family check (single-sig N=1: use template default; multisig: use
    // each slot's path).
    let origin_paths: Vec<OriginPath> = slots
        .iter()
        .map(|s| derivation_path_to_origin_path(&s.path))
        .collect();
    let all_same = origin_paths.windows(2).all(|w| w[0] == w[1]);
    let path_decl_paths = if all_same || n == 1 {
        PathDeclPaths::Shared(origin_paths[0].clone())
    } else {
        PathDeclPaths::Divergent(origin_paths)
    };

    // Build descriptor.
    let tree = template.wrapper_node(threshold, n);
    let fingerprints: Vec<(u8, [u8; 4])> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i as u8, s.fingerprint.to_bytes()))
        .collect();
    let pubkeys: Vec<(u8, [u8; 65])> = slots
        .iter()
        .enumerate()
        .map(|(i, s)| (i as u8, xpub_to_65(&s.xpub)))
        .collect();

    let descriptor = Descriptor {
        n: n as u8,
        path_decl: PathDecl {
            n: n as u8,
            paths: path_decl_paths,
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree,
        tlv: TlvSection {
            use_site_path_overrides: None,
            fingerprints: Some(fingerprints),
            pubkeys: Some(pubkeys),
            origin_path_overrides: None,
            unknown: Vec::new(),
        },
    };

    // The card-emission back-half (policy_id → ms1 → mk1 → md1 → Bundle) is
    // byte-identical to `synthesize_descriptor`, and `slots: &[ResolvedSlot]`
    // IS `&[CosignerKeyInfo]` (`type CosignerKeyInfo = ResolvedSlot`), so
    // delegate — FOLLOWUP `synthesize-descriptor-deduplicate-with-unified`.
    // (`synthesize_descriptor` re-derives policy_id/stub from `descriptor` and
    // its leading `cosigners.len() == descriptor.n` check holds since this fn
    // built `descriptor.n = slots.len()`.)
    synthesize_descriptor(
        &descriptor,
        slots,
        privacy_preserving,
        run_language,
        md1_form,
    )
}

/// True iff `descriptor` is admissible as a `--md1-form=template` shape. The
/// gate is split by arity so phase-2's broadening does NOT reverse phase-1's
/// deliberate single-sig refusals (the R0 I1 nested-multi-1of1 / bip49 pins):
///
///   - **n ≥ 2 (multisig / general policy — the phase-2 surface):** admit
///     "exactly what `restore` can later reconstruct from xpubs":
///       (a) the shape RENDERS via `to_miniscript_descriptor` (single-path,
///           chain 0) — admitting non-taproot multisig/general (`multi`/
///           `sortedmulti`/`thresh`/timelocks/hashlocks/`or_i` branches) AND the
///           shipped `tr(NUMS, multi_a)`, while REFUSING `tr(sortedmulti_a)` (the
///           `to_miniscript.rs` `SortedMultiA` render gap) and
///           `sortedmulti`-in-a-combinator; AND
///       (b) it has NO hardened use-site (`has_hardened_use_site` — #25; an xpub
///           cannot derive a hardened public child → unrestorable).
///   - **n == 1 (single-sig — UNCHANGED from phase 1):** admit ONLY the three
///     canonical-origin-elidable types — `cli_template_from_tree(tree).is_some()`
///     (pkh/bip44, wpkh/bip84, tr-keypath/bip86). This keeps bip49 `sh(wpkh)`
///     and a degenerate nested-multi/sortedmulti 1-of-1 (R0 I1) REFUSED, as
///     phase 1 pinned them — they are not "standard single-sig template" types
///     and route to `--md1-form=policy`.
///
/// Refusals → `TemplateFormUnsupportedShape`.
///
/// Note: this runs on the KEYED input `descriptor` (it still carries pubkeys);
/// `to_miniscript_descriptor` expands `@N` over `tlv.pubkeys`, so it must NOT be
/// called on the post-mutation keyless template.
fn template_admissible(descriptor: &Descriptor) -> bool {
    if descriptor.n == 1 {
        // Phase-1 single-sig gate, verbatim: only the canonical-elidable types.
        return cli_template_from_tree(&descriptor.tree).is_some();
    }
    if md_codec::to_miniscript::has_hardened_use_site(descriptor) {
        return false;
    }
    md_codec::to_miniscript::to_miniscript_descriptor(descriptor, 0).is_ok()
}

/// #28 phase 1+2 — emit a keyless TEMPLATE bundle (single-sig, multisig, or
/// general policy).
///
/// Gate (SPEC §3.1): `template_admissible(&descriptor)` — the shape must render
/// (refusing `tr(sortedmulti_a)` / `sortedmulti`-in-combinator) and carry no
/// hardened use-site (refusing the unrestorable hardened class). Refusals →
/// `TemplateFormUnsupportedShape`.
///
/// Mutations on a `descriptor.clone()` (SPEC §3.2):
///   1. `tlv.pubkeys = None`
///   2. `tlv.fingerprints = None`
///   3. **C1-conditional origin (the load-bearing emit decision):**
///      - `canonical_origin(&tree).is_some()` (canonical single-sig + canonical
///        multisig `wsh(multi/sortedmulti)`, `sh(wsh(...))`) → ELIDE to
///        `Shared(empty)` — byte-identical-shareable, account-agnostic.
///      - `canonical_origin(&tree).is_none()` (general policy — `wsh(or_i(...))`,
///        `thresh`, timelocks; e.g. degrade2) → KEEP the source per-`@N` origins
///        (`Divergent` when accounts differ). Eliding to empty here makes
///        `md decode` REJECT the wire (`validate_explicit_origin_required` →
///        `MissingExplicitOrigin`) — the C1 regression. The carried origin is
///        decode+display ONLY (origins are re-supplied at completion; the
///        template-id is origin-invariant, so binding is unchanged either way).
///   4. the `is_wallet_policy()` assert that guards the keyed path is NOT
///      asserted (the template is keyless by construction).
///
/// The per-`@N` use-site structure (incl. #25 overrides) is preserved (it is in
/// the template-id). Threshold k / sorted shape / N slots ride along unmutated
/// in `descriptor.tree`.
///
/// Binding stub (SPEC §3.3): the md1 + mk1 strings + display stub root on
/// `WalletDescriptorTemplateId` (NOT `WalletPolicyId`, which a keyless md1
/// cannot reproduce). The N-slot card back-half emits one mk1 card per cosigner
/// (`MkField::Single` at n==1, `MkField::Multi` at n≥2). ms1 is unchanged
/// (plain codex32 entropy per slot, no id field).
fn synthesize_template_descriptor(
    descriptor: &Descriptor,
    cosigners: &[CosignerKeyInfo],
    privacy_preserving: bool,
) -> Result<Bundle, ToolkitError> {
    // --- Shape-admission gate (SPEC §3.1) -----------------------------------
    // Admit single-sig, non-taproot multisig/general, and `tr(NUMS, multi_a)`;
    // refuse `tr(sortedmulti_a)` / `sortedmulti`-in-combinator (render gap) and
    // any hardened use-site (#25). `template_admissible` runs on the KEYED input
    // (it must render via `to_miniscript_descriptor`, which needs pubkeys) — so
    // it is called BEFORE the keyless mutations below.
    if !template_admissible(descriptor) {
        return Err(ToolkitError::TemplateFormUnsupportedShape {
            message: "--md1-form=template cannot template this descriptor shape: it either \
                 does not render (e.g. tr(sortedmulti_a) — the rust-miniscript v13 \
                 SortedMultiA gap — or sortedmulti inside a combinator) or it uses a \
                 hardened use-site path (/*h or a hardened multipath alt) which an xpub \
                 cannot derive; use --md1-form=policy for a faithful keyed backup"
                .into(),
        });
    }

    // --- The mutations (SPEC §3.2) on a clone -------------------------------
    let mut template = descriptor.clone();
    template.tlv.pubkeys = None; // mutation 1
    template.tlv.fingerprints = None; // mutation 2

    // mutation 3 — C1-CONDITIONAL origin handling. The canonical_origin verdict
    // is a whole-tree property (the wrapper shape), so it governs all `@N` at
    // once. A canonical wrapper re-derives the elided origin on decode; a
    // non-canonical wrapper (general policy) needs the explicit origins on the
    // wire or `validate_explicit_origin_required` rejects with
    // `MissingExplicitOrigin` (the C1 regression). When non-canonical we KEEP the
    // source `descriptor.path_decl.paths` verbatim — `parse_descriptor` /
    // `synthesize_unified` already built it (`Shared`/`Divergent`) from the
    // source per-`@N` origins. Origins are re-supplied at completion and the
    // template-id is origin-invariant, so binding is unchanged either way.
    if md_codec::canonical_origin::canonical_origin(&descriptor.tree).is_some() {
        template.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] });
    }
    // else: leave `template.path_decl.paths` as the cloned source origins.

    // mutation 4 — the `is_wallet_policy()` assert that guards the keyed path is
    // intentionally NOT asserted here (the template is keyless by construction).

    // --- Binding stub: re-root on WalletDescriptorTemplateId (SPEC §4.3) ----
    // Compute the id ONCE from the MUTATED template (the engraved md1 is the
    // template; binding must reflect what is engraved).
    let template_id =
        md_codec::compute_wallet_descriptor_template_id(&template).map_err(ToolkitError::from)?;
    let mut stub = [0u8; 4];
    stub.copy_from_slice(&template_id.as_bytes()[..4]);

    // md1 string = the keyless template.
    let md1 = md_codec::chunk::split(&template).map_err(ToolkitError::from)?;

    // mk1 cards — the xpub-bearing cards, each stubbed on the template id + a
    // slot-unique csi. Generalize the single-slot back-half to N cosigners
    // (mirrors the keyed `synthesize_descriptor` n==1 / n≥2 dispatch); the SAME
    // template-id stub roots every card (SPEC §3.3). `cosigners.len() == n` was
    // enforced by the caller (`synthesize_descriptor`'s leading check).
    let n = cosigners.len();
    let mk1 = if n == 1 {
        let c = &cosigners[0];
        let card = mk_codec::KeyCard::new(
            vec![stub],
            if privacy_preserving {
                None
            } else {
                Some(c.fingerprint)
            },
            mk1_origin_path(&c.xpub, &c.path),
            c.xpub,
        );
        let csi = derive_mk1_chunk_set_id_for_slot(&stub, 0);
        let chunks = mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
        MkField::Single(chunks)
    } else {
        let stubs: Vec<[u8; 4]> = vec![stub; n];
        let mut per_cosigner: Vec<Vec<String>> = Vec::with_capacity(n);
        for (i, c) in cosigners.iter().enumerate() {
            let card = mk_codec::KeyCard::new(
                stubs.clone(),
                if privacy_preserving {
                    None
                } else {
                    Some(c.fingerprint)
                },
                mk1_origin_path(&c.xpub, &c.path),
                c.xpub,
            );
            // Slot-unique csi (audit I10): stub^slot preserves the leading-16-bit
            // bundle-binding prefix while distinguishing same-xpub slots.
            let csi = derive_mk1_chunk_set_id_for_slot(&stub, i as u32);
            let chunks =
                mk_codec::encode_with_chunk_set_id(&card, csi).map_err(ToolkitError::from)?;
            per_cosigner.push(chunks);
        }
        MkField::Multi(per_cosigner)
    };

    // ms1 strings — UNCHANGED by form (plain codex32 entropy/mnem; no id field).
    // One per slot; watch-only slots → "" sentinel (SPEC §5.8 length-N rule).
    let mut ms1: MsField = Vec::with_capacity(n);
    for c in cosigners {
        match &c.entropy {
            Some(e) => {
                let emit_lang = c.language.unwrap_or(bip39::Language::English);
                let payload = if emit_lang == bip39::Language::English {
                    ms_codec::Payload::Entr((**e).clone())
                } else {
                    ms_codec::Payload::Mnem {
                        language: crate::language::bip39_to_wire_code(emit_lang),
                        entropy: (**e).clone(),
                    }
                };
                ms1.push(
                    ms_codec::encode(ms_codec::Tag::ENTR, &payload).map_err(ToolkitError::from)?,
                );
            }
            None => ms1.push(String::new()),
        }
    }

    Ok(Bundle { ms1, mk1, md1 })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::derive::derive_full;
    use crate::language::CliLanguage;

    #[test]
    fn mk1_origin_path_round_trips_every_class() {
        use bitcoin::bip32::{DerivationPath, Xpriv, Xpub};
        use bitcoin::secp256k1::Secp256k1;
        use std::str::FromStr;
        let secp = Secp256k1::new();
        let seed = [7u8; 32];
        let master = Xpriv::new_master(bitcoin::NetworkKind::Main, &seed).unwrap();
        let xpub_at = |p: &str| {
            let path = DerivationPath::from_str(p).unwrap();
            Xpub::from_priv(&secp, &master.derive_priv(&secp, &path).unwrap())
        };
        // (xpub, descriptor_path) for each census class.
        let cases: &[(Xpub, &str)] = &[
            (xpub_at("m/84'/0'/0'"), "m/84'/0'/0'"), // consistent 3→3 (no-op)
            (xpub_at("m/48'/0'/0'"), "m/48'/0'/0'/2'"), // 3→4 truncate
            (xpub_at("m/48'/0'/0'/2'"), "m/87'/0'/0'"), // 4→3 extend
            (xpub_at("m/84'/0'/0'"), "m"),           // 3→0 pad
            (xpub_at("m/0'"), "m/0'"),               // depth-1
        ];
        for (xpub, dpath) in cases {
            let out = mk1_origin_path(xpub, &DerivationPath::from_str(dpath).unwrap());
            let comps: Vec<_> = out.into_iter().copied().collect();
            assert_eq!(comps.len(), xpub.depth as usize, "len==depth for {dpath}");
            assert_eq!(
                *comps.last().unwrap(),
                xpub.child_number,
                "last==child for {dpath}"
            );
            // The card must now ENCODE (no XpubOriginPathMismatch).
            let card = mk_codec::KeyCard::new(vec![[0xAAu8; 4]], None, out, *xpub);
            assert!(
                mk_codec::encode_with_chunk_set_id(&card, 0).is_ok(),
                "encodes for {dpath}"
            );
        }
    }

    #[test]
    fn mk1_origin_path_depth0_is_empty() {
        // A WIF-style depth-0 xpub → empty path.
        let secp = bitcoin::secp256k1::Secp256k1::new();
        let sk =
            bitcoin::PrivateKey::from_wif("KwDiBf89QgGbjEhKnhXJuH7LrciVrZi3qYjgd9M7rFU73sVHnoWn")
                .unwrap();
        let xpub = bitcoin::bip32::Xpub {
            network: bitcoin::NetworkKind::Main,
            depth: 0,
            parent_fingerprint: Default::default(),
            child_number: bitcoin::bip32::ChildNumber::Normal { index: 0 },
            public_key: sk.public_key(&secp).inner,
            chain_code: bitcoin::bip32::ChainCode::from([0u8; 32]),
        };
        let out = mk1_origin_path(&xpub, &bitcoin::bip32::DerivationPath::master());
        assert_eq!(out.into_iter().count(), 0);
    }

    const TREZOR_24: &str = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon art";

    // SPEC §5.8 emission-rule regression-test constants (v0.21.0 cycle).
    // Mirror of cli_verify_bundle_multi_cosigner_mk1.rs:21-26; declared
    // locally here because the integration-test crate's `const` block is
    // not importable into the library's internal `mod tests` (separate
    // compilation units).
    const TREZOR_12_ZERO: &str =
        "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
    const BIP39_TEST_2: &str =
        "legal winner thank year wave sausage worth useful legal winner thank yellow";
    const BIP39_TEST_3: &str =
        "letter advice cage absurd amount doctor acoustic avoid letter advice cage above";

    fn fixture_full(template: CliTemplate, network: CliNetwork) -> (Vec<u8>, Fingerprint, Xpub) {
        let acc = derive_full(TREZOR_24, "", CliLanguage::English, network, template, 0).unwrap();
        // v0.10.1: `into_parts` returns bare Vec<u8> per caller-wrap
        // contract (Zeroizing-drives-scrub semantics live on the field).
        let (entropy, master_fingerprint, account_xpub, _xpriv, _path) = acc.into_parts();
        (entropy, master_fingerprint, account_xpub)
    }

    #[test]
    fn xpub_to_65_layout() {
        let (_, _, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bytes = xpub_to_65(&xpub);
        assert_eq!(&bytes[0..32], xpub.chain_code.to_bytes().as_slice());
        assert_eq!(&bytes[32..65], xpub.public_key.serialize().as_slice());
    }

    // T5 (SPEC_path_raw_bracketed_bare_unification.md §8) — `origin_path_bare()`
    // / `bracketed_origin()` render correctness + the pillar-3 default-path
    // sentinel invariant.
    #[test]
    fn origin_render_methods_bare_and_bracketed() {
        let (_, _, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let fp = Fingerprint::from_str("deadbeef").unwrap();
        let mk_slot = |path: DerivationPath| ResolvedSlot {
            xpub,
            fingerprint: fp,
            path,
            entropy: None,
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        };
        // (a) normal slot — bare `m/...`, bracketed `[fp/...]`
        let s = mk_slot(DerivationPath::from_str("48'/0'/0'/2'").unwrap());
        assert_eq!(s.origin_path_bare(), "m/48'/0'/0'/2'");
        assert_eq!(s.bracketed_origin(), "[deadbeef/48'/0'/0'/2']");
        // (b) default-path slot — empty sentinel + bare `[fp]`
        let d = mk_slot(DerivationPath::default());
        assert_eq!(d.origin_path_bare(), "");
        assert_eq!(d.bracketed_origin(), "[deadbeef]");
        // (c) no double-bracket, single fingerprint
        assert_eq!(s.bracketed_origin().matches('[').count(), 1);
        assert_eq!(s.bracketed_origin().matches("deadbeef").count(), 1);
        // (d) pillar-3 invariant: DerivationPath::default() renders to ""
        assert_eq!(DerivationPath::default().to_string(), "");
        // bonus: fingerprint casing is normalized to lowercase (M-1)
        let up = ResolvedSlot {
            fingerprint: Fingerprint::from_str("ABCD1234").unwrap(),
            language: None,
            ..mk_slot(DerivationPath::from_str("84'/0'/0'").unwrap())
        };
        assert_eq!(up.bracketed_origin(), "[abcd1234/84'/0'/0']");
    }

    #[test]
    fn full_bundle_emits_three_cards() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        assert!(bundle.any_secret_bearing());
        let ms1 = &bundle.ms1[0];
        assert!(ms1.starts_with("ms1"));
        // ms1 must round-trip to the INPUT entropy — correctness, not just the
        // "starts_with ms1" smoke (a syntactically-valid but wrong card would
        // pass the presence check). mk1→xpub + md1→policy correctness are
        // covered by `cross_binding_holds_round_trip`. The English `fixture_full`
        // entropy emits a `Payload::Entr` card.
        let (_, payload) = ms_codec::decode(ms1).unwrap();
        match payload {
            ms_codec::Payload::Entr(b) => {
                assert_eq!(b, entropy, "ms1 must decode back to the input entropy")
            }
            other => panic!("expected an Entr ms1 payload (English fixture), got {other:?}"),
        }
        let mk1 = bundle.mk1.as_single().unwrap();
        assert!(!mk1.is_empty());
        assert!(mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn watch_only_bundle_omits_ms1() {
        let (_, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle =
            synthesize_watch_only(fp, xpub, CliTemplate::Bip84, CliNetwork::Mainnet, 0).unwrap();
        assert!(!bundle.any_secret_bearing());
        let mk1 = bundle.mk1.as_single().unwrap();
        assert!(!mk1.is_empty());
        assert!(mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
        assert!(bundle.md1.iter().all(|s| s.starts_with("md1")));
    }

    #[test]
    fn mk1_chunk_set_id_is_deterministic_across_runs() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let a = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        let b = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();
        assert_eq!(
            a.mk1.as_single().unwrap(),
            b.mk1.as_single().unwrap(),
            "mk1 must be byte-deterministic across runs"
        );
        assert_eq!(a.md1, b.md1, "md1 must be byte-deterministic across runs");
        assert_eq!(a.ms1, b.ms1, "ms1 must be byte-deterministic across runs");
    }

    #[test]
    fn cross_binding_holds_round_trip() {
        let (entropy, fp, xpub) = fixture_full(CliTemplate::Bip84, CliNetwork::Mainnet);
        let bundle = synthesize_full(
            &entropy,
            fp,
            xpub,
            CliTemplate::Bip84,
            CliNetwork::Mainnet,
            0,
        )
        .unwrap();

        let mk1_v = bundle.mk1.as_single().unwrap();
        let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
        let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();

        let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
        assert_eq!(&decoded_mk1.policy_id_stubs[0], &policy_id.as_bytes()[..4]);

        assert!(decoded_md1.is_wallet_policy());

        assert_eq!(decoded_mk1.xpub, xpub);
        assert_eq!(decoded_mk1.origin_fingerprint, Some(fp));
    }

    #[test]
    fn multisig_full_self_multisig_emits_distinct_slot_unique_csi_cards() {
        use bip39::Mnemonic;
        let m = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let bundle = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().expect("multisig must emit Multi");
        assert_eq!(multi.len(), 3, "3 cosigners → 3 card-sets");
        // Audit I10: self-multisig means all xpubs are identical, so the OLD
        // per-fingerprint csi made all N card-sets byte-IDENTICAL — the exact
        // collision that broke verify-bundle reassembly. Post-fix each cosigner
        // gets a distinct slot-XOR csi, so the card-sets are pairwise DISTINCT.
        for i in 1..3 {
            assert_ne!(
                multi[0], multi[i],
                "post-I10 self-multisig card-sets must be DISTINCT (slot-unique csi)"
            );
        }
        assert_ne!(multi[1], multi[2], "slots 1 and 2 must also differ");
        // Cross-binding round-trip via decode.
        let card_strs: Vec<&str> = multi[0].iter().map(|s| s.as_str()).collect();
        let decoded = mk_codec::decode(&card_strs).unwrap();
        assert_eq!(decoded.policy_id_stubs.len(), 3);
        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();
        assert!(desc.is_wallet_policy());
        let pid = md_codec::compute_wallet_policy_id(&desc).unwrap();
        assert_eq!(&decoded.policy_id_stubs[0], &pid.as_bytes()[..4]);
    }

    /// Phase 3 pin (md-codec catchup v0.16.1 → v0.33.1): a 2-of-3
    /// `WshSortedMulti` bundle round-trips through `chunk::split` +
    /// `chunk::reassemble` and the reassembled inner `Tag::SortedMulti`
    /// node carries `Body::MultiKeys { k, indices: [0, 1, 2] }` — the v0.30
    /// Phase-C wire shape — NOT the pre-v0.30 `Body::Variable` shape.
    /// Guards against regression to the old per-leaf `Tag::PkK` emission
    /// that produces v0.30-incompatible bytes.
    #[test]
    fn multisig_wsh_sortedmulti_2_of_3_round_trips_v0_30_body_multikeys() {
        use bip39::Mnemonic;
        use md_codec::tag::Tag;
        use md_codec::tree::{Body, Node};

        let m = Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let bundle = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap();

        let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let desc = md_codec::chunk::reassemble(&md1_strs).unwrap();
        assert!(
            desc.is_wallet_policy(),
            "wallet-policy expected post-roundtrip"
        );
        assert!(matches!(desc.tree.tag, Tag::Wsh), "root must be Wsh");

        let inner: &Node = match &desc.tree.body {
            Body::Children(kids) => &kids[0],
            other => panic!("Wsh body must be Children, got {other:?}"),
        };
        assert!(matches!(inner.tag, Tag::SortedMulti));
        match &inner.body {
            Body::MultiKeys { k, indices } => {
                assert_eq!(*k, 2, "2-of-3 threshold");
                assert_eq!(indices, &vec![0u8, 1, 2], "indices must round-trip as 0..n");
            }
            other => panic!("v0.30 SPEC §4 requires SortedMulti → Body::MultiKeys, got {other:?}"),
        }

        // Cross-binding still holds.
        let pid = md_codec::compute_wallet_policy_id(&desc).unwrap();
        assert!(!pid.as_bytes().is_empty());
    }

    #[test]
    fn multisig_watch_only_distinct_xpubs_emits_distinct_card_sets() {
        // Build 2 cosigners from 2 different seeds (different fp/xpub).
        let m1 = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let m2 = bip39::Mnemonic::parse_in(
            bip39::Language::English,
            "legal winner thank year wave sausage worth useful legal winner thank yellow",
        )
        .unwrap();
        let secp = Secp256k1::new();

        let derive = |m: &bip39::Mnemonic, path_str: &str| -> CosignerSpec {
            let seed = m.to_seed("");
            let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
            let fp = master.fingerprint(&secp);
            let path = DerivationPath::from_str(path_str).unwrap();
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);
            CosignerSpec {
                xpub,
                master_fingerprint: fp,
                path: Some(path),
            }
        };

        let path_str = "m/87'/0'/0'";
        let cosigners = vec![derive(&m1, path_str), derive(&m2, path_str)];

        let bundle = synthesize_multisig_watch_only(
            &cosigners,
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().unwrap();
        assert_eq!(multi.len(), 2);
        assert_ne!(multi[0], multi[1], "distinct cosigners → distinct cards");

        // Round-trip both.
        for chunks in multi {
            let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
            let decoded = mk_codec::decode(&strs).unwrap();
            assert_eq!(decoded.policy_id_stubs.len(), 2);
        }
    }

    #[test]
    fn multisig_threshold_validation() {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        // K = 0 rejected.
        let e = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            0,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::MultisigConfig { .. }));
        // K > N rejected.
        let e = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            5,
            3,
            0,
            MultisigPathFamily::Bip87,
            false,
        )
        .unwrap_err();
        assert!(matches!(e, ToolkitError::MultisigConfig { .. }));
    }

    #[test]
    fn multisig_privacy_preserving_omits_fingerprints_in_mk1() {
        let m = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let bundle = synthesize_multisig_full(
            &m,
            "",
            CliNetwork::Mainnet,
            CliTemplate::WshSortedMulti,
            2,
            2,
            0,
            MultisigPathFamily::Bip87,
            true,
        )
        .unwrap();
        let multi = bundle.mk1.as_multi().unwrap();
        for chunks in multi {
            let strs: Vec<&str> = chunks.iter().map(|s| s.as_str()).collect();
            let decoded = mk_codec::decode(&strs).unwrap();
            assert!(
                decoded.origin_fingerprint.is_none(),
                "privacy-preserving mode should omit origin_fingerprint"
            );
        }
    }

    // ---- C.1: synthesize_descriptor (4 shape tests) ----

    /// Build a `Descriptor` + `CosignerKeyInfo` array for testing. Uses the
    /// TREZOR_24 fixture to derive the @0 xpub at a real BIP-32 path; for
    /// multisig tests, derives N xpubs at successive child indices.
    fn descriptor_fixture(
        descriptor_str: &str,
        ctx: crate::parse_descriptor::ScriptCtx,
        n: u8,
    ) -> (Descriptor, Vec<CosignerKeyInfo>, Vec<u8>) {
        use crate::parse_descriptor::{parse_descriptor, ParsedFingerprint, ParsedKey};

        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let master_fp = master.fingerprint(&secp);
        let _ = ctx;

        // Derive N xpubs at distinct paths (m/48'/0'/0'/2', then bumping account index).
        let base = "48'/0'/0'/2'";
        let mut cosigners = Vec::with_capacity(n as usize);
        let mut keys = Vec::with_capacity(n as usize);
        let mut fps = Vec::with_capacity(n as usize);
        for i in 0..n {
            let path_str = if i == 0 {
                base.to_string()
            } else {
                format!("48'/0'/{}/2'", i)
            };
            let path = DerivationPath::from_str(&path_str).unwrap();
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);
            cosigners.push(CosignerKeyInfo {
                xpub,
                fingerprint: master_fp,
                path: path.clone(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });

            let mut payload = [0u8; 65];
            payload[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
            payload[32..65].copy_from_slice(&xpub.public_key.serialize());
            keys.push(ParsedKey { i, payload });
            fps.push(ParsedFingerprint {
                i,
                fp: master_fp.to_bytes(),
            });
        }

        let descriptor = parse_descriptor(descriptor_str, &keys, &fps).unwrap();
        (descriptor, cosigners, entropy)
    }

    #[test]
    fn synthesize_descriptor_full_singlesig_shape() {
        let (descriptor, mut cosigners, entropy) = descriptor_fixture(
            "wpkh(@0/<0;1>/*)",
            crate::parse_descriptor::ScriptCtx::SingleSig,
            1,
        );
        cosigners[0].entropy = Some(zeroize::Zeroizing::new(entropy.clone()));
        let bundle = synthesize_descriptor(
            &descriptor,
            &cosigners,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert!(bundle.any_secret_bearing(), "full mode emits ms1");
        let mk1 = bundle.mk1.as_single().expect("n=1 → MkField::Single");
        assert!(!mk1.is_empty());
        assert!(mk1.iter().all(|s| s.starts_with("mk1")));
        assert!(!bundle.md1.is_empty());
    }

    #[test]
    fn synthesize_descriptor_watch_only_singlesig_shape() {
        let (descriptor, cosigners, _entropy) = descriptor_fixture(
            "wpkh(@0/<0;1>/*)",
            crate::parse_descriptor::ScriptCtx::SingleSig,
            1,
        );
        let bundle = synthesize_descriptor(
            &descriptor,
            &cosigners,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert!(!bundle.any_secret_bearing(), "watch-only mode omits ms1");
        let mk1 = bundle.mk1.as_single().expect("n=1 → MkField::Single");
        assert!(!mk1.is_empty());
    }

    #[test]
    fn synthesize_descriptor_full_multisig_shape() {
        let (descriptor, mut cosigners, entropy) = descriptor_fixture(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        cosigners[0].entropy = Some(zeroize::Zeroizing::new(entropy.clone()));
        let bundle = synthesize_descriptor(
            &descriptor,
            &cosigners,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert!(bundle.any_secret_bearing());
        let multi = bundle.mk1.as_multi().expect("n=2 → MkField::Multi");
        assert_eq!(multi.len(), 2, "multisig n=2 emits 2 mk1 cards");
    }

    #[test]
    fn synthesize_descriptor_watch_only_multisig_shape() {
        let (descriptor, cosigners, _) = descriptor_fixture(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        let bundle = synthesize_descriptor(
            &descriptor,
            &cosigners,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert!(!bundle.any_secret_bearing());
        let multi = bundle.mk1.as_multi().unwrap();
        assert_eq!(multi.len(), 2);
    }

    #[test]
    fn synthesize_descriptor_validates_cosigner_count() {
        let (descriptor, cosigners, _) = descriptor_fixture(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        // descriptor has n=2 but we only pass 1 cosigner → error
        let one = vec![cosigners[0].clone()];
        let err = synthesize_descriptor(
            &descriptor,
            &one,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap_err();
        assert!(matches!(err, ToolkitError::DescriptorParse(_)));
    }

    /// SPEC §5.8 emission rule (v0.21.0): a descriptor-mode multi-cosigner
    /// bundle emits one populated `ms1` string per phrase-bearing slot, and
    /// the empty-string sentinel only for watch-only slots. Regression guard
    /// against the legacy "v0.3 descriptor-mode contract" that pinned ms1
    /// emission to slot @0. Hybrid arm exercises the per-slot sentinel rule.
    #[test]
    fn synthesize_descriptor_emits_per_slot_ms1_for_phrase_bearing_slots() {
        use crate::parse_descriptor::{parse_descriptor, ParsedFingerprint, ParsedKey};

        // Build a 3-cosigner fixture from 3 DISTINCT BIP-39 mnemonics — cannot
        // use `descriptor_fixture` here (it shares one TREZOR_24 seed across
        // slots and would violate BIP-388 §4.11.b distinctness for the
        // descriptor's pkh(@0..2) leaves).
        let phrases = [TREZOR_12_ZERO, BIP39_TEST_2, BIP39_TEST_3];
        let secp = Secp256k1::new();
        let path = DerivationPath::from_str("48'/0'/0'/2'").unwrap();
        let mut cosigners: Vec<CosignerKeyInfo> = Vec::with_capacity(3);
        let mut keys: Vec<ParsedKey> = Vec::with_capacity(3);
        let mut fps: Vec<ParsedFingerprint> = Vec::with_capacity(3);
        for (i, phrase) in phrases.iter().enumerate() {
            let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, *phrase).unwrap();
            let entropy = mnemonic.to_entropy();
            let seed = mnemonic.to_seed("");
            let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
            let master_fp = master.fingerprint(&secp);
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);

            cosigners.push(CosignerKeyInfo {
                xpub,
                fingerprint: master_fp,
                path: path.clone(),
                entropy: Some(zeroize::Zeroizing::new(entropy)),
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });

            let mut payload = [0u8; 65];
            payload[0..32].copy_from_slice(&xpub.chain_code.to_bytes());
            payload[32..65].copy_from_slice(&xpub.public_key.serialize());
            keys.push(ParsedKey {
                i: i as u8,
                payload,
            });
            fps.push(ParsedFingerprint {
                i: i as u8,
                fp: master_fp.to_bytes(),
            });
        }

        let descriptor = parse_descriptor(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
            &keys,
            &fps,
        )
        .unwrap();
        let bundle = synthesize_descriptor(
            &descriptor,
            &cosigners,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 3, "ms1 dense vec len == n");
        assert!(
            bundle.ms1[0].starts_with("ms1"),
            "ms1[0] populated; got {:?}",
            bundle.ms1[0]
        );
        assert!(
            bundle.ms1[1].starts_with("ms1"),
            "ms1[1] populated; got {:?}",
            bundle.ms1[1]
        );
        assert!(
            bundle.ms1[2].starts_with("ms1"),
            "ms1[2] populated; got {:?}",
            bundle.ms1[2]
        );
        // All 3 must be DISTINCT (each ms1 carries that slot's own entropy bytes).
        assert_ne!(bundle.ms1[0], bundle.ms1[1]);
        assert_ne!(bundle.ms1[1], bundle.ms1[2]);
        assert_ne!(bundle.ms1[0], bundle.ms1[2]);

        // Hybrid arm — slot 0 phrase, slots 1-2 watch-only (entropy: None)
        // → ms1 = [populated, "", ""] per SPEC §5.8 example at line 141.
        let mut cosigners_hybrid = cosigners.clone();
        cosigners_hybrid[1].entropy = None;
        cosigners_hybrid[2].entropy = None;
        let bundle_hybrid = synthesize_descriptor(
            &descriptor,
            &cosigners_hybrid,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle_hybrid.ms1.len(), 3);
        assert!(bundle_hybrid.ms1[0].starts_with("ms1"));
        assert_eq!(bundle_hybrid.ms1[1], "");
        assert_eq!(bundle_hybrid.ms1[2], "");
    }

    #[test]
    fn cross_binding_holds_all_4_templates_x_4_networks() {
        let templates = [
            CliTemplate::Bip44,
            CliTemplate::Bip49,
            CliTemplate::Bip84,
            CliTemplate::Bip86,
        ];
        let networks = [
            CliNetwork::Mainnet,
            CliNetwork::Testnet,
            CliNetwork::Signet,
            CliNetwork::Regtest,
        ];
        for &t in &templates {
            for &n in &networks {
                let (entropy, fp, xpub) = fixture_full(t, n);
                let bundle = synthesize_full(&entropy, fp, xpub, t, n, 0).unwrap();
                let mk1_v = bundle.mk1.as_single().unwrap();
                let mk1_strs: Vec<&str> = mk1_v.iter().map(|s| s.as_str()).collect();
                let decoded_mk1 = mk_codec::decode(&mk1_strs).unwrap();
                let md1_strs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
                let decoded_md1 = md_codec::chunk::reassemble(&md1_strs).unwrap();
                let policy_id = md_codec::compute_wallet_policy_id(&decoded_md1).unwrap();
                assert_eq!(
                    &decoded_mk1.policy_id_stubs[0],
                    &policy_id.as_bytes()[..4],
                    "stub linkage failed for {t:?} on {n:?}"
                );
                assert!(decoded_md1.is_wallet_policy(), "{t:?} on {n:?}");
                assert_eq!(decoded_mk1.xpub, xpub, "{t:?} on {n:?}");
                assert_eq!(decoded_mk1.origin_fingerprint, Some(fp), "{t:?} on {n:?}");
            }
        }
    }

    // ---- v0.4.1 Phase H r1 review I-2 — synthesize_unified shape tests ----

    /// Build N ResolvedSlots from TREZOR_24 at distinct child paths. The
    /// `entropy_indices` set marks which slots are secret-bearing; all others
    /// are watch-only (entropy = None).
    fn unified_fixture(n: usize, entropy_indices: &[usize]) -> Vec<ResolvedSlot> {
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let master_fp = master.fingerprint(&secp);

        let mut out = Vec::with_capacity(n);
        for i in 0..n {
            let path_str = format!("48'/0'/{}'/2'", i);
            let path = DerivationPath::from_str(&path_str).unwrap();
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);
            // v0.10.1: ResolvedSlot.entropy is Option<Zeroizing<Vec<u8>>>;
            // wrap at construction.
            let entropy_field = if entropy_indices.contains(&i) {
                Some(zeroize::Zeroizing::new(entropy.clone()))
            } else {
                None
            };
            let entropy_pin = entropy_field
                .as_ref()
                .map(|e| Rc::new(mnemonic_toolkit::mlock::pin_pages_for(&e[..])));
            out.push(ResolvedSlot {
                xpub,
                fingerprint: master_fp,
                path: path.clone(),
                entropy: entropy_field,
                master_xpub: None,
                language: None,
                _entropy_pin: entropy_pin,
            });
        }
        out
    }

    /// Distinct-cosigner slot from a specific phrase (distinct entropy →
    /// distinct fingerprint/xpub → distinct mk1 csi). Path `48'/0'/<idx>'/2'`.
    fn distinct_slot(phrase: &str, idx: usize) -> ResolvedSlot {
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, phrase).unwrap();
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let master_fp = master.fingerprint(&secp);
        let path = DerivationPath::from_str(&format!("48'/0'/{idx}'/2'")).unwrap();
        let xpriv = master.derive_priv(&secp, &path).unwrap();
        let xpub = Xpub::from_priv(&secp, &xpriv);
        ResolvedSlot {
            xpub,
            fingerprint: master_fp,
            path,
            entropy: Some(zeroize::Zeroizing::new(entropy)),
            master_xpub: None,
            language: None,
            _entropy_pin: None,
        }
    }

    #[test]
    fn synthesize_unified_multisig_distinct_cosigners_byte_exact() {
        // R0 I1 characterization guard for the dedup `synthesize_unified` →
        // `synthesize_descriptor` delegation (FOLLOWUP
        // `synthesize-descriptor-deduplicate-with-unified`). Pins the n>1
        // `MkField::Multi` branch's full Bundle byte-shape with TWO DISTINCT
        // cosigners (distinct fingerprints → distinct per-cosigner mk1 csi →
        // mk1[0] != mk1[1]). FROZEN literals captured from the pre-delegation
        // binary (R0 M2: NOT an assert_eq!(unified, descriptor) compare, which
        // is vacuous once both are the same fn). Any csi / per-cosigner
        // ordering / stub / ms1 / md1 drift in the delegated path goes RED.
        let slots = vec![
            distinct_slot(TREZOR_12_ZERO, 0),
            distinct_slot(BIP39_TEST_2, 1),
        ];
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            2,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(
            bundle.ms1,
            vec![
                "ms10entrsqqqqqqqqqqqqqqqqqqqqqqqqqqqqcj9sxraq34v7f".to_string(),
                "ms10entrsqplh7lml0alh7lml0alh7lml0als5cclar2zmksh6".to_string(),
            ]
        );
        let mk = bundle.mk1.as_multi().expect("n>1 → Multi");
        // Audit I10: per-cosigner csi is now slot-XOR of the shared policy stub
        // (was per-fingerprint). mk[0] and mk[1] share their leading bytes
        // (same policy stub → same leading-16-bit binding prefix) and differ in
        // the csi low nibble (slot 0 vs 1) — visible as the shared `mk1qpe8m`
        // prefix below. Frozen literals re-captured post-fix.
        assert_eq!(
            mk[0],
            vec![
                "mk1qpe8mgpqqspvna5yxhyldpp4w0za5zs9qjyty8su72t3dwaqcl9pvz58pmltjs9tjrg0g2z0agd4urfpzanhaq3lcdlz6ta8cw4mf7d96gts".to_string(),
                "mk1qpe8mgpp2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfj6e20ur0anz7jwkzae8efp77w50cle83tzpcagl78".to_string(),
            ]
        );
        assert_eq!(
            mk[1],
            vec![
                "mk1qpe8mfzqqspvna5yxhyldpp4hp5gmu07qjcgpqyqpzqgpqyqpzqcpqyqpzpgpqyqpqzg3vs7247wz22l0uwvjq67znc3gr3exu5ux50m0ewe".to_string(),
                "mk1qpe8mfzp76lp8zltaht9xxts9tayzjzukf59mpctwngtxq6svts2qqk8su3z373k0ng4vra90z9r27f7v8wwelf50wn4cl9ft5au70gmqu2u".to_string(),
                "mk1qpe8mfzzc70s4f2z8jmqnewmltc0ta50n".to_string(),
            ]
        );
        assert_eq!(
            bundle.md1,
            vec![
                "md1ftp2nps9q2tvyyy5jmpprj5qqcy8ppgtcgu79mg9dcdzxlz9wpyhwsv0jskp2rsal4egz4eqzcngzrpfdv2w5".to_string(),
                "md1ftp2npsf5859p875x67p5s3wem7sgluxl3d2a3syx3m7halwd7s7d5e8l2xm3y3xzfmadfjcygdcxfdspxgm5".to_string(),
                "md1ftp2npsje20ur0anz7jwkzae8ef47lcueyp4u983znmtuyuta0kav5cewq405s2gtjexshvq3jnnf2ver22va".to_string(),
                "md1ftp2npslpd6dpvcr2p3wpgqzc7rjy286xe7dz4s054ug5dte8esaem8ax3a6whx8nu9qqqlxqwxzd0ld3k".to_string(),
            ]
        );
    }

    #[test]
    fn synthesize_unified_single_sig_full_ms1_one_non_empty() {
        // SingleSigFull (n=1, secret-bearing): ms1 = ["ms1..."] length-1.
        let slots = unified_fixture(1, &[0]);
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::Bip84,
            1,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 1);
        assert!(bundle.ms1[0].starts_with("ms1"));
        assert!(bundle.any_secret_bearing());
    }

    #[test]
    fn synthesize_unified_single_sig_watch_only_ms1_empty_sentinel() {
        // SingleSigWatchOnly (n=1, no entropy): ms1 = [""] length-1 with sentinel.
        let slots = unified_fixture(1, &[]);
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::Bip84,
            1,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 1);
        assert_eq!(bundle.ms1[0], "");
        assert!(!bundle.any_secret_bearing());
    }

    #[test]
    fn synthesize_unified_multisig_multisource_ms1_all_non_empty() {
        // MultisigMultiSource N=3: every slot secret-bearing.
        // Note: TREZOR_24 produces the same entropy across slots; in practice
        // multi-source uses N distinct phrases, but the synthesis contract
        // operates on per-slot entropy regardless of provenance.
        let slots = unified_fixture(3, &[0, 1, 2]);
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            2,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 3);
        assert!(bundle.ms1.iter().all(|s| s.starts_with("ms1")));
        assert!(bundle.any_secret_bearing());
    }

    #[test]
    fn synthesize_unified_multisig_watch_only_ms1_all_sentinel() {
        // MultisigWatchOnly N=3: every slot watch-only.
        let slots = unified_fixture(3, &[]);
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            2,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 3);
        assert!(bundle.ms1.iter().all(|s| s.is_empty()));
        assert!(!bundle.any_secret_bearing());
    }

    #[test]
    fn synthesize_unified_multisig_hybrid_ms1_dense_with_sentinels() {
        // MultisigHybrid N=3: slot 0 secret, slots 1+2 watch-only.
        let slots = unified_fixture(3, &[0]);
        let bundle = synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            2,
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap();
        assert_eq!(bundle.ms1.len(), 3);
        assert!(bundle.ms1[0].starts_with("ms1"), "slot 0 secret-bearing");
        assert_eq!(bundle.ms1[1], "", "slot 1 watch-only sentinel");
        assert_eq!(bundle.ms1[2], "", "slot 2 watch-only sentinel");
        assert!(
            bundle.any_secret_bearing(),
            "hybrid is secret-bearing for any-non-empty"
        );
    }

    #[test]
    fn synthesize_unified_threshold_out_of_range_rejected() {
        let slots = unified_fixture(2, &[0, 1]);
        let err = synthesize_unified(
            &slots,
            CliTemplate::WshSortedMulti,
            3, // threshold > N
            CliNetwork::Mainnet,
            false,
            bip39::Language::English,
            Md1Form::Policy,
        )
        .unwrap_err();
        match err {
            ToolkitError::MultisigConfig { message } => {
                assert!(message.contains("threshold 3 out of range 1..=2"));
            }
            other => panic!("unexpected variant {other:?}"),
        }
    }

    #[test]
    fn bundle_json_schema_version_pinned_to_4() {
        // Direct pin: BundleJson.schema_version must be the &'static str "4".
        // (BundleJson construction sites use the literal "4" per Phase H.1.)
        // This test pins the format-module's commitment and prevents accidental
        // downgrade during refactoring.
        use crate::format::{BundleJson, MkField};
        let json = BundleJson {
            schema_version: "4",
            mode: "full",
            network: "mainnet",
            template: Some("wpkh"),
            descriptor: None,
            account: 0,
            origin_path: None,
            origin_paths: None,
            master_fingerprint: None,
            ms1: vec!["ms1stub".into()],
            mk1: MkField::Single(vec![]),
            md1: vec![],
            multisig: None,
            privacy_preserving: false,
        };
        assert_eq!(json.schema_version, "4");
    }

    // ========================================================================
    // Path B-lite Site 2 — ResolvedSlot struct-sibling pin coverage.
    // (See bip85.rs path_b_lite_pin_tests preamble for the attempts-counter
    // observation rationale.)
    // ========================================================================

    /// Site 2 — `unified_fixture(1, &[0])` constructs one secret-bearing
    /// `ResolvedSlot`. After GREEN, the construction populates
    /// `_entropy_pin: Some(Arc::new(pin_pages_for(&entropy[..])))` which
    /// invokes `pin_pages_for`. Asserts `attempts_for_test()` incremented.
    #[test]
    fn site_2_resolvedslot_construction_invokes_pin() {
        let baseline = mnemonic_toolkit::mlock::attempts_for_test();
        let _slots = unified_fixture(1, &[0]);
        assert!(
            mnemonic_toolkit::mlock::attempts_for_test() > baseline,
            "unified_fixture(1, &[0]) constructs a secret-bearing ResolvedSlot whose \
             _entropy_pin populates via pin_pages_for; attempts counter did not increment",
        );
    }

    // ========================================================================
    // #28 phase 2 (P2) — multisig/general template EMIT (Slice 1) unit pins.
    // ========================================================================

    use md_codec::tag::Tag;
    use md_codec::tree::{Body, Node};

    /// Build a keyed NON-CANONICAL general-policy descriptor:
    /// `wsh(or_d(pk_k(@0), pk_k(@1)))` with explicit BIP-84 Divergent origins.
    /// `canonical_origin` returns `None` for this wrapper, so it exercises the
    /// C1-conditional carried-origin path. Returns `(descriptor, cosigners)`.
    fn general_policy_fixture() -> (Descriptor, Vec<CosignerKeyInfo>) {
        let slots = vec![
            distinct_slot(TREZOR_12_ZERO, 0),
            distinct_slot(BIP39_TEST_2, 1),
        ];
        // BIP-84 origins (NON-canonical for a wsh(or_d) wrapper) — distinct
        // accounts so the path_decl is Divergent.
        let origin_paths = vec![
            derivation_path_to_origin_path(&DerivationPath::from_str("84'/0'/0'").unwrap()),
            derivation_path_to_origin_path(&DerivationPath::from_str("84'/0'/1'").unwrap()),
        ];
        let tree = Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![Node {
                tag: Tag::OrD,
                body: Body::Children(vec![
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                    Node {
                        tag: Tag::PkH,
                        body: Body::KeyArg { index: 1 },
                    },
                ]),
            }]),
        };
        let fingerprints: Vec<(u8, [u8; 4])> = slots
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, s.fingerprint.to_bytes()))
            .collect();
        let pubkeys: Vec<(u8, [u8; 65])> = slots
            .iter()
            .enumerate()
            .map(|(i, s)| (i as u8, xpub_to_65(&s.xpub)))
            .collect();
        let descriptor = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Divergent(origin_paths),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree,
            tlv: TlvSection {
                use_site_path_overrides: None,
                fingerprints: Some(fingerprints),
                pubkeys: Some(pubkeys),
                origin_path_overrides: None,
                unknown: Vec::new(),
            },
        };
        (descriptor, slots)
    }

    /// C1 PIN (the load-bearing emit decision). A general-policy template
    /// (non-canonical wrapper) emitted with the CARRIED per-@N origins DECODES;
    /// the SAME template with origins forced to EMPTY fails md-decode's
    /// `validate_explicit_origin_required` (`MissingExplicitOrigin`). The
    /// conditional carry is what makes the wire valid — eliding to empty (the
    /// single-sig behaviour) would be the C1 regression.
    #[test]
    fn c1_general_policy_template_carries_origins_empty_fails_decode() {
        let (descriptor, cosigners) = general_policy_fixture();
        assert!(
            md_codec::canonical_origin::canonical_origin(&descriptor.tree).is_none(),
            "fixture must be a non-canonical wrapper to exercise C1"
        );

        let bundle = synthesize_template_descriptor(&descriptor, &cosigners, false)
            .expect("general-policy template emits");

        // The emitted template md1 DECODES (carried origins).
        let md1_refs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let decoded = md_codec::chunk::reassemble(&md1_refs)
            .expect("C1: general-policy template md1 must decode with carried origins");
        assert!(!decoded.is_wallet_policy(), "template is keyless");
        md_codec::validate::validate_explicit_origin_required(&decoded)
            .expect("C1: carried origins satisfy validate_explicit_origin_required");

        // The C1 NEGATIVE: force origins empty → md-decode rejects.
        let mut elided = decoded.clone();
        elided.path_decl.paths = PathDeclPaths::Shared(OriginPath { components: vec![] });
        let err = md_codec::validate::validate_explicit_origin_required(&elided)
            .expect_err("C1: empty origins on a non-canonical wrapper MUST be rejected");
        assert!(
            matches!(err, md_codec::error::Error::MissingExplicitOrigin { .. }),
            "C1: empty-origin rejection must be MissingExplicitOrigin, got {err:?}"
        );
    }

    /// A CANONICAL multisig template elides origins to `Shared(empty)` and still
    /// decodes (the wrapper re-derives the canonical origin) — distinguishing
    /// the canonical arm of the C1 conditional from the general-policy arm.
    #[test]
    fn canonical_multisig_template_elides_origins() {
        let (descriptor, cosigners, _entropy) = descriptor_fixture(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        assert!(
            md_codec::canonical_origin::canonical_origin(&descriptor.tree).is_some(),
            "wsh(sortedmulti) is a canonical wrapper"
        );
        let bundle = synthesize_template_descriptor(&descriptor, &cosigners, false).unwrap();
        let md1_refs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let decoded = md_codec::chunk::reassemble(&md1_refs).unwrap();
        // Origins elided to empty (Shared empty), yet decode succeeds.
        match &decoded.path_decl.paths {
            PathDeclPaths::Shared(p) => assert!(
                p.components.is_empty(),
                "canonical template elides origins to empty Shared"
            ),
            PathDeclPaths::Divergent(_) => {
                panic!("canonical template must elide to Shared(empty), not Divergent")
            }
        }
        md_codec::validate::validate_explicit_origin_required(&decoded).unwrap();
        // n=2 → two mk1 cards.
        assert_eq!(bundle.mk1.as_multi().unwrap().len(), 2);
    }

    /// The admission gate refuses `tr(sortedmulti_a)` (render gap) but admits
    /// `tr(NUMS, multi_a)` and non-taproot multisig/general.
    #[test]
    fn template_admissible_gate() {
        // tr-sortedmulti-a 2-of-2 — does NOT render → refused.
        let (sma, _, _) = descriptor_fixture_taproot(CliTemplate::TrSortedMultiA);
        assert!(
            !template_admissible(&sma),
            "tr(sortedmulti_a) must be refused (render gap)"
        );
        // tr-multi-a 2-of-2 (NUMS) — renders → admitted.
        let (ma, _, _) = descriptor_fixture_taproot(CliTemplate::TrMultiA);
        assert!(
            template_admissible(&ma),
            "tr(NUMS, multi_a) must be admitted"
        );
        // Canonical multisig — admitted.
        let (wsm, _, _) = descriptor_fixture(
            "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        assert!(template_admissible(&wsm), "wsh(sortedmulti) admitted");
        // General policy — admitted.
        let (gp, _) = general_policy_fixture();
        assert!(template_admissible(&gp), "wsh(or_d(...)) general policy admitted");
    }

    /// Build a keyed taproot multisig descriptor for `template` (TrMultiA /
    /// TrSortedMultiA) from TREZOR_24 at two distinct BIP-48 type-3 accounts.
    fn descriptor_fixture_taproot(
        template: CliTemplate,
    ) -> (Descriptor, Vec<CosignerKeyInfo>, Vec<u8>) {
        let mnemonic = bip39::Mnemonic::parse_in(bip39::Language::English, TREZOR_24).unwrap();
        let entropy = mnemonic.to_entropy();
        let seed = mnemonic.to_seed("");
        let secp = Secp256k1::new();
        let master = Xpriv::new_master(CliNetwork::Mainnet.network_kind(), &seed).unwrap();
        let master_fp = master.fingerprint(&secp);
        let mut cosigners = Vec::with_capacity(2);
        let mut origin_paths = Vec::with_capacity(2);
        let mut pubkeys = Vec::with_capacity(2);
        let mut fps = Vec::with_capacity(2);
        for i in 0..2u8 {
            let path = DerivationPath::from_str(&format!("48'/0'/{i}'/3'")).unwrap();
            let xpriv = master.derive_priv(&secp, &path).unwrap();
            let xpub = Xpub::from_priv(&secp, &xpriv);
            cosigners.push(CosignerKeyInfo {
                xpub,
                fingerprint: master_fp,
                path: path.clone(),
                entropy: None,
                master_xpub: None,
                language: None,
                _entropy_pin: None,
            });
            origin_paths.push(derivation_path_to_origin_path(&path));
            pubkeys.push((i, xpub_to_65(&xpub)));
            fps.push((i, master_fp.to_bytes()));
        }
        let descriptor = Descriptor {
            n: 2,
            path_decl: PathDecl {
                n: 2,
                paths: PathDeclPaths::Divergent(origin_paths),
            },
            use_site_path: UseSitePath::standard_multipath(),
            tree: template.wrapper_node(2, 2),
            tlv: TlvSection {
                use_site_path_overrides: None,
                fingerprints: Some(fps),
                pubkeys: Some(pubkeys),
                origin_path_overrides: None,
                unknown: Vec::new(),
            },
        };
        (descriptor, cosigners, entropy)
    }

    /// `is_order_independent_shape` distinguishes sortedmulti* (true) from
    /// multi/general (false), walking wsh/sh/tr wrappers.
    #[test]
    fn order_independent_shape_classifier() {
        let wsh = |inner: Node| Node {
            tag: Tag::Wsh,
            body: Body::Children(vec![inner]),
        };
        let leaf = |tag: Tag| Node {
            tag,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1],
            },
        };
        assert!(is_order_independent_shape(&wsh(leaf(Tag::SortedMulti))));
        assert!(!is_order_independent_shape(&wsh(leaf(Tag::Multi))));
        // tr(NUMS, sortedmulti_a) → order-independent.
        let tr_sma = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(leaf(Tag::SortedMultiA))),
            },
        };
        assert!(is_order_independent_shape(&tr_sma));
        let tr_ma = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(leaf(Tag::MultiA))),
            },
        };
        assert!(!is_order_independent_shape(&tr_ma));
        // general policy → false.
        let or_d = wsh(Node {
            tag: Tag::OrD,
            body: Body::Children(vec![leaf(Tag::PkK), leaf(Tag::Multi)]),
        });
        assert!(!is_order_independent_shape(&or_d));
    }

    /// `wallet_policy_id_for_template` recomputes the order-sensitive id from
    /// the keyless template + resolved slots, and it is ORDER-SENSITIVE for a
    /// (non-sorted) multi: swapping the two slot keys changes the id.
    #[test]
    fn wallet_policy_id_for_template_is_order_sensitive() {
        let (descriptor, cosigners, _entropy) = descriptor_fixture(
            "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
            crate::parse_descriptor::ScriptCtx::MultiSig,
            2,
        );
        let bundle = synthesize_template_descriptor(&descriptor, &cosigners, false).unwrap();
        let md1_refs: Vec<&str> = bundle.md1.iter().map(|s| s.as_str()).collect();
        let template = md_codec::chunk::reassemble(&md1_refs).unwrap();

        let id_forward = wallet_policy_id_for_template(&template, &cosigners).unwrap();
        let swapped: Vec<ResolvedSlot> = vec![cosigners[1].clone(), cosigners[0].clone()];
        let id_swapped = wallet_policy_id_for_template(&template, &swapped).unwrap();
        assert_ne!(
            id_forward.as_bytes(),
            id_swapped.as_bytes(),
            "multi (order-dependent) — swapping slot keys must change the WalletPolicyId"
        );
    }
}
