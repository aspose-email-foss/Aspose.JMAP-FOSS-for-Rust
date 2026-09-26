use aspose_jmap_foss::EmailSubmission;
use serde_json::{self, json, Value};

#[test]
fn deserialize_full_email_submission() {
    let raw = r#"
    {
        "id": "sub123",
        "identityId": "identity456",
        "emailId": "email789",
        "threadId": "thread001",
        "envelope": {
            "mailFrom": { "email": "sender@example.com" },
            "rcptTo": [ { "email": "rcpt@example.com" } ]
        },
        "sendAt": "2023-01-01T00:00:00Z",
        "undoStatus": "pending",
        "deliveryStatus": {
            "rcpt@example.com": {
                "smtpReply": "250 OK",
                "delivered": "yes",
                "displayed": "yes"
            }
        },
        "dsnBlobIds": ["dsn1"],
        "mdnBlobIds": ["mdn1"]
    }
    "#;

    let submission: EmailSubmission = serde_json::from_str(raw).expect("deserialization failed");

    // required fields
    assert_eq!(submission.identity_id, "identity456");
    assert_eq!(submission.email_id, "email789");

    // optional fields present
    assert_eq!(submission.id.as_deref(), Some("sub123"));
    assert_eq!(submission.thread_id.as_deref(), Some("thread001"));
    assert!(submission.envelope.is_some());
    assert_eq!(submission.send_at.as_deref(), Some("2023-01-01T00:00:00Z"));
    assert_eq!(submission.undo_status.as_deref(), Some("pending"));
    assert!(submission.delivery_status.is_some());
    assert_eq!(submission.dsn_blob_ids.as_deref(), Some(["dsn1".to_string()].as_slice()));
    assert_eq!(submission.mdn_blob_ids.as_deref(), Some(["mdn1".to_string()].as_slice()));

    // check a nested delivery status entry
    let ds_map = submission.delivery_status.unwrap();
    let rcpt_status = ds_map.get("rcpt@example.com").expect("missing delivery status");
    assert_eq!(rcpt_status.smtp_reply, "250 OK");
    assert_eq!(rcpt_status.delivered, "yes");
    assert_eq!(rcpt_status.displayed, "yes");
}

#[test]
fn deserialize_with_null_and_missing_optionals() {
    let raw = r#"
    {
        "identityId": "identity456",
        "emailId": "email789",
        "envelope": null
    }
    "#;

    let submission: EmailSubmission = serde_json::from_str(raw).expect("deserialization failed");

    // required fields
    assert_eq!(submission.identity_id, "identity456");
    assert_eq!(submission.email_id, "email789");

    // optional fields should be None
    assert!(submission.id.is_none());
    assert!(submission.thread_id.is_none());
    assert!(submission.envelope.is_none());
    assert!(submission.send_at.is_none());
    assert!(submission.undo_status.is_none());
    assert!(submission.delivery_status.is_none());
    assert!(submission.dsn_blob_ids.is_none());
    assert!(submission.mdn_blob_ids.is_none());

    // Serializing back should omit all None fields
    let serialized = serde_json::to_string(&submission).expect("serialization failed");
    let value: Value = serde_json::from_str(&serialized).expect("parsing serialized JSON failed");

    // The resulting JSON must contain only the required fields
    let expected = json!({
        "identityId": "identity456",
        "emailId": "email789"
    });
    assert_eq!(value, expected);
}
