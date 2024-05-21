use crate::crepr::Representable;
use crate::crepr::RepresentationContext;
use crate::ty::CType;
use std::fmt::{self, Debug};

#[derive(Clone, PartialEq, Eq)]
pub struct CVarDef {
    local_id: usize,
    name: String,
    ty: CType,
}

impl CVarDef {
    pub fn new(local_id: usize, name: String, ty: CType) -> Self {
        Self { local_id, name, ty }
    }
    pub fn same_type(&self, other: &CVarDef) -> bool {
        self.ty == other.ty
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
}

impl Representable for CVarDef {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        self.ty.repr(
            f,
            &RepresentationContext {
                indent: _context.indent,
                indent_string: _context.indent_string.clone(),
                include_newline: _context.include_newline,
                include_comments: _context.include_comments,
                var_name: Some(self.name.clone()),
                ..Default::default()
            },
        )
    }
}

impl Debug for CVarDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
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
    pub fn get_var_type(&self) -> &CType {
        &self.var.ty
    }
    pub fn get_name(&self) -> String {
        self.var.get_name()
    }
    pub fn get_id(&self) -> usize {
        self.var.local_id
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
                self.var.repr(f, _context)?;
                write!(f, ";")
            }
        }
    }
}

impl Debug for CVarDecl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}
