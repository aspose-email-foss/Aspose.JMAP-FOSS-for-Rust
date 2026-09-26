use std::collections::HashMap;
use std::iter::FromIterator;

use crate::common_types::{JmapError, SetError};
use crate::email_submission::EmailSubmission;
use crate::invocation::Invocation;
use crate::JmapClient;

use serde::Deserialize;
use serde_json::{json, Map, Value};

/// Response for `EmailSubmission/get`.
#[derive(Debug, Deserialize)]
struct EmailSubmissionGetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "list")]
    list: Vec<EmailSubmission>,
    #[serde(rename = "notFound")]
    not_found: Option<Vec<String>>,
}

/// Response for `EmailSubmission/set`.
#[derive(Debug, Deserialize)]
struct EmailSubmissionSetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "oldState")]
    old_state: Option<String>,
    #[serde(rename = "newState")]
    new_state: String,
    // Per RFC 8620 5.3, a "created" entry is PARTIAL and omits fields (like
    // identityId/emailId) already known from the client's create payload.
    // EmailSubmission::identity_id/email_id are required (non-Option), so
    // deserializing this straight into EmailSubmission would fail on any
    // real server response. Captured as raw JSON and merged onto the
    // client-submitted object in `send` before conversion.
    #[serde(rename = "created")]
    created: Option<HashMap<String, Value>>,
    #[serde(rename = "updated")]
    updated: Option<HashMap<String, Option<EmailSubmission>>>,
    #[serde(rename = "destroyed")]
    destroyed: Option<Vec<String>>,
    #[serde(rename = "notCreated")]
    not_created: Option<HashMap<String, SetError>>,
    #[serde(rename = "notUpdated")]
    not_updated: Option<HashMap<String, SetError>>,
    #[serde(rename = "notDestroyed")]
    not_destroyed: Option<HashMap<String, SetError>>,
}

/// Response for `EmailSubmission/query`.
#[derive(Debug, Deserialize)]
struct EmailSubmissionQueryResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "queryState")]
    query_state: String,
    #[serde(rename = "canCalculateChanges")]
    can_calculate_changes: bool,
    #[serde(rename = "position")]
    position: u64,
    #[serde(rename = "ids")]
    ids: Vec<String>,
    #[serde(rename = "total")]
    total: Option<u64>,
}

/// Merge two JSON `Value`s where both are objects. Fields from `overlay` win.
fn merge_json(original: Value, overlay: Value) -> Value {
    match (original, overlay) {
        (Value::Object(mut o), Value::Object(ov)) => {
            for (k, v) in ov {
                o.insert(k, v);
            }
            Value::Object(o)
        }
        (_, ov) => ov,
    }
}

impl JmapClient {
    /// Sends (creates) an `EmailSubmission`.
    ///
    /// The supplied `submission` is used as the creation payload. The server‑assigned
    /// fields (e.g. `id`, `sendAt`) are merged onto the original `submission` and
    /// returned.
    pub fn send(
        &mut self,
        account_id: &str,
        submission: EmailSubmission,
    ) -> Result<EmailSubmission, JmapError> {
        // Serialize the creation object.
        let create_val = serde_json::to_value(&submission)
            .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;

        // Build arguments.
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("create".to_string(), json!({ "c1": create_val }));

        let invocation = Invocation {
            name: "EmailSubmission/set".to_string(),
            arguments: args,
            method_call_id: "c1".to_string(),
        };

        let resp_env = self.send_request(
            vec![invocation],
            vec!["urn:ietf:params:jmap:submission".to_string()],
        )?;
        let method_resp = resp_env
            .method_responses
            .get(0)
            .ok_or_else(|| JmapError::InvalidResponse("missing method response".into()))?;

        if method_resp.name != "EmailSubmission/set" {
            return Err(JmapError::InvalidResponse(format!(
                "unexpected method response: {}",
                method_resp.name
            )));
        }

        // Convert the arguments map into a serde_json::Value::Object.
        let args_value = Value::Object(Map::from_iter(
            method_resp.arguments.clone().into_iter(),
        ));
        let set_resp: EmailSubmissionSetResponse = serde_json::from_value(args_value)
            .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;

        // Successful creation.
        if let Some(created_map) = set_resp.created {
            if let Some(created_val) = created_map.get("c1") {
                // Merge the server's partial "created" fields onto the original
                // (client-submitted) submission - the server value is raw JSON,
                // not yet deserialized into EmailSubmission, since it may omit
                // required fields like identityId/emailId that only the
                // client-submitted object has.
                let orig_val = serde_json::to_value(&submission)
                    .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;
                let merged = merge_json(orig_val, created_val.clone());
                let result: EmailSubmission = serde_json::from_value(merged)
                    .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;
                return Ok(result);
            }
        }

        // Creation failed – surface the first SetError as a protocol error.
        if let Some(not_created) = set_resp.not_created {
            if let Some(err) = not_created.values().next() {
                return Err(JmapError::Protocol {
                    error_type: err.error_type.clone(),
                    description: err
                        .description
                        .clone()
                        .unwrap_or_else(|| "submission not created".into()),
                });
            }
        }

        Err(JmapError::InvalidResponse(
            "submission creation response missing".into(),
        ))
    }

