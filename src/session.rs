//! JMAP Session resource models.
//!
//! This module defines the data structures representing the JMAP Session object
//! and its related sub‑objects as described in RFC 8620.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The JMAP Session resource.
///
/// Represents the server's session information, fetched once from the well‑known
/// JMAP URL and cached by the client. It contains server capabilities, account
/// information, and URL templates for subsequent JMAP requests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Session {
    /// Map of capability URNs to capability objects.
    ///
    /// For the core capability (`urn:ietf:params:jmap:core`) the value can be
    /// deserialized into [`CoreCapability`]; other capabilities are represented
    /// as generic JSON objects.
    #[serde(rename = "capabilities")]
    pub capabilities: HashMap<String, Value>,

    /// Map of account IDs to account objects.
    #[serde(rename = "accounts")]
    pub accounts: HashMap<String, Account>,

    /// Map of capability URNs to the default account ID for that capability.
    #[serde(rename = "primaryAccounts")]
    pub primary_accounts: HashMap<String, String>,

    /// The authenticated username.
    #[serde(rename = "username")]
    pub username: String,

    /// URL for POSTing JMAP method‑call requests. May be relative.
    #[serde(rename = "apiUrl")]
    pub api_url: String,

    /// URL template for downloading blobs. May be relative.
    #[serde(rename = "downloadUrl")]
    pub download_url: String,

    /// URL template for uploading blobs. May be relative.
    #[serde(rename = "uploadUrl")]
    pub upload_url: String,

    /// URL template for the push EventSource stream. May be relative.
    #[serde(rename = "eventSourceUrl")]
    pub event_source_url: String,

    /// Opaque state string that changes whenever the Session object changes.
    #[serde(rename = "state")]
    pub state: String,
}

/// An account description within a JMAP Session.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Account {
    /// Human‑readable name of the account.
    #[serde(rename = "name")]
    pub name: String,

    /// Indicates whether the account is a personal (true) or shared (false) account.
    #[serde(rename = "isPersonal")]
    pub is_personal: bool,

    /// Indicates whether the account is read‑only.
    #[serde(rename = "isReadOnly")]
    pub is_read_only: bool,

    /// Map of capability URNs to capability‑specific information for this account.
    #[serde(rename = "accountCapabilities")]
    pub account_capabilities: HashMap<String, Value>,
}

/// Core capability object (value of `capabilities["urn:ietf:params:jmap:core"]`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CoreCapability {
    /// Maximum size of a single uploaded blob, in bytes.
    #[serde(rename = "maxSizeUpload")]
    pub max_size_upload: u64,

    /// Maximum number of concurrent uploads allowed.
    #[serde(rename = "maxConcurrentUpload")]
    pub max_concurrent_upload: u64,

    /// Maximum size of a single request body, in bytes.
    #[serde(rename = "maxSizeRequest")]
    pub max_size_request: u64,

    /// Maximum number of concurrent requests allowed.
    #[serde(rename = "maxConcurrentRequests")]
    pub max_concurrent_requests: u64,

    /// Maximum number of method calls allowed in a single request.
    #[serde(rename = "maxCallsInRequest")]
    pub max_calls_in_request: u64,

    /// Maximum number of objects that can be returned by a `get` method.
    #[serde(rename = "maxObjectsInGet")]
    pub max_objects_in_get: u64,

    /// Maximum number of objects that can be set by a `set` method.
    #[serde(rename = "maxObjectsInSet")]
    pub max_objects_in_set: u64,

    /// List of supported collation algorithm identifiers.
    #[serde(rename = "collationAlgorithms")]
    pub collation_algorithms: Vec<String>,
}
