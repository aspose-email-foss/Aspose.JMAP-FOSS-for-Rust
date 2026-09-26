use aspose_jmap_foss::EmailHeader;
use serde_json;

/// Test that a fully populated `EmailHeader` serializes to the expected JSON string.
#[test]
fn serialize_email_header() {
    let header = EmailHeader {
        name: "Subject".to_string(),
        value: "Hello, world!".to_string(),
    };
    let json = serde_json::to_string(&header).expect("serialization should succeed");
    assert_eq!(json, r#"{"name":"Subject","value":"Hello, world!"}"#);
}

/// Test that a valid JSON representation deserializes into an `EmailHeader` correctly.
#[test]
fn deserialize_email_header() {
    let json = r#"{"name":"From","value":"alice@example.com"}"#;
    let header: EmailHeader =
        serde_json::from_str(json).expect("deserialization should succeed");
    assert_eq!(header.name, "From");
    assert_eq!(header.value, "alice@example.com");
}

/// Edge case: deserialization should fail when a required field is missing.
#[test]
fn deserialize_missing_field_error() {
    // The `value` field is omitted, which is required.
    let json = r#"{"name":"To"}"#;
    let result: Result<EmailHeader, _> = serde_json::from_str(json);
    assert!(result.is_err(), "deserialization should error on missing required fields");
}
