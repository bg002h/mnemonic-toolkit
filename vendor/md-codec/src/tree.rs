//! Tree (operator AST) per spec §3.6 + §6.

use crate::bitstream::{BitReader, BitWriter};
use crate::error::Error;
use crate::tag::Tag;

/// A node in the operator AST: a tag plus its body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Node {
    /// Operator tag identifying this node's kind.
    pub tag: Tag,
    /// Body fields and/or children, shape determined by `tag`.
    pub body: Body,
}

/// Body shape for a [`Node`], determined by its [`Tag`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    /// No body fields beyond N child nodes (Class 1 fixed-arity).
    Children(Vec<Node>),
    /// Variable-arity body for `Tag::Thresh` only (post-v0.30 Phase C).
    /// Encodes `k` + N child Nodes. Multi-family tags use [`Body::MultiKeys`]
    /// per SPEC v0.30 §4.
    Variable {
        /// Threshold `k`.
        k: u8,
        /// Child nodes; `n = children.len()`.
        children: Vec<Node>,
    },
    /// Multi-family body (`Tag::Multi`, `SortedMulti`, `MultiA`,
    /// `SortedMultiA`): k-of-n with raw `kiw`-width key indices, NOT full
    /// child Nodes. Per SPEC v0.30 §4: wire layout is
    /// `tag | (k-1)(5) | (n-1)(5) | n × index(kiw)`.
    MultiKeys {
        /// Threshold `k`.
        k: u8,
        /// Placeholder indices `@i`; `n = indices.len()`. Each entry is
        /// emitted as `kiw` bits.
        indices: Vec<u8>,
    },
    /// Tr's body: NUMS flag, key index, optional tap-script-tree root.
    /// Per SPEC v0.30 §7: wire shape is
    /// `Tag::Tr | is_nums(1) | [key_index(kiw) iff !is_nums] | has_tree(1) | [tree iff has_tree]`.
    /// When `is_nums = true`, the internal key is the BIP-341 NUMS H-point
    /// `50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0`
    /// and `key_index` is unused on the wire (encoder writes 0 by convention;
    /// decoder ignores). When `is_nums = false`, `key_index` is a `0..n`
    /// placeholder index encoded at `kiw` bits.
    Tr {
        /// `true` iff the internal key is the BIP-341 NUMS H-point.
        is_nums: bool,
        /// Internal-key index into the descriptor's key table. Unused when
        /// `is_nums = true` (no wire representation).
        key_index: u8,
        /// Optional tap-script-tree root.
        tree: Option<Box<Node>>,
    },
    /// Single key-arg (Pkh, Wpkh, PkK, PkH, multi-family children).
    /// Wire bit-width for `index` is determined by the parent Descriptor's
    /// key_index_width(); not carried in the AST.
    KeyArg {
        /// Key index into the descriptor's key table.
        index: u8,
    },
    /// 256-bit hash literal (Sha256, Hash256).
    Hash256Body([u8; 32]),
    /// 160-bit hash literal (Hash160, Ripemd160, RawPkH).
    Hash160Body([u8; 20]),
    /// 32-bit Bitcoin-native u32 (After, Older).
    Timelock(u32),
    /// No body (False, True).
    Empty,
}

