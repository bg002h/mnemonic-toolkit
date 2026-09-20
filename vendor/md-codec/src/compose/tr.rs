//! Taproot lowering (spec §5, tr rows; C17, C18, M1): internal-key extraction,
//! the right spine in listed order, NUMS when no key is extracted.

use super::lowering::{Numbered, experimental, finish, number, path_body};
use super::{ComposeError, Composed, PathList, SlotOrigin, SpendPath};
use crate::tag::Tag;
use crate::tree::{Body, Node};

/// The first-listed unlocked, unhashed one-key path, if any (spec §5, M1).
fn internal_key_path(list: &PathList) -> Option<usize> {
    list.paths.iter().position(SpendPath::is_bare_single)
}

/// Right spine in listed order: `{P1,{P2,{P3,P4}}}`; one leaf is bare; no leaf
/// is no tree.
fn spine(mut leaves: Vec<Node>) -> Option<Box<Node>> {
    let mut acc = leaves.pop()?;
    for leaf in leaves.into_iter().rev() {
        acc = Node {
            tag: Tag::TapTree,
            body: Body::Children(vec![leaf, acc]),
        };
    }
    Some(Box::new(acc))
}

pub(super) fn lower_tr(
    list: &PathList,
    declared: &[Option<SlotOrigin>],
) -> Result<Composed, ComposeError> {
    let ik = internal_key_path(list);
    let (numbered, slots) = number(list, ik);
    let leaf_paths: Vec<&Numbered<'_>> = numbered
        .iter()
        .filter(|n| Some(n.path_index) != ik)
        .collect();
    let m = leaf_paths.len();
    let leaves: Vec<Node> = leaf_paths
        .iter()
        .map(|n| path_body(n, true, m == 1 && n.path.is_bare_multi()))
        .collect();
    let tree = Node {
        tag: Tag::Tr,
        body: Body::Tr {
            is_nums: ik.is_none(),
            key_index: 0,
            tree: spine(leaves),
        },
    };
    let exp = experimental(list, |i| {
        m == 1 && Some(i) != ik && list.paths[i].is_bare_multi()
    });
    finish(list, declared, tree, slots, ik, exp)
}
