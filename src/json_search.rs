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

}

impl JsonSearch {
    pub fn resolve(&self, target: &Value) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        self.resolve_inner(&self.parts, target)
    }

    fn resolve_inner(&self, parts: &[SearchPart], target: &Value) -> Result<Vec<JsonPath>, JsonSearchResolveError> {
        todo!()
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
    use crate::json_search::{JsonSearch, JsonSearchParseError};
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
        let search = JsonSearch::default();
    }
}
