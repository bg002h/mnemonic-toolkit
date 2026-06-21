//! `descriptor-builder` engine (SPEC `design/SPEC_descriptor_builder_engine.md`,
//! Release A / v0.50.0) — a deterministic renderer + validator that turns a
//! versioned JSON `PolicyNode` fragment tree into a validated `wsh(M)`
//! descriptor + BIP-388 wallet-policy + cost preview + node-addressed
//! diagnostics. NOT a compiler (`miniscript/compiler` stays OFF).
//!
//! Release A shipped the IR ([`ir`]), the versioned `--spec-schema` grammar
//! ([`schema`]), the validation gate ([`gate`]), and the `build-descriptor`
//! clap surface + emit (v0.50.0). Release B adds the archetype preset
//! producers ([`archetype`]) over the frozen IR (presets SPEC
//! `design/SPEC_descriptor_builder_presets.md`).

pub mod archetype;
pub mod gate;
pub mod ir;
pub mod schema;

#[cfg(test)]
mod fixtures_test {
    //! Phase-1 archetype fixtures (`tests/fixtures/descriptor_builder/*.json`):
    //! parse + render-skeleton. Proves the v1 fragment set is expressive enough
    //! for all 5 archetypes and freezes the schema against reality. The
    //! descriptor + bip388 GOLDENS are pinned in Phase 3 (need emit); here we
    //! pin only the pre-canonicalization render skeleton (keys masked).
    use super::ir::SpecDoc;

    const KEY_A: &str = "[11111111/48h/0h/0h/2h]xpub661MyMwAqRbcEZVB4dScxMAdx6d4nFc9nvyvH3v4gJL378CSRZiYmhRoP7mBy6gSPSCYk6SzXPTf3ND1cZAceL7SfJ1Z3GC8vBgp2epUt13";
    const KEY_B: &str = "[22222222/48h/0h/0h/2h]xpub661MyMwAqRbcFtXgS5sYJABqqG9YLmC4Q1Rdap9gSE8NqtwybGhePY2gZ29ESFjqJoCu1Rupje8YtGqsefD265TMg7usUDFdp6W1EGMcet8";
    const KEY_C: &str = "[33333333/48h/0h/0h/2h]xpub661MyMwAqRbcFW31YEwpkMuc5THy2PSt5bDMsktWQcFF8syAmRUapSCGu8ED9W6oDMSgv6Zz8idoc4a6mr8BDzTJY47LJhkJ8UB7WEGuduB";
    const KEY_D: &str = "[44444444/48h/0h/0h/2h]xpub661MyMwAqRbcGczjuMoRm6dXaLDEhW1u34gKenbeYqAix21mdUKJyuyu5F1rzYGVxyL6tmgBUAEPrEz92mBXjByMRiJdba9wpnN37RLLAXa";
    const KEY_E: &str = "[55555555/48h/0h/0h/2h]xpub68Gmy5EdvgibQVfPdqkBBCHxA5htiqg55crXYuXoQRKfDBFA1WEjWgP6LHhwBZeNK1VTsfTFUHCdrfp1bgwQ9xv5ski8PX9rL2dZXvgGDnw";

    /// Mask the (long) key expressions + multipath suffix so the render
    /// skeleton is hand-verifiable. NOTE: the `KEY_A..E` constants must be
    /// mutually non-substring (they are — distinct full xpubs) or this masking
    /// mis-substitutes; a new fixture key must preserve that property.
    fn skeleton(rendered: &str) -> String {
        rendered
            .replace(KEY_A, "A")
            .replace(KEY_B, "B")
            .replace(KEY_C, "C")
            .replace(KEY_D, "D")
            .replace(KEY_E, "E")
            .replace("/<0;1>/*", "")
    }

    fn check(json: &str, expected_skeleton: &str) {
        let doc = SpecDoc::parse(json).expect("fixture parses");
        assert_eq!(skeleton(&doc.render_descriptor()), expected_skeleton);
    }

    #[test]
    fn simple_timelocked_inheritance() {
        check(
            include_str!(
                "../../tests/fixtures/descriptor_builder/simple-timelocked-inheritance.json"
            ),
            "wsh(or_d(pk(A),and_v(v:pkh(B),older(65535))))",
        );
    }

    #[test]
    fn decaying_multisig() {
        check(
            include_str!("../../tests/fixtures/descriptor_builder/decaying-multisig.json"),
            "wsh(andor(multi(2,A,B),older(1000),andor(multi(2,C,D),older(2000),and_v(v:pk(E),after(4000000)))))",
        );
    }

    #[test]
    fn kofn_recovery() {
        check(
            include_str!("../../tests/fixtures/descriptor_builder/kofn-recovery.json"),
            "wsh(or_d(multi(2,A,B,C),and_v(v:pk(D),older(52560))))",
        );
    }

    #[test]
    fn tiered_recovery() {
        check(
            include_str!("../../tests/fixtures/descriptor_builder/tiered-recovery.json"),
            "wsh(or_i(sortedmulti(2,A,B),and_v(v:older(4032),thresh(2,pk(C),s:pk(D),s:pk(E)))))",
        );
    }

    #[test]
    fn hashlock_gated() {
        check(
            include_str!("../../tests/fixtures/descriptor_builder/hashlock-gated.json"),
            "wsh(andor(pk(A),sha256(926a54995ca48600920a19bf7bc502ca5f2f7d07e6f804c4f00ebf0325084dbc),and_v(v:pk(B),older(144))))",
        );
    }
}
