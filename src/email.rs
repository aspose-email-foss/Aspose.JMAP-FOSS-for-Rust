use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::email_address::EmailAddress;
use crate::email_body_part::EmailBodyPart;

/// A single email message (RFC 8621 "Email" object). Immutable content (headers, body)
/// plus mutable per‑mailbox metadata (mailboxIds, keywords).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Email {
    /// Server‑assigned identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "id")]
    pub id: Option<String>,

    /// Blob id of the full raw RFC 5322 message (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "blobId")]
    pub blob_id: Option<String>,

    /// Server‑assigned thread identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "threadId")]
    pub thread_id: Option<String>,

    /// Set of mailbox ids this Email is in.
    #[serde(rename = "mailboxIds")]
    pub mailbox_ids: HashMap<String, bool>,

    /// Keywords such as `$seen`, `$flagged`, etc. (default = {}).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    #[serde(rename = "keywords")]
    pub keywords: HashMap<String, bool>,

    /// Size in octets (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "size")]
    pub size: Option<u64>,

    /// UTC timestamp when the message was received (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "receivedAt")]
    pub received_at: Option<String>,

    /// Message‑Id header values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "messageId")]
    pub message_id: Option<Vec<String>>,

    /// In‑Reply‑To header values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "inReplyTo")]
    pub in_reply_to: Option<Vec<String>>,

    /// References header values.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "references")]
    pub references: Option<Vec<String>>,

    /// Sender addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sender")]
    pub sender: Option<Vec<EmailAddress>>,

    /// From addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "from")]
    pub from: Option<Vec<EmailAddress>>,

    /// To addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "to")]
    pub to: Option<Vec<EmailAddress>>,

    /// Cc addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "cc")]
    pub cc: Option<Vec<EmailAddress>>,

    /// Bcc addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bcc")]
    pub bcc: Option<Vec<EmailAddress>>,

    /// Reply‑To addresses.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "replyTo")]
    pub reply_to: Option<Vec<EmailAddress>>,

    /// Subject line.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "subject")]
    pub subject: Option<String>,

    /// Sent timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "sentAt")]
    pub sent_at: Option<String>,

    /// Body structure (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bodyStructure")]
    pub body_structure: Option<EmailBodyPart>,

    /// Map of body part ids to their values (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "bodyValues")]
    pub body_values: Option<HashMap<String, EmailBodyValue>>,

    /// Text body parts (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "textBody")]
    pub text_body: Option<Vec<EmailBodyPart>>,

    /// HTML body parts (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "htmlBody")]
    pub html_body: Option<Vec<EmailBodyPart>>,

    /// Attachments (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "attachments")]
    pub attachments: Option<Vec<EmailBodyPart>>,

    /// Whether the message has any attachment (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "hasAttachment")]
    pub has_attachment: Option<bool>,

    /// Preview text (server‑assigned).
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(rename = "preview")]
    pub preview: Option<String>,
}

/// Represents a body value with its content and flags.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailBodyValue {
    /// The actual body content.
    #[serde(rename = "value")]
    pub value: String,

    /// Indicates an encoding problem.
    #[serde(rename = "isEncodingProblem")]
    pub is_encoding_problem: bool,

    /// Indicates whether the value was truncated.
    #[serde(rename = "isTruncated")]
    pub is_truncated: bool,
}
