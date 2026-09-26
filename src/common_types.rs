//! Shared JMAP types and error hierarchy.

use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt;

/// Identifier used throughout JMAP objects.
///
/// Must be a string of 1 to 255 characters from the base64url alphabet
/// (`[A-Za-z0-9_-]`). Case‑sensitive and never re‑used for a different object
/// once assigned.
pub type Id = String;

/// Signed integer (JSON number without a fractional part).
pub type Int = i64;

/// Non‑negative integer (JSON number without a fractional part).
pub type UnsignedInt = u64;

/// RFC 3339 date‑time string, e.g. `2026-08-18T10:00:00Z`.
pub type Date = String;

/// RFC 3339 date‑time string with a zero UTC offset, e.g. `2026-08-18T10:00:00Z`.
pub type UTCDate = String;

/// Standard error shape returned per‑id in `notCreated`, `notUpdated` or
/// `notDestroyed` responses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetError {
    /// Machine‑readable error type, e.g. `invalidProperties`, `notFound`,
    /// `forbidden`, `tooLarge`.
    #[serde(rename = "type")]
    pub error_type: String,

    /// Human‑readable description, if any.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// List of property names that caused the error, if applicable.
    #[serde(rename = "properties", skip_serializing_if = "Option::is_none")]
    pub properties: Option<Vec<String>>,
}

/// Standard top‑level method‑call error, returned as an `error` method response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MethodError {
    /// Machine‑readable error type, e.g. `unknownMethod`, `invalidArguments`,
    /// `accountNotFound`, `serverFail`.
    #[serde(rename = "type")]
    pub error_type: String,

    /// Human‑readable description, if any.
    #[serde(rename = "description", skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Back‑reference used inside a request argument to point at a value produced by
/// an earlier method call in the same request.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResultReference {
    /// Identifier of the earlier method call.
    #[serde(rename = "resultOf")]
    pub result_of: String,

    /// Name of the property within that method's result.
    #[serde(rename = "name")]
    pub name: String,

    /// JSON Pointer into the referenced result.
    #[serde(rename = "path")]
    pub path: String,
}

/// Errors that can be returned by the JMAP client.
///
/// The library never panics for ordinary error conditions; all such
/// situations are represented by this enum.
#[derive(Debug, Clone)]
pub enum JmapError {
    /// Network‑level failure (e.g. DNS error, connection refused, timeout).
    Network(String),

    /// Protocol‑level error returned by the JMAP server.
    Protocol {
        /// The `type` field from the server's error object.
        error_type: String,
        /// Human‑readable description, if any.
        description: String,
    },

    /// The server response could not be parsed or did not conform to the
    /// expected schema.
    InvalidResponse(String),
}

impl fmt::Display for JmapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            JmapError::Network(msg) => write!(f, "network error: {}", msg),
            JmapError::Protocol { error_type, description } => {
                write!(f, "protocol error ({}): {}", error_type, description)
            }
            JmapError::InvalidResponse(msg) => write!(f, "invalid response: {}", msg),
        }
    }
}

impl Error for JmapError {}

impl From<ureq::Error> for JmapError {
    fn from(err: ureq::Error) -> Self {
        JmapError::Network(err.to_string())
    }
}
