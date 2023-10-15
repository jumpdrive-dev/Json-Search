use std::fmt::{Display, Formatter};
use std::str::FromStr;
use serde_json::Value;
use thiserror::Error;
use crate::json_path::JsonPath;
use crate::json_path::path_part::PathPart;
use crate::json_search::search_part::SearchPart;

#[cfg(feature = "serde")]
use serde::{Serialize, Serializer, Deserialize, Deserializer};
#[cfg(feature = "serde")]
use crate::json_search::json_search_visitor::JsonSearchVisitor;

pub mod search_part;

#[cfg(feature = "serde")]
mod json_search_visitor;

#[derive(Debug, Default, Clone, PartialEq)]
pub struct JsonSearch {
    parts: Vec<SearchPart>,
    optional: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum JsonSearchResolveError {
    #[error("Expected an object at '{0}'")]
    NotAnObject(JsonPath),

    #[error("Expected an array at '{0}'")]
    NotAnArray(JsonPath),

    #[error("Expected an array or an object at '{0}'")]
    NotAnArrayOrObject(JsonPath),

    #[error("Missing required key '{1}' at '{0}'")]
    MissingRequiredKey(JsonPath, String),

    #[error("Missing required index '{1}' at '{0}'")]
    MissingRequiredIndex(JsonPath, usize),
}

impl JsonSearch {
    pub fn new<const U: usize>(parts: &[&str; U]) -> Self {
        Self {
            parts: parts
                .iter()
                .map(|value| value.to_string().into())
                .collect(),
            optional: false,
        }
    }

    pub fn new_optional<const U: usize>(parts: &[&str; U]) -> Self {
        let mut new = JsonSearch::new(parts);
        new.optional = true;
        new
    }

    pub fn resolve(&self, target: &Value) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        self.resolve_inner(&self.parts, target, JsonPath::default())
    }

    fn resolve_inner(&self, parts: &[SearchPart], target: &Value, parent: JsonPath) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let mut results = vec![];
        let remaining = if parts.len() > 0 {
            &parts[1..]
        } else {
            &parts[0..]
        };

        if parts.is_empty() {
            return Ok(vec![parent]);
        }


        if let Some(part) = parts.get(0) {
            let resolved = match part {
                SearchPart::Key(key) => self.resolve_key(remaining, target, parent, key)?,
                SearchPart::Index(index) => self.resolve_index(remaining, target, parent, index)?,
                SearchPart::Wildcard => self.resolve_wildcard(remaining, target, parent)?,
            };

            results.extend(resolved);
        };

        Ok(results)
    }

    fn resolve_key(&self, parts: &[SearchPart], target: &Value, mut parent: JsonPath, key: &String) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let Value::Object(map) = target else {
            return Err(JsonSearchResolveError::NotAnObject(parent));
        };

        match map.get(key) {
            Some(value) => {
                parent.push(PathPart::Key(key.clone()));
                self.resolve_inner(parts, value, parent)
            },
            None if self.optional => Ok(vec![]),
            None => Err(JsonSearchResolveError::MissingRequiredKey(parent, key.to_string())),
        }
    }

    fn resolve_index(&self, parts: &[SearchPart], target: &Value, mut parent: JsonPath, index: &usize) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let Value::Array(array) = target else {
            return Err(JsonSearchResolveError::NotAnArray(parent));
        };

        match array.get(*index) {
            Some(value) => {
                parent.push(PathPart::Index(*index));
                self.resolve_inner(parts, value, parent)
            },
            None if self.optional => Ok(vec![]),
            None => Err(JsonSearchResolveError::MissingRequiredIndex(parent, *index)),
        }
    }

    fn resolve_wildcard(&self, parts: &[SearchPart], target: &Value, parent: JsonPath) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        match target {
            Value::Array(_) => self.resolve_array_wildcard(parts, target, parent),
            Value::Object(_) => self.resolve_object_wildcard(parts, target, parent),
            _ => Err(JsonSearchResolveError::NotAnArrayOrObject(parent)),
        }
    }

    fn resolve_array_wildcard(&self, parts: &[SearchPart], target: &Value, mut parent: JsonPath) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let Value::Array(array) = target else {
            return Err(JsonSearchResolveError::NotAnArray(parent));
        };

        let parts: Vec<Vec<JsonPath>> = array.iter()
            .enumerate()
            .filter_map(|(i, value)| {
                let mut local = parent.clone();
                local.push(PathPart::Index(i));

                self.resolve_inner(parts, value, local).ok()
            })
            .collect();

        Ok(parts.into_iter().flatten().collect())
    }

    fn resolve_object_wildcard(&self, parts: &[SearchPart], target: &Value, mut parent: JsonPath) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let Value::Object(map) = target else {
            return Err(JsonSearchResolveError::NotAnObject(parent));
        };

        let parts: Vec<Vec<JsonPath>> = map.iter()
            .filter_map(|(key, value)| {
                let mut local = parent.clone();
                local.push(PathPart::Key(key.to_string()));

                self.resolve_inner(parts, value, local).ok()
            })
            .collect();

        Ok(parts.into_iter().flatten().collect())
    }
}

