//! Canonical `Descriptor` → BIP 388 `@N`-template renderer.
//!
//! Walks an md1 [`Descriptor`] AST (`Tag` / `Body` / `Node` / `UseSitePath`,
//! no rust-miniscript) and emits the keyless wallet-policy template string with
//! `@i` placeholders — e.g.
//! `wsh(or_i(and_v(v:after(1000000),...),multi(3,@0/<0;1>/*,...)))`.
//!
//! This is the **single source of truth** for the template rendering: the
//! `md` CLI (`md decode` / `md inspect`) delegates here, and the
//! `mnemonic` toolkit's `inspect` renders the same `template:` line by calling
//! [`descriptor_to_template`] — guaranteeing byte-identical output across both
//! binaries.
//!
//! Lifted verbatim from `md-cli`'s `format/text.rs` (the renderer previously
//! lived only in the CLI). The structural-guard error type changed from the
//! CLI's `CliError::TemplateParse` to the dedicated [`RenderError`] so
//! `md_codec::Error` stays a pure wire/decode taxonomy.

use crate::compose::{HashKind, HashLock};
use crate::encode::Descriptor;
use crate::nums::NUMS_H_POINT_X_ONLY_HEX;
use crate::policy_shape::{LockKind, lock_from_wire};
use crate::tag::Tag;
use crate::tree::{Body, InternalKey, Node};
use crate::use_site_path::UseSitePath;
use std::fmt::Write as _;

/// Error returned by the [`descriptor_to_template`] renderer.
///
/// The renderer's `Err` arms are **fail-closed structural guards** that never
/// fire on a decoder-produced [`Descriptor`] (a decoded AST is always
/// well-formed). They exist so a foreign/test-fabricated tree with an
/// impossible tag/body pairing produces a typed error instead of malformed
/// output. Kept a *separate* type from [`crate::Error`] (the wire/decode
/// taxonomy) because a text-render failure is not a wire error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RenderError {
    /// A tree node had a structurally-malformed tag/body pairing the renderer
    /// cannot serialize (e.g. a `Tag::Tr` without a `Body::Tr`).
    MalformedTree(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RenderError::MalformedTree(m) => write!(f, "malformed descriptor tree: {m}"),
        }
    }
}

impl std::error::Error for RenderError {}

/// Which literal values `render_node` writes for lock operands and hash
/// digests. `Literal` is the original, wire-faithful behavior; `Abstract`
/// replaces both with `kind#class` equality-class labels (see
/// [`descriptor_to_abstract_template`]).
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Literal,
    Abstract,
}

/// Per-render state for [`Mode::Abstract`]: one first-seen-order list per
/// lock kind and per hash kind, so equal values within a kind share a class
/// number and distinct values don't — assigned in the template's own
/// left-to-right traversal order. Unused (and left empty) under
/// [`Mode::Literal`].
struct RenderCtx {
    mode: Mode,
    older_blocks: Vec<u32>,
    older_units: Vec<u32>,
    after_height: Vec<u32>,
    after_time: Vec<u32>,
    sha256: Vec<HashLock>,
    hash256: Vec<HashLock>,
    ripemd160: Vec<HashLock>,
    hash160: Vec<HashLock>,
}

impl RenderCtx {
    fn new(mode: Mode) -> Self {
        Self {
            mode,
            older_blocks: Vec::new(),
            older_units: Vec::new(),
            after_height: Vec::new(),
            after_time: Vec::new(),
            sha256: Vec::new(),
            hash256: Vec::new(),
            ripemd160: Vec::new(),
            hash160: Vec::new(),
        }
    }

    /// 1-based class number for `value` within lock kind `kind`: the first
    /// distinct value seen for that kind gets 1, the next distinct value 2,
    /// and so on; a value equal to one already seen reuses its class.
    fn class_for_lock(&mut self, kind: LockKind, value: u32) -> usize {
        let list = match kind {
            LockKind::OlderBlocks => &mut self.older_blocks,
            LockKind::OlderUnits => &mut self.older_units,
            LockKind::AfterHeight => &mut self.after_height,
            LockKind::AfterTime => &mut self.after_time,
        };
        class_index(list, value)
    }

