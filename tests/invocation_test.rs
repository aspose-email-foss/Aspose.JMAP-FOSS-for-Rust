use aspose_jmap_foss::Invocation;
use serde_json::{json, Value};
use std::collections::HashMap;

#[test]
fn test_invocation_serde_roundtrip() {
    // Prepare arguments map
    let mut args = HashMap::new();
    args.insert("accountId".to_string(), json!("a1"));

    // Construct the Invocation instance
    let inv = Invocation {
        name: "Mailbox/get".to_string(),
        arguments: args.clone(),
        method_call_id: "c123".to_string(),
    };

    // Serialize to JSON string
    let serialized = serde_json::to_string(&inv).expect("serialization failed");
    let expected = json!(["Mailbox/get", args, "c123"]).to_string();
    assert_eq!(serialized, expected, "serialized JSON does not match expected");

    // Deserialize back and compare
    let deserialized: Invocation =
        serde_json::from_str(&serialized).expect("deserialization failed");
    assert_eq!(deserialized, inv, "deserialized value differs from original");
}

#[test]
fn test_invocation_deserialize_invalid_length() {
    // JSON array missing the methodCallId element (invalid length)
    let bad_json = json!(["Mailbox/get", {"accountId": "a1"}]);
    let s = bad_json.to_string();

    let result: Result<Invocation, _> = serde_json::from_str(&s);
    assert!(
        result.is_err(),
        "deserialization should fail for an array with insufficient elements"
    );
}
