//! mk-codec v0.2.2 encoder/decoder round-trip: BIP-84 mainnet, 1-stub w/ fp.
use bitcoin::bip32::{DerivationPath, Fingerprint, Xpub};
use mk_codec::{KeyCard, decode, encode_with_chunk_set_id};
use std::error::Error;
use std::str::FromStr;

fn main() -> Result<(), Box<dyn Error>> {
    let path = DerivationPath::from_str("m/84'/0'/0'")?;
    let xpub = Xpub::from_str(
        "xpub6BmeGmRo4LosAcU21HDaGcvtaQ7GrqQcY48nBkE22qM6KVwQUjRJ1BGzk84SFVHgLcd61Vcnhr8\
         petHexjjn5WbQ9PriVrRhphw4oCp2z6a",
    )?;
    let card = KeyCard::new(
        vec![[0xC0, 0xFF, 0xEE, 0x00]],
        Some(Fingerprint::from([0xDE, 0xAD, 0xBE, 0xEF])),
        path,
        xpub,
    );
    let strings = encode_with_chunk_set_id(&card, 144470)?;
    println!("encoded: {} chunk(s)", strings.len());
    for s in &strings {
        println!("  {}", s);
    }
    let refs: Vec<&str> = strings.iter().map(String::as_str).collect();
    let back = decode(&refs)?;
    println!(
        "decode ok: stubs={} path={}",
        back.policy_id_stubs.len(),
        back.origin_path
    );
    Ok(())
}
