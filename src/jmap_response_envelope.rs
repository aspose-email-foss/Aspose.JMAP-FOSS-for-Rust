use std::collections::HashMap;
use crate::invocation::Invocation;
use serde::{Deserialize, Serialize};

/// The JSON body returned from apiUrl. Deliberately NOT named bare "Response" for the same
/// reason JmapRequestEnvelope isn't named "Request" - see that object's description.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JmapResponseEnvelope {
    #[serde(rename = "methodResponses")]
    pub method_responses: Vec<Invocation>,

    #[serde(rename = "createdIds")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_ids: Option<HashMap<String, String>>,

    #[serde(rename = "sessionState")]
    pub session_state: String,
}
