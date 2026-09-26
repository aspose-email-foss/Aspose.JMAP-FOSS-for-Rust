//! Core client implementation for Aspose JMAP FOSS.

use std::collections::HashMap;

use crate::common_types::{JmapError, JmapError::Protocol};
use crate::invocation::Invocation;
use crate::jmap_request_envelope::JmapRequestEnvelope;
use crate::jmap_response_envelope::JmapResponseEnvelope;
use crate::session::Session;
use crate::transport::{Transport, UreqTransport, HttpRequest};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};
use serde_json::Value;

/// RFC 3986 unreserved characters (beyond alphanumerics) are left un-encoded, matching the
/// behavior of the other language targets' URL-encoding calls (e.g. Python's
/// urllib.parse.quote, JS's encodeURIComponent) - only `-`, `.`, `_`, `~` need this carve-out.
const URL_SEGMENT_ENCODE_SET: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

/// Percent-encodes a value substituted into a URL template placeholder (e.g. `{accountId}`).
/// These values are spliced verbatim into a URL path segment, and `/`, `?`, `#`, etc. in
/// them would otherwise rewrite the request path or smuggle extra query parameters into the
/// request.
fn encode_url_segment(value: &str) -> String {
    utf8_percent_encode(value, URL_SEGMENT_ENCODE_SET).to_string()
}

/// Configuration options for creating a [`JmapClient`].
#[derive(Debug, Clone)]
pub struct ClientOptions {
    /// URL of the JMAP session resource (e.g. `https://example.com/.well-known/jmap`).
    pub session_url: String,
    /// Username for HTTP Basic authentication.
    pub username: String,
    /// Password for HTTP Basic authentication.
    pub password: String,
    /// Optional OAuth 2.0 bearer token (RFC 6750). When set to a non-empty
    /// string, every request authenticates with `Authorization: Bearer
    /// <token>` instead of HTTP Basic auth, and `username`/`password` are
    /// ignored. `None` (or `Some("")`, treated the same as unset) falls back
    /// to HTTP Basic auth using `username`/`password`.
    pub bearer_token: Option<String>,
    /// Optional custom transport; `None` selects the default `UreqTransport`.
    pub transport: Option<Box<dyn Transport>>,
}

/// Primary JMAP client.
///
/// The client holds the session information and URLs required for subsequent
/// JMAP method calls. Fields that need to be accessed by other client modules
/// are `pub(crate)`.
#[derive(Debug, Clone)]
pub struct JmapClient {
    options: ClientOptions,
    pub transport: Box<dyn Transport>,
    pub(crate) session: Option<Session>,
    pub(crate) api_url: Option<String>,
    pub(crate) upload_url: Option<String>,
    pub(crate) download_url: Option<String>,
    pub(crate) event_source_url: Option<String>,
}

impl JmapClient {
    /// Creates a new client from the supplied options.
    pub fn new(mut options: ClientOptions) -> Self {
        let transport: Box<dyn Transport> = options
            .transport
            .take()
            .unwrap_or_else(|| Box::new(UreqTransport::default()));
        Self {
            options,
            transport,
            session: None,
            api_url: None,
            upload_url: None,
            download_url: None,
            event_source_url: None,
        }
    }

    /// Connects to the JMAP server, fetching the session resource.
    ///
    /// The returned [`Session`] is also stored internally for later use.
    pub fn connect(&mut self) -> Result<Session, JmapError> {
        let req = HttpRequest {
            method: "GET".to_string(),
            url: self.options.session_url.clone(),
            headers: self.auth_headers(),
            body: None,
        };
        let resp = self.transport.send(&req)?;
        let session: Session = serde_json::from_slice(&resp.body).map_err(|e| Protocol {
            error_type: "invalidResponse".to_string(),
            description: e.to_string(),
        })?;

        // Resolve URLs against the session URL's origin.
        let base = Self::origin(&self.options.session_url);
        let api_url = Self::resolve(&base, &session.api_url);
        let upload_url = Self::resolve(&base, &session.upload_url);
        let download_url = Self::resolve(&base, &session.download_url);
        let event_source_url = Self::resolve(&base, &session.event_source_url);

        self.session = Some(session.clone());
        self.api_url = Some(api_url);
        self.upload_url = Some(upload_url);
        self.download_url = Some(download_url);
        self.event_source_url = Some(event_source_url);

        Ok(session)
    }

