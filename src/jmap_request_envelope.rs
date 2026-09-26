use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::invocation::Invocation;

/// The JSON envelope sent to the JMAP server's `apiUrl`.
///
/// It contains the capabilities used, the list of method calls, and an optional map of
/// client‑generated IDs to server‑assigned IDs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JmapRequestEnvelope {
    /// Capability URNs this request depends on; must include `urn:ietf:params:jmap:core`.
    #[serde(rename = "using")]
    pub using: Vec<String>,

    /// The method calls to invoke.
    #[serde(rename = "methodCalls")]
    pub method_calls: Vec<Invocation>,

    /// Optional map of client‑generated IDs to server‑assigned IDs.
    #[serde(
        rename = "createdIds",
        skip_serializing_if = "Option::is_none"
    )]
    pub created_ids: Option<HashMap<String, String>>,
}