#[derive(Debug, Error, PartialEq)]
pub enum JsonSearchParseError {
    #[error("JSON search string should have a '$' or '?' as the first character")]
    MissingRoot,

    #[error("JSON search string should start with a '$' or '?', but got '{0}'")]
    IncorrectRoot(String),
}

impl FromStr for JsonSearch {
    type Err = JsonSearchParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parts = s.split('.');

        let optional = match parts.next() {
            Some("?") => Ok(true),
            Some("$") => Ok(false),
            Some(value) => Err(JsonSearchParseError::IncorrectRoot(value.to_string())),
            None => Err(JsonSearchParseError::MissingRoot),
        }?;

        Ok(JsonSearch {
            parts: parts
                .map(|part| SearchPart::from(part.to_string()))
                .collect(),
            optional,
        })
    }
}

impl<const U: usize> From<[&str; U]> for JsonSearch {
    fn from(value: [&str; U]) -> Self {
        JsonSearch {
            parts: value.iter()
                .map(|part| part.to_string().into())
                .collect(),
            optional: false,
        }
    }
}

impl Display for JsonSearch {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        if self.optional {
            write!(f, "?")?;
        } else {
            write!(f, "$")?;
        }

        for part in &self.parts {
            write!(f, "{}", part)?;
        }

        Ok(())
    }
}

#[cfg(feature = "serde")]
impl Serialize for JsonSearch {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[cfg(feature = "serde")]
impl<'de> Deserialize<'de> for JsonSearch {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_string(JsonSearchVisitor)
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use serde_json::json;
    use crate::json_path::JsonPath;
    use crate::json_search::{JsonSearch, JsonSearchParseError, JsonSearchResolveError};
    use crate::json_search::search_part::SearchPart;

    #[test]
    fn correctly_formatted_json_search_strings_are_parsed_correctly() {
        assert_eq!(JsonSearch::from_str("$").unwrap(), JsonSearch {
            parts: vec![],
            optional: false,
        });

        assert_eq!(JsonSearch::from_str("$.a").unwrap(), JsonSearch {
            parts: vec![SearchPart::Key("a".to_string())],
            optional: false,
        });

        assert_eq!(JsonSearch::from_str("$.a.b").unwrap(), JsonSearch {
            parts: vec![SearchPart::Key("a".to_string()), SearchPart::Key("b".to_string())],
            optional: false,
        });

        assert_eq!(JsonSearch::from_str("$.0").unwrap(), JsonSearch {
            parts: vec![SearchPart::Index(0)],
            optional: false,
        });

        assert_eq!(JsonSearch::from_str("$.*.a").unwrap(), JsonSearch {
            parts: vec![SearchPart::Wildcard, SearchPart::Key("a".to_string())],
            optional: false,
        });

        assert_eq!(JsonSearch::from_str("?.*.a").unwrap(), JsonSearch {
            parts: vec![SearchPart::Wildcard, SearchPart::Key("a".to_string())],
            optional: true,
        });
    }

    #[test]
    fn incorrectly_formatted_json_search_strings_return_errors() {
        assert_eq!(JsonSearch::from_str(""), Err(JsonSearchParseError::IncorrectRoot("".to_string())));
        assert_eq!(JsonSearch::from_str("!"), Err(JsonSearchParseError::IncorrectRoot("!".to_string())));
    }

