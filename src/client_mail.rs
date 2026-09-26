//! JMAP mail module client extensions.

use std::collections::HashMap;

use serde::Deserialize;
use serde_json::{json, Value};

use crate::common_types::{JmapError, SetError};
use crate::comparator::Comparator;
use crate::email::Email;
use crate::email_query_response::EmailQueryResponse;
use crate::identity::Identity;
use crate::invocation::Invocation;
use crate::mailbox::Mailbox;

/// Internal response shapes for selected JMAP calls.
#[derive(Debug, Deserialize)]
struct MailboxGetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "list")]
    list: Vec<Mailbox>,
    #[serde(rename = "notFound")]
    not_found: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct MailboxSetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "oldState")]
    old_state: Option<String>,
    #[serde(rename = "newState")]
    new_state: String,
    // Per RFC 8620 5.3, a "created" entry is PARTIAL - it only carries
    // server-set properties, omitting ones (like `name`) the client already
    // submitted. Deserializing straight into `Mailbox` would fail on any real
    // server response because `Mailbox::name` is a required (non-Option)
    // field. Captured as raw JSON here and merged onto the client-submitted
    // object in `create_mailbox` before conversion to `Mailbox`.
    #[serde(rename = "created")]
    created: Option<HashMap<String, Value>>,
    #[serde(rename = "updated")]
    updated: Option<HashMap<String, Mailbox>>,
    #[serde(rename = "destroyed")]
    destroyed: Option<Vec<String>>,
    #[serde(rename = "notCreated")]
    not_created: Option<HashMap<String, SetError>>,
    #[serde(rename = "notUpdated")]
    not_updated: Option<HashMap<String, SetError>>,
    #[serde(rename = "notDestroyed")]
    not_destroyed: Option<HashMap<String, SetError>>,
}

/// Merges `overlay` onto `base` where both are JSON objects; `overlay`'s
/// fields win on conflict. Used to reconstruct a full object from a client-
/// submitted create payload plus the server's partial "created" response.
fn merge_json(base: Value, overlay: Value) -> Value {
    match (base, overlay) {
        (Value::Object(mut b), Value::Object(ov)) => {
            for (k, v) in ov {
                b.insert(k, v);
            }
            Value::Object(b)
        }
        (_, ov) => ov,
    }
}

#[derive(Debug, Deserialize)]
struct EmailGetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "list")]
    list: Vec<Email>,
    #[serde(rename = "notFound")]
    not_found: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmailSetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "oldState")]
    old_state: Option<String>,
    #[serde(rename = "newState")]
    new_state: String,
    #[serde(rename = "created")]
    created: Option<HashMap<String, Email>>,
    #[serde(rename = "updated")]
    updated: Option<HashMap<String, Email>>,
    #[serde(rename = "destroyed")]
    destroyed: Option<Vec<String>>,
    #[serde(rename = "notCreated")]
    not_created: Option<HashMap<String, SetError>>,
    #[serde(rename = "notUpdated")]
    not_updated: Option<HashMap<String, SetError>>,
    #[serde(rename = "notDestroyed")]
    not_destroyed: Option<HashMap<String, SetError>>,
}

#[derive(Debug, Deserialize)]
struct IdentityGetResponse {
    #[serde(rename = "accountId")]
    account_id: String,
    #[serde(rename = "state")]
    state: String,
    #[serde(rename = "list")]
    list: Vec<Identity>,
    #[serde(rename = "notFound")]
    not_found: Vec<String>,
}

/// Helper to deserialize an `Invocation`'s arguments into a concrete response type.
fn deserialize_invocation<T: serde::de::DeserializeOwned>(inv: Invocation) -> Result<T, JmapError> {
    let map: serde_json::Map<String, Value> = inv.arguments.into_iter().collect();
    serde_json::from_value(Value::Object(map)).map_err(|e| JmapError::InvalidResponse(e.to_string()))
}

impl crate::JmapClient {
    /// Retrieves all mailboxes for the given account.
    pub fn list_mailboxes(&mut self, account_id: &str) -> Result<Vec<Mailbox>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("ids".to_string(), Value::Null);

