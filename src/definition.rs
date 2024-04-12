use crate::ty::CType;
use crate::ty::NAME_TOKEN;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CVarDef {
    name: String,
    ty: CType,
}

impl CVarDef {
    pub fn new(name: String, ty: CType) -> Self {
        Self { name, ty }
    }
}

impl fmt::Display for CVarDef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.ty {
            CType::Array(_, _) | CType::FunctionPtr(_, _) => {
                write!(f, "{}", format!("{}", self.ty).replace(NAME_TOKEN, &self.name))
            }
            _ => write!(f, "{} {}", self.ty, self.name),
        }
    }
}
