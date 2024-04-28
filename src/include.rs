use std::fmt::{self, Debug};

use crate::crepr::Representable;

#[derive(Clone, PartialEq, Eq)]
pub struct Include {
    pub name: String,
    pub is_system: bool,
}

impl Representable for Include {
    fn repr(
        &self,
        f: &mut fmt::Formatter<'_>,
        _context: &crate::crepr::RepresentationContext,
    ) -> fmt::Result {
        if self.is_system {
            write!(f, "#include <{}>", self.name)
        } else {
            write!(f, "#include \"{}\"", self.name)
        }
    }
}

impl Debug for Include {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

impl Include {
    pub fn new(name: String, is_system: bool) -> Self {
        Self { name, is_system }
    }
}
