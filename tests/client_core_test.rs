//! Integration tests for the core client implementation.

use std::cell::RefCell;
use std::collections::HashMap;

use aspose_jmap_foss::{
    ClientOptions, HttpRequest, HttpResponse, JmapClient, JmapError, Transport,
};
use serde_json::{json, Value};

/// Simple fake transport that returns pre‑programmed responses and records the
/// requests it receives.
#[derive(Debug)]
struct FakeTransport {
    responses: RefCell<Vec<HttpResponse>>,
    captured: RefCell<Vec<HttpRequest>>,
}

impl FakeTransport {
    fn new(responses: Vec<HttpResponse>) -> Self {
        Self {
            responses: RefCell::new(responses),
            captured: RefCell::new(Vec::new()),
        }
    }

    fn requests(&self) -> Vec<HttpRequest> {
        self.captured.borrow().clone()
    }
}

impl Transport for FakeTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError> {
        // Record a shallow copy of the request.
        let recorded = HttpRequest {
            method: req.method.clone(),
            url: req.url.clone(),
            headers: req.headers.clone(),
            body: req.body.clone(),
        };
        self.captured.borrow_mut().push(recorded);

        // Return the next programmed response.
        Ok(self.responses.borrow_mut().remove(0))
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(FakeTransport {
            responses: RefCell::new(self.responses.borrow().clone()),
            captured: RefCell::new(self.captured.borrow().clone()),
        })
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper to build a minimal Session JSON payload. Only the fields accessed by
/// `client_core.rs` are required.
fn session_json(base: &str) -> Vec<u8> {
    let obj = json!({
        "capabilities": {},
        "accounts": {},
        "primaryAccounts": {},
        "username": "user@example.com",
        "state": "s1",
        "apiUrl": "/jmap/api",
        "uploadUrl": "/jmap/upload/{accountId}",
        "downloadUrl": "/jmap/download/{accountId}/{blobId}/{type}/{name}",
        "eventSourceUrl": "/jmap/events"
    });
    obj.to_string().into_bytes()
}

/// Constructs a `ClientOptions` that uses the given fake transport, authenticating
/// with HTTP Basic auth (username/password, no bearer token).
fn client_with_fake(transport: FakeTransport) -> JmapClient {
    let options = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(transport)),
    };
    JmapClient::new(options)
}

/// Constructs a `ClientOptions` that uses the given fake transport, authenticating
/// with the given OAuth 2.0 bearer token instead of username/password.
fn client_with_fake_bearer(transport: FakeTransport, bearer_token: &str) -> JmapClient {
    let options = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: Some(bearer_token.to_string()),
        transport: Some(Box::new(transport)),
    };
    JmapClient::new(options)
}

#[test]
fn test_connect_success() {
    // Prepare the fake transport with a single Session response.
    let resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let fake = FakeTransport::new(vec![resp]);

    let mut client = client_with_fake(fake);
    let session = client.connect().expect("connect should succeed");

    // Verify the returned Session contains the expected (relative) URLs.
    assert_eq!(session.api_url, "/jmap/api");
    assert_eq!(session.upload_url, "/jmap/upload/{accountId}");
    assert_eq!(session.download_url, "/jmap/download/{accountId}/{blobId}/{type}/{name}");
    assert_eq!(session.event_source_url, "/jmap/events");

    // Verify the HTTP request that was issued.
    let captured = client.transport.as_any().downcast_ref::<FakeTransport>()
        .expect("transport is FakeTransport")
        .requests();
    assert_eq!(captured.len(), 1);
    let req = &captured[0];
    assert_eq!(req.method, "GET");
    assert_eq!(req.url, "https://example.com/.well-known/jmap");
    assert!(req.headers.contains_key("Authorization"));
    assert!(req.body.is_none());

    // Basic auth: base64("user:pass") == "dXNlcjpwYXNz".
    assert_eq!(
        req.headers.get("Authorization").map(String::as_str),
        Some("Basic dXNlcjpwYXNz")
    );
}

#[test]
fn test_connect_uses_bearer_token_when_configured() {
    // When `bearer_token` is set, requests must authenticate with
    // `Authorization: Bearer <token>` instead of HTTP Basic auth, even though
    // username/password are still populated on the options.
    let resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let fake = FakeTransport::new(vec![resp]);

    let mut client = client_with_fake_bearer(fake, "my-oauth-token");
    client.connect().expect("connect should succeed");

    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .expect("transport is FakeTransport")
        .requests();
    assert_eq!(captured.len(), 1);
    let req = &captured[0];
    assert_eq!(
        req.headers.get("Authorization").map(String::as_str),
        Some("Bearer my-oauth-token")
    );
}

#[test]
fn test_connect_empty_bearer_token_falls_back_to_basic_auth() {
    // `Some("")` is treated the same as `None`: it falls back to HTTP Basic
    // auth built from username/password, rather than sending an empty
    // `Authorization: Bearer ` header.
    let resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let fake = FakeTransport::new(vec![resp]);

    let mut client = client_with_fake_bearer(fake, "");
    client.connect().expect("connect should succeed");

    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .expect("transport is FakeTransport")
        .requests();
    let req = &captured[0];
    assert_eq!(
        req.headers.get("Authorization").map(String::as_str),
        Some("Basic dXNlcjpwYXNz")
    );
}