    /// Calls the `Core/echo` method, returning the server‑echoed arguments.
    pub fn echo(&mut self, arguments: Value) -> Result<Value, JmapError> {
        let args_map = match arguments {
            Value::Object(map) => map.into_iter().collect(),
            _ => {
                return Err(Protocol {
                    error_type: "invalidArgument".to_string(),
                    description: "echo arguments must be a JSON object".to_string(),
                })
            }
        };
        let invocation = Invocation {
            name: "Core/echo".to_string(),
            arguments: args_map,
            method_call_id: "c1".to_string(),
        };
        let resp = self.send_request(
            vec![invocation],
            vec!["urn:ietf:params:jmap:core".to_string()],
        )?;
        for inv in resp.method_responses {
            if inv.name == "Core/echo" && inv.method_call_id == "c1" {
                return Ok(Value::Object(inv.arguments.into_iter().collect()));
            }
        }
        Err(Protocol {
            error_type: "missingResponse".to_string(),
            description: "echo response not found".to_string(),
        })
    }

    /// Uploads a binary blob to the server.
    ///
    /// Returns the JSON response object containing `accountId`, `blobId`, `type`,
    /// and `size`.
    pub fn upload_blob(
        &mut self,
        account_id: &str,
        content_type: &str,
        data: &[u8],
    ) -> Result<Value, JmapError> {
        let upload_url_template = self
            .upload_url
            .as_ref()
            .ok_or_else(|| Protocol {
                error_type: "missingUrl".to_string(),
                description: "upload URL not set".to_string(),
            })?;
        let url = upload_url_template.replace("{accountId}", &encode_url_segment(account_id));

        let req = HttpRequest {
            method: "POST".to_string(),
            url,
            headers: {
                let mut h = self.auth_headers();
                h.insert("Content-Type".to_string(), content_type.to_string());
                h
            },
            body: Some(data.to_vec()),
        };
        let resp = self.transport.send(&req)?;
        let json: Value = serde_json::from_slice(&resp.body).map_err(|e| Protocol {
            error_type: "invalidResponse".to_string(),
            description: e.to_string(),
        })?;
        Ok(json)
    }

    /// Downloads a previously uploaded blob.
    ///
    /// Returns the raw bytes of the blob.
    pub fn download_blob(
        &mut self,
        account_id: &str,
        blob_id: &str,
        mime_type: &str,
        name: Option<&str>,
    ) -> Result<Vec<u8>, JmapError> {
        let download_url_template = self
            .download_url
            .as_ref()
            .ok_or_else(|| Protocol {
                error_type: "missingUrl".to_string(),
                description: "download URL not set".to_string(),
            })?;
        let mut url = download_url_template
            .replace("{accountId}", &encode_url_segment(account_id))
            .replace("{blobId}", &encode_url_segment(blob_id))
            .replace("{type}", &encode_url_segment(mime_type));
        if let Some(n) = name {
            url = url.replace("{name}", &encode_url_segment(n));
        }

        let req = HttpRequest {
            method: "GET".to_string(),
            url,
            headers: self.auth_headers(),
            body: None,
        };
        let resp = self.transport.send(&req)?;
        Ok(resp.body)
    }

