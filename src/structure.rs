use crate::{
    crepr::{indent, Representable},
    definition::CVarDef,
    ty::CType,
};
use std::fmt::{self, Debug};

#[derive(Clone)]
pub struct CStruct {
    name: String,
    fields: Vec<CVarDef>,
}

impl CStruct {
    pub fn new(name: String, fields: Option<Vec<CVarDef>>) -> Self {
        match fields {
            Some(fields) => Self { name, fields },
            None => Self { name, fields: Vec::new() },
        }
    }

    #[allow(dead_code)]
    pub fn push(&mut self, field: CVarDef) {
        self.fields.push(field);
    }

    pub fn get_name(&self) -> &String {
        &self.name
    }
    pub fn get_field(&self, idx: usize) -> &CVarDef {
        &self.fields[idx]
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
        self.default_repr(f)
    }
}

impl Representable for CStruct {
    fn repr(
        &self,
        f: &mut std::fmt::Formatter<'_>,
        context: &crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        write!(f, "typedef struct {{\n")?;
        for field in &self.fields {
            indent(f, context)?;
            field.repr(f, context)?;
            write!(f, ";\n")?;
        }
        write!(f, "}} {};", self.name)
    }
}

impl<'tcx> From<&Vec<CType>> for CStruct {
    // We can change the way we name tuples, for now all that matters is its unique.
    fn from(list: &Vec<CType>) -> Self {
        let mut struct_name = String::from("s_");
        for ty in list {
            struct_name.push_str(&format!("{:?}", ty));
        }

        // TODO(@Luka) I hate having to do this, I think we should rework how structures get generated in general
        // should work for now though :)
        struct_name = struct_name.replace("*", "_PTR_");

        let fields = list
            .iter()
            .enumerate()
            .map(|(idx, ty)| CVarDef::new(idx, format!("field_{idx}"), ty.clone()))
            .collect();
        Self::new(struct_name, Some(fields))
    }
}
