use std::fmt::Formatter;
use std::str::FromStr;
use serde::de::{Error, Visitor};
use crate::json_path::JsonPath;

pub struct JsonPathVisitor;

impl<'de> Visitor<'de> for JsonPathVisitor {
    type Value = JsonPath;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "a json path")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
    {
        JsonPath::from_str(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
    {
        JsonPath::from_str(&v).map_err(|err| E::custom(err.to_string()))
    }
}