    /// Same as [`Self::class_for_lock`], scoped per [`HashKind`] instead.
    fn class_for_hash(&mut self, kind: HashKind, lock: HashLock) -> usize {
        let list = match kind {
            HashKind::Sha256 => &mut self.sha256,
            HashKind::Hash256 => &mut self.hash256,
            HashKind::Ripemd160 => &mut self.ripemd160,
            HashKind::Hash160 => &mut self.hash160,
        };
        class_index(list, lock)
    }
}

/// Return `value`'s 1-based position in `seen` (first-seen order),
/// appending it as a new class if it has not been seen before.
fn class_index<T: PartialEq>(seen: &mut Vec<T>, value: T) -> usize {
    match seen.iter().position(|v| *v == value) {
        Some(pos) => pos + 1,
        None => {
            seen.push(value);
            seen.len()
        }
    }
}

/// The abstract-mode class label for a lock kind — distinct from the bare
/// hash-kind token because `older(...)`/`after(...)` alone doesn't say
/// whether the operand is blocks vs. 512-second units, or a height vs. a
/// time; the label carries what the wrapper name can't.
fn lock_class_label(kind: LockKind) -> &'static str {
    match kind {
        LockKind::OlderBlocks => "older-blocks",
        LockKind::OlderUnits => "older-units",
        LockKind::AfterHeight => "after-height",
        LockKind::AfterTime => "after-time",
    }
}

/// Render a `Descriptor` back to a BIP 388 template string with `@i` placeholders.
pub fn descriptor_to_template(d: &Descriptor) -> Result<String, RenderError> {
    render_with(d, Mode::Literal)
}

/// Render with lock values and digests replaced by `kind#class`.
/// Used as the coordinator-compatibility key's template component.
pub fn descriptor_to_abstract_template(d: &Descriptor) -> Result<String, RenderError> {
    render_with(d, Mode::Abstract)
}

fn render_with(d: &Descriptor, mode: Mode) -> Result<String, RenderError> {
    let mut out = String::new();
    let mut ctx = RenderCtx::new(mode);
    render_node(
        &d.tree,
        d.n,
        &d.use_site_path,
        d.tlv.use_site_path_overrides.as_deref(),
        &mut ctx,
        &mut out,
    )?;
    Ok(out)
}

