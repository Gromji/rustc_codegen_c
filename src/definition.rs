use crate::crepr::Representable;
use crate::crepr::RepresentationContext;
use crate::ty::CType;
use std::fmt::{self, Debug};

#[derive(Clone, PartialEq, Eq)]
pub struct CVarDef {
    name: String,
    ty: CType,
}

impl CVarDef {
    pub fn new(name: String, ty: CType) -> Self {
        Self { name, ty }
    }
}

impl Representable for CVarDef {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        match self.ty {
            CType::Array(_, _) | CType::FunctionPtr(_, _) => self.ty.repr(
                f,
                &RepresentationContext {
                    indent: _context.indent,
                    indent_string: _context.indent_string.clone(),
                    include_newline: _context.include_newline,
                    include_comments: _context.include_comments,
                    var_name: Some(self.name.clone()),
                },
            ),
            _ => write!(f, "{:?} {}", self.ty, self.name),
        }
    }
}

impl Debug for CVarDef {
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

#[derive(Clone, PartialEq, Eq)]

pub struct CVarDecl {
    pub var: CVarDef,
    pub value: Option<String>,
}

impl CVarDecl {
    pub fn new(var: CVarDef, value: Option<String>) -> Self {
        Self { var, value }
    }
}

impl Representable for CVarDecl {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        match &self.value {
            Some(value) => {
                self.var.repr(f, _context)?;
                write!(f, " = {};", value)
            }
            None => {
                self.var.repr(f, _context);
                write!(f, ";")
            }
        }
    }
}

impl Debug for CVarDecl {
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
