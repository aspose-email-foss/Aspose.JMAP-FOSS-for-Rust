use aspose_jmap_foss::{EmailBodyPart, EmailHeader};
use serde_json::{json, Value};

#[test]
fn test_deserialize_full() {
    // JSON containing every field (optional fields are present with concrete values)
    let raw = r#"
    {
        "partId": "part-1",
        "blobId": "blob-abc",
        "size": 2048,
        "headers": [
            { "name": "Subject", "value": "Test email" },
            { "name": "From", "value": "sender@example.com" }
        ],
        "name": "attachment.txt",
        "type": "text/plain",
        "charset": "utf-8",
        "disposition": "attachment",
        "cid": "<cid@example.com>",
        "language": ["en", "fr"],
        "location": "/attachments/1",
        "subParts": [
            {
                "size": 512,
                "headers": [],
                "type": "image/png"
            }
        ]
    }
    "#;

    let part: EmailBodyPart = serde_json::from_str(raw).expect("deserialization should succeed");

    assert_eq!(part.part_id, Some("part-1".to_string()));
    assert_eq!(part.blob_id, Some("blob-abc".to_string()));
    assert_eq!(part.size, 2048);
    assert_eq!(
        part.headers,
        vec![
            EmailHeader {
                name: "Subject".to_string(),
                value: "Test email".to_string(),
            },
            EmailHeader {
                name: "From".to_string(),
                value: "sender@example.com".to_string(),
            },
        ]
    );
    assert_eq!(part.name, Some("attachment.txt".to_string()));
    assert_eq!(part.type_, "text/plain".to_string());
    assert_eq!(part.charset, Some("utf-8".to_string()));
    assert_eq!(part.disposition, Some("attachment".to_string()));
    assert_eq!(part.cid, Some("<cid@example.com>".to_string()));
    assert_eq!(part.language, Some(vec!["en".to_string(), "fr".to_string()]));
    assert_eq!(part.location, Some("/attachments/1".to_string()));
    // subParts should contain one nested EmailBodyPart with only required fields set
    let sub = part.sub_parts.unwrap();
    assert_eq!(sub.len(), 1);
    let sub_part = &sub[0];
    assert_eq!(sub_part.part_id, None);
    assert_eq!(sub_part.blob_id, None);
    assert_eq!(sub_part.size, 512);
    assert!(sub_part.headers.is_empty());
    assert_eq!(sub_part.type_, "image/png".to_string());
    assert_eq!(sub_part.charset, None);
    assert_eq!(sub_part.disposition, None);
    assert_eq!(sub_part.cid, None);
    assert_eq!(sub_part.language, None);
    assert_eq!(sub_part.location, None);
    assert_eq!(sub_part.sub_parts, None);
}

#[test]
fn test_deserialize_with_nulls_and_missing_optionals() {
    // JSON where optional fields are either omitted or explicitly null
    let raw = r#"
    {
        "size": 1234,
        "headers": [],
        "type": "application/json",
        "partId": null,
        "blobId": null,
        "name": null,
        "charset": null,
        "disposition": null,
        "cid": null,
        "language": null,
        "location": null,
        "subParts": null
    }
    "#;

    let part: EmailBodyPart = serde_json::from_str(raw).expect("deserialization should succeed");

    // All optional fields must be None
    assert_eq!(part.part_id, None);
    assert_eq!(part.blob_id, None);
    assert_eq!(part.name, None);
    assert_eq!(part.charset, None);
    assert_eq!(part.disposition, None);
    assert_eq!(part.cid, None);
    assert_eq!(part.language, None);
    assert_eq!(part.location, None);
    assert_eq!(part.sub_parts, None);

    // Required fields must be present
    assert_eq!(part.size, 1234);
    assert!(part.headers.is_empty());
    assert_eq!(part.type_, "application/json".to_string());
}

#[test]
fn test_serialize_full() {
    // Build a fully populated EmailBodyPart instance
    let part = EmailBodyPart {
        part_id: Some("part-42".to_string()),
        blob_id: Some("blob-xyz".to_string()),
        size: 4096,
        headers: vec![
            EmailHeader {
                name: "Content-Type".to_string(),
                value: "text/html".to_string(),
            },
            EmailHeader {
                name: "X-Custom".to_string(),
                value: "value".to_string(),
            },
        ],
        name: Some("document.html".to_string()),
        type_: "text/html".to_string(),
        charset: Some("utf-8".to_string()),
        disposition: Some("inline".to_string()),
        cid: Some("<doc@example.com>".to_string()),
        language: Some(vec!["en".to_string()]),
        location: Some("/docs/1".to_string()),
        sub_parts: Some(vec![]),
    };

    // Serialize to JSON
    let serialized = serde_json::to_string(&part).expect("serialization should succeed");
    let value: Value = serde_json::from_str(&serialized).expect("must be valid JSON");

    // Expected JSON structure (order of object keys is not significant)
    let expected = json!({
        "partId": "part-42",
        "blobId": "blob-xyz",
        "size": 4096,
        "headers": [
            { "name": "Content-Type", "value": "text/html" },
            { "name": "X-Custom", "value": "value" }
        ],
        "name": "document.html",
        "type": "text/html",
        "charset": "utf-8",
        "disposition": "inline",
        "cid": "<doc@example.com>",
        "language": ["en"],
        "location": "/docs/1",
        "subParts": []
    });

    assert_eq!(value, expected);
}
