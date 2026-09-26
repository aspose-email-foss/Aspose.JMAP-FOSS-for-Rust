use crate::delivery_status::DeliveryStatus;
use crate::envelope::Envelope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// One attempt to submit an Email for delivery. Analogous to what Aspose.Email's
/// `SmtpClient.Send` does synchronously over SMTP; here creating an `EmailSubmission`
/// is what actually dispatches the message via the server's outbound MTA.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailSubmission {
    /// Server‑assigned identifier (omitted when constructing a create payload).
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Must reference an existing Identity (required).
    #[serde(rename = "identityId")]
    pub identity_id: String,

    /// The Email to send; typically a draft created via `Email/set` just before this call (required).
    #[serde(rename = "emailId")]
    pub email_id: String,

    /// Server‑assigned thread identifier (omitted when constructing a create payload).
    #[serde(rename = "threadId", skip_serializing_if = "Option::is_none")]
    pub thread_id: Option<String>,

    /// If `null`, the server derives it from the Email's From/To/Cc/Bcc headers.
    #[serde(rename = "envelope", skip_serializing_if = "Option::is_none")]
    pub envelope: Option<Envelope>,

    /// Server‑assigned UTC date when the submission is to be sent (omitted when constructing a create payload).
    #[serde(rename = "sendAt", skip_serializing_if = "Option::is_none")]
    pub send_at: Option<String>,

    /// Server‑assigned undo status (`pending` | `final` | `canceled`) (omitted when constructing a create payload).
    #[serde(rename = "undoStatus", skip_serializing_if = "Option::is_none")]
    pub undo_status: Option<String>,

    /// Server‑assigned delivery status map (omitted when constructing a create payload).
    #[serde(rename = "deliveryStatus", skip_serializing_if = "Option::is_none")]
    pub delivery_status: Option<HashMap<String, DeliveryStatus>>,

    /// Server‑assigned DSN blob identifiers (omitted when constructing a create payload).
    #[serde(rename = "dsnBlobIds", skip_serializing_if = "Option::is_none")]
    pub dsn_blob_ids: Option<Vec<String>>,

    /// Server‑assigned MDN blob identifiers (omitted when constructing a create payload).
    #[serde(rename = "mdnBlobIds", skip_serializing_if = "Option::is_none")]
    pub mdn_blob_ids: Option<Vec<String>>,
}