fn render_node(
    node: &Node,
    n: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    match node.tag {
        Tag::Wpkh => render_wrapper("wpkh", node, n, default_usp, overrides, ctx, out),
        Tag::Pkh => render_wrapper("pkh", node, n, default_usp, overrides, ctx, out),
        Tag::Wsh => render_wrapper("wsh", node, n, default_usp, overrides, ctx, out),
        Tag::Sh => render_wrapper("sh", node, n, default_usp, overrides, ctx, out),
        Tag::Tr => {
            out.push_str("tr(");
            match &node.body {
                Body::Tr { internal_key, tree } => {
                    // SPEC v0.30 §7 / stage 1b SPEC §4: NumsPoint encodes the
                    // BIP-341 NUMS H-point as the implicit internal key;
                    // render as the literal x-only hex. LianaUnspendable
                    // (wire kind 1) is a DIFFERENT internal key -- an xpub
                    // derived from the tap-tree's own leaf keys (SPEC §2) --
                    // but that derivation needs the already-built leaf keys
                    // AND a network to pick the base58 prefix, neither of
                    // which a keyless template carries. §4's table (row 2 and
                    // row 3): BOTH keyless render modes (this function serves
                    // both `Mode::Literal` and `Mode::Abstract` -- neither
                    // reads `ctx.mode` at this site) emit the fixed
                    // `LIANA_UNSPENDABLE_MARKER` text instead; only the
                    // *keyed* descriptor (`to_miniscript`, §4 row 1) embeds
                    // the real derived xpub. A Slot renders @{i}.
                    match internal_key {
                        InternalKey::NumsPoint => {
                            out.push_str(NUMS_H_POINT_X_ONLY_HEX);
                        }
                        InternalKey::LianaUnspendable => {
                            out.push_str(crate::nums::LIANA_UNSPENDABLE_MARKER);
                        }
                        InternalKey::Slot(i) => {
                            render_key(*i, default_usp, overrides, out)?;
                        }
                    }
                    if let Some(t) = tree {
                        out.push(',');
                        render_tap_node(t, n, default_usp, overrides, ctx, out)?;
                    }
                }
                _ => {
                    return Err(RenderError::MalformedTree(
                        "Tag::Tr without Body::Tr".into(),
                    ));
                }
            }
            out.push(')');
            Ok(())
        }
        Tag::Multi => render_multi("multi", node, default_usp, overrides, out),
        Tag::SortedMulti => render_multi("sortedmulti", node, default_usp, overrides, out),
        Tag::MultiA => render_multi("multi_a", node, default_usp, overrides, out),
        Tag::SortedMultiA => render_multi("sortedmulti_a", node, default_usp, overrides, out),
        Tag::PkK | Tag::PkH => match node.body {
            Body::KeyArg { index } => {
                // Tag::PkK on the wire encodes miniscript's `Terminal::PkK(K)`
                // (BIP-379 sugar `pk(K)` = `c:pk_k(K)`, type B). Tag::PkH
                // similarly encodes `Terminal::PkH(K)` (sugar `pkh(K)` =
                // `c:pk_h(K)`, type B). Render as the sugar form so the
                // emitted text re-parses through miniscript at the same
                // type the encoder accepted; the bare `pk_k(K)` / `pk_h(K)`
                // forms are type K, only valid as a `c:` child.
                if matches!(node.tag, Tag::PkH) {
                    out.push_str("pkh(");
                } else {
                    out.push_str("pk(");
                }
                render_key(index, default_usp, overrides, out)?;
                out.push(')');
                Ok(())
            }
            _ => Err(RenderError::MalformedTree(
                "PkK/PkH without KeyArg body".into(),
            )),
        },
        Tag::AndV => {
            // and_v(left, right) — function-call syntax. Used inside tap-script
            // leaves for and-conjunction / inheritance patterns.
            let kids = match &node.body {
                Body::Children(v) if v.len() == 2 => v,
                _ => {
                    return Err(RenderError::MalformedTree(
                        "AndV body must be Children([2])".into(),
                    ));
                }
            };
            out.push_str("and_v(");
            render_node(&kids[0], n, default_usp, overrides, ctx, out)?;
            out.push(',');
            render_node(&kids[1], n, default_usp, overrides, ctx, out)?;
            out.push(')');
            Ok(())
        }
        Tag::Older => {
            let v = match node.body {
                Body::Timelock(v) => v,
                _ => {
                    return Err(RenderError::MalformedTree(
                        "Older body must be Timelock".into(),
                    ));
                }
            };
            match ctx.mode {
                Mode::Literal => write!(out, "older({v})").unwrap(),
                Mode::Abstract => {
                    let lock = lock_from_wire(Tag::Older, v);
                    let class = ctx.class_for_lock(lock.kind, lock.value);
                    write!(out, "older({}#{class})", lock_class_label(lock.kind)).unwrap();
                }
            }
            Ok(())
        }
        Tag::After => {
            let v = match node.body {
                Body::Timelock(v) => v,
                _ => {
                    return Err(RenderError::MalformedTree(
                        "After body must be Timelock".into(),
                    ));
                }
            };
            match ctx.mode {
                Mode::Literal => write!(out, "after({v})").unwrap(),
                Mode::Abstract => {
                    let lock = lock_from_wire(Tag::After, v);
                    let class = ctx.class_for_lock(lock.kind, lock.value);
                    write!(out, "after({}#{class})", lock_class_label(lock.kind)).unwrap();
                }
            }
            Ok(())
        }
        Tag::AndB => render_binary("and_b", node, n, default_usp, overrides, ctx, out),
        Tag::OrB => render_binary("or_b", node, n, default_usp, overrides, ctx, out),
        Tag::OrC => render_binary("or_c", node, n, default_usp, overrides, ctx, out),
        Tag::OrD => render_binary("or_d", node, n, default_usp, overrides, ctx, out),
        Tag::OrI => render_binary("or_i", node, n, default_usp, overrides, ctx, out),
        Tag::AndOr => {
            // andor(a, b, c) — ternary "if a then b else c". Only ternary
            // fragment in miniscript; Body::Children must have length 3.
            let kids = match &node.body {
                Body::Children(v) if v.len() == 3 => v,
                _ => {
                    return Err(RenderError::MalformedTree(
                        "AndOr body must be Children([3])".into(),
                    ));
                }
            };
            out.push_str("andor(");
            render_node(&kids[0], n, default_usp, overrides, ctx, out)?;
            out.push(',');
            render_node(&kids[1], n, default_usp, overrides, ctx, out)?;
            out.push(',');
            render_node(&kids[2], n, default_usp, overrides, ctx, out)?;
            out.push(')');
            Ok(())
        }
        Tag::Sha256 => render_hash256(HashKind::Sha256, &node.body, ctx, out),
        Tag::Hash256 => render_hash256(HashKind::Hash256, &node.body, ctx, out),
        Tag::Ripemd160 => render_hash160(HashKind::Ripemd160, &node.body, ctx, out),
        Tag::Hash160 => render_hash160(HashKind::Hash160, &node.body, ctx, out),
        // `Tag::Verify` belongs HERE, not in an arm of its own. It used to have
        // one, which pushed a literal `"v:"` and recursed into `render_node` —
        // so a `v:` sitting above another wrapper emitted a second colon,
        // rendering `vj:` as `v:j:`. A doubled colon is not canonical
        // miniscript and rust-miniscript's own parser REJECTS it, meaning the
        // renderer could return a string no consumer could read back.
        Tag::Check
        | Tag::Swap
        | Tag::Alt
        | Tag::DupIf
        | Tag::NonZero
        | Tag::ZeroNotEqual
        | Tag::Verify => render_wrapper_chain(node, n, default_usp, overrides, ctx, out),
        Tag::True => {
            out.push('1');
            Ok(())
        }
        Tag::False => {
            out.push('0');
            Ok(())
        }
        Tag::RawPkH => {
            // Decode-side only — Tag::RawPkH carries a 20-byte hash in the
            // wire format. Miniscript's RawPkH variant is constructible only
            // from raw scripts (per upstream doc-comment), never from policy
            // or descriptor APIs, so this arm exists for round-trip fidelity
            // when md-codec encounters a RawPkH wire tag emitted by some
            // other producer.
            //
            // Rendering choice: emit `expr_raw_pkh(<hex>)` (no underscore
            // between `pk` and `h`; the parser-accepted checked form), not
            // the bare-K Display form `expr_raw_pk_h(<hex>)` (with
            // underscore). miniscript-rs's parser at `mod.rs:1017` only
            // accepts `expr_raw_pkh`, which produces
            // `Terminal::Check(Terminal::RawPkH(<hash>))` — type B. The bare
            // `expr_raw_pk_h` Display form is type K (display.rs:248) and is
            // an internal artifact, not a spec-level string. Emitting the
            // checked form matches the v0.4.2 PkH pattern (bare `Tag::PkH`
            // → `pkh(K)`, the type-B sugar; `Tag::RawPkH` → `expr_raw_pkh(<hex>)`,
            // its type-B sugar) and produces output that re-parses through
            // miniscript. Absorbs the would-be `Check(RawPkH)` shorthand-
            // collapse case at the bare arm, so `render_wrapper_chain`
            // needs no parallel extension.
            let h = match &node.body {
                Body::Hash160Body(h) => h,
                _ => {
                    return Err(RenderError::MalformedTree(
                        "RawPkH body must be Hash160Body".into(),
                    ));
                }
            };
            out.push_str("expr_raw_pkh(");
            for byte in h {
                write!(out, "{byte:02x}").unwrap();
            }
            out.push(')');
            Ok(())
        }
        Tag::Thresh => {
            // thresh(k, c1, c2, ..., cn) — k-of-n threshold over arbitrary
            // miniscript fragments (distinct from Multi/MultiA which take only
            // keys). Each child is rendered recursively.
            let (k, children) = match &node.body {
                Body::Variable { k, children } => (*k, children),
                _ => {
                    return Err(RenderError::MalformedTree(
                        "Thresh body must be Variable".into(),
                    ));
                }
            };
            write!(out, "thresh({k}").unwrap();
            for child in children {
                out.push(',');
                render_node(child, n, default_usp, overrides, ctx, out)?;
            }
            out.push(')');
            Ok(())
        }
        other => Err(RenderError::MalformedTree(format!(
            "unsupported tag in render: {other:?}"
        ))),
    }
}

