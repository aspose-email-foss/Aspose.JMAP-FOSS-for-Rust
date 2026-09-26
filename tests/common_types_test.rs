//! Tests for the shared JMAP types defined in `common_types.rs`.

use aspose_jmap_foss::{MethodError, ResultReference, SetError, JmapError};
use serde_json;
#[cfg(feature = "ureq")]
use ureq;

/// Verify that a fully populated `SetError` round‑trips through JSON
/// (serialization then deserialization) unchanged.
#[test]
fn set_error_roundtrip_full() {
    let original = SetError {
        error_type: "invalidProperties".to_string(),
        description: Some("Invalid field values".to_string()),
        properties: Some(vec!["name".to_string(), "email".to_string()]),
    };
    let json = serde_json::to_string(&original).expect("serialization failed");
    let parsed: SetError =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(original, parsed);
}

/// Verify that optional fields of `SetError` are correctly handled when omitted.
#[test]
fn set_error_deserialize_missing_optionals() {
    let json = r#"{"type":"notFound"}"#;
    let err: SetError =
        serde_json::from_str(json).expect("deserialization failed");
    assert_eq!(err.error_type, "notFound");
    assert!(err.description.is_none());
    assert!(err.properties.is_none());
}

/// Verify that a fully populated `MethodError` round‑trips through JSON unchanged.
#[test]
fn method_error_roundtrip_full() {
    let original = MethodError {
        error_type: "unknownMethod".to_string(),
        description: Some("The method does not exist".to_string()),
    };
    let json = serde_json::to_string(&original).expect("serialization failed");
    let parsed: MethodError =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(original, parsed);
}

/// Verify that optional fields of `MethodError` are correctly handled when omitted.
#[test]
fn method_error_deserialize_missing_optionals() {
    let json = r#"{"type":"accountNotFound"}"#;
    let err: MethodError =
        serde_json::from_str(json).expect("deserialization failed");
    assert_eq!(err.error_type, "accountNotFound");
    assert!(err.description.is_none());
}

/// Verify that `ResultReference` round‑trips through JSON unchanged.
#[test]
fn result_reference_roundtrip() {
    let original = ResultReference {
        result_of: "c1".to_string(),
        name: "created".to_string(),
        path: "/ids/0".to_string(),
    };
    let json = serde_json::to_string(&original).expect("serialization failed");
    let parsed: ResultReference =
        serde_json::from_str(&json).expect("deserialization failed");
    assert_eq!(original, parsed);
}

/// Ensure the `Display` implementation of `JmapError` produces the expected strings.
#[test]
fn jmap_error_display() {
    let net = JmapError::Network("connection refused".to_string());
    assert_eq!(format!("{}", net), "network error: connection refused");

    let proto = JmapError::Protocol {
        error_type: "invalidArguments".to_string(),
        description: "Missing required field".to_string(),
    };
    assert_eq!(
        format!("{}", proto),
        "protocol error (invalidArguments): Missing required field"
    );

    let invalid = JmapError::InvalidResponse("unexpected token".to_string());
    assert_eq!(format!("{}", invalid), "invalid response: unexpected token");
}

#[cfg(feature = "ureq")]
/// Verify that a `ureq::Error` converts into a `JmapError::Network`.
#[test]
fn jmap_error_from_ureq_error() {
    // Create a simple ureq::Error via a failed request (status error).
    let err = ureq::Error::Status(404, ureq::Response::new(
        404,
        "Not Found",
        std::collections::HashMap::new(),
        std::io::empty(),
    ));
    let jerr: JmapError = err.into();
    match jerr {
        JmapError::Network(msg) => {
            assert!(msg.contains("404"));
        }
        _ => panic!("expected JmapError::Network variant"),
    }
}
