use std::collections::HashMap;

use aspose_jmap_foss::{Invocation, JmapResponseEnvelope};
use serde_json::{json, Value};

#[test]
fn test_serialize_deserialize_full() {
    // Build a sample Invocation
    let mut args = HashMap::new();
    args.insert("accountId".to_string(), json!("a1"));
    args.insert("ids".to_string(), json!(["mailbox1", "mailbox2"]));

    let invocation = Invocation {
        name: "Mailbox/get".to_string(),
        arguments: args,
        method_call_id: "c1".to_string(),
    };

    // Build the envelope with all fields present
    let mut created = HashMap::new();
    created.insert("newId".to_string(), "oldId".to_string());

    let envelope = JmapResponseEnvelope {
        method_responses: vec![invocation.clone()],
        created_ids: Some(created.clone()),
        session_state: "12345".to_string(),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&envelope).expect("serialization failed");

    // Deserialize back
    let deserialized: JmapResponseEnvelope =
        serde_json::from_str(&json_str).expect("deserialization failed");

    // Verify round‑trip equality
    assert_eq!(deserialized.session_state, envelope.session_state);
    assert_eq!(deserialized.created_ids, envelope.created_ids);
    assert_eq!(deserialized.method_responses, envelope.method_responses);
    // Ensure the inner Invocation matches
    assert_eq!(deserialized.method_responses[0], invocation);
}

#[test]
fn test_deserialize_missing_created_ids() {
    // JSON without the optional `createdIds` field
    let json_payload = json!({
        "methodResponses": [
            [
                "Mailbox/get",
                {
                    "accountId": "a1",
                    "ids": []
                },
                "c1"
            ]
        ],
        "sessionState": "abcde"
    });

    let json_str = json_payload.to_string();

    let envelope: JmapResponseEnvelope =
        serde_json::from_str(&json_str).expect("deserialization failed");

    assert_eq!(envelope.session_state, "abcde");
    assert!(envelope.created_ids.is_none());
    assert_eq!(envelope.method_responses.len(), 1);
    let inv = &envelope.method_responses[0];
    assert_eq!(inv.name, "Mailbox/get");
    assert_eq!(inv.method_call_id, "c1");
    // arguments should contain the expected keys
    assert_eq!(inv.arguments.get("accountId"), Some(&json!("a1")));
    assert_eq!(inv.arguments.get("ids"), Some(&json!([])));
}

#[test]
fn test_deserialize_null_created_ids() {
    // JSON with `createdIds` explicitly set to null
    let json_payload = json!({
        "methodResponses": [
            [
                "Mailbox/get",
                {
                    "accountId": "a2",
                    "ids": ["mailbox3"]
                },
                "c2"
            ]
        ],
        "createdIds": null,
        "sessionState": "fghij"
    });

    let json_str = json_payload.to_string();

    let envelope: JmapResponseEnvelope =
        serde_json::from_str(&json_str).expect("deserialization failed");

    assert_eq!(envelope.session_state, "fghij");
    assert!(envelope.created_ids.is_none());
    assert_eq!(envelope.method_responses.len(), 1);
    let inv = &envelope.method_responses[0];
    assert_eq!(inv.name, "Mailbox/get");
    assert_eq!(inv.method_call_id, "c2");
    assert_eq!(inv.arguments.get("accountId"), Some(&json!("a2")));
    assert_eq!(inv.arguments.get("ids"), Some(&json!(["mailbox3"])));
}