/// Render a 32-byte-hash literal (sha256, hash256). Body must be Hash256Body.
/// In [`Mode::Abstract`], writes `{kind}(#{class})` instead of the digest —
/// `HashLock::new` takes the digest at its full alloc-gate width, and
/// `Hash256Body` is already exactly `kind.digest_len()` (32) bytes wide, so
/// no padding is introduced here.
fn render_hash256(
    kind: HashKind,
    body: &Body,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    let h = match body {
        Body::Hash256Body(h) => h,
        _ => {
            return Err(RenderError::MalformedTree(format!(
                "{} body must be Hash256Body",
                kind.token()
            )));
        }
    };
    match ctx.mode {
        Mode::Literal => {
            out.push_str(kind.token());
            out.push('(');
            for byte in h {
                write!(out, "{byte:02x}").unwrap();
            }
            out.push(')');
        }
        Mode::Abstract => {
            let class = ctx.class_for_hash(kind, HashLock::new(kind, *h));
            write!(out, "{}(#{class})", kind.token()).unwrap();
        }
    }
    Ok(())
}

/// Render a 20-byte-hash literal (ripemd160, hash160). Body must be
/// Hash160Body. In [`Mode::Abstract`], writes `{kind}(#{class})` instead of
/// the digest. `HashLock::new` takes the full 32-byte alloc-gate slot;
/// `Hash160Body`'s 20 real bytes are placed at the front and the rest
/// zero-padded — `HashLock::digest()` never reads past `kind.digest_len()`
/// (20 for this arm), so that padding is never observed. NEVER read past
/// `digest()`'s slice: committing the padding into a lowering produces a
/// wallet nobody can spend.
fn render_hash160(
    kind: HashKind,
    body: &Body,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    let h = match body {
        Body::Hash160Body(h) => h,
        _ => {
            return Err(RenderError::MalformedTree(format!(
                "{} body must be Hash160Body",
                kind.token()
            )));
        }
    };
    match ctx.mode {
        Mode::Literal => {
            out.push_str(kind.token());
            out.push('(');
            for byte in h {
                write!(out, "{byte:02x}").unwrap();
            }
            out.push(')');
        }
        Mode::Abstract => {
            let mut digest = [0u8; 32];
            digest[..20].copy_from_slice(h);
            let class = ctx.class_for_hash(kind, HashLock::new(kind, digest));
            write!(out, "{}(#{class})", kind.token()).unwrap();
        }
    }
    Ok(())
}

