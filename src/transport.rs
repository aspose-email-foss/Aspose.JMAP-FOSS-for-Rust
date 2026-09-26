//! Transport seam for the JMAP client.
//!
//! This module defines the low‑level HTTP request/response types, the `Transport`
//! trait that abstracts over the actual HTTP implementation, and the default
//! implementation based on the `ureq` crate.

use std::collections::HashMap;
use std::fmt::Debug;
use std::io::Read;

use crate::common_types::{JmapError, JmapError::Network, JmapError::Protocol};

/// A low‑level HTTP request used by the transport layer.
///
/// The fields correspond directly to the wire format required by JMAP
/// endpoints.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HttpRequest {
    /// HTTP method, e.g. `"POST"`.
    pub method: String,
    /// Full request URL.
    pub url: String,
    /// Header map where each key/value is a string.
    pub headers: HashMap<String, String>,
    /// Optional raw request body bytes. Deliberately bytes, not a string: a string body
    /// would force every caller - including blob uploads, which are not text - through a
    /// lossy UTF-8 round trip. JMAP method-call bodies are UTF-8-encoded JSON text; blob
    /// uploads are the blob's raw bytes verbatim.
    pub body: Option<Vec<u8>>,
}

/// A low‑level HTTP response returned by a transport implementation.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct HttpResponse {
    /// Numeric HTTP status code (e.g. `200`).
    pub status_code: u16,
    /// Header map.
    pub headers: HashMap<String, String>,
    /// Raw response body bytes (typically JSON text, but a blob download's body is
    /// arbitrary binary data - see the note on [`HttpRequest::body`]).
    pub body: Vec<u8>,
}

/// Trait representing an HTTP transport used by the JMAP client.
///
/// Implementations translate a generic [`HttpRequest`] into a concrete HTTP
/// request, perform the I/O, and return a [`HttpResponse`] on success. Network‑
/// level failures are reported as [`JmapError::Network`]; protocol‑level
/// problems (e.g. unexpected status codes) are reported as
/// [`JmapError::Protocol`].
///
/// The trait is required to be `Debug` and cloneable so that it can be stored
/// inside `JmapClient`, which derives `Debug` and `Clone`.
pub trait Transport: Debug {
    /// Sends the given request and returns the server's response.
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError>;

    /// Clone the boxed transport. This method enables `Box<dyn Transport>` to
    /// implement `Clone`.
    fn box_clone(&self) -> Box<dyn Transport>;

    /// Returns `self` as `&dyn Any`, so tests can `downcast_ref` a `Box<dyn Transport>`
    /// back to their own fake implementation (e.g. to inspect captured requests).
    fn as_any(&self) -> &dyn std::any::Any;
}

impl Clone for Box<dyn Transport> {
    fn clone(&self) -> Box<dyn Transport> {
        self.box_clone()
    }
}

/// Default transport implementation based on the `ureq` crate.
#[derive(Debug, Clone)]
pub struct UreqTransport {
    agent: ureq::Agent,
}

impl UreqTransport {
    /// Creates a new `UreqTransport` with a fresh `ureq::Agent`.
    ///
    /// Automatic redirect-following is disabled (`redirects(0)`): ureq drops
    /// the `Authorization` header on any redirect it follows itself, and real
    /// JMAP servers (e.g. Stalwart) 307-redirect `/.well-known/jmap` to their
    /// actual session endpoint - so an auto-followed redirect silently lands
    /// on an unauthenticated response instead of erroring. `send` below
    /// implements its own redirect loop that re-sends every original header,
    /// including `Authorization`, on each SAME-ORIGIN hop - but drops the
    /// credential headers (`Authorization` / `Cookie` / `Proxy-Authorization`)
    /// once a redirect crosses to a different origin, so a hostile redirect
    /// cannot leak them to another host.
    pub fn new() -> Self {
        Self {
            agent: ureq::AgentBuilder::new().redirects(0).build(),
        }
    }

    /// scheme://host[:port] prefix of an absolute URL, or the whole string if it has no path.
    fn origin_of(url: &str) -> &str {
        match url.find("://") {
            Some(pos) => match url[pos + 3..].find('/') {
                Some(slash) => &url[..pos + 3 + slash],
                None => url,
            },
            None => url,
        }
    }

    /// Resolves a `Location` header value against the origin of `base_url`.
    fn resolve_location(base_url: &str, location: &str) -> String {
        if location.starts_with("http://") || location.starts_with("https://") {
            return location.to_string();
        }
        let origin = Self::origin_of(base_url);
        if location.starts_with('/') {
            format!("{origin}{location}")
        } else {
            format!("{origin}/{location}")
        }
    }

