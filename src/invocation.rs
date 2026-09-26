//! JMAP method invocation representation.
//! This type is serialized as a JSON array `[name, arguments, methodCallId]`.

use serde::{
    de::{self, SeqAccess, Visitor},
    ser::SerializeTuple,
    Deserialize, Deserializer, Serialize, Serializer,
};
use serde_json::Value;
use std::collections::HashMap;

/// A JMAP method invocation.
///
/// It is represented on the wire as a three‑element JSON array:
/// `[name, arguments, methodCallId]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invocation {
    /// The JMAP method name (e.g. `"Mailbox/get"`).
    pub name: String,
    /// The method arguments as a map from argument name to JSON value.
    pub arguments: HashMap<String, Value>,
    /// Client‑chosen identifier for this call, echoed back in the response.
    #[allow(clippy::struct_field_names)]
    pub method_call_id: String,
}

impl Serialize for Invocation {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut tup = serializer.serialize_tuple(3)?;
        tup.serialize_element(&self.name)?;
        tup.serialize_element(&self.arguments)?;
        tup.serialize_element(&self.method_call_id)?;
        tup.end()
    }
}

impl<'de> Deserialize<'de> for Invocation {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct InvocationVisitor;

        impl<'de> Visitor<'de> for InvocationVisitor {
            type Value = Invocation;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a JMAP invocation array of [name, arguments, methodCallId]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Invocation, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let name: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;
                let arguments: HashMap<String, Value> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;
                let method_call_id: String = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(2, &self))?;
                Ok(Invocation {
                    name,
                    arguments,
                    method_call_id,
                })
            }
        }

        deserializer.deserialize_tuple(3, InvocationVisitor)
    }
}
