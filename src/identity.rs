use serde::{Deserialize, Serialize};
use crate::email_address::{EmailAddress};

fn is_empty_string(v: &str) -> bool {
    v.is_empty()
}

/// A sending identity (name/email/replyTo used as the From when submitting mail);
/// analogous to configuring a MailAddress + display name on Aspose's SmtpClient.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Identity {
    /// Server‑assigned identifier. Omitted when constructing a create payload.
    #[serde(rename = "id", skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Display name for the identity. Defaults to an empty string.
    #[serde(rename = "name", default, skip_serializing_if = "is_empty_string")]
    pub name: String,

    /// Primary email address for the identity. Required.
    #[serde(rename = "email")]
    pub email: String,

    /// Optional list of reply‑to addresses.
    #[serde(rename = "replyTo", skip_serializing_if = "Option::is_none")]
    pub reply_to: Option<Vec<EmailAddress>>,

    /// Optional list of blind‑carbon‑copy addresses.
    #[serde(rename = "bcc", skip_serializing_if = "Option::is_none")]
    pub bcc: Option<Vec<EmailAddress>>,

    /// Text signature appended to outgoing messages. Defaults to an empty string.
    #[serde(rename = "textSignature", default, skip_serializing_if = "is_empty_string")]
    pub text_signature: String,

    /// HTML signature appended to outgoing messages. Defaults to an empty string.
    #[serde(rename = "htmlSignature", default, skip_serializing_if = "is_empty_string")]
    pub html_signature: String,

    /// Server‑assigned flag indicating whether the identity may be deleted.
    /// Omitted when constructing a create payload.
    #[serde(rename = "mayDelete", skip_serializing_if = "Option::is_none")]
    pub may_delete: Option<bool>,
}