    /// `true` if a redirect header (`Authorization` etc.) must not be forwarded to `url`.
    fn is_credential_header(name: &str) -> bool {
        let n = name.to_ascii_lowercase();
        n == "authorization" || n == "cookie" || n == "proxy-authorization"
    }

    fn send_once(
        &self,
        method: &str,
        url: &str,
        req: &HttpRequest,
        strip_credentials: bool,
    ) -> Result<ureq::Response, ureq::Error> {
        let mut builder = self.agent.request(method, url);
        for (name, value) in &req.headers {
            if strip_credentials && Self::is_credential_header(name) {
                continue;
            }
            builder = builder.set(name, value);
        }
        if let Some(body) = &req.body {
            builder.send_bytes(body)
        } else {
            builder.call()
        }
    }

    /// Builds a `Protocol` error for a non-2xx HTTP status, including an excerpt of the
    /// response body so a failed request can be debugged from more than the bare status code.
    fn describe_http_error(status_code: u16, body: &[u8]) -> JmapError {
        const MAX_ERROR_BODY_EXCERPT: usize = 2000;
        let text = String::from_utf8_lossy(body);
        let text = text.trim();
        let description = if text.is_empty() {
            format!("Unexpected HTTP status {}", status_code)
        } else if text.chars().count() > MAX_ERROR_BODY_EXCERPT {
            // Truncate by char count, not byte index, so a multi-byte UTF-8 character
            // straddling the cutoff can't panic on a non-char-boundary slice.
            let excerpt: String = text.chars().take(MAX_ERROR_BODY_EXCERPT).collect();
            format!("Unexpected HTTP status {}: {}...", status_code, excerpt)
        } else {
            format!("Unexpected HTTP status {}: {}", status_code, text)
        };
        Protocol {
            error_type: "httpError".to_string(),
            description,
        }
    }
}

impl Default for UreqTransport {
    fn default() -> Self {
        Self::new()
    }
}

const MAX_REDIRECTS: u8 = 5;

impl Transport for UreqTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError> {
        let start_origin = Self::origin_of(&req.url).to_string();
        let mut url = req.url.clone();
        // Once a redirect crosses to another origin, the caller's credentials
        // (Authorization / Cookie / Proxy-Authorization) must not be forwarded to it.
        let mut strip_credentials = false;
        let mut response = None;

        for _ in 0..=MAX_REDIRECTS {
            let result = self.send_once(&req.method, &url, req, strip_credentials);
            match result {
                // ureq treats any status below 400 as Ok, including 3xx
                // (redirects(0) only stops it from auto-following them) - so
                // a redirect surfaces here, not as an Err.
                Ok(resp) if (300..400).contains(&resp.status()) => {
                    let location = resp.header("Location").map(|s| s.to_string());
                    match location {
                        Some(loc) => {
                            url = Self::resolve_location(&url, &loc);
                            if Self::origin_of(&url) != start_origin {
                                strip_credentials = true;
                            }
                            continue;
                        }
                        None => {
                            return Err(Protocol {
                                error_type: "httpError".to_string(),
                                description: format!(
                                    "redirect status {} missing Location header",
                                    resp.status()
                                ),
                            });
                        }
                    }
                }
                Ok(resp) => {
                    response = Some(resp);
                    break;
                }
                // ureq surfaces any 4xx/5xx response as `Err(Error::Status(code, resp))`,
                // not as `Ok(resp)` - unlike the 3xx case above, which it still returns as
                // Ok. The response (including its body) is still attached to the error, so
                // extract it here rather than discarding it: a real JMAP server's error
                // response commonly carries an RFC 8620 section 3.6.1 "problem details" JSON
                // body (type/title/detail) explaining exactly what went wrong, and losing it
                // would leave only the bare status code to debug from.
                Err(ureq::Error::Status(code, resp)) => {
                    let mut body = Vec::new();
                    let _ = resp.into_reader().read_to_end(&mut body);
                    return Err(Self::describe_http_error(code, &body));
                }
                Err(e) => return Err(Network(e.to_string())),
            }
        }

        let response = response.ok_or_else(|| Protocol {
            error_type: "httpError".to_string(),
            description: format!("exceeded {MAX_REDIRECTS} redirects"),
        })?;

        // Extract status, headers, and body from the ureq response.
        let status_code = response.status();
        let mut resp_headers = HashMap::new();
        for name in response.headers_names() {
            if let Some(value) = response.header(&name) {
                resp_headers.insert(name.to_string(), value.to_string());
            }
        }
        let mut body = Vec::new();
        response
            .into_reader()
            .read_to_end(&mut body)
            .map_err(|e| Network(e.to_string()))?;

        Ok(HttpResponse {
            status_code,
            headers: resp_headers,
            body,
        })
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
