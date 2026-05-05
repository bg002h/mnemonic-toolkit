#![allow(dead_code)]

mod derive;
mod error;
mod format;
mod language;
mod network;
mod parse;
mod synthesize;
mod template;

mod friendly {
    /* stub for Phase 1 — real impl in Phase 3 */
    pub fn friendly_bip39(_: &bip39::Error) -> String {
        unimplemented!("Phase 3")
    }
    pub fn friendly_bitcoin(_: &crate::error::BitcoinErrorKind) -> String {
        unimplemented!("Phase 3")
    }
    pub fn friendly_ms_codec(_: &ms_codec::Error) -> String {
        unimplemented!("Phase 3")
    }
    pub fn friendly_mk_codec(_: &mk_codec::Error) -> String {
        unimplemented!("Phase 3")
    }
    pub fn friendly_md_codec(_: &md_codec::Error) -> String {
        unimplemented!("Phase 3")
    }
}

fn main() {}
