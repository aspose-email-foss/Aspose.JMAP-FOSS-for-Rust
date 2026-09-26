use aspose_jmap_foss::{EmailAddress, EmailAddressGroup};
use serde_json::{json, Value};

#[test]
fn serialize_with_name() {
    let group = EmailAddressGroup {
        name: Some("Team".to_string()),
        addresses: vec![EmailAddress {
            name: Some("Alice".to_string()),
            email: "alice@example.com".to_string(),
        }],
    };

    let serialized: Value = serde_json::to_value(&group).expect("serialization failed");
    let expected = json!({
        "name": "Team",
        "addresses": [
            {
                "name": "Alice",
                "email": "alice@example.com"
            }
        ]
    });

    assert_eq!(serialized, expected);
}

#[test]
fn serialize_without_name() {
    let group = EmailAddressGroup {
        name: None,
        addresses: vec![EmailAddress {
            name: None,
            email: "bob@example.com".to_string(),
        }],
    };

    let serialized: Value = serde_json::to_value(&group).expect("serialization failed");
    let expected = json!({
        "addresses": [
            {
                "email": "bob@example.com"
            }
        ]
    });

    assert_eq!(serialized, expected);
}

#[test]
fn deserialize_with_null_name() {
    let json_data = r#"
    {
        "name": null,
        "addresses": [
            { "name": "Carol", "email": "carol@example.com" },
            { "email": "dave@example.com" }
        ]
    }
    "#;

    let group: EmailAddressGroup =
        serde_json::from_str(json_data).expect("deserialization failed");

    assert_eq!(group.name, None);
    assert_eq!(group.addresses.len(), 2);
    assert_eq!(group.addresses[0].name.as_deref(), Some("Carol"));
    assert_eq!(group.addresses[0].email, "carol@example.com");
    assert_eq!(group.addresses[1].name, None);
    assert_eq!(group.addresses[1].email, "dave@example.com");
}
