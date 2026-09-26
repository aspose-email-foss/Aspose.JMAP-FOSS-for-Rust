use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SMTP MAIL FROM / RCPT TO envelope for a submission, distinct from the message's own From/To headers.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Envelope {
    /// The SMTP MAIL FROM address.
    #[serde(rename = "mailFrom")]
    pub mail_from: Address,

    /// The SMTP RCPT TO addresses.
    #[serde(rename = "rcptTo")]
    pub rcpt_to: Vec<Address>,
}

/// Address
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Address {
    /// The email address.
    #[serde(rename = "email")]
    pub email: String,

    /// SMTP MAIL/RCPT parameters, e.g. `{"RET": "HDRS"}`.
    #[serde(
        rename = "parameters",
        skip_serializing_if = "Option::is_none"
    )]
    pub parameters: Option<HashMap<String, Option<String>>>,
}
