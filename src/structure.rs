use crate::{
    crepr::{indent, Representable},
    definition::CVarDef,
    ty::CType,
};
use std::fmt::{self, Debug};

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CStructDef {
    pub name: String,
    pub fields: Vec<CVarDef>,
}

impl CStructDef {
    pub fn get_field(&self, idx: usize) -> CVarDef {
        return self.fields[idx].clone();
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct CTaggedUnionDef {
    pub name: String,
    pub tag: CVarDef,
    pub union_var: CVarDef,
}

impl CTaggedUnionDef {
    pub const TAG_NAME: &'static str = "tag";
    pub const UNION_NAME: &'static str = "union_type";

    pub fn new(name: String, tag_type: CType, union_type: CType) -> Self {
        Self {
            name,
            tag: CVarDef::new(0, CTaggedUnionDef::TAG_NAME.to_string(), tag_type),
            union_var: CVarDef::new(1, CTaggedUnionDef::UNION_NAME.to_string(), union_type),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
pub enum CComposite {
    Struct(CStructDef),
    Union(CStructDef),
    TaggedUnion(CTaggedUnionDef),
}

impl CComposite {
    pub fn get_name(&self) -> String {
        match self {
            CComposite::Struct(s) | CComposite::Union(s) => s.get_name(),
            CComposite::TaggedUnion(t) => t.name.clone(),
        }
    }

    pub fn as_struct_def(&self) -> CStructDef {
        match self {
            CComposite::Struct(s) | CComposite::Union(s) => s.clone(),
            _ => panic!("as_struct_def: not a struct type"),
        }
    }

    pub fn as_tagged_union_def(&self) -> CTaggedUnionDef {
        match self {
            CComposite::TaggedUnion(s) => s.clone(),

            _ => panic!("as_union_def: not a tagged union type"),
        }
    }
}

impl Debug for CComposite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

impl Representable for CComposite {
    fn repr(
        &self,
        f: &mut (dyn fmt::Write),
        context: &mut crate::crepr::RepresentationContext,
    ) -> std::fmt::Result {
        match self {
            CComposite::Struct(s) | CComposite::Union(s) => {
                write!(f, "typedef ")?;

                match self {
                    CComposite::Struct(_) => write!(f, "struct ")?,
                    CComposite::Union(_) => write!(f, "union ")?,
                    _ => unreachable!(),
                }

                write!(f, " {{\n")?;
                for field in &s.fields {
                    indent(f, context)?;
                    field.repr(f, context)?;
                    write!(f, ";\n")?;
                }

                write!(f, "}} {};", s.name)
            }

            CComposite::TaggedUnion(t) => {
                write!(f, "typedef struct  {{\n")?;

                indent(f, context)?;
                t.tag.repr(f, context)?;
                write!(f, ";\n")?;

                indent(f, context)?;
                t.union_var.repr(f, context)?;
                write!(f, ";\n")?;

                write!(f, "}} {};", t.name)
            }
        }
    }
}
