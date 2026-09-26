use std::cell::RefCell;
use std::collections::HashMap;
use std::io::{Read as _, Write as _};
use std::net::TcpListener;
use std::thread;

use aspose_jmap_foss::{HttpRequest, HttpResponse, JmapError, Transport, UreqTransport};

/// A simple fake transport that records the last request it received and returns a
/// pre‑configured response.
#[derive(Debug, Clone)]
struct FakeTransport {
    /// The request that was passed to `send`, stored for later inspection.
    last_req: RefCell<Option<HttpRequest>>,
    /// The response that `send` should return.
    response: Result<HttpResponse, JmapError>,
}

impl FakeTransport {
    fn new(response: Result<HttpResponse, JmapError>) -> Self {
        Self {
            last_req: RefCell::new(None),
            response,
        }
    }
}

impl Transport for FakeTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError> {
        // Record the request (clone is required because `HttpRequest` is owned by the caller).
        *self.last_req.borrow_mut() = Some(req.clone());
        // Return the pre‑configured response.
        self.response.clone()
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[test]
fn test_fake_transport_success() {
    // Prepare a fake transport that will return a successful 200 response.
    let fake = FakeTransport::new(Ok(HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: b"OK".to_vec(),
    }));

    // Build a simple GET request.
    let req = HttpRequest {
        method: "GET".to_string(),
        url: "https://example.com/api".to_string(),
        headers: HashMap::new(),
        body: None,
    };

    // Send the request through the fake transport.
    let resp = fake.send(&req).expect("expected successful response");

    // Verify the response contents.
    assert_eq!(resp.status_code, 200);
    assert_eq!(resp.body, b"OK".to_vec());

    // Verify that the transport recorded the exact request we sent.
    let last_req = fake.last_req.borrow();
    let recorded = last_req
        .as_ref()
        .expect("request should have been recorded");
    assert_eq!(recorded.method, "GET");
    assert_eq!(recorded.url, "https://example.com/api");
    assert!(recorded.headers.is_empty());
    assert!(recorded.body.is_none());
}

#[test]
fn test_fake_transport_error() {
    // Prepare a fake transport that will simulate a network failure.
    let fake = FakeTransport::new(Err(JmapError::Network("network down".into())));

    // Build a POST request with a JSON body.
    let mut headers = HashMap::new();
    headers.insert("Content-Type".to_string(), "application/json".to_string());

    let req = HttpRequest {
        method: "POST".to_string(),
        url: "https://example.com/submit".to_string(),
        headers,
        body: Some(b"{\"a\":1}".to_vec()),
    };

    // Send the request and expect an error.
    let err = fake.send(&req).unwrap_err();

    // Verify that the error is the expected network error.
    match err {
        JmapError::Network(msg) => assert_eq!(msg, "network down"),
        _ => panic!("expected JmapError::Network"),
    }

    // Verify that the request was still recorded even though the transport failed.
    let last_req = fake.last_req.borrow();
    let recorded = last_req
        .as_ref()
        .expect("request should have been recorded");
    assert_eq!(recorded.method, "POST");
    assert_eq!(recorded.url, "https://example.com/submit");
}

/// Regression test: a prior version of `UreqTransport::send` discarded the response body
/// entirely on a non-2xx status, keeping only the bare status code. Real JMAP servers
/// commonly return an RFC 8620 section 3.6.1 "problem details" JSON body explaining exactly
/// what went wrong - silently dropping it makes debugging a failed request much harder than
/// necessary. This exercises the real `UreqTransport` (not `FakeTransport`) against a raw
/// local TCP listener, since the non-2xx-handling logic under test lives inside
/// `UreqTransport::send` itself, not in client-level code.
#[test]
fn test_ureq_transport_non_2xx_includes_body_excerpt() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("local_addr");

    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut buf = [0u8; 4096];
        let _ = stream.read(&mut buf); // discard the request; we only care about the response

        let body = br#"{"type":"urn:ietf:params:jmap:error:notJSON","detail":"bad request body"}"#;
        let response = format!(
            "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        );
        stream.write_all(response.as_bytes()).expect("write headers");
        stream.write_all(body).expect("write body");
        stream.flush().expect("flush");
    });

    let transport = UreqTransport::new();
    let req = HttpRequest {
        method: "GET".to_string(),
        url: format!("http://{}/api", addr),
        headers: HashMap::new(),
        body: None,
    };

    let err = transport.send(&req).expect_err("expected a protocol error for a 400 response");
    match err {
        JmapError::Protocol { description, .. } => {
            assert!(
                description.contains("bad request body"),
                "expected error description to include the response body excerpt, got: {description}"
            );
        }
        other => panic!("expected JmapError::Protocol, got: {other:?}"),
    }

    server.join().expect("server thread panicked");
}
