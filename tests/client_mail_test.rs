//! Integration tests for the `client_mail.rs` module.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use aspose_jmap_foss::{
    ClientOptions, HttpRequest, HttpResponse, JmapClient, JmapError, Mailbox, Transport,
};

/// Simple fake transport that returns a predefined sequence of responses and
/// records the requests it receives for later inspection. Uses `Rc<RefCell<...>>`
/// (not a bare `RefCell`) so that `.clone()` produces a handle sharing the SAME
/// underlying storage - `ClientOptions` takes ownership of one clone, and the test
/// keeps another to inspect captured requests after the call; a bare `RefCell`
/// clone would deep-copy and the two handles would silently diverge.
#[derive(Debug, Clone)]
struct FakeTransport {
    /// Responses to return, in order.
    responses: Rc<RefCell<Vec<HttpResponse>>>,
    /// Captured requests.
    captured: Rc<RefCell<Vec<HttpRequest>>>,
}

impl FakeTransport {
    fn new(responses: Vec<HttpResponse>) -> Self {
        Self {
            responses: Rc::new(RefCell::new(responses)),
            captured: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn captured(&self) -> std::cell::Ref<'_, Vec<HttpRequest>> {
        self.captured.borrow()
    }
}

impl Transport for FakeTransport {
    fn send(&self, req: &HttpRequest) -> Result<HttpResponse, JmapError> {
        // Record the request.
        self.captured.borrow_mut().push(req.clone());

        // Return the next prepared response.
        self.responses
            .borrow_mut()
            .pop()
            .ok_or_else(|| JmapError::Network("no more fake responses".into()))
    }

    fn box_clone(&self) -> Box<dyn Transport> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Helper to build a minimal JMAP session JSON string. The fields match the
/// `Session` struct used by the client (only the ones required for URL
/// resolution are included).
fn session_json() -> Vec<u8> {
    // The client will resolve the API URL against the origin of the session URL.
    // Here we set `apiUrl` to `/jmap` (relative) and provide dummy values for the
    // other URLs.
    let obj = serde_json::json!({
        "apiUrl": "/jmap",
        "uploadUrl": "/upload/{accountId}",
        "downloadUrl": "/download/{accountId}/{blobId}/{type}/{name}",
        "eventSourceUrl": "/events",
        "capabilities": {},
        "accounts": {},
        "username": "user@example.com",
        "state": "s1",
        "primaryAccounts": {
            "urn:ietf:params:jmap:mail": "u1"
        }
    });
    obj.to_string().into_bytes()
}

/// Helper to build a JMAP response envelope containing a single method response.
fn envelope_json(name: &str, arguments: serde_json::Value, method_call_id: &str) -> Vec<u8> {
    let resp = serde_json::json!({
        "methodResponses": [
            [name, arguments, method_call_id]
        ],
        "sessionState": "dummy"
    });
    resp.to_string().into_bytes()
}

/// Constructs a `JmapClient` with a fake transport that will first return a
/// session response, then the provided `subsequent` responses in order.
fn client_with_fake(subsequent: Vec<HttpResponse>) -> (JmapClient, FakeTransport) {
    // FakeTransport.send() returns responses via Vec::pop() (removes from the END), so
    // the vec must be built in REVERSE chronological order: the method-specific
    // responses (reversed) first, then the session response LAST so it pops FIRST.
    let mut responses: Vec<HttpResponse> = subsequent.into_iter().rev().collect();
    responses.push(HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: session_json(),
    });

    let fake = FakeTransport::new(responses);
    let options = ClientOptions {
        session_url: "https://example.com/.well-known/jmap".to_string(),
        username: "user".to_string(),
        password: "pass".to_string(),
        bearer_token: None,
        transport: Some(Box::new(fake.clone())),
    };
    let client = JmapClient::new(options);
    (client, fake)
}

#[test]
fn list_mailboxes_success() {
    // Prepare a Mailbox/get response with a single mailbox.
    let mailbox = serde_json::json!({
        "accountId": "u1",
        "state": "1",
        "list": [{
            "id": "mb1",
            "name": "Inbox",
            "parentId": null,
            "role": "inbox",
            "sortOrder": 0,
            "totalEmails": 3,
            "unreadEmails": 1,
            "totalThreads": 3,
            "unreadThreads": 1,
            "myRights": {
                "mayReadItems": true,
                "mayAddItems": true,
                "mayRemoveItems": true,
                "maySetSeen": true,
                "maySetKeywords": true,
                "mayCreateChild": true,
                "mayRename": false,
                "mayDelete": false,
                "maySubmit": true
            },
            "isSubscribed": true
        }],
        "notFound": []
    });
    let resp_body = envelope_json("Mailbox/get", mailbox, "c1");
    let mailbox_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: resp_body,
    };

    let (mut client, fake) = client_with_fake(vec![mailbox_resp]);

    // Connect to initialise URLs.
    client.connect().expect("connect failed");

    // Call the method under test.
    let result = client.list_mailboxes("u1").expect("list_mailboxes failed");

