// Auto-generated crate root (template-rendered, not an LLM task).
// Declares every generated module and re-exports the public model types.

mod common_types;
mod transport;
mod session;
mod invocation;
mod jmap_request_envelope;
mod jmap_response_envelope;
mod mailbox;
mod email_address;
mod email_address_group;
mod email_body_part;
mod email_header;
mod email;
mod thread;
mod identity;
mod comparator;
mod search_snippet;
mod email_query_response;
mod envelope;
mod delivery_status;
mod email_submission;
mod client_core;
mod client_mail;
mod client_submission;

pub use common_types::{SetError, MethodError, ResultReference, JmapError, JmapError::Protocol, JmapError::Network};
pub use transport::{Transport, UreqTransport, HttpRequest, HttpResponse};
pub use session::{Session, Account, CoreCapability};
pub use invocation::{Invocation};
pub use jmap_request_envelope::{JmapRequestEnvelope};
pub use jmap_response_envelope::{JmapResponseEnvelope};
pub use mailbox::{Mailbox, MailboxRights};
pub use email_address::{EmailAddress};
pub use email_address_group::{EmailAddressGroup};
pub use email_body_part::{EmailBodyPart};
pub use email_header::{EmailHeader};
pub use email::{Email, EmailBodyValue};
pub use thread::{Thread};
pub use identity::{Identity};
pub use comparator::{Comparator};
pub use search_snippet::{SearchSnippet};
pub use email_query_response::{EmailQueryResponse};
pub use envelope::{Envelope, Address};
pub use delivery_status::{DeliveryStatus};
pub use email_submission::{EmailSubmission};
pub use client_core::{JmapClient, ClientOptions};
