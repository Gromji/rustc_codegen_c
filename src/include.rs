use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Include {
    pub name: String,
    pub is_system: bool,
}

impl fmt::Display for Include {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.is_system {
            write!(f, "#include <{}>", self.name)
        } else {
            write!(f, "#include \"{}\"", self.name)
        }
    }
}

impl Include {
    pub fn new(name: String, is_system: bool) -> Self {
        Self { name, is_system }
    }
}
