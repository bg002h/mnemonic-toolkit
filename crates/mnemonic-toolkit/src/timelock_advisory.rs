//! Non-blocking consensus-masked `older()` advisory (SPEC_older_timelock_advisory).
//!
//! Intake/round-trip surfaces accept BIP-68 consensus-masked relative timelocks
//! that `build-descriptor`'s authoring gate refuses. This module is THE single
//! source of the bit-math (`older_consensus_masked`, shared with the gate) plus
//! two walk adapters that collect non-blocking advisories.

use std::collections::BTreeSet;
use std::io::Write;

use md_codec::tag::Tag;
use md_codec::tree::{Body, Node};
use miniscript::descriptor::ShInner;
use miniscript::miniscript::decode::Terminal;
use miniscript::{Descriptor, Miniscript, MiniscriptKey, ScriptContext};

/// Unit of a BIP-68 relative timelock value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelockUnit {
    Blocks,
    Seconds512,
}

/// What consensus does to a footgun `older()` operand.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimelockMaskConsequence {
    /// Bit-31 disable flag set ⇒ CSV is a no-op (no timelock at all). Reachable
    /// from the gate's IR path and the A-raw-card path; never post-`from_str`.
    Bit31Disabled,
    /// Consensus masks the operand to `effective` in `unit`.
    Masked { effective: u16, unit: TimelockUnit },
}

/// A collected advisory: the literal operand + its consensus consequence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimelockAdvisory {
    pub operand: u32,
    pub consequence: TimelockMaskConsequence,
}

/// THE shared predicate (SPEC §3.1). `Some` iff `n` is a BIP-68 footgun: a bit
/// outside {low-16 value, bit-22 type-flag} is set, OR the 16-bit value is zero.
/// `None` for clean operands (1..=65535 blocks, or 0x400001..=0x40FFFF 512-second
/// units). Note `0` (zero 16-bit value, no stray bits) is itself a footgun → a
/// no-op lock that yields `Masked{effective:0}`, NOT clean (`None`). Mirrors
/// `descriptor_builder::gate`'s former inline logic verbatim.
pub fn older_consensus_masked(n: u32) -> Option<TimelockMaskConsequence> {
    if (n & !0x0040_FFFFu32) != 0 || (n & 0x0000_FFFFu32) == 0 {
        if n & 0x8000_0000 != 0 {
            Some(TimelockMaskConsequence::Bit31Disabled)
        } else {
            let unit = if n & 0x0040_0000 != 0 {
                TimelockUnit::Seconds512
            } else {
                TimelockUnit::Blocks
            };
            Some(TimelockMaskConsequence::Masked {
                effective: (n & 0x0000_FFFF) as u16,
                unit,
            })
        }
    } else {
        None
    }
}

impl TimelockAdvisory {
    /// The stderr advisory line (SPEC §5). Two forms by consequence.
    pub fn message(&self) -> String {
        match self.consequence {
            TimelockMaskConsequence::Bit31Disabled => format!(
                "advisory: older({}) has the BIP-68 bit-31 disable flag set — consensus treats this \
                 CHECKSEQUENCEVERIFY as a no-op, so there is no relative timelock at all.",
                self.operand
            ),
            TimelockMaskConsequence::Masked { effective, unit } => {
                let unit_str = match unit {
                    TimelockUnit::Blocks => "blocks",
                    TimelockUnit::Seconds512 => "512-second units",
                };
                if effective == 0 {
                    format!(
                        "advisory: older({}) is consensus-masked — BIP-68 uses only the low 16 bits, \
                         so this relative timelock has NO effective value (0 {unit_str}); the literal \
                         overstates the lock.",
                        self.operand
                    )
                } else {
                    format!(
                        "advisory: older({}) is consensus-masked — BIP-68 uses only the low 16 bits, \
                         so this relative timelock has an effective value of {effective} {unit_str}; \
                         the literal overstates the lock.",
                        self.operand
                    )
                }
            }
        }
    }
}

/// Write each advisory's message to `stderr` (best-effort; mirrors `secret_advisory`).
pub fn emit_advisories<E: Write>(advisories: &[TimelockAdvisory], stderr: &mut E) {
    for a in advisories {
        let _ = writeln!(stderr, "{}", a.message());
    }
}

fn merge_deduped(
    advs: Vec<TimelockAdvisory>,
    seen: &mut BTreeSet<u32>,
    out: &mut Vec<TimelockAdvisory>,
) {
    for a in advs {
        if seen.insert(a.operand) {
            out.push(a);
        }
    }
}

