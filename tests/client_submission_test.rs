use aspose_jmap_foss::{
    ClientOptions, EmailSubmission, JmapClient, JmapError, Transport, HttpRequest,
    HttpResponse,
};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

/// Simple fake transport that returns a queued response for each request
/// and records the requests it receives. Uses `Rc<RefCell<...>>` (not a bare
/// `RefCell`) so that `.clone()` shares the SAME underlying storage - the client
/// owns one clone, and the test keeps another to inspect recorded requests; a
/// bare `RefCell` clone would deep-copy and the two handles would diverge.
#[derive(Debug, Clone)]
struct FakeTransport {
    responses: Rc<RefCell<Vec<HttpResponse>>>,
    requests: Rc<RefCell<Vec<HttpRequest>>>,
}

impl FakeTransport {
    fn new(resps: Vec<HttpResponse>) -> Self {
        Self {
            responses: Rc::new(RefCell::new(resps)),
            requests: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn recorded_requests(&self) -> Vec<HttpRequest> {
        self.requests.borrow().clone()
    }
}

impl Transport for FakeTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError> {
        self.requests.borrow_mut().push(req.clone());
        let resp = self
            .responses
            .borrow_mut()
            .remove(0);
        Ok(resp)
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Minimal Session JSON required by `JmapClient::connect`.
fn session_response() -> HttpResponse {
    let body = json!({
        "apiUrl": "https://example.com/api",
        "uploadUrl": "https://example.com/upload",
        "downloadUrl": "https://example.com/download",
        "eventSourceUrl": "https://example.com/events",
        "primaryAccounts": { "submission": "a1" },
        "capabilities": {},
        "accounts": {},
        "username": "user@example.com",
        "state": "s1"
    })
    .to_string()
    .into_bytes();

    HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body,
    }
}

#[test]
fn test_send_success() {
    // 1️⃣ Session GET response.
    let sess_resp = session_response();

    // 2️⃣ EmailSubmission/set response with a created object.
    let method_body = json!({
        "sessionState": "0",
        "methodResponses": [
            ["EmailSubmission/set",
                {
                    "accountId": "a1",
                    "newState": "1",
                    "created": {
                        "c1": {
                            "id": "s1",
                            "identityId": "id1",
                            "emailId": "e1"
                        }
                    }
                },
                "c1"
            ]
        ]
    })
    .to_string()
    .into_bytes();

    let method_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: method_body,
    };

    // Fake transport with the two queued responses.
    let fake = FakeTransport::new(vec![sess_resp, method_resp]);

    // Build a client that uses the fake transport.
    let opts = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(fake.clone())),
    };
    let mut client = JmapClient::new(opts);
    client.connect().unwrap();

    // The submission we want to send.
    let sub = EmailSubmission {
        id: None,
        identity_id: "id1".to_string(),
        email_id: "e1".to_string(),
        thread_id: None,
        envelope: None,
        send_at: None,
        undo_status: None,
        delivery_status: None,
        dsn_blob_ids: None,
        mdn_blob_ids: None,
    };

    // Call the method under test.
    let created = client.send("a1", sub).unwrap();
    assert_eq!(created.id.unwrap(), "s1");

    // Verify the POST request that was issued.
    let recorded = fake.recorded_requests();
    // first request is the session GET, second is the POST we care about.
    assert_eq!(recorded.len(), 2);
    let post = &recorded[1];
    assert_eq!(post.method, "POST");
    let body_json: Value = serde_json::from_slice(post.body.as_ref().unwrap()).unwrap();
    let method_calls = body_json.get("methodCalls").unwrap().as_array().unwrap();
    let call = &method_calls[0];
    assert_eq!(call[0].as_str().unwrap(), "EmailSubmission/set");
    let args = &call[1];
    assert_eq!(args.get("accountId").unwrap(), "a1");
    // The `create` map must contain our client‑generated id "c1".
    let create_map = args.get("create").unwrap().as_object().unwrap();
    assert!(create_map.contains_key("c1"));
}

