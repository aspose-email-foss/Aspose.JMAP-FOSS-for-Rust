use serde::{Deserialize, Serialize};

/// A named set of Emails (JMAP's analogue of a mail folder / IMAP mailbox). Top-level, addressable JMAP data type.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Mailbox {
    /// Server-assigned identifier. Omitted when constructing a create payload.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// The name of the mailbox. Required.
    #[serde(rename = "name")]
    pub name: String,

    /// Identifier of the parent mailbox, if any.
    #[serde(rename = "parentId", skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<String>,

    /// Role of the mailbox (e.g. inbox, sent, drafts, trash, junk, archive).
    #[serde(rename = "role", skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,

    /// Sort order for the mailbox. Defaults to `0`.
    #[serde(rename = "sortOrder", skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<u64>,

    /// Total number of emails in the mailbox. Server-assigned.
    #[serde(rename = "totalEmails", skip_serializing_if = "Option::is_none")]
    pub total_emails: Option<u64>,

    /// Number of unread emails in the mailbox. Server-assigned.
    #[serde(rename = "unreadEmails", skip_serializing_if = "Option::is_none")]
    pub unread_emails: Option<u64>,

    /// Total number of threads in the mailbox. Server-assigned.
    #[serde(rename = "totalThreads", skip_serializing_if = "Option::is_none")]
    pub total_threads: Option<u64>,

    /// Number of unread threads in the mailbox. Server-assigned.
    #[serde(rename = "unreadThreads", skip_serializing_if = "Option::is_none")]
    pub unread_threads: Option<u64>,

    /// Rights the authenticated user has on this mailbox. Server-assigned.
    #[serde(rename = "myRights", skip_serializing_if = "Option::is_none")]
    pub my_rights: Option<MailboxRights>,

    /// Whether the mailbox is subscribed. Defaults to `false`.
    #[serde(
        rename = "isSubscribed",
        default,
        skip_serializing_if = "std::ops::Not::not"
    )]
    pub is_subscribed: bool,
}

/// Rights the authenticated user has on a mailbox.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MailboxRights {
    #[serde(rename = "mayReadItems")]
    pub may_read_items: bool,

    #[serde(rename = "mayAddItems")]
    pub may_add_items: bool,

    #[serde(rename = "mayRemoveItems")]
    pub may_remove_items: bool,

    #[serde(rename = "maySetSeen")]
    pub may_set_seen: bool,

    #[serde(rename = "maySetKeywords")]
    pub may_set_keywords: bool,

    #[serde(rename = "mayCreateChild")]
    pub may_create_child: bool,

    #[serde(rename = "mayRename")]
    pub may_rename: bool,

    #[serde(rename = "mayDelete")]
    pub may_delete: bool,

    #[serde(rename = "maySubmit")]
    pub may_submit: bool,
}