// ---- Adapter B: miniscript AST (post-`from_str`; bit-31 unreachable) ---------

/// Generic core. Walks every sub-node of `ms` for `Terminal::Older` and collects
/// deduped advisories. Adapter B is ALWAYS post-`from_str`, so a bit-31 operand
/// cannot occur (the `debug_assert` documents the invariant; see SPEC §3.3).
pub fn older_advisories_ms<Pk: MiniscriptKey, Ctx: ScriptContext>(
    ms: &Miniscript<Pk, Ctx>,
) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    for node in ms.iter() {
        if let Terminal::Older(lt) = &node.node {
            let n = lt.to_consensus_u32();
            if let Some(consequence) = older_consensus_masked(n) {
                debug_assert!(
                    consequence != TimelockMaskConsequence::Bit31Disabled,
                    "Adapter B is post-from_str; a bit-31 older() cannot parse"
                );
                if seen.insert(n) {
                    out.push(TimelockAdvisory {
                        operand: n,
                        consequence,
                    });
                }
            }
        }
    }
    out
}

/// Unwrap a parsed `Descriptor` to its inner miniscript(s) and collect advisories
/// (deduped across all leaves). Adapter B.
pub fn older_advisories_descriptor<Pk: MiniscriptKey>(d: &Descriptor<Pk>) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    match d {
        Descriptor::Wsh(wsh) => {
            merge_deduped(older_advisories_ms(wsh.as_inner()), &mut seen, &mut out)
        }
        Descriptor::Sh(sh) => match sh.as_inner() {
            ShInner::Wsh(wsh) => {
                merge_deduped(older_advisories_ms(wsh.as_inner()), &mut seen, &mut out)
            }
            // Legacy P2SH miniscript sh(<ms>) — the toolkit supports it (parse_descriptor.rs:451)
            // and it can carry older(); reachable on Adapter-B surfaces. (older_advisories_ms is
            // generic over Ctx, so the Legacy-context inner miniscript works.)
            ShInner::Ms(ms) => merge_deduped(older_advisories_ms(ms), &mut seen, &mut out),
            // ShInner::Wpkh carries no older().
            ShInner::Wpkh(_) => {}
        },
        Descriptor::Tr(tr) => {
            for leaf in tr.leaves() {
                // leaf.miniscript() = &Arc<Miniscript<_,Tap>>; &Arc<T> coerces to &T at the call.
                merge_deduped(older_advisories_ms(leaf.miniscript()), &mut seen, &mut out);
            }
        }
        // Bare / Pkh / Wpkh carry no miniscript with older().
        _ => {}
    }
    out
}

// ---- Adapter A: md_codec Node tree (md1-card decode; bit-31 REACHABLE) --------

/// Walk an `md_codec` descriptor's node tree for `Tag::Older` + `Body::Timelock`.
/// Used by md1-card decode paths (A-raw-card: bit-31 REACHABLE → NO debug_assert)
/// AND by `parse_descriptor`-sourced MdDescriptors (A-post-from_str, bit-31-free).
pub fn older_advisories_tree(desc: &md_codec::Descriptor) -> Vec<TimelockAdvisory> {
    older_advisories_node(&desc.tree)
}

/// Walk a `Node` tree directly — unit-testable without constructing a full
/// `md_codec::Descriptor` (avoids `PathDecl`/`UseSitePath`/`TlvSection` field fragility).
pub(crate) fn older_advisories_node(root: &Node) -> Vec<TimelockAdvisory> {
    let mut seen = BTreeSet::new();
    let mut out = Vec::new();
    walk_node(root, &mut seen, &mut out);
    out
}

