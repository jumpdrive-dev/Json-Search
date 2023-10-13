#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
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
