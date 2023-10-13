use std::str::FromStr;
use thiserror::Error;
use crate::json_path::path_part::PathPart;

pub mod path_part;

#[derive(Debug, Default)]
#[cfg_attr(test, derive(PartialEq))]
pub struct JsonPath {
    parts: Vec<PathPart>,
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

#[cfg(test)]
mod tests {
    use std::str::FromStr;
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
}
