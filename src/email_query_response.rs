use serde::{Deserialize, Serialize};

/// Result of an `Email/query` call (RFC 8620 section 5.5).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailQueryResponse {
    /// The id of the account used for the call.
    #[serde(rename = "accountId")]
    pub account_id: String,

    /// A string encoding the current state of the query on the server.
    #[serde(rename = "queryState")]
    pub query_state: String,

    /// Whether the server supports calculating changes to the query result set.
    #[serde(rename = "canCalculateChanges")]
    pub can_calculate_changes: bool,

    /// The zero-based index of the first result in `ids` within the full query result set.
    #[serde(rename = "position")]
    pub position: u64,

    /// The list of email ids matching the query, in the requested sort order.
    #[serde(rename = "ids")]
    pub ids: Vec<String>,

    /// The total number of results matching the filter, if requested.
    #[serde(rename = "total")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,

    /// The limit applied when running the query, if any.
    #[serde(rename = "limit")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,
}
