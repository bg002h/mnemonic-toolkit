//! Drift-guard KAT for the canonical `descriptor_to_template` renderer.
//!
//! md-codec **cannot** dev-depend on md-cli (that would be circular:
//! md-cli → md-codec), so this freezes a hardcoded `const` snapshot of the
//! **md-cli 0.11.2** rendered output and asserts the md-codec renderer
//! reproduces it byte-for-byte. The `(md1 → expected template)` pairs were
//! captured by driving the pre-move `md` binary:
//!   `md encode '<template>' [--path <p>] --group-size 0`  →  the md1 wire,
//!   `md decode '<md1>'`                                    →  the frozen template.
//! (The optional `--path` only adds an origin so non-canonical wrappers decode;
//! it does not appear in the keyless `@N` template.)
//!
//! This freezes the wire→text mapping so the *move* of the renderer from md-cli
//! into md-codec cannot silently change a rendered template (a funds-display
//! regression). True cross-binary equality is additionally proven by the
//! toolkit end-to-end leg (spec §6).
//!
//! The corpus exercises **all ten** lifted renderer fns:
//!   descriptor_to_template, render_node, render_wrapper, render_wrapper_chain,
//!   render_tap_node, render_multi, render_key, render_hash256, render_hash160,
//!   render_binary — plus the NUMS literal, non-NUMS taproot, raw-pkh (covered
//!   by the relocated unit test in `src/render.rs`), and use-site overrides.

use md_codec::decode::decode_md1_string;
use md_codec::descriptor_to_template;

/// `(label, md1_wire_string, expected_template)` — frozen against md-cli 0.11.2.
const CORPUS: &[(&str, &str, &str)] = &[
    // render_wrapper (wpkh leaf / KeyArg), render_key
    ("wpkh", "md1yqpqqxqq8xtwhw4xwn4qh", "wpkh(@0/<0;1>/*)"),
    // render_wrapper (sh + wsh nesting), render_multi
    (
        "sh_wsh_multi",
        "md1yppqqxpsscy96gddy0v67f8tp",
        "sh(wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*)))",
    ),
    // render_wrapper (wsh), render_multi (multi)
    (
        "wsh_multi",
        "md1yppqqxppsg2vlumagltz27le",
        "wsh(multi(2,@0/<0;1>/*,@1/<0;1>/*))",
    ),
    // render_multi (sortedmulti)
    (
        "wsh_sortedmulti",
        "md1yppqqxppcg2zwgjsnaf20fmv",
        "wsh(sortedmulti(2,@0/<0;1>/*,@1/<0;1>/*))",
    ),
    // render_node Tag::Tr (NUMS literal internal key), render_tap_node,
    // render_multi (multi_a)
    (
        "tr_nums_multi_a",
        "md1yz80tgggqps8yq3psc6g0fzjxsj620",
        "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,multi_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
    ),
    // render_multi (sortedmulti_a) under NUMS taproot
    (
        "tr_nums_sortedmulti_a",
        "md1yz80tgggqps8ys3psu9rrkfee0tpv2",
        "tr(50929b74c1a04954b78b4b6035e97a5e078a5a0f28ec96d547bfee9ace803ac0,sortedmulti_a(2,@0/<0;1>/*,@1/<0;1>/*,@2/<0;1>/*))",
    ),
    // render_node Tag::Tr (non-NUMS internal key via render_key)
    (
        "tr_nonnums_keypath",
        "md1yqpqqxqsqgprhfjpjaz6d",
        "tr(@0/<0;1>/*)",
    ),
    // non-NUMS taproot with a tap-script leaf (render_tap_node leaf + render_node pk)
    (
        "tr_nonnums_leaf",
        "md1yp80tgggqpsyj5zpmragnq8pn7h",
        "tr(@0/<0;1>/*,pk(@1/<0;1>/*))",
    ),
    // pathological: render_binary (or_i), and_v, v: (verify), after, render_multi
    (
        "pathological_or_i",
        "md1yzfdsssj5qqcyefnfgdsqr6zgqvzzcrfln7t3kzht2u",
        "wsh(or_i(and_v(v:pk(@0/<0;1>/*),after(1000000)),multi(2,@1/<0;1>/*,@2/<0;1>/*)))",
    ),
    // render_key with a per-@N use-site-path override (@1 differs)
    (
        "use_site_override",
        "md1yppqqxppsg2qknq2zc2ktzhwekmddzh",
        "wsh(multi(2,@0/<0;1>/*,@1/<2;3>/*))",
    ),
    // render_hash256 (sha256)
    (
        "sha256_leaf",
        "md1yp80tgggqpsy5e54wsqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgtstwnn9zjd3s7",
        "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),sha256(0000000000000000000000000000000000000000000000000000000000000001)))",
    ),
    // render_hash160 (hash160)
    (
        "hash160_leaf",
        "md1yp80tgggqpsy5e540qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsdjl8890lrfs9z",
        "tr(@0/<0;1>/*,and_v(v:pk(@1/<0;1>/*),hash160(0000000000000000000000000000000000000001)))",
    ),
    // render_wrapper_chain (`s:`, `snj:` letter chains), older
    (
        "wrapper_chain_thresh",
        "md1yzfdsssj5qqcy6pz9qu2fey3fnf2wqqqqqjqu5afd8p60nnsc",
        "wsh(thresh(2,pk(@0/<0;1>/*),s:pk(@1/<0;1>/*),snj:and_v(v:pk(@2/<0;1>/*),older(144))))",
    ),
];

#[test]
fn renderer_matches_frozen_md_cli_0_11_2_snapshot() {
    for (label, md1, expected) in CORPUS {
        let d = decode_md1_string(md1)
            .unwrap_or_else(|e| panic!("[{label}] decode_md1_string failed: {e}"));
        let got = descriptor_to_template(&d)
            .unwrap_or_else(|e| panic!("[{label}] descriptor_to_template failed: {e}"));
        assert_eq!(
            &got, expected,
            "[{label}] renderer drifted from the frozen md-cli 0.11.2 snapshot",
        );
    }
}
