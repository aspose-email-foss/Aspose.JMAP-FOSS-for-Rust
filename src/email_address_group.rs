use crate::email_address::{EmailAddress};
use serde::{Deserialize, Serialize};

/// Group syntax as in From/To headers, e.g. `Team: a@x, b@x;`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmailAddressGroup {
    /// The name of the group, or `null` if none.
    #[serde(rename = "name", skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The email addresses belonging to the group.
    #[serde(rename = "addresses")]
    pub addresses: Vec<EmailAddress>,
}