    /// Sends a JMAP request envelope containing the given method calls.
    ///
    /// The `using` list must contain the capability URNs required by the calls.
    pub fn send_request(
        &mut self,
        method_calls: Vec<Invocation>,
        using: Vec<String>,
    ) -> Result<JmapResponseEnvelope, JmapError> {
        let api_url = self
            .api_url
            .as_ref()
            .ok_or_else(|| Protocol {
                error_type: "missingUrl".to_string(),
                description: "API URL not set".to_string(),
            })?;
        let envelope = JmapRequestEnvelope {
            using,
            method_calls,
            created_ids: None,
        };
        let body = serde_json::to_vec(&envelope).map_err(|e| Protocol {
            error_type: "serializationError".to_string(),
            description: e.to_string(),
        })?;

        let req = HttpRequest {
            method: "POST".to_string(),
            url: api_url.clone(),
            headers: {
                let mut h = self.auth_headers();
                h.insert("Content-Type".to_string(), "application/json".to_string());
                h
            },
            body: Some(body),
        };
        let resp = self.transport.send(&req)?;
        let envelope: JmapResponseEnvelope = serde_json::from_slice(&resp.body).map_err(|e| Protocol {
            error_type: "invalidResponse".to_string(),
            description: e.to_string(),
        })?;

        // Detect protocol‑level error responses.
        for inv in &envelope.method_responses {
            if inv.name == "error" {
                let err_type = inv
                    .arguments
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let description = inv
                    .arguments
                    .get("description")
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string();
                return Err(Protocol {
                    error_type: err_type,
                    description,
                });
            }
        }

        Ok(envelope)
    }

    /// Builds a header map containing a single `Authorization` entry for
    /// every request.
    ///
    /// If a non-empty `bearer_token` is configured, the header is
    /// `Authorization: Bearer <token>` per the OAuth 2.0 Bearer Token usage
    /// scheme (RFC 6750). Otherwise it falls back to
    /// `Authorization: Basic ...` built from the configured
    /// username/password, per the HTTP Basic auth scheme (RFC 7617) that
    /// JMAP servers expect on every request.
    fn auth_headers(&self) -> HashMap<String, String> {
        let mut h = HashMap::new();
        let value = match self.options.bearer_token.as_deref() {
            Some(token) if !token.is_empty() => format!("Bearer {}", token),
            _ => {
                let credentials = format!("{}:{}", self.options.username, self.options.password);
                format!("Basic {}", base64_encode(credentials.as_bytes()))
            }
        };
        h.insert("Authorization".to_string(), value);
        h
    }

    /// Returns the scheme + authority part of a URL (e.g. `https://example.com`).
    fn origin(url: &str) -> String {
        if let Some(pos) = url.find("://") {
            let after = &url[pos + 3..];
            if let Some(slash) = after.find('/') {
                return format!("{}://{}", &url[..pos], &after[..slash]);
            }
            return url.to_string();
        }
        url.to_string()
    }

    /// Resolves a possibly‑relative URL against the given base origin.
    fn resolve(base: &str, maybe_rel: &str) -> String {
        if maybe_rel.starts_with("http://") || maybe_rel.starts_with("https://") {
            maybe_rel.to_string()
        } else if maybe_rel.starts_with('/') {
            format!("{}{}", base, maybe_rel)
        } else {
            format!("{}/{}", base, maybe_rel)
        }
    }
}

/// Standard (RFC 4648) base64 encoding, hand-rolled since `allowed_dependencies`
/// permits only serde/serde_json/ureq - no third-party base64 crate.
fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity((input.len() + 2) / 3 * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);

        out.push(ALPHABET[(b0 >> 2) as usize] as char);
        out.push(ALPHABET[(((b0 & 0x03) << 4) | (b1 >> 4)) as usize] as char);
        out.push(if chunk.len() > 1 {
            ALPHABET[(((b1 & 0x0f) << 2) | (b2 >> 6)) as usize] as char
        } else {
            '='
        });
        out.push(if chunk.len() > 2 {
            ALPHABET[(b2 & 0x3f) as usize] as char
        } else {
            '='
        });
    }
    out
}
