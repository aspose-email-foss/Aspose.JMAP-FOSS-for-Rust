use serde::{Deserialize, Serialize};

/// Highlighted subject/preview snippet for an Email id matched by a filter's text search.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SearchSnippet {
    /// The identifier of the email this snippet belongs to.
    #[serde(rename = "emailId")]
    pub email_id: String,

    /// May contain `<mark></mark>` tags around matches.
    #[serde(rename = "subject")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,

    /// Preview text snippet.
    #[serde(rename = "preview")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}
