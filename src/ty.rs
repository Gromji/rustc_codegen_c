use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CType {
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
            CType::Array(ty, size) => write!(f, "{} {}[{}]", ty, NAME_TOKEN, size),
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
