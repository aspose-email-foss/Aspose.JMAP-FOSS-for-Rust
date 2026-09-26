use std::collections::HashMap;

use aspose_jmap_foss::{Email, EmailBodyValue, EmailAddress, EmailBodyPart};
use serde_json::{json, Value};

#[test]
fn test_email_serialization_deserialization_normal() {
    // Prepare data
    let mut mailbox_ids = HashMap::new();
    mailbox_ids.insert("mailboxA".to_string(), true);
    mailbox_ids.insert("mailboxB".to_string(), false);

    let mut keywords = HashMap::new();
    keywords.insert("$seen".to_string(), true);
    keywords.insert("$flagged".to_string(), false);

    let sender = vec![EmailAddress {
        name: Some("Alice".to_string()),
        email: "alice@example.com".to_string(),
    }];

    let email = Email {
        id: Some("email123".to_string()),
        blob_id: Some("blob456".to_string()),
        thread_id: Some("thread789".to_string()),
        mailbox_ids,
        keywords,
        size: Some(1024),
        received_at: Some("2023-01-01T12:00:00Z".to_string()),
        message_id: Some(vec!["<msgid@example.com>".to_string()]),
        in_reply_to: None,
        references: None,
        sender: Some(sender.clone()),
        from: Some(sender.clone()),
        to: None,
        cc: None,
        bcc: None,
        reply_to: None,
        subject: Some("Test Subject".to_string()),
        sent_at: Some("2023-01-01T11:00:00Z".to_string()),
        body_structure: None,
        body_values: None,
        text_body: None,
        html_body: None,
        attachments: None,
        has_attachment: Some(false),
        preview: Some("preview text".to_string()),
    };

    // Serialize to JSON
    let json_str = serde_json::to_string(&email).expect("serialization failed");
    let json_val: Value =
        serde_json::from_str(&json_str).expect("parsing serialized JSON failed");

    // Verify required fields are present with correct names
    assert_eq!(json_val["id"], "email123");
    assert_eq!(json_val["blobId"], "blob456");
    assert_eq!(json_val["threadId"], "thread789");
    assert_eq!(json_val["mailboxIds"]["mailboxA"], true);
    assert_eq!(json_val["mailboxIds"]["mailboxB"], false);
    assert_eq!(json_val["keywords"]["$seen"], true);
    assert_eq!(json_val["keywords"]["$flagged"], false);
    assert_eq!(json_val["size"], 1024);
    assert_eq!(json_val["receivedAt"], "2023-01-01T12:00:00Z");
    assert_eq!(json_val["subject"], "Test Subject");
    assert_eq!(json_val["sentAt"], "2023-01-01T11:00:00Z");
    assert_eq!(json_val["hasAttachment"], false);
    assert_eq!(json_val["preview"], "preview text");

    // Deserialize back and compare
    let deserialized: Email =
        serde_json::from_str(&json_str).expect("deserialization failed");
    assert_eq!(deserialized, email);
}

#[test]
fn test_email_serialization_edge_missing_optional() {
    // Only required fields; keywords left empty (should be omitted)
    let mut mailbox_ids = HashMap::new();
    mailbox_ids.insert("inbox".to_string(), true);

    let email = Email {
        id: None,
        blob_id: None,
        thread_id: None,
        mailbox_ids,
        keywords: HashMap::new(), // empty map
        size: None,
        received_at: None,
        message_id: None,
        in_reply_to: None,
        references: None,
        sender: None,
        from: None,
        to: None,
        cc: None,
        bcc: None,
        reply_to: None,
        subject: None,
        sent_at: None,
        body_structure: None,
        body_values: None,
        text_body: None,
        html_body: None,
        attachments: None,
        has_attachment: None,
        preview: None,
    };

    // Serialize
    let json_str = serde_json::to_string(&email).expect("serialization failed");
    let json_val: Value =
        serde_json::from_str(&json_str).expect("parsing serialized JSON failed");

    // Required field must be present
    assert_eq!(json_val["mailboxIds"]["inbox"], true);

    // Optional fields must be omitted
    assert!(json_val.get("id").is_none());
    assert!(json_val.get("blobId").is_none());
    assert!(json_val.get("keywords").is_none()); // empty map omitted
    assert!(json_val.get("size").is_none());

    // Deserialize from minimal JSON
    let minimal_json = json!({
        "mailboxIds": { "inbox": true }
    })
    .to_string();

    let deserialized: Email =
        serde_json::from_str(&minimal_json).expect("deserialization failed");
    assert_eq!(deserialized.id, None);
    assert_eq!(deserialized.keywords.is_empty(), true);
    assert_eq!(deserialized.mailbox_ids.get("inbox"), Some(&true));
}

#[test]
fn test_email_body_value_serialization() {
    let body_value = EmailBodyValue {
        value: "Hello, world!".to_string(),
        is_encoding_problem: false,
        is_truncated: true,
    };

    let json_str = serde_json::to_string(&body_value).expect("serialization failed");
    let json_val: Value =
        serde_json::from_str(&json_str).expect("parsing serialized JSON failed");

    assert_eq!(json_val["value"], "Hello, world!");
    assert_eq!(json_val["isEncodingProblem"], false);
    assert_eq!(json_val["isTruncated"], true);

    let deserialized: EmailBodyValue =
        serde_json::from_str(&json_str).expect("deserialization failed");
    assert_eq!(deserialized, body_value);
}
