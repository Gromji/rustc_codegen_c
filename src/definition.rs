use crate::crepr::Representable;
use crate::crepr::RepresentationContext;
use crate::expression::Expression;
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
    pub fn get_id(&self) -> usize {
        self.local_id
    }
    pub fn get_name(&self) -> String {
        self.name.clone()
    }
    pub fn get_type(&self) -> &CType {
        &self.ty
    }
}

impl Representable for CVarDef {
    fn repr(&self, f: &mut (dyn fmt::Write), _context: &RepresentationContext) -> fmt::Result {
        let mut new_context = _context.clone();
        new_context.var_name = Some(self.name.clone());
        self.ty.repr(f, &new_context)
    }
}

impl Debug for CVarDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

#[derive(Clone, PartialEq, Eq)]

pub struct CVarDecl {
    var: CVarDef,
    value: Option<Box<Expression>>,
}

impl CVarDecl {
    pub fn new(var: CVarDef, value: Option<Box<Expression>>) -> Self {
        Self { var, value }
    }
    pub fn get_var(&self) -> &CVarDef {
        &self.var
    }

    #[allow(dead_code)]
    pub fn get_var_type(&self) -> &CType {
        &self.var.ty
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> String {
        self.var.get_name()
    }
    pub fn get_id(&self) -> usize {
        self.var.local_id
    }
}

impl Representable for CVarDecl {
    fn repr(&self, f: &mut (dyn fmt::Write), _context: &RepresentationContext) -> fmt::Result {
        match &self.value {
            Some(value) => {
                self.var.repr(f, _context)?;
                write!(f, " = ")?;
                value.repr(f, _context)?;
                write!(f, ";")
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