/// Render a chain of single-letter prefix wrappers as miniscript's canonical
/// concatenated form: e.g. `Swap(NonZero(DupIf(X)))` → `snj:X` (not
/// `s:n:j:X`). Walks down the wrapper spine, accumulating letters, then
/// renders the innermost non-wrapper fragment after a single `:`.
///
/// # Caller contract
///
/// Reachable from exactly one site: the wrapper-chain dispatch arm in
/// [`render_node`] for tags `Check | Swap | Alt | DupIf | NonZero |
/// ZeroNotEqual | Verify`. The function MUST NOT be called with any other tag —
/// its first-iteration loop body relies on the head being a wrapper, and
/// passing a non-wrapper would emit a malformed bare `:` followed by the
/// inner render. The `debug_assert!` below pins this invariant in tests
/// and debug builds. A structural restructure that peels the first letter
/// unconditionally would also work but adds complexity for no live-bug
/// benefit; the assert is sufficient.
///
/// Special cases:
/// - `Check(PkK)` / `Check(PkH)` collapse to `pk(K)` / `pkh(K)`. v0.30 SPEC
///   §5.1 (Q12 — walker normalization) makes the v0.30 md-cli walker emit
///   bare `Tag::PkK` / `Tag::PkH` at every key-leaf position, so this arm
///   is unreachable on v0.30-produced wires (the bare-PkK/PkH arm in
///   [`render_node`] handles the shorthand directly). Retained as defensive
///   coverage for foreign/legacy/test-fabricated wires that still carry the
///   wrapped shape.
/// - When `n:` (Tag::ZeroNotEqual) appears immediately before a `0` literal,
///   miniscript prints `n0` not `n:0`. Phase 4b doesn't pin this corner; bare
///   `0` at top-level is structurally degenerate. Handle if a future test
///   surfaces it.
fn render_wrapper_chain(
    node: &Node,
    n: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    // The single dispatch arm at render_node guarantees `node.tag` is one of
    // the SEVEN wrapper tags (Check/Swap/Alt/DupIf/NonZero/ZeroNotEqual/Verify),
    // so the first iteration of the loop below always assigns a non-None letter.
    // Guard the empty-prefix case in debug builds to make the invariant
    // explicit (release builds skip the assertion; the invariant is upheld by
    // the dispatch site, not by render_wrapper_chain itself).
    debug_assert!(
        matches!(
            node.tag,
            Tag::Check
                | Tag::Swap
                | Tag::Alt
                | Tag::DupIf
                | Tag::NonZero
                | Tag::ZeroNotEqual
                | Tag::Verify
        ),
        "render_wrapper_chain called on non-wrapper tag {:?}",
        node.tag
    );
    let mut prefix = String::new();
    let mut current = node;
    loop {
        let letter = match current.tag {
            Tag::Check => Some('c'),
            Tag::Swap => Some('s'),
            Tag::Alt => Some('a'),
            Tag::DupIf => Some('d'),
            Tag::NonZero => Some('j'),
            Tag::ZeroNotEqual => Some('n'),
            Tag::Verify => Some('v'),
            _ => None,
        };
        match letter {
            Some(c) => {
                prefix.push(c);
                current = match &current.body {
                    Body::Children(v) if v.len() == 1 => &v[0],
                    _ => {
                        return Err(RenderError::MalformedTree(format!(
                            "{c}: wrapper body must be Children([1])"
                        )));
                    }
                };
            }
            None => break,
        }
    }
    // After collapsing the chain, if the deepest inner is PkK or PkH and the
    // chain ends in `c`, emit the miniscript shorthand `pk(K)` / `pkh(K)`.
    // (See the bare-PkK/PkH arm above for the BIP-379-sugar-form rationale.)
    if prefix.ends_with('c') && matches!(current.tag, Tag::PkK | Tag::PkH) {
        let prefix_no_c = &prefix[..prefix.len() - 1];
        if !prefix_no_c.is_empty() {
            out.push_str(prefix_no_c);
            out.push(':');
        }
        let idx = match current.body {
            Body::KeyArg { index } => index,
            _ => {
                return Err(RenderError::MalformedTree(
                    "Check(PkK/PkH) inner body must be KeyArg".into(),
                ));
            }
        };
        if matches!(current.tag, Tag::PkH) {
            out.push_str("pkh(");
        } else {
            out.push_str("pk(");
        }
        render_key(idx, default_usp, overrides, out)?;
        out.push(')');
        return Ok(());
    }
    out.push_str(&prefix);
    out.push(':');
    render_node(current, n, default_usp, overrides, ctx, out)
}

