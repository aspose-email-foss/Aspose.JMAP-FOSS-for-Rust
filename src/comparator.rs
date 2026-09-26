use serde::{Deserialize, Serialize};

fn default_is_ascending() -> bool {
    true
}

fn is_default_ascending(v: &bool) -> bool {
    *v
}

/// One entry of an Email/query `sort` argument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Comparator {
    /// The property to sort by, e.g. `receivedAt`, `from`, `subject`, `size`.
    #[serde(rename = "property")]
    pub property: String,

    /// Determines whether the sort is ascending. Defaults to `true`.
    #[serde(
        rename = "isAscending",
        default = "default_is_ascending",
        skip_serializing_if = "is_default_ascending"
    )]
    pub is_ascending: bool,

    /// Optional collation identifier.
    #[serde(rename = "collation", skip_serializing_if = "Option::is_none")]
    pub collation: Option<String>,
}
