use aspose_jmap_foss::{Address, Envelope};
use serde_json::{json, Value};

#[test]
fn test_envelope_serialization_deserialization() {
    // Build a full Envelope instance with all required fields and optional parameters.
    let envelope = Envelope {
        mail_from: Address {
            email: "sender@example.com".to_string(),
            parameters: Some(
                [("RET".to_string(), Some("HDRS".to_string()))]
                    .iter()
                    .cloned()
                    .collect(),
            ),
        },
        rcpt_to: vec![
            Address {
                email: "rcpt1@example.com".to_string(),
                parameters: Some(
                    [("SIZE".to_string(), Some("1234".to_string()))]
                        .iter()
                        .cloned()
                        .collect(),
                ),
            },
            Address {
                email: "rcpt2@example.com".to_string(),
                parameters: None,
            },
        ],
    };

    // Serialize to JSON.
    let serialized = serde_json::to_string(&envelope).expect("serialization failed");
    let serialized_value: Value =
        serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    // Expected JSON structure.
    let expected = json!({
        "mailFrom": {
            "email": "sender@example.com",
            "parameters": {
                "RET": "HDRS"
            }
        },
        "rcptTo": [
            {
                "email": "rcpt1@example.com",
                "parameters": {
                    "SIZE": "1234"
                }
            },
            {
                "email": "rcpt2@example.com"
            }
        ]
    });

    assert_eq!(serialized_value, expected);

    // Deserialize back to a struct and compare with the original.
    let deserialized: Envelope =
        serde_json::from_str(&serialized).expect("deserialization failed");
    assert_eq!(deserialized, envelope);
}

#[test]
fn test_address_deserialize_with_null_parameters() {
    // JSON where the optional `parameters` field is explicitly null.
    let json_data = r#"
    {
        "email": "nullparams@example.com",
        "parameters": null
    }
    "#;

    let address: Address =
        serde_json::from_str(json_data).expect("deserialization of address failed");

    // The `parameters` field should be None when the JSON value is null.
    assert_eq!(address.email, "nullparams@example.com");
    assert!(address.parameters.is_none());
}
