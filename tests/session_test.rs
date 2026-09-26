use aspose_jmap_foss::{Account, CoreCapability, Session};
use serde_json::{json, Value};

#[test]
fn deserialize_full_session() {
    // A complete Session JSON payload with all required fields.
    let raw = json!({
        "capabilities": {
            "urn:ietf:params:jmap:core": {
                "maxSizeUpload": 50000000,
                "maxConcurrentUpload": 4,
                "maxSizeRequest": 10000000,
                "maxConcurrentRequests": 8,
                "maxCallsInRequest": 16,
                "maxObjectsInGet": 500,
                "maxObjectsInSet": 200,
                "collationAlgorithms": ["i;unicode-casemap"]
            },
            "urn:ietf:params:jmap:mail": {}
        },
        "accounts": {
            "acc123": {
                "name": "Test Account",
                "isPersonal": true,
                "isReadOnly": false,
                "accountCapabilities": {}
            }
        },
        "primaryAccounts": {
            "urn:ietf:params:jmap:mail": "acc123"
        },
        "username": "user@example.test",
        "apiUrl": "/jmap/",
        "downloadUrl": "/download/{accountId}/{blobId}/{type}/{name}",
        "uploadUrl": "/upload/{accountId}",
        "eventSourceUrl": "/events",
        "state": "abc123"
    })
    .to_string();

    let session: Session = serde_json::from_str(&raw).expect("deserialization should succeed");

    // Verify top‑level fields.
    assert_eq!(session.username, "user@example.test");
    assert_eq!(session.api_url, "/jmap/");
    assert_eq!(session.state, "abc123");

    // Verify capabilities map contains the core capability and can be deserialized.
    let core_cap_json = session
        .capabilities
        .get("urn:ietf:params:jmap:core")
        .expect("core capability must be present");
    let core_cap: CoreCapability =
        serde_json::from_value(core_cap_json.clone()).expect("core capability deserialization");
    assert_eq!(core_cap.max_size_upload, 50_000_000);
    assert_eq!(core_cap.collation_algorithms, vec!["i;unicode-casemap"]);

    // Verify account entry.
    let account = session
        .accounts
        .get("acc123")
        .expect("account must be present");
    assert_eq!(account.name, "Test Account");
    assert!(account.is_personal);
    assert!(!account.is_read_only);
    assert!(account.account_capabilities.is_empty());

    // Verify primaryAccounts mapping.
    let primary = session
        .primary_accounts
        .get("urn:ietf:params:jmap:mail")
        .expect("primary account for mail must be present");
    assert_eq!(primary, "acc123");
}

#[test]
fn serialize_and_roundtrip_session() {
    // Build a Session struct programmatically.
    let mut capabilities = std::collections::HashMap::new();
    let core = CoreCapability {
        max_size_upload: 10,
        max_concurrent_upload: 1,
        max_size_request: 20,
        max_concurrent_requests: 2,
        max_calls_in_request: 3,
        max_objects_in_get: 4,
        max_objects_in_set: 5,
        collation_algorithms: vec!["i;unicode-casemap".to_string()],
    };
    capabilities.insert(
        "urn:ietf:params:jmap:core".to_string(),
        serde_json::to_value(&core).unwrap(),
    );
    capabilities.insert(
        "urn:ietf:params:jmap:mail".to_string(),
        Value::Object(serde_json::Map::new()),
    );

    let mut accounts = std::collections::HashMap::new();
    accounts.insert(
        "acc123".to_string(),
        Account {
            name: "Test Account".to_string(),
            is_personal: true,
            is_read_only: false,
            account_capabilities: std::collections::HashMap::new(),
        },
    );

    let mut primary_accounts = std::collections::HashMap::new();
    primary_accounts.insert("urn:ietf:params:jmap:mail".to_string(), "acc123".to_string());

    let session = Session {
        capabilities,
        accounts,
        primary_accounts,
        username: "user@example.test".to_string(),
        api_url: "/jmap/".to_string(),
        download_url: "/download/{accountId}/{blobId}/{type}/{name}".to_string(),
        upload_url: "/upload/{accountId}".to_string(),
        event_source_url: "/events".to_string(),
        state: "state123".to_string(),
    };

    // Serialize to JSON string.
    let json_str = serde_json::to_string(&session).expect("serialization should succeed");

    // Deserialize back and compare.
    let roundtrip: Session = serde_json::from_str(&json_str).expect("deserialization should succeed");
    assert_eq!(roundtrip.username, session.username);
    assert_eq!(roundtrip.api_url, session.api_url);
    assert_eq!(roundtrip.state, session.state);
    assert_eq!(roundtrip.accounts.len(), 1);
    assert_eq!(roundtrip.capabilities.len(), 2);
}

#[test]
fn deserialize_missing_required_field_fails() {
    // Omit the required `username` field.
    let raw = json!({
        "capabilities": {},
        "accounts": {},
        "primaryAccounts": {},
        "apiUrl": "/jmap/",
        "downloadUrl": "/download/",
        "uploadUrl": "/upload/",
        "eventSourceUrl": "/events",
        "state": "xyz"
    })
    .to_string();

    let err = serde_json::from_str::<Session>(&raw).expect_err("deserialization must fail");
    // Ensure the error is a missing field error.
    assert!(
        err.to_string().contains("username"),
        "error message should mention the missing field"
    );
}
