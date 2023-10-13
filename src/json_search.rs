use std::str::FromStr;
use serde_json::Value;
use thiserror::Error;
use crate::json_path::JsonPath;
use crate::json_search::search_part::SearchPart;

pub mod search_part;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct JsonSearch {
    parts: Vec<SearchPart>,
    optional: bool,
}

#[derive(Debug, Error, PartialEq)]
pub enum JsonSearchResolveError {
    #[error("")]
    NotAnObject,

    #[error("")]
    MissingRequiredPart,

    #[error("")]
    MissingPart,
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
        self.resolve_inner(&self.parts, target)
    }

    fn resolve_inner(&self, parts: &[SearchPart], target: &Value) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let mut results = vec![];
        let remaining = if parts.len() > 0 {
            &parts[1..]
        } else {
            &parts[0..]
        };

        if let Some(part) = parts.get(0) {
            match part {
                SearchPart::Key(key) => {}
                SearchPart::Index(_) => {}
                SearchPart::Wildcard => {}
            }
        };

        Ok(results)
    }

    fn resolve_key(&self, parts: &[SearchPart], target: &Value, key: &String) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        let Value::Object(map) = target else {
            return Err(JsonSearchResolveError::NotAnObject);
        };

        let key_value = self.resolve_optional(map.get(key))?;

        match key_value {
            Some(value) => self.resolve_inner(parts, value),
            None => Ok(vec![])
        }
    }

    fn resolve_optional<T>(&self, optional: Option<T>) -> Result<Option<T>, JsonSearchResolveError> {
        match optional {
            Some(value) => Ok(Some(value)),
            None if self.optional => Ok(None),
            _ => Err(JsonSearchResolveError::MissingRequiredPart),
        }
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
    fn multiple_values_in_array_are_resolved_correctly_using_a_wildcard() {
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
    fn multiple_nested_value_in_array_are_resolved_correctly() {
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
    fn different_nested_value_in_array_are_resolved_correctly() {
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
    fn required_search_returns_an_err_when_a_path_does_not_exist() {
        assert_eq!(JsonSearch::from(["b"]).resolve(&json!({ "a": 10 })), Err(JsonSearchResolveError::MissingPart));
        assert_eq!(JsonSearch::from(["b"]).resolve(&json!("hello world")), Err(JsonSearchResolveError::MissingPart));
        assert_eq!(JsonSearch::from(["0"]).resolve(&json!({ "a": 10 })), Err(JsonSearchResolveError::MissingPart));
    }

    // #[test]
    // fn optional_search_with_no_matches_resolves_correctly() {
    //     let target_value = json!({
    //         "a": 10,
    //     });
    // }
}
