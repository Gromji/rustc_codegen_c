use std::fmt;

use rustc_middle::ty::{Ty, FnSig};

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CType {
    Unit,
    Void,
    Bool,
    Char,
    Int(CIntTy),
    UInt(CUIntTy),
    Float(CFloatTy),
    Struct,
    Union,
    Enum,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    FunctionPtr(Box<CFuncPtrInfo>),
}

pub const NAME_TOKEN: &str = "<<name>>";

impl fmt::Display for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            // Custom struct for Rust's Unit type
            CType::Unit => write!(f, "struct __Unit"),
            CType::Void => write!(f, "void"),
            CType::Bool => write!(f, "bool"),
            CType::Char => write!(f, "wchar_t"),
            CType::Int(i) => write!(f, "{}", i.name_str()),
            CType::UInt(u) => write!(f, "{}", u.name_str()),
            CType::Float(float) => write!(f, "{}", float.name_str()),
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
            CType::FunctionPtr(func_info) => {
                write!(f, "{} (*{})(", func_info.ret, NAME_TOKEN)?;
                for (i, arg) in func_info.args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", arg)?;
                }
                if func_info.args.len() == 0 {
                    write!(f, "{}", CType::Void)?;
                }
                write!(f, ")")
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CIntTy {
    Int8,
    Int16,
    Int32,
    Int64,
    Int128,
}
impl CIntTy {
    const DEFAULT_BIT_WIDTH: u64 = 64;
    pub fn name_str(&self) -> &'static str {
        match *self {
            CIntTy::Int8 => "int8_t",
            CIntTy::Int16 => "int16_t",
            CIntTy::Int32 => "int32_t",
            CIntTy::Int64 => "int64_t",
            // Might cause trouble on some gcc versions
            CIntTy::Int128 => "__int128_t",
        }
    }

    pub fn bit_width(&self) -> u64 {
        match *self {
            CIntTy::Int8 => 8,
            CIntTy::Int16 => 16,
            CIntTy::Int32 => 32,
            CIntTy::Int64 => 64,
            CIntTy::Int128 => 128,
        }
    }
}

impl From<u64> for CIntTy {
    fn from(bits: u64) -> Self {
        match bits {
            8 => CIntTy::Int8,
            16 => CIntTy::Int16,
            32 => CIntTy::Int32,
            64 => CIntTy::Int64,
            128 => CIntTy::Int128,
            _ => panic!("Unsupported int type size"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CUIntTy {
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    UInt128,
}
impl CUIntTy {
    const DEFAULT_BIT_WIDTH: u64 = 64;
    pub fn name_str(&self) -> &'static str {
        match *self {
            CUIntTy::UInt8 => "uint8_t",
            CUIntTy::UInt16 => "uint16_t",
            CUIntTy::UInt32 => "uint32_t",
            CUIntTy::UInt64 => "uint64_t",
            // Might cause trouble on some gcc versions
            CUIntTy::UInt128 => "unsigned __int128_t",
        }
    }

    pub fn bit_width(&self) -> u64 {
        match *self {
            CUIntTy::UInt8 => 8,
            CUIntTy::UInt16 => 16,
            CUIntTy::UInt32 => 32,
            CUIntTy::UInt64 => 64,
            CUIntTy::UInt128 => 128,
        }
    }
}

impl From<u64> for CUIntTy {
    fn from(bits: u64) -> Self {
        match bits {
            8 => CUIntTy::UInt8,
            16 => CUIntTy::UInt16,
            32 => CUIntTy::UInt32,
            64 => CUIntTy::UInt64,
            128 => CUIntTy::UInt128,
            _ => panic!("Unsupported uint type size"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CFloatTy {
    Float16,
    Float32,
    Float64,
    Float128,
}
impl CFloatTy {
    pub fn name_str(&self) -> &'static str {
        match *self {
            CFloatTy::Float16 => "float",
            CFloatTy::Float32 => "float",
            CFloatTy::Float64 => "double",
            // Might cause trouble on some gcc versions
            CFloatTy::Float128 => "__float128",
        }
    }

    pub fn bit_width(&self) -> u64 {
        match *self {
            CFloatTy::Float16 => 16,
            CFloatTy::Float32 => 32,
            CFloatTy::Float64 => 64,
            CFloatTy::Float128 => 128,
        }
    }
}

impl From<u64> for CFloatTy {
    fn from(bits: u64) -> Self {
        match bits {
            16 => CFloatTy::Float16,
            32 => CFloatTy::Float32,
            64 => CFloatTy::Float64,
            128 => CFloatTy::Float128,
            _ => panic!("Unsupported float type size"),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFuncPtrInfo {
    pub args: Vec<CType>,
    pub ret: Box<CType>,
}
impl<'tcx> From<FnSig<'tcx>> for CFuncPtrInfo {
    fn from(value: FnSig<'tcx>) -> Self {
        let types: Vec<CType> = value.inputs_and_output.iter().map(|x| CType::from(&x)).collect();
        let last_idx = types.len() - 1;
        CFuncPtrInfo{args: types[0..last_idx].to_vec(), ret: Box::new(types[last_idx].clone())}
    }
}



// TODO: This is not yet done. Probably shouldn't be returning Unit for everything
impl<'tcx> From<&Ty<'tcx>> for CType {
    fn from(ty: &Ty) -> Self {
        match ty.kind() {
            rustc_middle::ty::Bool => CType::Bool,
            rustc_middle::ty::Char => CType::Char,
            rustc_middle::ty::Uint(u) => CType::UInt(CUIntTy::from(u.bit_width().unwrap_or(CUIntTy::DEFAULT_BIT_WIDTH))),
            rustc_middle::ty::Int(i) => CType::Int(CIntTy::from(i.bit_width().unwrap_or(CIntTy::DEFAULT_BIT_WIDTH))),
            rustc_middle::ty::Float(float) => CType::Float(CFloatTy::from(float.bit_width())),
            rustc_middle::ty::FnPtr(s) => CType::FunctionPtr(Box::new(CFuncPtrInfo::from(s.skip_binder()))),
            rustc_middle::ty::Ref(_, ty, _) => CType::Pointer(Box::new(CType::from(ty))),
            rustc_middle::ty::Array(ty, size) => {
                CType::Array(Box::new(CType::from(ty)), size.try_to_scalar().unwrap().to_u64().unwrap().try_into().unwrap())
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