    // Verify the returned data.
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].name, "Inbox");
    assert_eq!(result[0].id.as_deref(), Some("mb1"));

    // Verify the request that was sent.
    let captured = fake.captured();
    // The second request (index 1) is the Mailbox/get call.
    let req = &captured[1];
    assert_eq!(req.method, "POST");
    assert!(req.url.ends_with("/jmap"));
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"Mailbox/get\""));
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"accountId\":\"u1\""));
}

#[test]
fn list_mailboxes_empty() {
    // Mailbox/get response with an empty list.
    let empty = serde_json::json!({
        "accountId": "u1",
        "state": "1",
        "list": [],
        "notFound": []
    });
    let resp_body = envelope_json("Mailbox/get", empty, "c1");
    let mailbox_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: resp_body,
    };

    let (mut client, fake) = client_with_fake(vec![mailbox_resp]);
    client.connect().expect("connect failed");
    let result = client.list_mailboxes("u1").expect("list_mailboxes failed");
    assert!(result.is_empty());

    // Ensure a request was made.
    let captured = fake.captured();
    let req = &captured[1];
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"Mailbox/get\""));
}

#[test]
fn create_mailbox_merges_partial_created_response() {
    // Per RFC 8620 5.3, the server's "created" entry is PARTIAL: it omits
    // properties already known from the client's create payload (like
    // `name`) and only carries server-set ones. It always includes the real
    // server-assigned id, which is deliberately DIFFERENT here from "new1"
    // (the client-chosen creation id, meaningless outside this response) so
    // a fix that wrongly copies the creation id into the result's `id`
    // fails loudly.
    let created = serde_json::json!({
        "accountId": "u1",
        "newState": "3",
        "created": {
            "new1": {
                "id": "mb-server-assigned-42",
                "totalEmails": 0,
                "unreadEmails": 0,
                "totalThreads": 0,
                "unreadThreads": 0
            }
        }
    });
    let resp_body = envelope_json("Mailbox/set", created, "c1");
    let set_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: resp_body,
    };

    let (mut client, _fake) = client_with_fake(vec![set_resp]);
    client.connect().expect("connect failed");

    let mut create = HashMap::new();
    create.insert(
        "new1".to_string(),
        Mailbox {
            id: None,
            name: "Drafts".to_string(),
            parent_id: None,
            role: None,
            sort_order: None,
            total_emails: None,
            unread_emails: None,
            total_threads: None,
            unread_threads: None,
            my_rights: None,
            is_subscribed: false,
        },
    );

    let result = client
        .create_mailbox("u1", create)
        .expect("create_mailbox failed");
    let mailbox = result.get("new1").expect("missing new1 in result");

    assert_eq!(mailbox.id.as_deref(), Some("mb-server-assigned-42"));
    assert_eq!(mailbox.name, "Drafts");
}

#[test]
fn list_messages_success() {
    // Prepare an Email/query response with all fields populated.
    let query_result = serde_json::json!({
        "accountId": "u1",
        "queryState": "qs1",
        "canCalculateChanges": true,
        "position": 0,
        "ids": ["m1", "m2"],
        "total": 42,
        "limit": 10
    });
    let resp_body = envelope_json("Email/query", query_result, "c1");
    let query_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: resp_body,
    };

    let (mut client, fake) = client_with_fake(vec![query_resp]);
    client.connect().expect("connect failed");

    let result = client
        .list_messages("u1", None, None, None)
        .expect("list_messages failed");

    // Verify the returned data.
    assert_eq!(result.account_id, "u1");
    assert_eq!(result.query_state, "qs1");
    assert!(result.can_calculate_changes);
    assert_eq!(result.position, 0);
    assert_eq!(result.ids, vec!["m1".to_string(), "m2".to_string()]);
    assert_eq!(result.total, Some(42));
    assert_eq!(result.limit, Some(10));

    // Verify the request that was sent, and that only a single call (Email/query,
    // no follow-up Email/get) was made.
    let captured = fake.captured();
    assert_eq!(captured.len(), 2);
    let req = &captured[1];
    assert_eq!(req.method, "POST");
    assert!(req.url.ends_with("/jmap"));
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"Email/query\""));
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"accountId\":\"u1\""));
}

#[test]
fn list_mailboxes_protocol_error() {
    // Simulate a protocol‑level error response.
    let error_args = serde_json::json!({
        "type": "accountNotFound",
        "description": "The account does not exist."
    });
    let resp_body = envelope_json("error", error_args, "c1");
    let error_resp = HttpResponse {
        status_code: 200,
        headers: HashMap::new(),
        body: resp_body,
    };

    let (mut client, fake) = client_with_fake(vec![error_resp]);
    client.connect().expect("connect failed");
    let err = client
        .list_mailboxes("u1")
        .expect_err("expected protocol error");

    match err {
        JmapError::Protocol { error_type, description } => {
            assert_eq!(error_type, "accountNotFound");
            assert_eq!(description, "The account does not exist.");
        }
        _ => panic!("unexpected error variant: {:?}", err),
    }

    // Verify that the request was still sent.
    let captured = fake.captured();
    let req = &captured[1];
    assert!(String::from_utf8_lossy(req.body.as_ref().unwrap()).contains("\"Mailbox/get\""));
}
