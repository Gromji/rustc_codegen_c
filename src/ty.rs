use std::fmt;

use rustc_middle::ty::Ty;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CType {
    Unit,
    Void,
    Bool,
    Char,
    UChar,
    Short,
    UShort,
    Int,
    UInt,
    Long,
    ULong,
    LongLong,
    ULongLong,
    Float,
    Double,
    LongDouble,
    Struct,
    Union,
    Enum,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    FunctionPtr(Box<CType>, Vec<CType>),
}

pub const NAME_TOKEN: &str = "<<name>>";

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Custom struct for Rust's Unit type
            CType::Unit => write!(f, "struct __Unit"),
            CType::Void => write!(f, "void"),
            CType::Bool => write!(f, "bool"),
            CType::Char => write!(f, "char"),
            CType::UChar => write!(f, "unsigned char"),
            CType::Short => write!(f, "short"),
            CType::UShort => write!(f, "unsigned short"),
            CType::Int => write!(f, "int"),
            CType::UInt => write!(f, "unsigned int"),
            CType::Long => write!(f, "long"),
            CType::ULong => write!(f, "unsigned long"),
            CType::LongLong => write!(f, "long long"),
            CType::ULongLong => write!(f, "unsigned long long"),
            CType::Float => write!(f, "float"),
            CType::Double => write!(f, "double"),
            CType::LongDouble => write!(f, "long double"),
            CType::Struct => write!(f, "struct"),
            CType::Union => write!(f, "union"),
            CType::Enum => write!(f, "enum"),
            CType::Pointer(ty) => write!(f, "{}*", ty),
            CType::Array(ty, size) => {
                if *size as u32 == 0 {
                    write!(f, "{} {}[]", ty, NAME_TOKEN)
                } else {
                    write!(f, "{} {}[{}]", ty, NAME_TOKEN, size)
                }
            }
            CType::FunctionPtr(ty, args) => {
                write!(f, "{} (*{}", ty, NAME_TOKEN)?;
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                write!(f, ")")
            }
        }
    }
}

// TODO: This is not yet done. Probably shouldn't be returning Unit for everything
impl<'tcx> From<&Ty<'tcx>> for CType {
    fn from(ty: &Ty) -> Self {
        match ty.kind() {
            rustc_middle::ty::Bool => CType::Bool,
            rustc_middle::ty::Char => CType::Char,
            rustc_middle::ty::Uint(_) => CType::UInt,
            rustc_middle::ty::Int(_) => CType::Int,
            rustc_middle::ty::Float(_) => CType::Double,
            rustc_middle::ty::FnPtr(_) => CType::FunctionPtr(Box::new(CType::Void), Vec::new()),
            rustc_middle::ty::Ref(_, ty, _) => CType::Pointer(Box::new(CType::from(ty))),
            rustc_middle::ty::Array(ty, size) => {
                CType::Array(Box::new(CType::from(ty)), todo!("Set array size"))
            }
            rustc_middle::ty::Adt(adt, _) => match adt.adt_kind() {
                rustc_middle::ty::AdtKind::Struct => CType::Struct,
                rustc_middle::ty::AdtKind::Union => CType::Union,
                rustc_middle::ty::AdtKind::Enum => CType::Enum,
            },
            _ => CType::Unit,
        }
    }
}