/// Render a binary fragment `name(left, right)` — used for and_b, or_b, or_c,
/// or_d, or_i. Body::Children must have exactly 2 elements.
fn render_binary(
    name: &str,
    node: &Node,
    n: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    let kids = match &node.body {
        Body::Children(v) if v.len() == 2 => v,
        _ => {
            return Err(RenderError::MalformedTree(format!(
                "{name} body must be Children([2])"
            )));
        }
    };
    out.push_str(name);
    out.push('(');
    render_node(&kids[0], n, default_usp, overrides, ctx, out)?;
    out.push(',');
    render_node(&kids[1], n, default_usp, overrides, ctx, out)?;
    out.push(')');
    Ok(())
}

/// Render a single-arity wrapper (wsh, sh, wpkh, pkh) — both `Children([inner])`
/// and `KeyArg{index}` (Wpkh/Pkh leaf form) work.
fn render_wrapper(
    name: &str,
    node: &Node,
    n: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    out.push_str(name);
    out.push('(');
    match &node.body {
        Body::KeyArg { index } => render_key(*index, default_usp, overrides, out)?,
        Body::Children(v) if v.len() == 1 => {
            render_node(&v[0], n, default_usp, overrides, ctx, out)?
        }
        _ => {
            return Err(RenderError::MalformedTree(format!(
                "{name} body must be KeyArg or Children([1])"
            )));
        }
    }
    out.push(')');
    Ok(())
}

