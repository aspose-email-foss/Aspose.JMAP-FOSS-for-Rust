use aspose_jmap_foss::{EmailAddress, Identity};
use serde_json::{json, Value};

#[test]
fn test_identity_serialization_full() {
    let identity = Identity {
        id: Some("id123".to_string()),
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
        reply_to: Some(vec![EmailAddress {
            name: Some("Reply".to_string()),
            email: "reply@example.com".to_string(),
        }]),
        bcc: Some(vec![EmailAddress {
            name: None,
            email: "bcc@example.com".to_string(),
        }]),
        text_signature: "Best".to_string(),
        html_signature: "<b>Best</b>".to_string(),
        may_delete: Some(true),
    };

    let serialized = serde_json::to_string(&identity).expect("serialization failed");
    let value: Value = serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    let expected = json!({
        "id": "id123",
        "name": "John Doe",
        "email": "john@example.com",
        "replyTo": [
            {
                "name": "Reply",
                "email": "reply@example.com"
            }
        ],
        "bcc": [
            {
                "email": "bcc@example.com"
            }
        ],
        "textSignature": "Best",
        "htmlSignature": "<b>Best</b>",
        "mayDelete": true
    });

    assert_eq!(value, expected);
}

#[test]
fn test_identity_deserialization_missing_optional_and_defaults() {
    let json_input = r#"
    {
        "email": "jane@example.com",
        "replyTo": null
    }
    "#;

    let identity: Identity =
        serde_json::from_str(json_input).expect("deserialization should succeed");

    // Required field
    assert_eq!(identity.email, "jane@example.com");

    // Optional fields omitted or null become None
    assert!(identity.id.is_none());
    assert!(identity.reply_to.is_none());
    assert!(identity.bcc.is_none());
    assert!(identity.may_delete.is_none());

    // Fields with defaults should be empty strings
    assert_eq!(identity.name, "");
    assert_eq!(identity.text_signature, "");
    assert_eq!(identity.html_signature, "");

    // Serializing back should omit all None/empty-default fields
    let serialized = serde_json::to_string(&identity).expect("serialization failed");
    let value: Value = serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    let expected = json!({
        "email": "jane@example.com"
    });

    assert_eq!(value, expected);
}
