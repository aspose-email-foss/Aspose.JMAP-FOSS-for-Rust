use serde::{Deserialize, Serialize};

/// Per-recipient delivery outcome, keyed by recipient email address on
/// `EmailSubmission.deliveryStatus`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DeliveryStatus {
    /// The SMTP reply string from the server.
    #[serde(rename = "smtpReply")]
    pub smtp_reply: String,

    /// Delivery status: one of `"queued"`, `"yes"`, `"no"`, or `"unknown"`.
    #[serde(rename = "delivered")]
    pub delivered: String,

    /// Display status: either `"unknown"` or `"yes"`.
    #[serde(rename = "displayed")]
    pub displayed: String,
}