        let resp = self.send_request(
            vec![Invocation {
                name: "Mailbox/get".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let get_resp: MailboxGetResponse = deserialize_invocation(inv)?;
        Ok(get_resp.list)
    }

    /// Retrieves specific mailboxes by their ids.
    pub fn get_mailbox(&mut self, account_id: &str, ids: Vec<String>) -> Result<Vec<Mailbox>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("ids".to_string(), json!(ids));

        let resp = self.send_request(
            vec![Invocation {
                name: "Mailbox/get".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let get_resp: MailboxGetResponse = deserialize_invocation(inv)?;
        Ok(get_resp.list)
    }

    /// Creates one or more mailboxes.
    ///
    /// `create` is a map from client‑chosen temporary ids to `Mailbox` objects.
    /// Returns a map from those temporary ids to the server‑created `Mailbox` objects.
    pub fn create_mailbox(
        &mut self,
        account_id: &str,
        create: HashMap<String, Mailbox>,
    ) -> Result<HashMap<String, Mailbox>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        let create_json: HashMap<_, _> =
            create.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
        args.insert("create".to_string(), json!(create_json));

        let resp = self.send_request(
            vec![Invocation {
                name: "Mailbox/set".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let set_resp: MailboxSetResponse = deserialize_invocation(inv)?;

        let mut result = HashMap::new();
        if let Some(created) = set_resp.created {
            for (creation_id, server_value) in created {
                let submitted = create.get(&creation_id).ok_or_else(|| {
                    JmapError::InvalidResponse(format!(
                        "server returned unknown creation id {creation_id}"
                    ))
                })?;
                let merged = merge_json(json!(submitted), server_value);
                let mailbox: Mailbox = serde_json::from_value(merged)
                    .map_err(|e| JmapError::InvalidResponse(e.to_string()))?;
                result.insert(creation_id, mailbox);
            }
        }
        Ok(result)
    }

    /// Deletes the specified mailboxes.
    ///
    /// Returns the list of ids that were successfully destroyed.
    pub fn delete_mailbox(&mut self, account_id: &str, destroy: Vec<String>) -> Result<Vec<String>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("destroy".to_string(), json!(destroy));

        let resp = self.send_request(
            vec![Invocation {
                name: "Mailbox/set".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let set_resp: MailboxSetResponse = deserialize_invocation(inv)?;
        Ok(set_resp.destroyed.unwrap_or_default())
    }

    /// Lists messages matching the given filter.
    ///
    /// `filter` is a raw JSON object describing the search criteria.
    ///
    /// Returns the raw `Email/query` result (RFC 8620 section 5.5), including
    /// paging metadata such as `position` and `total`. Callers who need
    /// hydrated `Email` objects should pass the returned `ids` to
    /// `fetch_message`.
    pub fn list_messages(
        &mut self,
        account_id: &str,
        filter: Option<Value>,
        sort: Option<Vec<Comparator>>,
        limit: Option<u64>,
    ) -> Result<EmailQueryResponse, JmapError> {
        let mut query_args = HashMap::new();
        query_args.insert("accountId".to_string(), json!(account_id));
        if let Some(f) = filter {
            query_args.insert("filter".to_string(), f);
        }
        if let Some(s) = sort {
            query_args.insert("sort".to_string(), json!(s));
        }
        if let Some(l) = limit {
            query_args.insert("limit".to_string(), json!(l));
        }

        let query_resp = self.send_request(
            vec![Invocation {
                name: "Email/query".to_string(),
                arguments: query_args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let query_inv = query_resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;

        deserialize_invocation(query_inv)
    }

    /// Retrieves full message objects for the given ids.
    pub fn fetch_message(
        &mut self,
        account_id: &str,
        ids: Vec<String>,
        properties: Option<Vec<String>>,
    ) -> Result<Vec<Email>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("ids".to_string(), json!(ids));
        if let Some(p) = properties {
            args.insert("properties".to_string(), json!(p));
        }

        let resp = self.send_request(
            vec![Invocation {
                name: "Email/get".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let get_resp: EmailGetResponse = deserialize_invocation(inv)?;
        Ok(get_resp.list)
    }

    /// Moves messages to a different mailbox by updating their `mailboxIds`.
    ///
    /// `updates` maps message ids to a map of mailbox id → boolean.
    pub fn move_message(
        &mut self,
        account_id: &str,
        updates: HashMap<String, HashMap<String, bool>>,
    ) -> Result<HashMap<String, Email>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        let update_json: HashMap<_, _> = updates
            .into_iter()
            .map(|(id, mbids)| (id, json!({ "mailboxIds": mbids })))
            .collect();
        args.insert("update".to_string(), json!(update_json));

        let resp = self.send_request(
            vec![Invocation {
                name: "Email/set".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let set_resp: EmailSetResponse = deserialize_invocation(inv)?;
        Ok(set_resp.updated.unwrap_or_default())
    }

    /// Sets a keyword (e.g. `$seen`) on the given messages.
    ///
    /// `updates` maps message ids to a map of keyword → boolean.
    pub fn set_message_keyword(
        &mut self,
        account_id: &str,
        updates: HashMap<String, HashMap<String, bool>>,
    ) -> Result<HashMap<String, Email>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        let update_json: HashMap<_, _> = updates
            .into_iter()
            .map(|(id, kw)| (id, json!({ "keywords": kw })))
            .collect();
        args.insert("update".to_string(), json!(update_json));

        let resp = self.send_request(
            vec![Invocation {
                name: "Email/set".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let set_resp: EmailSetResponse = deserialize_invocation(inv)?;
        Ok(set_resp.updated.unwrap_or_default())
    }

    /// Deletes the specified messages.
    ///
    /// Returns the list of ids that were successfully destroyed.
    pub fn delete_message(&mut self, account_id: &str, destroy: Vec<String>) -> Result<Vec<String>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("destroy".to_string(), json!(destroy));

        let resp = self.send_request(
            vec![Invocation {
                name: "Email/set".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let set_resp: EmailSetResponse = deserialize_invocation(inv)?;
        Ok(set_resp.destroyed.unwrap_or_default())
    }

    /// Retrieves all identities for the given account.
    pub fn list_identities(&mut self, account_id: &str) -> Result<Vec<Identity>, JmapError> {
        let mut args = HashMap::new();
        args.insert("accountId".to_string(), json!(account_id));
        args.insert("ids".to_string(), Value::Null);

        let resp = self.send_request(
            vec![Invocation {
                name: "Identity/get".to_string(),
                arguments: args,
                method_call_id: "c1".to_string(),
            }],
            vec!["urn:ietf:params:jmap:mail".to_string()],
        )?;

        let inv = resp
            .method_responses
            .into_iter()
            .next()
            .ok_or_else(|| JmapError::InvalidResponse("no method response".to_string()))?;
        let get_resp: IdentityGetResponse = deserialize_invocation(inv)?;
        Ok(get_resp.list)
    }
}
