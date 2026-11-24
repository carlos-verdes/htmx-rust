//! A value's fields, read from a URL and printed back as one. The two are
//! inverse: what is printed here is parsed here.

use std::any::type_name;

use axum::extract::RawPathParams;
use form_urlencoded::{Serializer, byte_serialize};
use serde::{
    Serialize,
    de::{DeserializeOwned, Deserializer, Visitor},
    forward_to_deserialize_any,
};

pub use serde_urlencoded::de::Error;

/// What a request's URL carries, as one list of fields: the path parameters,
/// then the query string.
pub fn fields(parameters: &RawPathParams, query: Option<&str>) -> String {
    let parameters = Serializer::new(String::new())
        .extend_pairs(parameters)
        .finish();
    format!("{parameters}&{}", query.unwrap_or_default())
}

pub fn parse<T: DeserializeOwned>(fields: &str) -> Result<T, Error> {
    T::deserialize(Fields(serde_urlencoded::Deserializer::new(
        form_urlencoded::parse(fields.as_bytes()),
    )))
}

/// Prints `path` with its parameters filled from the fields of `value` that
/// they name, then the other fields as the query string.
///
/// # Panics
///
/// If a parameter of the path has no field to fill it, or the value is not made
/// of named fields that each hold one plain value.
pub fn print<T: Serialize>(path: &str, value: &T) -> String {
    let fields = serde_urlencoded::to_string(value)
        .unwrap_or_else(|error| panic!("{} does not print as a URL: {error}", type_name::<T>()));
    let mut path = path.to_owned();
    let mut query = Serializer::new(String::new());
    for (name, value) in form_urlencoded::parse(fields.as_bytes()) {
        let parameter = format!("{{{name}}}");
        if path.contains(&parameter) {
            // In a path a space is `%20`: a `+` there stands for itself.
            let value = byte_serialize(value.as_bytes()).collect::<String>();
            path = path.replace(&parameter, &value.replace('+', "%20"));
        } else {
            query.append_pair(&name, &value);
        }
    }
    assert!(
        !path.contains('{'),
        "{} has no field for a parameter of its path: {path}",
        type_name::<T>()
    );
    match query.finish() {
        query if query.is_empty() => path,
        query => format!("{path}?{query}"),
    }
}

/// Reads fields as `serde_urlencoded` does, and also into a type that has none
/// (`struct Home;`), whatever the URL carries.
struct Fields<'de>(serde_urlencoded::Deserializer<'de>);

impl<'de> Deserializer<'de> for Fields<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        self.0.deserialize_any(visitor)
    }

    fn deserialize_unit_struct<V: Visitor<'de>>(
        self,
        _: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_unit()
    }

    forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
        bytes byte_buf option unit newtype_struct seq tuple tuple_struct map
        struct enum identifier ignored_any
    }
}