    #[test]
    fn root_value_is_resolved_correctly() {
        let target_value = json!("Hello world");
        let search = JsonSearch::default();

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from([]),
        ]));
    }

    #[test]
    fn nested_object_value_is_resolved_correctly() {
        let target_value = json!({
            "a": 10
        });
        let search = JsonSearch::from(["a"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a"]),
        ]));
    }

    #[test]
    fn deeply_nested_object_value_is_resolved_correctly() {
        let target_value = json!({
            "a": { "b": { "c": { "d": 10 } } }
        });
        let search = JsonSearch::from(["a", "b", "c", "d"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a", "b", "c", "d"]),
        ]));
    }

    #[test]
    fn array_exact_index_is_resolved_correctly() {
        let target_value = json!([
            10
        ]);
        let search = JsonSearch::from(["0"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0"]),
        ]));
    }

    #[test]
    fn deeply_nested_array_exact_index_is_resolved_correctly() {
        let target_value = json!([
            [10, [20, [30, [40]]]]
        ]);
        let search = JsonSearch::from(["0", "1", "1", "1", "0"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0", "1", "1", "1", "0"]),
        ]));
    }

    #[test]
    fn multiple_values_in_array_wildcard_are_resolved_correctly_using_a_wildcard() {
        let target_value = json!([10, 20, 30, 40, 50]);
        let search = JsonSearch::from(["*"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0"]),
            JsonPath::from(["1"]),
            JsonPath::from(["2"]),
            JsonPath::from(["3"]),
            JsonPath::from(["4"]),
        ]));
    }

    #[test]
    fn multiple_nested_value_in_array_wildcard_are_resolved_correctly() {
        let target_value = json!([
            { "a": 10 },
            { "a": 20 },
            { "a": 30 },
            { "a": 40 },
            { "a": 50 }
        ]);
        let search = JsonSearch::from(["*", "a"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0", "a"]),
            JsonPath::from(["1", "a"]),
            JsonPath::from(["2", "a"]),
            JsonPath::from(["3", "a"]),
            JsonPath::from(["4", "a"]),
        ]));
    }

    #[test]
    fn different_nested_value_in_array_wildcard_are_resolved_correctly() {
        let target_value = json!([
            { "a": 10 },
            { "a": 20 },
            { "a": 30 },
            { "b": 40 },
            { "b": 50 }
        ]);
        let search = JsonSearch::from(["*", "a"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0", "a"]),
            JsonPath::from(["1", "a"]),
            JsonPath::from(["2", "a"]),
        ]));
    }

    #[test]
    fn different_nested_value_types_in_array_wildcard_are_resolved_correctly() {
        let target_value = json!([
            { "a": { "b": 10 } },
            { "a": { "b": 20 } },
            { "a": { "b": 30 } },
            { "a": 40 },
            { "a": 50 }
        ]);
        let search = JsonSearch::from(["*", "a", "b"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0", "a", "b"]),
            JsonPath::from(["1", "a", "b"]),
            JsonPath::from(["2", "a", "b"]),
        ]));
    }

    #[test]
    fn nested_array_wildcards_are_resolved_correctly() {
        let target_value = json!([
            { "a": [ 10, 20 ] },
            { "a": [ 10, 20 ] },
            { "b": [ 10, 20 ] },
        ]);

        let search = JsonSearch::from(["*", "a", "*"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["0", "a", "0"]),
            JsonPath::from(["0", "a", "1"]),
            JsonPath::from(["1", "a", "0"]),
            JsonPath::from(["1", "a", "1"]),
        ]));
    }

    #[test]
    fn multiple_values_in_object_wildcard_are_resolved_correctly_using_a_wildcard() {
        let target_value = json!({
            "a": 10,
            "b": 20,
            "c": 30,
            "d": 40,
            "e": 50,
        });
        let search = JsonSearch::from(["*"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a"]),
            JsonPath::from(["b"]),
            JsonPath::from(["c"]),
            JsonPath::from(["d"]),
            JsonPath::from(["e"]),
        ]));
    }

    #[test]
    fn multiple_nested_value_in_object_wildcard_are_resolved_correctly() {
        let target_value = json!({
            "a": [10],
            "b": [20],
            "c": [30],
            "d": [40],
            "e": [50],
        });
        let search = JsonSearch::from(["*", "0"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a", "0"]),
            JsonPath::from(["b", "0"]),
            JsonPath::from(["c", "0"]),
            JsonPath::from(["d", "0"]),
            JsonPath::from(["e", "0"]),
        ]));
    }

    #[test]
    fn different_nested_value_in_object_wildcard_are_resolved_correctly() {
        let target_value = json!({
            "a": [10, 60],
            "b": [20, 70],
            "c": [30, 80],
            "d": [40],
            "e": [50],
        });
        let search = JsonSearch::from(["*", "1"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a", "1"]),
            JsonPath::from(["b", "1"]),
            JsonPath::from(["c", "1"]),
        ]));
    }

    #[test]
    fn nested_object_wildcards_are_resolved_correctly() {
        let target_value = json!({
            "a": { "b": 10, "c": 20 },
            "b": { "d": 10, "e": 20 },
            "c": 10,
        });

        let search = JsonSearch::from(["*", "*"]);

        let result = search.resolve(&target_value);

        assert_eq!(result, Ok(vec![
            JsonPath::from(["a", "b"]),
            JsonPath::from(["a", "c"]),
            JsonPath::from(["b", "d"]),
            JsonPath::from(["b", "e"]),
        ]));
    }

    #[test]
    fn required_search_returns_an_err_when_a_path_does_not_exist() {
        assert_eq!(JsonSearch::from(["b"]).resolve(&json!({ "a": 10 })), Err(JsonSearchResolveError::MissingRequiredKey(JsonPath::default(), "b".to_string())));
        assert_eq!(JsonSearch::from(["b"]).resolve(&json!("hello world")), Err(JsonSearchResolveError::NotAnObject(JsonPath::default())));
        assert_eq!(JsonSearch::from(["0"]).resolve(&json!({ "a": 10 })), Err(JsonSearchResolveError::NotAnArray(JsonPath::default())));
    }
}
