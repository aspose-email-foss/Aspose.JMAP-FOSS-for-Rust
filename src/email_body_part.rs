use serde::{Deserialize, Serialize};
use crate::email_header::{EmailHeader};

/// One node of the Email's MIME bodyStructure tree.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailBodyPart {
    /// Part identifier, may be null.
    #[serde(rename = "partId", skip_serializing_if = "Option::is_none")]
    pub part_id: Option<String>,

    /// Blob identifier, may be null.
    #[serde(rename = "blobId", skip_serializing_if = "Option::is_none")]
    pub blob_id: Option<String>,

    /// Size in bytes.
    #[serde(rename = "size")]
    pub size: u64,

    /// List of headers.
    #[serde(rename = "headers")]
    pub headers: Vec<EmailHeader>,

    /// Filename, from Content‑Disposition or Content‑Type, may be null.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// MIME type, e.g. `"text/plain"`.
    #[serde(rename = "type")]
    pub type_: String,

    /// Character set, may be null.
    #[serde(rename = "charset", skip_serializing_if = "Option::is_none")]
    pub charset: Option<String>,

    /// Disposition, e.g. `"inline"` or `"attachment"`, may be null.
    #[serde(rename = "disposition", skip_serializing_if = "Option::is_none")]
    pub disposition: Option<String>,

    /// Content‑Id, may be null.
    #[serde(rename = "cid", skip_serializing_if = "Option::is_none")]
    pub cid: Option<String>,

    /// Languages, may be null.
    #[serde(rename = "language", skip_serializing_if = "Option::is_none")]
    pub language: Option<Vec<String>>,

    /// Location, may be null.
    #[serde(rename = "location", skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,

    /// Sub‑parts, may be null.
    #[serde(rename = "subParts", skip_serializing_if = "Option::is_none")]
    pub sub_parts: Option<Vec<EmailBodyPart>>,
}