fn walk_node(node: &Node, seen: &mut BTreeSet<u32>, out: &mut Vec<TimelockAdvisory>) {
    if node.tag == Tag::Older {
        if let Body::Timelock(n) = node.body {
            if let Some(consequence) = older_consensus_masked(n) {
                if seen.insert(n) {
                    out.push(TimelockAdvisory {
                        operand: n,
                        consequence,
                    });
                }
            }
        }
        return; // Older is a leaf
    }
    match &node.body {
        Body::Children(children) => children.iter().for_each(|c| walk_node(c, seen, out)),
        Body::Variable { children, .. } => children.iter().for_each(|c| walk_node(c, seen, out)),
        Body::Tr { tree: Some(t), .. } => walk_node(t, seen, out),
        _ => {} // KeyArg / MultiKeys / Hash* / Timelock(After) / Empty / Tr{tree:None} — no Older children
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn predicate_classifies_operands() {
        use TimelockMaskConsequence::*;
        use TimelockUnit::*;
        // masked-to-zero (stray high bit, low-16 == 0)
        assert_eq!(
            older_consensus_masked(65536),
            Some(Masked {
                effective: 0,
                unit: Blocks
            })
        );
        // stray bit 23 → consensus masks to low-16 value 100 (blocks)
        assert_eq!(
            older_consensus_masked(0x0080_0064),
            Some(Masked {
                effective: 100,
                unit: Blocks
            })
        );
        // bit-31 disable flag set
        assert_eq!(older_consensus_masked(0x8000_0001), Some(Bit31Disabled));
        // clean block values → None
        for n in [1u32, 2016, 52560, 65535] {
            assert_eq!(older_consensus_masked(n), None, "clean blocks {n}");
        }
        // clean 512-second-unit values (bit 22 set, nonzero low-16) → None
        for n in [0x0040_0001u32, 0x0040_FFFF] {
            assert_eq!(older_consensus_masked(n), None, "clean 512s {n:#x}");
        }
        // older(0): no stray bits but zero 16-bit value → Masked{effective:0, Blocks} (a no-op lock).
        assert_eq!(
            older_consensus_masked(0),
            Some(Masked {
                effective: 0,
                unit: Blocks
            })
        );
        // 0x0040_0000: only the bit-22 type-flag set, zero value → Masked{effective:0, Seconds512}.
        assert_eq!(
            older_consensus_masked(0x0040_0000),
            Some(Masked {
                effective: 0,
                unit: Seconds512
            })
        );
    }

    #[test]
    fn message_forms() {
        let masked = TimelockAdvisory {
            operand: 65536,
            consequence: TimelockMaskConsequence::Masked {
                effective: 0,
                unit: TimelockUnit::Blocks,
            },
        };
        assert!(masked
            .message()
            .contains("advisory: older(65536) is consensus-masked"));
        assert!(masked.message().contains("NO effective value"));
        // 0x0080_0064: bit-23 stray (NOT bit-22), so unit is Blocks, value 100.
        let stray_blocks = TimelockAdvisory {
            operand: 0x0080_0064,
            consequence: TimelockMaskConsequence::Masked {
                effective: 100,
                unit: TimelockUnit::Blocks,
            },
        };
        assert!(stray_blocks
            .message()
            .contains("effective value of 100 blocks"));
        let b31 = TimelockAdvisory {
            operand: 0x8000_0001,
            consequence: TimelockMaskConsequence::Bit31Disabled,
        };
        assert!(b31.message().contains("bit-31 disable flag set"));
        assert!(b31.message().contains("no relative timelock at all"));
    }

    #[test]
    fn adapter_a_tree_reaches_bit31_and_dedups() {
        use md_codec::tag::Tag;
        use md_codec::tree::{Body, Node};
        // Hand-built tree: andor(<older(0x80000001)>, <older(65536)>, <older(65536)>)
        // mirrors a crafted md1 card (md_codec decode does NO operand validation).
        let older = |v: u32| Node {
            tag: Tag::Older,
            body: Body::Timelock(v),
        };
        let root = Node {
            tag: Tag::AndOr,
            body: Body::Children(vec![older(0x8000_0001), older(65536), older(65536)]),
        };
        // Walk the Node directly — no full md_codec::Descriptor literal needed (R0 advisor #2).
        let advs = older_advisories_node(&root);
        // bit-31 IS reachable on the A-raw-card path; 65536 deduped to one entry.
        assert_eq!(advs.len(), 2);
        assert!(advs
            .iter()
            .any(|a| a.consequence == TimelockMaskConsequence::Bit31Disabled));
        assert!(advs.iter().any(|a| a.operand == 65536));
    }

    #[test]
    fn adapter_b_descriptor_wsh_collects_masked_only() {
        use miniscript::Descriptor;
        use std::str::FromStr;
        // wsh(andor(pk(K0),older(65536),and_v(v:pk(K1),older(2016)))) — 65536 masked, 2016 clean.
        let k0 = "0279be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798";
        let k1 = "03f9308a019258c31049344f85f89d5229b531c845836f99b08601f113bce036f9";
        let d = Descriptor::<miniscript::bitcoin::PublicKey>::from_str(&format!(
            "wsh(andor(pk({k0}),older(65536),and_v(v:pk({k1}),older(2016))))"
        ))
        .expect("masked policy parses");
        let advs = older_advisories_descriptor(&d);
        assert_eq!(
            advs.len(),
            1,
            "only older(65536) is masked; older(2016) is clean"
        );
        assert_eq!(advs[0].operand, 65536);
    }
}
