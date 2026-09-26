use aspose_jmap_foss::Comparator;
use serde_json::{json, Value};

#[test]
fn test_serialize_full() {
    let comp = Comparator {
        property: "subject".to_string(),
        is_ascending: false,
        collation: Some("en".to_string()),
    };

    let serialized = serde_json::to_string(&comp).expect("serialization failed");
    let parsed: Value = serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    let expected = json!({
        "property": "subject",
        "isAscending": false,
        "collation": "en"
    });

    assert_eq!(parsed, expected);
}

#[test]
fn test_deserialize_full() {
    let data = r#"
    {
        "property": "from",
        "isAscending": true,
        "collation": "fr"
    }
    "#;

    let comp: Comparator = serde_json::from_str(data).expect("deserialization failed");
    assert_eq!(comp.property, "from");
    assert_eq!(comp.is_ascending, true);
    assert_eq!(comp.collation, Some("fr".to_string()));
}

#[test]
fn test_deserialize_missing_optional_and_default() {
    // Only the required field is present; isAscending should default to true,
    // collation should be None.
    let data = r#"{ "property": "receivedAt" }"#;

    let comp: Comparator = serde_json::from_str(data).expect("deserialization failed");
    assert_eq!(comp.property, "receivedAt");
    assert_eq!(comp.is_ascending, true); // default
    assert!(comp.collation.is_none());
}

#[test]
fn test_deserialize_null_collation() {
    // collation explicitly set to null should become None.
    let data = r#"{ "property": "size", "collation": null }"#;

    let comp: Comparator = serde_json::from_str(data).expect("deserialization failed");
    assert_eq!(comp.property, "size");
    assert_eq!(comp.is_ascending, true); // default
    assert!(comp.collation.is_none());
}

#[test]
fn test_serialize_omit_defaults() {
    // isAscending is true (default) and collation is None, both should be omitted.
    let comp = Comparator {
        property: "size".to_string(),
        is_ascending: true,
        collation: None,
    };

    let serialized = serde_json::to_string(&comp).expect("serialization failed");
    let parsed: Value = serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    // The resulting object should contain only the "property" field.
    let expected = json!({ "property": "size" });
    assert_eq!(parsed, expected);
}
