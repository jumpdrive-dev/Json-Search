use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum SearchPart {
    Key(String),
    Index(usize),
    Wildcard,
}

impl From<String> for SearchPart {
    fn from(value: String) -> Self {
        if &value == "*" {
            return SearchPart::Wildcard;
        }

        if let Ok(index) = value.parse() {
            return SearchPart::Index(index);
        }

        SearchPart::Key(value)
    }
}

impl Display for SearchPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SearchPart::Key(key) => write!(f, "{}", key),
            SearchPart::Index(index) => write!(f, "{}", index),
            SearchPart::Wildcard => write!(f, "*"),
        }
    }
}
