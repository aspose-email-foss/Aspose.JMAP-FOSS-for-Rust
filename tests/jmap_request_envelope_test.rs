use std::collections::HashMap;

use aspose_jmap_foss::{JmapRequestEnvelope, Invocation};
use serde_json::{json, Value};

#[test]
fn test_serialize_full_envelope() {
    // Build a sample envelope with all fields present.
    let mut created = HashMap::new();
    created.insert("client1".to_string(), "server1".to_string());

    let mut args = HashMap::new(); // empty arguments map
    let invocation = Invocation {
        name: "Mailbox/get".to_string(),
        arguments: args,
        method_call_id: "c1".to_string(),
    };

    let envelope = JmapRequestEnvelope {
        using: vec!["urn:ietf:params:jmap:core".to_string()],
        method_calls: vec![invocation],
        created_ids: Some(created),
    };

    // Serialize to JSON.
    let serialized = serde_json::to_value(&envelope).expect("serialization failed");

    // Expected JSON structure.
    let expected: Value = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            [
                "Mailbox/get",
                {},               // empty arguments object
                "c1"
            ]
        ],
        "createdIds": {
            "client1": "server1"
        }
    });

    assert_eq!(serialized, expected);
}

#[test]
fn test_deserialize_with_null_created_ids() {
    // JSON where createdIds is explicitly null.
    let raw = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            [
                "Email/query",
                { "accountId": "acc123" },
                "q1"
            ]
        ],
        "createdIds": null
    });

    let envelope: JmapRequestEnvelope =
        serde_json::from_value(raw).expect("deserialization failed");

    assert_eq!(envelope.using, vec!["urn:ietf:params:jmap:core"]);
    assert_eq!(envelope.method_calls.len(), 1);
    assert_eq!(envelope.method_calls[0].name, "Email/query");
    assert_eq!(envelope.created_ids, None);
}

#[test]
fn test_deserialize_missing_created_ids() {
    // JSON without the createdIds field at all.
    let raw = json!({
        "using": ["urn:ietf:params:jmap:core"],
        "methodCalls": [
            [
                "Identity/get",
                { "accountId": "acc456" },
                "i1"
            ]
        ]
    });

    let envelope: JmapRequestEnvelope =
        serde_json::from_value(raw).expect("deserialization failed");

    assert_eq!(envelope.using, vec!["urn:ietf:params:jmap:core"]);
    assert_eq!(envelope.method_calls.len(), 1);
    assert_eq!(envelope.method_calls[0].name, "Identity/get");
    assert!(envelope.created_ids.is_none());
}
