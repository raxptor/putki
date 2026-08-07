//! Checks the Rust mangling against the shared vectors in
//! `tests/fixtures/mangling-vectors.txt`. The C# suite runs the same file
//! through `Putki.LocMangling`, so the two implementations cannot drift.

use putki_inki::{mangle, unmangle, Mangled, SUBST};
use std::path::PathBuf;

fn vectors() -> Vec<(String, String, String)> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../tests/fixtures/mangling-vectors.txt");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("read {}: {e}", path.display()));
    body.lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| {
            let mut f = l.split('\t');
            let (s, m, d) = (f.next(), f.next(), f.next());
            match (s, m, d) {
                (Some(s), Some(m), Some(d)) => (s.to_string(), m.to_string(), d.to_string()),
                _ => panic!("malformed vector line: {l}"),
            }
        })
        .collect()
}

#[test]
fn vectors_match_the_shared_fixture() {
    let vectors = vectors();
    assert!(!vectors.is_empty(), "no vectors loaded");
    for (source, expect_mangled, expect_display) in vectors {
        let m = mangle(&source);
        assert_eq!(m.text, expect_mangled, "mangling [{source}]");
        assert_eq!(unmangle(&m, false), expect_display, "display of [{source}]");
        assert_eq!(unmangle(&m, true), source, "round trip of [{source}]");
    }
}

#[test]
fn numeric_variants_collapse_to_one_key() {
    // The whole point: these must produce the same catalog entry.
    assert_eq!(mangle("Deal 20% damage").text, mangle("Deal 30% damage").text);
    assert_ne!(mangle("Deal 20% damage").text, mangle("Heal 20% damage").text);
}

#[test]
fn unmarked_braces_are_left_alone() {
    let m = mangle("Hi {player}, take {#this} literally");
    assert_eq!(m.text, "Hi {player}, take {12} literally");
    assert_eq!(m.replacements, vec!["#this"]);
}

#[test]
#[should_panic(expected = "more than 10 localization placeholders")]
fn overflowing_the_table_is_reported() {
    // The original threw a bare IndexOutOfRange here.
    let mut s = String::new();
    for i in 0..SUBST.len() + 1 {
        s.push_str(&format!("{{#m{i}}} "));
    }
    mangle(&s);
}

#[test]
fn a_missing_replacement_does_not_panic() {
    // Hand-built state that cannot come from mangle(); the original indexed an
    // empty list and threw.
    let m = Mangled { text: String::from("{12} and {34}"), replacements: vec![] };
    assert_eq!(unmangle(&m, false), "{12} and {34}");
}
