//! md-codec v0.32.0 encoder/decoder round-trip: BIP-84 single-key template.
use md_codec::origin_path::{OriginPath, PathComponent, PathDecl, PathDeclPaths};
use md_codec::tlv::TlvSection;
use md_codec::tree::{Body, Node};
use md_codec::use_site_path::UseSitePath;
use md_codec::{Descriptor, Tag, decode_md1_string, encode_md1_string};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let d = Descriptor {
        n: 1,
        path_decl: PathDecl {
            n: 1,
            paths: PathDeclPaths::Shared(OriginPath {
                components: vec![
                    PathComponent { hardened: true, value: 84 },
                    PathComponent { hardened: true, value: 0 },
                    PathComponent { hardened: true, value: 0 },
                ],
            }),
        },
        use_site_path: UseSitePath::standard_multipath(),
        tree: Node { tag: Tag::Wpkh, body: Body::KeyArg { index: 0 } },
        tlv: TlvSection::new_empty(),
    };
    let card = encode_md1_string(&d)?;
    println!("encoded: {}", card);
    let back = decode_md1_string(&card)?;
    println!("decode ok: n={} tag={:?}", back.n, back.tree.tag);
    Ok(())
}