#[test]
fn test_cancel_send_success() {
    let sess_resp = session_response();

    let method_body = json!({
        "sessionState": "0",
        "methodResponses": [
            ["EmailSubmission/set",
                {
                    "accountId": "a1",
                    "newState": "2",
                    "updated": {
                        "s1": {
                            "id": "s1",
                            "identityId": "id1",
                            "emailId": "e1",
                            "undoStatus": "canceled"
                        }
                    }
                },
                "c1"
            ]
        ]
    })
    .to_string()
    .into_bytes();

    let method_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: method_body,
    };

    let fake = FakeTransport::new(vec![sess_resp, method_resp]);

    let opts = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(fake.clone())),
    };
    let mut client = JmapClient::new(opts);
    client.connect().unwrap();

    let updated = client.cancel_send("a1", "s1").unwrap();
    assert_eq!(updated.undo_status.unwrap(), "canceled");

    // Verify the request body contains the correct patch.
    let recorded = fake.recorded_requests();
    assert_eq!(recorded.len(), 2);
    let post = &recorded[1];
    let body_json: Value = serde_json::from_slice(post.body.as_ref().unwrap()).unwrap();
    let method_calls = body_json.get("methodCalls").unwrap().as_array().unwrap();
    let call = &method_calls[0];
    assert_eq!(call[0].as_str().unwrap(), "EmailSubmission/set");
    let args = &call[1];
    let update_map = args.get("update").unwrap().as_object().unwrap();
    let patch = update_map.get("s1").unwrap().as_object().unwrap();
    // Regression test: a PatchObject key is a JSON Pointer (RFC 6901) relative to the
    // patched object - a bare top-level property name has no leading slash. A prior
    // version sent "/undoStatus" (pointing at a differently-named property instead),
    // which a real JMAP server rejects/ignores, silently breaking cancel-send.
    assert_eq!(patch.get("undoStatus").unwrap(), "canceled");
    assert!(patch.get("/undoStatus").is_none());
}

#[test]
fn test_list_submissions_empty() {
    let sess_resp = session_response();

    // Query response with no ids.
    let query_body = json!({
        "sessionState": "0",
        "methodResponses": [
            ["EmailSubmission/query",
                {
                    "accountId": "a1",
                    "queryState": "0",
                    "canCalculateChanges": false,
                    "position": 0,
                    "ids": [],
                    "total": null
                },
                "c1"
            ]
        ]
    })
    .to_string()
    .into_bytes();

    let query_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: query_body,
    };

    let fake = FakeTransport::new(vec![sess_resp, query_resp]);

    let opts = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(fake.clone())),
    };
    let mut client = JmapClient::new(opts);
    client.connect().unwrap();

    let list = client.list_submissions("a1").unwrap();
    assert!(list.is_empty());

    // Only the session GET and the query POST should have been sent.
    let recorded = fake.recorded_requests();
    assert_eq!(recorded.len(), 2);
    let post = &recorded[1];
    let body_json: Value = serde_json::from_slice(post.body.as_ref().unwrap()).unwrap();
    let method_calls = body_json.get("methodCalls").unwrap().as_array().unwrap();
    let call = &method_calls[0];
    assert_eq!(call[0].as_str().unwrap(), "EmailSubmission/query");
}

#[test]
fn test_send_not_created_error() {
    let sess_resp = session_response();

    // Response where creation failed and a SetError is returned.
    let method_body = json!({
        "sessionState": "0",
        "methodResponses": [
            ["EmailSubmission/set",
                {
                    "accountId": "a1",
                    "newState": "1",
                    "notCreated": {
                        "c1": {
                            "type": "invalidProperties",
                            "description": "missing identityId"
                        }
                    }
                },
                "c1"
            ]
        ]
    })
    .to_string()
    .into_bytes();

    let method_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: method_body,
    };

    let fake = FakeTransport::new(vec![sess_resp, method_resp]);

    let opts = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(fake.clone())),
    };
    let mut client = JmapClient::new(opts);
    client.connect().unwrap();

    let sub = EmailSubmission {
        id: None,
        identity_id: "id1".to_string(),
        email_id: "e1".to_string(),
        thread_id: None,
        envelope: None,
        send_at: None,
        undo_status: None,
        delivery_status: None,
        dsn_blob_ids: None,
        mdn_blob_ids: None,
    };

    let err = client.send("a1", sub).unwrap_err();
    match err {
        JmapError::Protocol { error_type, description } => {
            assert_eq!(error_type, "invalidProperties");
            assert_eq!(description, "missing identityId");
        }
        _ => panic!("expected protocol error"),
    }
}
