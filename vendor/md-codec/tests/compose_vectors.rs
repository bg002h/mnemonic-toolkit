//! Spec §12 item 1: TAGGED coverage of the lowering. Every compose vector in
//! `MANIFEST` is listed in `support::family()` with the spec rows it exercises;
//! every non-singular tag must appear in at least two vectors; every listed
//! vector's stored template must be exactly what `compose` renders (with
//! origins inlined) for its path list.

use std::collections::BTreeMap;

#[path = "compose_support.rs"]
mod support;
use support::*;

use md_codec::compose::{compose, template_with_origins};
use md_codec::render::descriptor_to_template;
use md_codec::test_vectors::MANIFEST;

#[test]
fn every_family_entry_renders_as_listed() {
    // Gate-runnable without the MANIFEST: the lowering against the listed text.
    for (name, list, expected, _) in &family() {
        let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(
            &descriptor_to_template(&c.descriptor).unwrap(),
            expected,
            "{name}"
        );
    }
}

#[test]
fn every_compose_vector_in_the_manifest_is_exactly_what_compose_renders() {
    // MANIFEST templates carry inline origins (the parse-input form), so the
    // comparison is against `template_with_origins`.
    for (name, list, _, tags) in &family() {
        if tags.contains(&"no-corpus") {
            assert!(
                MANIFEST.iter().all(|v| v.name != *name),
                "{name}: a no-corpus vector must not be in MANIFEST (the exporter would refuse it)"
            );
            continue;
        }
        let v = MANIFEST
            .iter()
            .find(|v| v.name == *name)
            .unwrap_or_else(|| panic!("MANIFEST lacks {name}"));
        let c = compose(list).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(template_with_origins(&c).unwrap(), v.template, "{name}");
        assert_eq!(
            v.path, None,
            "{name}: compose vectors carry inline origins, never a --path override"
        );
    }
}

#[test]
fn every_compose_manifest_entry_is_in_the_family() {
    let fam = family();
    for v in MANIFEST.iter().filter(|v| v.name.contains("compose_")) {
        assert!(
            fam.iter().any(|(n, _, _, _)| *n == v.name),
            "untagged compose vector {}",
            v.name
        );
    }
}

#[test]
fn every_tag_appears_in_at_least_two_vectors() {
    let mut count: BTreeMap<&str, usize> = BTreeMap::new();
    for (_, _, _, tags) in family() {
        for t in tags {
            *count.entry(t).or_default() += 1;
        }
    }
    let thin: Vec<_> = count
        .iter()
        .filter(|(t, c)| **c < 2 && !SINGULAR_TAGS.contains(t))
        .collect();
    assert!(
        thin.is_empty(),
        "tags with fewer than two vectors: {thin:?}"
    );
    for t in SINGULAR_TAGS {
        assert_eq!(
            count.get(t),
            Some(&1),
            "a singular tag has exactly one vector: {t}"
        );
    }
    // The spec's required tag list, every member present (spec §12 item 1).
    for required in [
        "w:tr",
        "w:wsh",
        "w:sh-wsh",
        "w:sh",
        "paths:1",
        "paths:2",
        "paths:3",
        "paths:4",
        "slots:32",
        "spine:0",
        "spine:1",
        "spine:2",
        "spine:3",
        "spine:7",
        "ik:extracted-first",
        "ik:extracted-later",
        "ik:nums",
        "lock:none",
        "lock:blocks",
        "lock:units",
        "lock:height",
        "lock:time",
        "hash",
        "sorted",
        "unsorted",
        "keyless-wsh",
        "fp:distinct",
        "fp:one-seed-one-path",
        "fp:one-seed-two-paths",
        "origins:default-tr",
        "origins:default-wsh",
        "origins:default-sh-wsh",
        "origins:default-sh",
    ] {
        assert!(
            count.contains_key(required),
            "required tag missing from the family: {required}"
        );
    }
}

#[test]
fn keyed_compose_vectors_bind_at_most_the_four_journey_keys() {
    for v in MANIFEST
        .iter()
        .filter(|v| v.name.starts_with("keyed_compose_"))
    {
        assert!(
            !v.keys.is_empty(),
            "{}: a keyed_ vector must bind keys so md vectors emits .conformance.json",
            v.name
        );
        assert!(
            v.keys.len() <= XPUB.len(),
            "{}: the journey fixture has four keys",
            v.name
        );
        assert_eq!(v.keys.len(), v.fingerprints.len(), "{}", v.name);
    }
}

#[test]
fn print_family_templates_for_the_manifest() {
    for (name, list, _, tags) in family() {
        if tags.contains(&"no-corpus") {
            continue;
        }
        let c = compose(&list).unwrap();
        println!("{name}\t{}", template_with_origins(&c).unwrap());
    }
}
