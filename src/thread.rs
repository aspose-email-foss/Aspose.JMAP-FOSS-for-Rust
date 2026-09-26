use serde::{Deserialize, Serialize};

/// An ordered list of Email ids that make up a conversation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Thread {
    /// Server-assigned identifier of the thread.
    #[serde(rename = "id")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Ordered list of Email ids that belong to this thread.
    #[serde(rename = "emailIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email_ids: Option<Vec<String>>,
}
