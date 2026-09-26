use serde::{Deserialize, Serialize};

/// One address in a header such as From/To/Cc (RFC 8621 section 4.1.2.3).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAddress {
    /// The display name of the address, or `null` if not present.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The email address (required).
    #[serde(rename = "email")]
    pub email: String,
}
