//! String extraction, end to end: marked fields in the typedefs, through the
//! generated collectors, out as a gettext catalog.

use putki_inki::escape;

fn pot() -> String {
    let catalog = putkitest::extract_strings(&putkitest::data_dir());
    assert!(
        catalog.mismatches().is_empty(),
        "strings did not survive mangling: {:?}",
        catalog.mismatches()
    );
    catalog.to_pot("PutkiTest")
}

#[test]
fn marked_fields_are_collected() {
    let pot = pot();
    // DlgSay.Text and DlgMood.Text, from two different objects.
    assert!(pot.contains("msgctxt \"Dialogue\""), "no Dialogue context:\n{pot}");
    assert!(pot.contains("#. DlgSay.Text (dlg)"), "missing DlgSay comment:\n{pot}");
    assert!(pot.contains("#. DlgMood.Text (dlg2)"), "missing DlgMood comment:\n{pot}");
}

#[test]
fn numeric_variants_collapse_to_one_entry() {
    // "Deal 20% damage to {#Ada}" and "Deal 30% damage to {#Ada}" are authored
    // on two different objects and must share a single msgid.
    let pot = pot();
    let key = "msgid \"Deal {12}% damage to {34}\"";
    assert_eq!(pot.matches(key).count(), 1, "expected exactly one {key} in:\n{pot}");
    assert!(!pot.contains("20%"), "the number leaked into the catalog:\n{pot}");
    assert!(!pot.contains("Ada"), "the isolated span leaked into the catalog:\n{pot}");
}

#[test]
fn plural_fields_get_both_forms() {
    let pot = pot();
    assert!(pot.contains("msgid \"{12} found 3 torch\""), "no singular:\n{pot}");
    assert!(pot.contains("msgid_plural \"{12} found 3 torchs\""), "no plural:\n{pot}");
}

#[test]
fn unmarked_strings_are_not_collected() {
    // Dialog.Id and IDlgNode.Id carry no {Category}.
    let pot = pot();
    assert!(!pot.contains("DIALOG HEJ"), "unmarked string collected:\n{pot}");
    assert!(!pot.contains("Dialog.Id"), "unmarked field collected:\n{pot}");
}

#[test]
fn the_catalog_is_stable_across_runs() {
    // Object iteration is hash-ordered underneath; extraction must not be.
    assert_eq!(pot(), pot());
}

#[test]
fn quotes_and_backslashes_are_escaped() {
    // The bug this writer exists to prevent: a quote in authored text produced
    // a msgid that gettext cannot parse.
    assert_eq!(escape(r#"a "quoted" word"#), r#"a \"quoted\" word"#);
    assert_eq!(escape(r"back\slash"), r"back\\slash");
    assert_eq!(escape("line\nbreak\ttab"), "line\\nbreak\\ttab");
}