/// Encode a [`Node`] to the bit stream.
///
/// `key_index_width` is the bit width used for key-index fields, derived from
/// the descriptor's path-decl head. Filled in across phases 7-11.
pub fn write_node(w: &mut BitWriter, node: &Node, key_index_width: u8) -> Result<(), Error> {
    node.tag.write(w);
    match &node.body {
        Body::KeyArg { index } => {
            w.write_bits(u64::from(*index), key_index_width as usize);
        }
        Body::Children(children) => {
            for c in children {
                write_node(w, c, key_index_width)?;
            }
        }
        Body::Variable { k, children } => {
            // Thresh-only post-v0.30 Phase C. Encode k-1 in 5 bits per spec §4.2.
            if !(1..=32).contains(&(*k as u32)) {
                return Err(Error::ThresholdOutOfRange { k: *k });
            }
            if !(1..=32).contains(&(children.len() as u32)) {
                return Err(Error::ChildCountOutOfRange {
                    count: children.len(),
                });
            }
            // Reject k > n at encode, mirroring the decode-side reject below
            // (KGreaterThanN). Without this the encoder emits a card no decoder
            // will read back — an engrave-but-can't-restore gap.
            if *k as usize > children.len() {
                return Err(Error::KGreaterThanN {
                    k: *k,
                    n: children.len(),
                });
            }
            w.write_bits((*k - 1) as u64, 5);
            w.write_bits((children.len() - 1) as u64, 5);
            for c in children {
                write_node(w, c, key_index_width)?;
            }
        }
        Body::MultiKeys { k, indices } => {
            // Multi-family per SPEC v0.30 §4: k-of-n + raw kiw-width indices.
            if !(1..=32).contains(&(*k as u32)) {
                return Err(Error::ThresholdOutOfRange { k: *k });
            }
            if !(1..=32).contains(&(indices.len() as u32)) {
                return Err(Error::ChildCountOutOfRange {
                    count: indices.len(),
                });
            }
            // Reject k > n at encode, mirroring the decode-side reject below
            // (KGreaterThanN). Without this the encoder emits a card no decoder
            // will read back — an engrave-but-can't-restore gap.
            if *k as usize > indices.len() {
                return Err(Error::KGreaterThanN {
                    k: *k,
                    n: indices.len(),
                });
            }
            w.write_bits((*k - 1) as u64, 5);
            w.write_bits((indices.len() - 1) as u64, 5);
            for idx in indices {
                w.write_bits(u64::from(*idx), key_index_width as usize);
            }
        }
        Body::Tr {
            is_nums,
            key_index,
            tree,
        } => {
            // SPEC v0.30 §7: is_nums(1) | [key_index(kiw) iff !is_nums] |
            // has_tree(1) | [tree iff has_tree].
            debug_assert!(
                !(*is_nums && *key_index != 0),
                "is_nums=true implies key_index=0 (no wire representation otherwise)"
            );
            w.write_bits(u64::from(*is_nums), 1);
            if !*is_nums {
                w.write_bits(u64::from(*key_index), key_index_width as usize);
            }
            w.write_bits(u64::from(tree.is_some()), 1);
            if let Some(t) = tree {
                write_node(w, t, key_index_width)?;
            }
        }
        Body::Timelock(v) => {
            w.write_bits(u64::from(*v), 32);
        }
        Body::Hash256Body(h) => {
            for byte in h {
                w.write_bits(u64::from(*byte), 8);
            }
        }
        Body::Hash160Body(h) => {
            for byte in h {
                w.write_bits(u64::from(*byte), 8);
            }
        }
        Body::Empty => {}
    }
    Ok(())
}

/// Hard cap on `read_node` recursion depth. Shared across all recursive tags
/// (`Sh`, `AndV`, `AndOr`, `TapTree`, `Multi`, `Tr`, …) as a generic anti-DOS
/// hardening bound — not a spec-mandated value for non-taproot sites. The
/// value 128 happens to coincide with BIP-341 `TAPROOT_CONTROL_MAX_NODE_COUNT`,
/// but its role here is just "any depth a real miniscript expression could
/// plausibly reach + headroom"; P2WSH script-size limits cap practical
/// miniscript depth at well under 50.
pub const MAX_DECODE_DEPTH: u8 = 128;

