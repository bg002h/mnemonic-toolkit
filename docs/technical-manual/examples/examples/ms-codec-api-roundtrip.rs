//! ms-codec v0.1.1 encoder/decoder round-trip: 12-word BIP-39 entropy (Entr).
use ms_codec::{Payload, PayloadKind, Tag, decode, encode};
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // Canonical BIP-39 12-word test entropy: "abandon abandon ... about"
    // = 16 zero bytes. Deterministic; no CSPRNG.
    let payload = Payload::Entr(vec![0u8; 16]);
    let s = encode(Tag::ENTR, &payload)?;
    println!("encoded: {}", s);
    let (tag, recovered) = decode(&s)?;
    let kind = match recovered.kind() {
        PayloadKind::Entr => "Entr",
        _ => "<other>",
    };
    println!(
        "decode ok: tag={} kind={} bytes={}",
        tag.as_str(),
        kind,
        recovered.as_bytes().len()
    );
    Ok(())
}
