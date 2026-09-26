use aspose_jmap_foss::{Mailbox, MailboxRights};
use serde_json::{self, Value};

#[test]
fn test_mailbox_full_serialization_deserialization() {
    // A mailbox with every field populated.
    let json = r#"
    {
        "id": "mailbox123",
        "name": "Inbox",
        "parentId": "parent456",
        "role": "inbox",
        "sortOrder": 10,
        "totalEmails": 100,
        "unreadEmails": 5,
        "totalThreads": 80,
        "unreadThreads": 3,
        "myRights": {
            "mayReadItems": true,
            "mayAddItems": false,
            "mayRemoveItems": true,
            "maySetSeen": false,
            "maySetKeywords": true,
            "mayCreateChild": false,
            "mayRename": true,
            "mayDelete": false,
            "maySubmit": true
        },
        "isSubscribed": true
    }
    "#;

    // Deserialize JSON into the model.
    let mailbox: Mailbox = serde_json::from_str(json).expect("deserialization should succeed");

    // Verify each field.
    assert_eq!(mailbox.id.as_deref(), Some("mailbox123"));
    assert_eq!(mailbox.name, "Inbox");
    assert_eq!(mailbox.parent_id.as_deref(), Some("parent456"));
    assert_eq!(mailbox.role.as_deref(), Some("inbox"));
    assert_eq!(mailbox.sort_order, Some(10));
    assert_eq!(mailbox.total_emails, Some(100));
    assert_eq!(mailbox.unread_emails, Some(5));
    assert_eq!(mailbox.total_threads, Some(80));
    assert_eq!(mailbox.unread_threads, Some(3));
    assert!(mailbox.is_subscribed);
    let rights = mailbox.my_rights.as_ref().expect("myRights should be present");
    assert!(rights.may_read_items);
    assert!(!rights.may_add_items);
    assert!(rights.may_remove_items);
    assert!(!rights.may_set_seen);
    assert!(rights.may_set_keywords);
    assert!(!rights.may_create_child);
    assert!(rights.may_rename);
    assert!(!rights.may_delete);
    assert!(rights.may_submit);

    // Serialize back to JSON and compare structural equality.
    let serialized = serde_json::to_string(&mailbox).expect("serialization should succeed");
    let original_val: Value = serde_json::from_str(json).unwrap();
    let serialized_val: Value = serde_json::from_str(&serialized).unwrap();
    assert_eq!(original_val, serialized_val);
}

#[test]
fn test_mailbox_minimal_deserialization() {
    // Only the required field `name` is present.
    let json = r#"{ "name": "Drafts" }"#;

    let mailbox: Mailbox = serde_json::from_str(json).expect("deserialization should succeed");

    // Required field.
    assert_eq!(mailbox.name, "Drafts");
    // Optional fields should be None / default.
    assert!(mailbox.id.is_none());
    assert!(mailbox.parent_id.is_none());
    assert!(mailbox.role.is_none());
    assert!(mailbox.sort_order.is_none());
    assert!(mailbox.total_emails.is_none());
    assert!(mailbox.unread_emails.is_none());
    assert!(mailbox.total_threads.is_none());
    assert!(mailbox.unread_threads.is_none());
    assert!(mailbox.my_rights.is_none());
    // `isSubscribed` defaults to false when omitted.
    assert!(!mailbox.is_subscribed);
}

#[test]
fn test_mailbox_parent_id_null() {
    // `parentId` explicitly set to null should become None.
    let json = r#"{ "name": "Folder", "parentId": null }"#;

    let mailbox: Mailbox = serde_json::from_str(json).expect("deserialization should succeed");

    assert_eq!(mailbox.name, "Folder");
    assert!(mailbox.parent_id.is_none());
    // Other optional fields remain None.
    assert!(mailbox.id.is_none());
    assert!(!mailbox.is_subscribed);
}

#[test]
fn test_mailboxrights_serialization() {
    let rights = MailboxRights {
        may_read_items: true,
        may_add_items: true,
        may_remove_items: false,
        may_set_seen: true,
        may_set_keywords: false,
        may_create_child: true,
        may_rename: false,
        may_delete: true,
        may_submit: false,
    };

    let json = serde_json::to_string(&rights).expect("serialization should succeed");
    let parsed: Value = serde_json::from_str(&json).unwrap();

    // Verify each property appears with the correct name and value.
    assert_eq!(parsed["mayReadItems"], true);
    assert_eq!(parsed["mayAddItems"], true);
    assert_eq!(parsed["mayRemoveItems"], false);
    assert_eq!(parsed["maySetSeen"], true);
    assert_eq!(parsed["maySetKeywords"], false);
    assert_eq!(parsed["mayCreateChild"], true);
    assert_eq!(parsed["mayRename"], false);
    assert_eq!(parsed["mayDelete"], true);
    assert_eq!(parsed["maySubmit"], false);
}