fn render_multi(
    name: &str,
    node: &Node,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    out: &mut String,
) -> Result<(), RenderError> {
    // v0.30 Phase C: multi-family bodies carry raw key indices, not child Nodes.
    let (k, indices) = match &node.body {
        Body::MultiKeys { k, indices } => (*k, indices),
        _ => {
            return Err(RenderError::MalformedTree(format!(
                "{name} body must be MultiKeys"
            )));
        }
    };
    write!(out, "{name}({k}").unwrap();
    for idx in indices {
        out.push(',');
        render_key(*idx, default_usp, overrides, out)?;
    }
    out.push(')');
    Ok(())
}

/// Render a tap-tree node. Branches → `{left,right}`; leaves → render their body
/// directly (no wrapper around the leaf).
fn render_tap_node(
    node: &Node,
    n: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    ctx: &mut RenderCtx,
    out: &mut String,
) -> Result<(), RenderError> {
    if matches!(node.tag, Tag::TapTree) {
        let children = match &node.body {
            Body::Children(v) if v.len() == 2 => v,
            _ => {
                return Err(RenderError::MalformedTree(
                    "TapTree must have Children([2])".into(),
                ));
            }
        };
        out.push('{');
        render_tap_node(&children[0], n, default_usp, overrides, ctx, out)?;
        out.push(',');
        render_tap_node(&children[1], n, default_usp, overrides, ctx, out)?;
        out.push('}');
        Ok(())
    } else {
        render_node(node, n, default_usp, overrides, ctx, out)
    }
}