    /// Cancels a previously sent `EmailSubmission` by setting its `undoStatus` to `"canceled"`.
    pub fn cancel_send(
        &mut self,
        account_id: &str,
        submission_id: &str,
    ) -> Result<EmailSubmission, JmapError> {
        // Build the patch object. A PatchObject key is a JSON Pointer (RFC 6901) relative
        // to the object being patched - a bare top-level property name has no leading
        // slash; "/undoStatus" would instead point at a property literally named the
        // empty string, which a real JMAP server rejects/ignores.
        let patch = json!({ "undoStatus": "canceled" });

        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("update".to_string(), json!({ submission_id: patch }));

        let invocation = Invocation {
            name: "EmailSubmission/set".to_string(),
            arguments: args,
            method_call_id: "c1".to_string(),
        };

        let resp_env = self.send_request(
            vec![invocation],
            vec!["urn:ietf:params:jmap:submission".to_string()],
        )?;
        let method_resp = resp_env
            .method_responses
            .get(0)
            .ok_or_else(|| JmapError::InvalidResponse("missing method response".into()))?;

        if method_resp.name != "EmailSubmission/set" {
            return Err(JmapError::InvalidResponse(format!(
                "unexpected method response: {}",
                method_resp.name
            )));
        }

        let args_value = Value::Object(Map::from_iter(
            method_resp.arguments.clone().into_iter(),
        ));
        let set_resp: EmailSubmissionSetResponse = serde_json::from_value(args_value)
            .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;

        // Successful update.
        if let Some(updated_map) = set_resp.updated {
            if let Some(Some(updated)) = updated_map.get(submission_id) {
                return Ok(updated.clone());
            }
        }

        // Update failed – surface the SetError.
        if let Some(not_updated) = set_resp.not_updated {
            if let Some(err) = not_updated.get(submission_id) {
                return Err(JmapError::Protocol {
                    error_type: err.error_type.clone(),
                    description: err
                        .description
                        .clone()
                        .unwrap_or_else(|| "submission not updated".into()),
                });
            }
        }

        Err(JmapError::InvalidResponse(
            "submission cancel response missing".into(),
        ))
    }

    /// Lists all `EmailSubmission` objects for the given account.
    ///
    /// This performs a `EmailSubmission/query` to obtain the ids and then a
    /// `EmailSubmission/get` to retrieve the full objects.
    pub fn list_submissions(
        &mut self,
        account_id: &str,
    ) -> Result<Vec<EmailSubmission>, JmapError> {
        // Query for all submission ids.
        let mut query_args = HashMap::new();
        query_args.insert("accountId".to_string(), json!(account_id));

        let query_inv = Invocation {
            name: "EmailSubmission/query".to_string(),
            arguments: query_args,
            method_call_id: "c1".to_string(),
        };

        let query_env = self.send_request(
            vec![query_inv],
            vec!["urn:ietf:params:jmap:submission".to_string()],
        )?;
        let query_resp_inv = query_env
            .method_responses
            .get(0)
            .ok_or_else(|| JmapError::InvalidResponse("missing query response".into()))?;

        if query_resp_inv.name != "EmailSubmission/query" {
            return Err(JmapError::InvalidResponse(format!(
                "unexpected method response: {}",
                query_resp_inv.name
            )));
        }

        let query_args_value = Value::Object(Map::from_iter(
            query_resp_inv.arguments.clone().into_iter(),
        ));
        let query_resp: EmailSubmissionQueryResponse = serde_json::from_value(query_args_value)
            .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;

        if query_resp.ids.is_empty() {
            return Ok(Vec::new());
        }

        // Fetch the full objects.
        let mut get_args = HashMap::new();
        get_args.insert("accountId".to_string(), json!(account_id));
        get_args.insert("ids".to_string(), json!(query_resp.ids));

        let get_inv = Invocation {
            name: "EmailSubmission/get".to_string(),
            arguments: get_args,
            method_call_id: "c1".to_string(),
        };

        let get_env = self.send_request(
            vec![get_inv],
            vec!["urn:ietf:params:jmap:submission".to_string()],
        )?;
        let get_resp_inv = get_env
            .method_responses
            .get(0)
            .ok_or_else(|| JmapError::InvalidResponse("missing get response".into()))?;

        if get_resp_inv.name != "EmailSubmission/get" {
            return Err(JmapError::InvalidResponse(format!(
                "unexpected method response: {}",
                get_resp_inv.name
            )));
        }

        let get_args_value = Value::Object(Map::from_iter(
            get_resp_inv.arguments.clone().into_iter(),
        ));
        let get_resp: EmailSubmissionGetResponse = serde_json::from_value(get_args_value)
            .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;

        Ok(get_resp.list)
    }
}
