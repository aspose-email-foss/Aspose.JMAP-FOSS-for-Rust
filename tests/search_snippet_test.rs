use aspose_jmap_foss::SearchSnippet;
use serde_json::{json, Value};

#[test]
fn deserialize_full() {
    let data = r#"
    {
        "emailId": "msg-123",
        "subject": "Hello <mark>world</mark>",
        "preview": "This is a <mark>test</mark>"
    }
    "#;
    let snippet: SearchSnippet = serde_json::from_str(data).expect("valid JSON");
    assert_eq!(snippet.email_id, "msg-123");
    assert_eq!(snippet.subject.as_deref(), Some("Hello <mark>world</mark>"));
    assert_eq!(snippet.preview.as_deref(), Some("This is a <mark>test</mark>"));
}

#[test]
fn deserialize_missing_optional() {
    let data = r#"{"emailId":"msg-456"}"#;
    let snippet: SearchSnippet = serde_json::from_str(data).expect("valid JSON");
    assert_eq!(snippet.email_id, "msg-456");
    assert!(snippet.subject.is_none());
    assert!(snippet.preview.is_none());
}

#[test]
fn serialize_full() {
    let snippet = SearchSnippet {
        email_id: "msg-789".to_string(),
        subject: Some("Subject <mark>match</mark>".to_string()),
        preview: Some("Preview <mark>match</mark>".to_string()),
    };
    let json_str = serde_json::to_string(&snippet).expect("serialization");
    let value: Value = serde_json::from_str(&json_str).expect("parse back to Value");
    let expected = json!({
        "emailId": "msg-789",
        "subject": "Subject <mark>match</mark>",
        "preview": "Preview <mark>match</mark>"
    });
    assert_eq!(value, expected);
}

#[test]
fn serialize_partial() {
    let snippet = SearchSnippet {
        email_id: "msg-101".to_string(),
        subject: None,
        preview: None,
    };
    let json_str = serde_json::to_string(&snippet).expect("serialization");
    let value: Value = serde_json::from_str(&json_str).expect("parse back to Value");
    let expected = json!({
        "emailId": "msg-101"
    });
    assert_eq!(value, expected);
}
