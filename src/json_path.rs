use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde_json::Value;
use thiserror::Error;
use crate::json_path::path_part::PathPart;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};
#[cfg(feature = "serde")]
use crate::json_path::json_path_visitor::JsonPathVisitor;


pub mod path_part;

#[cfg(feature = "serde")]
mod json_path_visitor;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct JsonPath {
    parts: Vec<PathPart>,
}

#[derive(Debug, Error, PartialEq)]
pub enum JsonPathResolveError {
    #[error("Failed to resolve part '{0}'")]
    FailedToResolvePart(PathPart),

    #[error("Missing key '{0}' on object")]
    MissingKey(String),

    #[error("Missing index '{0}' on array")]
    MissingIndex(usize),
}

impl JsonPath {
    pub fn push(&mut self, part: PathPart) {
        self.parts.push(part);
    }

    pub fn parent(&self) -> Option<JsonPath> {
        match self.parts.last() {
            Some(_) => {
                let mut local = self.clone();
                local.parts.pop();

                Some(local)
            }
            None => None,
        }
    }

    pub fn resolve<'a>(&self, value: &'a Value) -> Result<&'a Value, JsonPathResolveError> {
        let mut working_value = value;

        for part in &self.parts {
            match (working_value, part) {
                (Value::Object(object), PathPart::Key(key)) => {
                    let Some(value) = object.get(key) else {
                        return Err(JsonPathResolveError::MissingKey(key.to_string()));
                    };

                    working_value = value;
                }
                (Value::Array(array), PathPart::Index(index)) => {
                    let Some(value) = array.get(*index) else {
                        return Err(JsonPathResolveError::MissingIndex(*index));
                    };

                    working_value = value;
                },
                _ => {
                    return Err(JsonPathResolveError::FailedToResolvePart(part.clone()));
                }
            }
        }

        Ok(working_value)
    }

    pub fn resolve_mut<'a>(&mut self, value: &'a mut Value) -> Result<&'a mut Value, JsonPathResolveError> {
        let mut working_value = value;

        for part in &self.parts {
            match (working_value, part) {
                (Value::Object(object), PathPart::Key(key)) => {
                    let Some(value) = object.get_mut(key) else {
                        return Err(JsonPathResolveError::MissingKey(key.to_string()));
                    };

                    working_value = value;
                }
                (Value::Array(array), PathPart::Index(index)) => {
                    let Some(value) = array.get_mut(*index) else {
                        return Err(JsonPathResolveError::MissingIndex(*index));
                    };

                    working_value = value;
                },
                _ => {
                    return Err(JsonPathResolveError::FailedToResolvePart(part.clone()));
                }
            }
        }

        Ok(working_value)
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum JsonPathParseError {
    #[error("JSON path string should have a '$' first character")]
    MissingRoot,

    #[error("JSON path string should start with a '$', but got '{0}'")]
    IncorrectRoot(String),
}

impl FromStr for JsonPath {
    type Err = JsonPathParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        match parts.next() {
            Some("$") => Ok(()),
            Some(value) => Err(JsonPathParseError::IncorrectRoot(value.to_string())),
            None => Err(JsonPathParseError::MissingRoot),
        }?;

        Ok(JsonPath {
            parts: parts
                .map(|part| PathPart::from(part.to_string()))
                .collect(),
        })
    }
}

impl<const U: usize> From<[&str; U]> for JsonPath {
    fn from(value: [&str; U]) -> Self {
        JsonPath {
            parts: value
                .iter()
                .map(|value| value.to_string().into())
                .collect()
        }
    }
}

impl Display for JsonPath {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "$")?;

        for part in &self.parts {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl Serialize for JsonPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for JsonPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_string(JsonPathVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use serde_json::json;
    use crate::json_path::{JsonPath, JsonPathParseError};
    use crate::json_path::path_part::PathPart;

    #[test]
    fn correctly_formatted_json_paths_strings_are_parsed_correctly() {
        assert_eq!(JsonPath::from_str("$").unwrap(), JsonPath {
            parts: vec![],
        });

        assert_eq!(JsonPath::from_str("$.a").unwrap(), JsonPath {
            parts: vec![PathPart::Key("a".to_string())],
        });

        assert_eq!(JsonPath::from_str("$.a.b").unwrap(), JsonPath {
            parts: vec![PathPart::Key("a".to_string()), PathPart::Key("b".to_string())],
        });

        assert_eq!(JsonPath::from_str("$.0").unwrap(), JsonPath {
            parts: vec![PathPart::Index(0)],
        });

        assert_eq!(JsonPath::from_str("$.*.a").unwrap(), JsonPath {
            parts: vec![PathPart::Key("*".to_string()), PathPart::Key("a".to_string())],
        });
    }

    #[test]
    fn incorrectly_formatted_json_paths_strings_return_errors() {
        assert_eq!(JsonPath::from_str(""), Err(JsonPathParseError::IncorrectRoot("".to_string())));
        assert_eq!(JsonPath::from_str("?"), Err(JsonPathParseError::IncorrectRoot("?".to_string())));
        assert_eq!(JsonPath::from_str("!"), Err(JsonPathParseError::IncorrectRoot("!".to_string())));
    }

    #[test]
    fn parent_is_returned_correctly() {
        let a = JsonPath::from(["a", "b"]);
        assert_eq!(a.parent(), Some(JsonPath::from(["a"])));

        let b = JsonPath::default();
        assert_eq!(b.parent(), None);
    }

    #[test]
    fn paths_are_resolved_correctly() {
        assert_eq!(JsonPath::default().resolve(&json!({ "a": 10 })), Ok(&json!({ "a": 10 })));
        assert_eq!(JsonPath::from(["a"]).resolve(&json!({ "a": 10 })), Ok(&json!(10)));
        assert_eq!(JsonPath::from(["a", "0"]).resolve(&json!({ "a": [10] })), Ok(&json!(10)));
    }

    #[test]
    fn mut_paths_are_resolved_correctly() {
        assert_eq!(JsonPath::default().resolve_mut(&mut json!({ "a": 10 })), Ok(&mut json!({ "a": 10 })));
        assert_eq!(JsonPath::from(["a"]).resolve_mut(&mut json!({ "a": 10 })), Ok(&mut json!(10)));
        assert_eq!(JsonPath::from(["a", "0"]).resolve_mut(&mut json!({ "a": [10] })), Ok(&mut json!(10)));
    }
}
