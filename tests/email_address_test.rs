use aspose_jmap_foss::EmailAddress;
use serde_json::{json, Value};

#[test]
fn deserialize_full() {
    let data = r#"{"name":"Alice","email":"alice@example.com"}"#;
    let addr: EmailAddress = serde_json::from_str(data).expect("deserialization failed");
    assert_eq!(addr.name, Some("Alice".to_string()));
    assert_eq!(addr.email, "alice@example.com");
}

#[test]
fn deserialize_without_name() {
    let data = r#"{"email":"bob@example.com"}"#;
    let addr: EmailAddress = serde_json::from_str(data).expect("deserialization failed");
    assert_eq!(addr.name, None);
    assert_eq!(addr.email, "bob@example.com");
}

#[test]
fn serialize_full() {
    let addr = EmailAddress {
        name: Some("Carol".to_string()),
        email: "carol@example.com".to_string(),
    };
    let json_str = serde_json::to_string(&addr).expect("serialization failed");
    let v: Value = serde_json::from_str(&json_str).expect("parsing json string failed");
    let expected = json!({
        "name": "Carol",
        "email": "carol@example.com"
    });
    assert_eq!(v, expected);
}

#[test]
fn serialize_without_name() {
    let addr = EmailAddress {
        name: None,
        email: "dave@example.com".to_string(),
    };
    let json_str = serde_json::to_string(&addr).expect("serialization failed");
    let v: Value = serde_json::from_str(&json_str).expect("parsing json string failed");
    let expected = json!({
        "email": "dave@example.com"
    });
    assert_eq!(v, expected);
}
