use serde::{Serialize, Deserialize};

/// Represents an email header as defined by JMAP.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailHeader {
    /// The name of the header.
    #[serde(rename = "name")]
    pub name: String,
    /// The value of the header.
    #[serde(rename = "value")]
    pub value: String,
}
