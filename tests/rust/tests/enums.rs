//! Enum values coming out of hand-authored data.
//!
//! `From<&str>` maps anything unrecognized to the first variant, which turns a
//! typo into wrong data. Generated parsers use `parse_value` instead, which
//! keeps the "absent field" case and rejects the rest.

use gen_test_inki::inki::TestEnum;

#[test]
fn named_values_parse() {
    assert_eq!(i32::from(&TestEnum::parse_value("HEJ_KALLE_KULA", "f")), 3);
    assert_eq!(i32::from(&TestEnum::parse_value("GURKA_TRUBADUR", "f")), 1);
    assert_eq!(i32::from(&TestEnum::parse_value("TO_INDEX", "f")), 2);
}

#[test]
fn absent_field_takes_the_default() {
    // get_string yields "" when the field is missing and no default is declared.
    assert_eq!(
        i32::from(&TestEnum::parse_value("", "f")),
        i32::from(&TestEnum::default())
    );
}

#[test]
#[should_panic(expected = "unknown value 'TO_INDEXX' for enum TestEnum in field 'EnumValue'")]
fn a_typo_is_an_error() {
    TestEnum::parse_value("TO_INDEXX", "EnumValue");
}

#[test]
#[should_panic(expected = "expected one of: HEJ_KALLE_KULA, GURKA_TRUBADUR, TO_INDEX")]
fn the_error_lists_the_valid_values() {
    TestEnum::parse_value("nope", "EnumValue");
}
