use std::error::Error;
use std::fmt::Formatter;
use std::str::FromStr;
use serde::de::Visitor;
use crate::json_search::JsonSearch;

pub struct JsonSearchVisitor;

impl<'de> Visitor<'de> for JsonSearchVisitor {
    type Value = JsonSearch;

    fn expecting(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "a json path")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: Error,
    {
        JsonSearch::from_str(v).map_err(|err| E::custom(err.to_string()))
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: Error,
    {
        JsonSearch::from_str(&v).map_err(|err| E::custom(err.to_string()))
    }
}
