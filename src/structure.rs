use std::fmt::{self, Debug};

use crate::{
    crepr::{Representable, RepresentationContext},
    definition::CVarDef,
    ty::CType
};
use rustc_middle::ty::{Ty, List};

#[derive(Clone)]
pub struct CStruct {
    name: String,
    fields: Vec<CVarDef>,
}

impl CStruct {
    pub fn new(name: String, fields: Option<Vec<CVarDef>>) -> Self {
        match fields{
            Some(fields) => Self { name, fields },
            None => Self { name, fields: Vec::new() }
        }
    }

    pub fn push(&mut self, field: CVarDef) {
        self.fields.push(field);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }

    #[allow(dead_code)]
    pub fn validate_struct(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

impl PartialEq for CStruct {
    fn eq(&self, other: &CStruct) -> bool {
        let mut result = true;
        if self.fields.len() == other.fields.len() {
            for (i, field) in self.fields.iter().enumerate() {
                result &= field.same_type(&other.fields[i]);
            }
        } else {
            result = false;
        }
        result
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
                ..Default::default()
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

impl<'tcx> From<&List<Ty<'tcx>>> for CStruct {
    // We can change the way we name tuples, for now all that matters is its unique.
    fn from(list: &List<Ty>) -> Self {
        let mut struct_name = String::from("s_");
        for ty in list {
            struct_name.push_str(&format!("{:?}", ty));
        }
        // Change names instead of 
        let fields = list.iter()
            .enumerate().map(|(idx, ty)| CVarDef::new(format!("field_{idx}"), CType::from(&ty)))
            .collect();
        Self::new(struct_name, Some(fields))
    }
}
