use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq)]
pub enum PathPart {
    Key(String),
    Index(usize),
}

impl From<String> for PathPart {
    fn from(value: String) -> Self {
        if let Ok(index) = value.parse() {
            return PathPart::Index(index);
        }

        PathPart::Key(value)
    }
}

impl Display for PathPart {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let string = match self {
            PathPart::Key(value) => value.to_string(),
            PathPart::Index(value) => value.to_string(),
        };

        write!(f, "{}", string)
    }
}