/// Decode a [`Node`] from the bit stream.
///
/// `key_index_width` is the bit width used for key-index fields, derived from
/// the descriptor's path-decl head. Filled in across phases 7-11.
///
/// Top-level entry point. Internally threads a recursion-depth counter that
/// errors out at [`MAX_DECODE_DEPTH`] before parsing the next node, so a
/// hostile wire payload nesting recursive tags (`Tag::Sh`, `Tag::AndV`,
/// `Tag::TapTree`, etc.) arbitrarily deep cannot blow the Rust stack.
pub fn read_node(r: &mut BitReader, key_index_width: u8) -> Result<Node, Error> {
    read_node_with_depth(r, key_index_width, 0)
}

/// Inner recursive form of `read_node` that threads `depth`. Public callers
/// should use `read_node` instead, which starts at depth 0. Increments
/// `depth` once per call and errors if it reaches [`MAX_DECODE_DEPTH`].
fn read_node_with_depth(r: &mut BitReader, key_index_width: u8, depth: u8) -> Result<Node, Error> {
    if depth >= MAX_DECODE_DEPTH {
        return Err(Error::DecodeRecursionDepthExceeded {
            depth,
            max: MAX_DECODE_DEPTH,
        });
    }
    let tag = Tag::read(r)?;
    let body = match tag {
        Tag::PkK | Tag::PkH | Tag::Wpkh | Tag::Pkh => {
            let index = r.read_bits(key_index_width as usize)? as u8;
            Body::KeyArg { index }
        }
        Tag::Sh
        | Tag::Wsh
        | Tag::Check
        | Tag::Verify
        | Tag::Swap
        | Tag::Alt
        | Tag::DupIf
        | Tag::NonZero
        | Tag::ZeroNotEqual => {
            let child = read_node_with_depth(r, key_index_width, depth + 1)?;
            Body::Children(vec![child])
        }
        Tag::AndV | Tag::AndB | Tag::OrB | Tag::OrC | Tag::OrD | Tag::OrI => {
            let l = read_node_with_depth(r, key_index_width, depth + 1)?;
            let r2 = read_node_with_depth(r, key_index_width, depth + 1)?;
            Body::Children(vec![l, r2])
        }
        Tag::AndOr => {
            let a = read_node_with_depth(r, key_index_width, depth + 1)?;
            let b = read_node_with_depth(r, key_index_width, depth + 1)?;
            let c = read_node_with_depth(r, key_index_width, depth + 1)?;
            Body::Children(vec![a, b, c])
        }
        Tag::TapTree => {
            let l = read_node_with_depth(r, key_index_width, depth + 1)?;
            let r2 = read_node_with_depth(r, key_index_width, depth + 1)?;
            Body::Children(vec![l, r2])
        }
        Tag::Multi | Tag::SortedMulti | Tag::MultiA | Tag::SortedMultiA => {
            let k = (r.read_bits(5)? + 1) as u8;
            let count = (r.read_bits(5)? + 1) as usize;
            if k as usize > count {
                return Err(Error::KGreaterThanN { k, n: count });
            }
            let mut indices = Vec::with_capacity(count);
            for _ in 0..count {
                indices.push(r.read_bits(key_index_width as usize)? as u8);
            }
            Body::MultiKeys { k, indices }
        }
        Tag::Thresh => {
            let k = (r.read_bits(5)? + 1) as u8;
            let count = (r.read_bits(5)? + 1) as usize;
            if k as usize > count {
                return Err(Error::KGreaterThanN { k, n: count });
            }
            let mut children = Vec::with_capacity(count);
            for _ in 0..count {
                children.push(read_node_with_depth(r, key_index_width, depth + 1)?);
            }
            Body::Variable { k, children }
        }
        Tag::Tr => {
            // SPEC v0.30 §7: is_nums(1) | [key_index(kiw) iff !is_nums] |
            // has_tree(1) | [tree iff has_tree].
            let is_nums = r.read_bits(1)? != 0;
            let key_index = if is_nums {
                0
            } else {
                r.read_bits(key_index_width as usize)? as u8
            };
            let has_tree = r.read_bits(1)? != 0;
            let tree = if has_tree {
                Some(Box::new(read_node_with_depth(
                    r,
                    key_index_width,
                    depth + 1,
                )?))
            } else {
                None
            };
            Body::Tr {
                is_nums,
                key_index,
                tree,
            }
        }
        Tag::After | Tag::Older => {
            let v = r.read_bits(32)? as u32;
            Body::Timelock(v)
        }
        Tag::Sha256 => {
            let mut h = [0u8; 32];
            for byte in &mut h {
                *byte = r.read_bits(8)? as u8;
            }
            Body::Hash256Body(h)
        }
        Tag::Hash160 => {
            let mut h = [0u8; 20];
            for byte in &mut h {
                *byte = r.read_bits(8)? as u8;
            }
            Body::Hash160Body(h)
        }
        Tag::Hash256 => {
            let mut h = [0u8; 32];
            for byte in &mut h {
                *byte = r.read_bits(8)? as u8;
            }
            Body::Hash256Body(h)
        }
        Tag::Ripemd160 | Tag::RawPkH => {
            let mut h = [0u8; 20];
            for byte in &mut h {
                *byte = r.read_bits(8)? as u8;
            }
            Body::Hash160Body(h)
        }
        Tag::False | Tag::True => Body::Empty,
    };
    Ok(Node { tag, body })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bitstream::{BitReader, BitWriter};

    #[test]
    fn key_arg_n1_zero_bits() {
        // v0.30: at n=1, kiw = ⌈log₂(1)⌉ = 0; key-arg emits zero bits.
        let n = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag::PkK (6 bits) + key-arg (0 bits) = 6 bits total.
        assert_eq!(w.bit_len(), 6);
    }

    #[test]
    fn key_arg_n3_two_bits() {
        // v0.30: at n=3, kiw = ⌈log₂(3)⌉ = 2.
        let n = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 2 },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        // Tag::PkK (6 bits) + key-arg (2 bits) = 8 bits total.
        assert_eq!(w.bit_len(), 8);
    }

    #[test]
    fn key_arg_round_trip() {
        let n = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 1 },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn wrapper_chain_v_c_pk_round_trip() {
        // v:c:pk_k(@0) — three nested wrappers around PkK
        let n = Node {
            tag: Tag::Verify,
            body: Body::Children(vec![Node {
                tag: Tag::Check,
                body: Body::Children(vec![Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: 0 },
                }]),
            }]),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn sortedmulti_2of3_round_trip() {
        let n = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.30 Phase C — multi packing bit-cost pin.
    /// `Tag(6-bit) | k-1(5) | n-1(5) | 3×kiw(2 at n=3) = 22 bits` (SPEC §4.2).
    /// Saves 14 bits over v0.x's per-child encoding (which was 36 bits).
    #[test]
    fn sortedmulti_2of3_bit_cost() {
        let n = Node {
            tag: Tag::SortedMulti,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        assert_eq!(w.bit_len(), 22);
    }

    /// v0.30 Phase C — `Body::MultiKeys` round-trips under `Tag::Multi`.
    #[test]
    fn multi_keys_body_round_trip() {
        let n = Node {
            tag: Tag::Multi,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.30 Phase C — `Body::MultiKeys` round-trips under `Tag::SortedMultiA`.
    #[test]
    fn sortedmulti_a_indices_round_trip() {
        let n = Node {
            tag: Tag::SortedMultiA,
            body: Body::MultiKeys {
                k: 2,
                indices: vec![0, 1, 2],
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn tr_bip86_no_tree() {
        // v0.30: tr(@0) keypath-only at synthetic width=0 (n=1 in Descriptor
        // would yield kiw=0). Exercises the zero-width edge of write_node /
        // read_node directly.
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag::Tr (6) + is_nums (1) + key_index (0, kiw=0) + has_tree (1) = 8 bits.
        assert_eq!(w.bit_len(), 8);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn thresh_2of3_with_pk_children() {
        // thresh(2, pk_k(@0), pk_k(@1), pk_k(@2))
        let n = Node {
            tag: Tag::Thresh,
            body: Body::Variable {
                k: 2,
                children: vec![
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 1 },
                    },
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 2 },
                    },
                ],
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn tr_with_single_leaf() {
        // tr(@0, multi_a(2, @1, @2))
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::MultiA,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![1, 2],
                    },
                })),
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.30 Phase F — `Body::Tr { is_nums: true, .. }` round-trips. NUMS
    /// suppresses the kiw field on the wire entirely.
    #[test]
    fn tr_nums_flag_round_trip() {
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.30 Phase F — `Body::Tr { is_nums: false, key_index, .. }` round-
    /// trips with explicit key_index written at kiw width.
    #[test]
    fn tr_is_nums_false_round_trip() {
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 2,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn after_700_000_round_trip() {
        let n = Node {
            tag: Tag::After,
            body: Body::Timelock(700_000),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag(6) + u32(32) = 38 bits
        assert_eq!(w.bit_len(), 38);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn sha256_round_trip() {
        let h = [0xab; 32];
        let n = Node {
            tag: Tag::Sha256,
            body: Body::Hash256Body(h),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag(6) + 256 = 262 bits
        assert_eq!(w.bit_len(), 262);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn hash160_round_trip() {
        let h = [0xcd; 20];
        let n = Node {
            tag: Tag::Hash160,
            body: Body::Hash160Body(h),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag(6) + 160 = 166 bits
        assert_eq!(w.bit_len(), 166);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn hash256_round_trip() {
        let h = [0xef; 32];
        let n = Node {
            tag: Tag::Hash256,
            body: Body::Hash256Body(h),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        // Tag(6) + 256 = 262 bits (Hash256 primary 0x1F in v0.30).
        assert_eq!(w.bit_len(), 262);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn ripemd160_round_trip() {
        let h = [0x42; 20];
        let n = Node {
            tag: Tag::Ripemd160,
            body: Body::Hash160Body(h),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn false_round_trip() {
        let n = Node {
            tag: Tag::False,
            body: Body::Empty,
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        assert_eq!(w.bit_len(), 6); // Tag(6), no body (False primary 0x22 in v0.30)
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn true_round_trip() {
        let n = Node {
            tag: Tag::True,
            body: Body::Empty,
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn older_144_round_trip() {
        let n = Node {
            tag: Tag::Older,
            body: Body::Timelock(144),
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn tr_nums_n_1_bare_round_trip() {
        // v0.30: tr(<NUMS>) with no script tree at n=1. NUMS is now signalled
        // via the `is_nums` flag, not a reserved sentinel `key_index`. At n=1
        // the v0.30 kiw is 0 — but `is_nums=true` suppresses the kiw write
        // entirely, so the kiw width is irrelevant here.
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        // kiw=0 at n=1 (irrelevant — is_nums=true suppresses the kiw field).
        write_node(&mut w, &n, 0).unwrap();
        // Tag::Tr (6) + is_nums (1) + has_tree (1) = 8 bits.
        assert_eq!(w.bit_len(), 8);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 0).unwrap(), n);
    }

    #[test]
    fn tr_nums_n_2_and_v_inheritance_round_trip() {
        // v0.30: tr(<NUMS>, and_v(v:pk(@0), pk(@1))) — inheritance pattern via
        // NUMS internal key, signalled by `is_nums = true`. Exercises and_v +
        // verify wrapper inside the script-path branch.
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::AndV,
                    body: Body::Children(vec![
                        Node {
                            tag: Tag::Verify,
                            body: Body::Children(vec![Node {
                                tag: Tag::PkK,
                                body: Body::KeyArg { index: 0 },
                            }]),
                        },
                        Node {
                            tag: Tag::PkK,
                            body: Body::KeyArg { index: 1 },
                        },
                    ]),
                })),
            },
        };
        let mut w = BitWriter::new();
        // v0.30 width at n=2: ⌈log₂(2)⌉ = 1.
        write_node(&mut w, &n, 1).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 1).unwrap(), n);
    }

    #[test]
    fn tr_nums_n_3_multi_a_2_of_3_round_trip() {
        // v0.30: tr(<NUMS>, multi_a(2, @0, @1, @2)) — the canonical 2-of-3
        // hardware-wallet multisig encoding (the headline use case). NUMS
        // signalled by `is_nums = true`. At n=3 the v0.30 kiw is 2.
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::MultiA,
                    body: Body::MultiKeys {
                        k: 2,
                        indices: vec![0, 1, 2],
                    },
                })),
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    #[test]
    fn tr_nums_n_4_bare_round_trip() {
        // v0.30: tr(<NUMS>) at n=4. NUMS signalled by `is_nums = true`. At
        // n=4 the v0.30 kiw is ⌈log₂(4)⌉ = 2; is_nums=true suppresses the
        // kiw field, so it's irrelevant here.
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: true,
                key_index: 0,
                tree: None,
            },
        };
        let mut w = BitWriter::new();
        // kiw=2 at n=4 (irrelevant — is_nums=true suppresses the kiw field).
        write_node(&mut w, &n, 2).unwrap();
        // Tag::Tr (6) + is_nums (1) + has_tree (1) = 8 bits.
        assert_eq!(w.bit_len(), 8);
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.19 — multi-branch tap tree wire-format round-trip. Closes audit
    /// Concern B (no codec-level tests for `Tag::TapTree` with branching
    /// existed before v0.19; multi-branch was previously walker-rejected
    /// so there was no real input that exercised this wire shape).
    /// `tr(@0, {pk(@1), pk(@2)})` with key_index_width=2.
    /// Bit-length pin (v0.30): Tag::Tr (6) + is_nums (1) + kiw (2) + has_tree (1)
    ///                 + Tag::TapTree (6) + 2×(Tag::PkK (6) + kiw (2)) = 32 bits.
    #[test]
    fn tap_tree_two_leaf_round_trip() {
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![
                        Node {
                            tag: Tag::PkK,
                            body: Body::KeyArg { index: 1 },
                        },
                        Node {
                            tag: Tag::PkK,
                            body: Body::KeyArg { index: 2 },
                        },
                    ]),
                })),
            },
        };
        let mut w = BitWriter::new();
        write_node(&mut w, &n, 2).unwrap();
        assert_eq!(w.bit_len(), 32, "2-leaf TapTree wire layout pin");
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.19 — 4-leaf nested multi-branch tap tree:
    /// `tr(@0, {{pk(@1),pk(@2)}, {pk(@3),pk(@4)}})`. Verifies recursion
    /// through `read_node`/`write_node` on nested Tag::TapTree.
    #[test]
    fn tap_tree_nested_four_leaf_round_trip() {
        let mk_branch = |a: u8, b: u8| Node {
            tag: Tag::TapTree,
            body: Body::Children(vec![
                Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: a },
                },
                Node {
                    tag: Tag::PkK,
                    body: Body::KeyArg { index: b },
                },
            ]),
        };
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![mk_branch(1, 2), mk_branch(3, 4)]),
                })),
            },
        };
        let mut w = BitWriter::new();
        // 5 distinct indices (0..=4) → v0.30 kiw = ⌈log₂(5)⌉ = 3.
        write_node(&mut w, &n, 3).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 3).unwrap(), n);
    }

    /// v0.19 — 3-leaf unbalanced: `tr(@0, {pk(@1), {pk(@2),pk(@3)}})`.
    /// Asymmetric shape — the right child is a TapTree, the left is a
    /// bare PkK leaf. Verifies the wire format doesn't require balanced
    /// trees.
    #[test]
    fn tap_tree_unbalanced_round_trip() {
        let n = Node {
            tag: Tag::Tr,
            body: Body::Tr {
                is_nums: false,
                key_index: 0,
                tree: Some(Box::new(Node {
                    tag: Tag::TapTree,
                    body: Body::Children(vec![
                        Node {
                            tag: Tag::PkK,
                            body: Body::KeyArg { index: 1 },
                        },
                        Node {
                            tag: Tag::TapTree,
                            body: Body::Children(vec![
                                Node {
                                    tag: Tag::PkK,
                                    body: Body::KeyArg { index: 2 },
                                },
                                Node {
                                    tag: Tag::PkK,
                                    body: Body::KeyArg { index: 3 },
                                },
                            ]),
                        },
                    ]),
                })),
            },
        };
        let mut w = BitWriter::new();
        // v0.30 kiw at n=3: ⌈log₂(3)⌉ = 2.
        write_node(&mut w, &n, 2).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        assert_eq!(read_node(&mut r, 2).unwrap(), n);
    }

    /// v0.19 hardening — reject deeply-nested TapTree on the decode side.
    /// Encode-side has no cap (input here is a programmatically-constructed
    /// Node tree, not from the walker), but the decode-side cap fires
    /// when the deepest left-child read attempts at depth MAX_DECODE_DEPTH.
    #[test]
    fn read_node_rejects_excessive_taptree_nesting() {
        let mut left = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        // 128 TapTree wrappers: deepest leaf ends up at depth 128 on the
        // left chain; cap fires when reading that leaf.
        for _ in 0..128 {
            left = Node {
                tag: Tag::TapTree,
                body: Body::Children(vec![
                    left,
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                ]),
            };
        }
        let mut w = BitWriter::new();
        write_node(&mut w, &left, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let err = read_node(&mut r, 0).unwrap_err();
        assert_eq!(
            err,
            Error::DecodeRecursionDepthExceeded {
                depth: 128,
                max: MAX_DECODE_DEPTH,
            }
        );
    }

    /// v0.19 hardening — cap is tag-agnostic; fires for non-taproot
    /// recursive tags (AndV chain) the same way it fires for TapTree.
    #[test]
    fn read_node_rejects_excessive_andv_chain_nesting() {
        let mut left = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        // 128 AndV wrappers on the left, with PkK leaves on the right at
        // each level. Deepest left-leaf at depth 128 triggers the cap.
        for _ in 0..128 {
            left = Node {
                tag: Tag::AndV,
                body: Body::Children(vec![
                    left,
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                ]),
            };
        }
        let mut w = BitWriter::new();
        write_node(&mut w, &left, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let err = read_node(&mut r, 0).unwrap_err();
        assert_eq!(
            err,
            Error::DecodeRecursionDepthExceeded {
                depth: 128,
                max: MAX_DECODE_DEPTH,
            }
        );
    }

    /// v0.19 hardening — depth exactly at the limit (deepest leaf at
    /// depth 127, one shy of MAX_DECODE_DEPTH) round-trips successfully.
    #[test]
    fn read_node_accepts_max_depth_minus_one() {
        let mut left = Node {
            tag: Tag::PkK,
            body: Body::KeyArg { index: 0 },
        };
        // 127 TapTree wrappers: deepest leaf at depth 127, just under cap.
        for _ in 0..127 {
            left = Node {
                tag: Tag::TapTree,
                body: Body::Children(vec![
                    left,
                    Node {
                        tag: Tag::PkK,
                        body: Body::KeyArg { index: 0 },
                    },
                ]),
            };
        }
        let mut w = BitWriter::new();
        write_node(&mut w, &left, 0).unwrap();
        let bytes = w.into_bytes();
        let mut r = BitReader::new(&bytes);
        let decoded = read_node(&mut r, 0).unwrap();
        assert_eq!(decoded, left);
    }
}
