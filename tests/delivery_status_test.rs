use aspose_jmap_foss::DeliveryStatus;
use serde_json::{self, json};

#[test]
fn test_serialization_matches_wire_format() {
    let status = DeliveryStatus {
        smtp_reply: "250 OK".to_string(),
        delivered: "yes".to_string(),
        displayed: "unknown".to_string(),
    };

    // Serialize to JSON string.
    let serialized = serde_json::to_string(&status).expect("serialization should succeed");

    // Expected JSON object with exact wire property names.
    let expected = json!({
        "smtpReply": "250 OK",
        "delivered": "yes",
        "displayed": "unknown"
    });

    // Parse the serialized string back to a Value for comparison (ignores field order).
    let serialized_value: serde_json::Value =
        serde_json::from_str(&serialized).expect("serialized JSON should be valid");

    assert_eq!(serialized_value, expected);
}

#[test]
fn test_deserialization_from_wire_format() {
    let json_input = r#"
    {
        "smtpReply": "550 Mailbox unavailable",
        "delivered": "no",
        "displayed": "yes"
    }
    "#;

    let status: DeliveryStatus =
        serde_json::from_str(json_input).expect("deserialization should succeed");

    assert_eq!(status.smtp_reply, "550 Mailbox unavailable");
    assert_eq!(status.delivered, "no");
    assert_eq!(status.displayed, "yes");
}

#[test]
fn test_deserialization_missing_required_field_fails() {
    // Omit the required `smtpReply` field.
    let incomplete_json = r#"
    {
        "delivered": "queued",
        "displayed": "unknown"
    }
    "#;

    let result: Result<DeliveryStatus, _> = serde_json::from_str(incomplete_json);
    assert!(result.is_err(), "deserialization should fail when a required field is missing");
}
