use aspose_jmap_foss::Thread;
use serde_json::{self, json};

#[test]
fn test_thread_serialization_deserialization() {
    // Create a Thread with all fields populated
    let thread = Thread {
        id: Some("thread-123".to_string()),
        email_ids: Some(vec![
            "email-1".to_string(),
            "email-2".to_string(),
            "email-3".to_string(),
        ]),
    };

    // Serialize to JSON
    let serialized = serde_json::to_string(&thread).expect("serialization failed");
    // Expected JSON object (field order is not guaranteed, so compare as Value)
    let expected_json = json!({
        "id": "thread-123",
        "emailIds": ["email-1", "email-2", "email-3"]
    });
    let serialized_json: serde_json::Value =
        serde_json::from_str(&serialized).expect("parsing serialized JSON failed");
    assert_eq!(serialized_json, expected_json);

    // Deserialize back to a Thread
    let deserialized: Thread =
        serde_json::from_str(&serialized).expect("deserialization failed");
    assert_eq!(deserialized.id, Some("thread-123".to_string()));
    assert_eq!(
        deserialized.email_ids,
        Some(vec![
            "email-1".to_string(),
            "email-2".to_string(),
            "email-3".to_string()
        ])
    );
}

#[test]
fn test_thread_missing_optional_fields() {
    // JSON with no fields (both are optional)
    let empty_json = "{}";

    // Deserialize should succeed and produce a Thread with None fields
    let thread: Thread =
        serde_json::from_str(empty_json).expect("deserialization of empty object failed");
    assert!(thread.id.is_none());
    assert!(thread.email_ids.is_none());

    // Serializing the empty Thread should produce an empty JSON object
    let serialized = serde_json::to_string(&thread).expect("serialization failed");
    let serialized_json: serde_json::Value =
        serde_json::from_str(&serialized).expect("parsing serialized JSON failed");
    assert_eq!(serialized_json, json!({}));
}