#[test]
fn test_echo_success() {
    // Session response + echo response.
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let echo_body = json!({
        "methodResponses": [
            ["Core/echo", {"hello": true, "high": 5}, "c1"]
        ],
        "sessionState": "s1"
    })
    .to_string();
    let echo_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: echo_body.into_bytes(),
    };
    let fake = FakeTransport::new(vec![session_resp, echo_resp]);

    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    let args = json!({"hello": true, "high": 5});
    let result = client.echo(args.clone()).expect("echo should succeed");
    assert_eq!(result, args);

    // Verify the POST request to the API endpoint.
    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .unwrap()
        .requests();
    // Two requests: session GET and echo POST.
    assert_eq!(captured.len(), 2);
    let req = &captured[1];
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://example.com/jmap/api");
    assert_eq!(
        req.headers.get("Content-Type").map(String::as_str),
        Some("application/json")
    );

    // Parse the request body and check its structure.
    let body: Value = serde_json::from_slice(req.body.as_ref().unwrap()).unwrap();
    assert_eq!(
        body.get("using").unwrap(),
        &json!(["urn:ietf:params:jmap:core"])
    );
    let method_calls = body.get("methodCalls").unwrap().as_array().unwrap();
    assert_eq!(method_calls.len(), 1);
    let call = &method_calls[0];
    assert_eq!(call[0], "Core/echo");
    assert_eq!(call[1], args);
    assert_eq!(call[2], "c1");
}

#[test]
fn test_echo_invalid_arguments() {
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let fake = FakeTransport::new(vec![session_resp]);

    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    let bad_arg = Value::String("not an object".to_string());
    let err = client.echo(bad_arg).unwrap_err();
    match err {
        JmapError::Protocol { error_type, description } => {
            assert_eq!(error_type, "invalidArgument");
            assert!(description.contains("echo arguments must be a JSON object"));
        }
        _ => panic!("expected Protocol error"),
    }
}

#[test]
fn test_upload_blob_success() {
    // Session + upload response.
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let upload_body = json!({
        "accountId": "a1",
        "blobId": "b1",
        "type": "text/plain",
        "size": 4
    })
    .to_string();
    let upload_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: upload_body.clone().into_bytes(),
    };
    let fake = FakeTransport::new(vec![session_resp, upload_resp]);

    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    // Not valid UTF-8 (starts like a JPEG magic number) - a prior version lossily decoded
    // this via String::from_utf8_lossy before sending, corrupting real attachment content.
    let data: &[u8] = &[0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46];
    let result = client
        .upload_blob("a1", "text/plain", data)
        .expect("upload_blob should succeed");
    assert_eq!(result, serde_json::from_str::<Value>(&upload_body).unwrap());

    // Verify request.
    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .unwrap()
        .requests();
    assert_eq!(captured.len(), 2);
    let req = &captured[1];
    assert_eq!(req.method, "POST");
    assert_eq!(req.url, "https://example.com/jmap/upload/a1");
    assert_eq!(
        req.headers.get("Content-Type").map(String::as_str),
        Some("text/plain")
    );
    assert_eq!(req.body.as_ref().unwrap().as_slice(), data);
}

#[test]
fn test_download_blob_success() {
    // Session + download response.
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    // Not valid UTF-8 (starts like a JPEG magic number) - a prior version decoded this via
    // ureq's into_string(), which hard-errors on invalid UTF-8 instead of returning it.
    let binary_content: Vec<u8> = vec![0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46];
    let download_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: binary_content.clone(),
    };
    let fake = FakeTransport::new(vec![session_resp, download_resp]);

    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    let result = client
        .download_blob("a1", "b1", "application/octet-stream", Some("file.bin"))
        .expect("download_blob should succeed");
    assert_eq!(result, binary_content);

    // Verify request URL.
    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .unwrap()
        .requests();
    assert_eq!(captured.len(), 2);
    let req = &captured[1];
    assert_eq!(req.method, "GET");
    assert_eq!(
        req.url,
        "https://example.com/jmap/download/a1/b1/application%2Foctet-stream/file.bin"
    );
}

#[test]
fn test_upload_and_download_blob_percent_encode_url_segments() {
    // Regression test: account_id/blob_id/mime_type/name are spliced verbatim into a URL
    // path segment. A value containing "/", "?", or "#" must be percent-encoded, or it
    // would rewrite the request path or smuggle extra query parameters into the request.
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let upload_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: json!({"accountId": "a/b", "blobId": "b1", "type": "text/plain", "size": 1})
            .to_string()
            .into_bytes(),
    };
    let fake = FakeTransport::new(vec![session_resp, upload_resp]);
    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    client.upload_blob("a/b", "text/plain", b"x").unwrap();

    let captured = client
        .transport
        .as_any()
        .downcast_ref::<FakeTransport>()
        .unwrap()
        .requests();
    let req = &captured[1];
    assert_eq!(req.url, "https://example.com/jmap/upload/a%2Fb");
}

#[test]
fn test_protocol_error_propagates() {
    // Session + error response.
    let session_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json("https://example.com"),
    };
    let error_body = json!({
        "methodResponses": [
            ["error", {"type": "unknownMethod", "description": "bad call"}, "c1"]
        ],
        "sessionState": "s1"
    })
    .to_string();
    let error_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: error_body.into_bytes(),
    };
    let fake = FakeTransport::new(vec![session_resp, error_resp]);

    let mut client = client_with_fake(fake);
    client.connect().unwrap();

    let args = json!({"foo": "bar"});
    let err = client.echo(args).unwrap_err();
    match err {
        JmapError::Protocol { error_type, description } => {
            assert_eq!(error_type, "unknownMethod");
            assert_eq!(description, "bad call");
        }
        _ => panic!("expected Protocol error"),
    }
}
