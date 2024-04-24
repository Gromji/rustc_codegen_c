use std::fmt::{self, Debug};

use crate::{
    crepr::{Representable, RepresentationContext},
    definition::CVarDef,
};

#[derive(Clone, PartialEq, Eq)]
pub struct CStruct {
    name: String,
    fields: Vec<CVarDef>,
}

impl CStruct {
    pub fn new(name: String) -> Self {
        Self { name, fields: Vec::new() }
    }

    pub fn push(&mut self, field: CVarDef) {
        self.fields.push(field);
    }

    #[allow(dead_code)]
    pub fn validate_struct(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

impl Debug for CStruct {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr(
            f,
            &RepresentationContext {
                indent: 1,
                indent_string: "\t".into(),
                include_newline: true,
                include_comments: true,
                var_name: None,
            },
        )
    }
}

impl Representable for CStruct {
    fn repr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        _context: &crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        write!(f, "struct {} {{\n", self.name)?;
        for field in &self.fields {
            write!(f, "{}", _context.indent_string.as_str().repeat(_context.indent))?;
            field.repr(f, _context)?;
            write!(f, ";\n")?;
        }
        write!(f, "}};")
    }
}