fn render_key(
    idx: u8,
    default_usp: &UseSitePath,
    overrides: Option<&[(u8, UseSitePath)]>,
    out: &mut String,
) -> Result<(), RenderError> {
    let usp = overrides
        .and_then(|v| v.iter().find(|(i, _)| *i == idx).map(|(_, u)| u))
        .unwrap_or(default_usp);
    write!(out, "@{idx}").unwrap();
    if let Some(alts) = &usp.multipath {
        out.push_str("/<");
        for (n, alt) in alts.iter().enumerate() {
            if n > 0 {
                out.push(';');
            }
            write!(out, "{}", alt.value).unwrap();
            if alt.hardened {
                out.push('\'');
            }
        }
        out.push_str(">/*");
    } else {
        out.push_str("/*");
    }
    if usp.wildcard_hardened {
        out.push('\'');
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// v0.4.3 — bare `Tag::RawPkH` rendering, unit-pinned at the `render_node`
    /// level. Since v0.10.0 the walker DOES emit `Tag::RawPkH` (for the parseable
    /// `wsh(c:expr_raw_pkh(<hash>))` form → `Wsh → Check → RawPkH`); the walk half
    /// is pinned by `walk_rawpkh_wsh_check_emits_rawpkh_node` in `md-cli`'s
    /// `parse::template`. This test still constructs the Node directly to pin the
    /// bare-node rendering invariant in isolation (no `@N` placeholder is
    /// involved, so the full `parse_template` pipeline — which requires
    /// placeholders — is not the entry point here). Asserts the output matches
    /// the parser-accepted Display form `expr_raw_pkh(<hex>)` — see the
    /// doc-comment on the arm at `render_node`'s Tag::RawPkH match for the
    /// rationale.
    ///
    /// Relocated from `md-cli` `format/text.rs` when the renderer moved into
    /// md-codec (the md-cli copy called the now-removed local `render_node`).
    /// Pins the `pk`-shorthand collapse for a **multi-letter** chain ending in
    /// `c` — `Verify(Check(PkK))` → `v:pk(K)`, not `vc:pk_k(K)`.
    ///
    /// **Why this is a fabricated node rather than a `parse_template` case.**
    /// Since v0.30 the md-cli walker emits a **bare** `Tag::PkK` at every
    /// key-leaf position, so no wire the current encoder produces ever carries
    /// an explicit `Tag::Check` above a key. The shorthand branch therefore
    /// exists only as defensive coverage for foreign, legacy or
    /// test-fabricated wires — and was, until this test, **reachable by
    /// nothing**: an exec review mutated `prefix.ends_with('c')` to
    /// `prefix == "c"` and all 782 tests stayed green.
    ///
    /// That mutation is exactly what this test kills: under it the chain `vc`
    /// no longer takes the shorthand and renders `vc:pk_k(@0/<0;1>/*)`.
    #[test]
    fn render_verify_over_check_pkk_uses_pk_shorthand() {
        let node = Node {
            tag: Tag::Verify,
            body: Body::Children(vec![Node {
                tag: Tag::Check,
                body: Body::Children(vec![Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                }]),
            }]),
        };
        let usp = UseSitePath::standard_multipath();
        let mut out = String::new();
        let mut ctx = RenderCtx::new(Mode::Literal);
        render_node(
            &node, /* n */ 1, &usp, /* overrides */ None, &mut ctx, &mut out,
        )
        .expect("render_node Verify(Check(PkK)) must succeed");
        assert_eq!(
            out, "v:pk(@0/<0;1>/*)",
            "a multi-letter chain ending in `c` must still take the pk shorthand"
        );
    }

    #[test]
    fn render_bare_rawpkh_emits_expr_raw_pkh() {
        let node = Node {
            tag: Tag::RawPkH,
            body: Body::Hash160Body([0u8; 20]),
        };
        let usp = UseSitePath::standard_multipath();
        let mut out = String::new();
        let mut ctx = RenderCtx::new(Mode::Literal);
        render_node(
            &node, /* n */ 1, &usp, /* overrides */ None, &mut ctx, &mut out,
        )
        .expect("render_node Tag::RawPkH must succeed");
        assert_eq!(
            out,
            "expr_raw_pkh(0000000000000000000000000000000000000000)",
        );
    }
}
