use std::fmt::{self, Debug};

use rustc_middle::ty::{FnSig, Ty, VariantDef};

use crate::crepr::{Representable, RepresentationContext};

#[derive(Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum CType {
    Unit,
    Void,
    Bool,
    Char,
    Int(CIntTy),
    UInt(CUIntTy),
    Float(CFloatTy),
    Struct(CStructInfo),
    Union,
    Enum,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    FunctionPtr(Box<CFuncPtrInfo>),
}

impl Representable for CType {
    fn repr(
        &self,
        f: &mut fmt::Formatter<'_>,
        context: &crate::crepr::RepresentationContext,
    ) -> fmt::Result {
        match self {
            // Custom struct for Rust's Unit type
            CType::Unit => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("struct __Unit{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Void => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("void{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Bool => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("bool{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Char => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("char32_t{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Int(i) => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("{}{}", i.name_str(), ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::UInt(u) => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("{}{}", u.name_str(), ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Float(float) => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("{}{}", float.name_str(), ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            CType::Struct(info) => {
                let struct_name = &info.name;
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("struct {struct_name}{ptrs}");
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            // Incorrectly implemented, needs fix!
            CType::Union => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("union{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            // Incorrectly implemented, needs fix!
            CType::Enum => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("enum{}", ptrs);
                match &context.var_name {
                    Some(name) => write!(f, "enum {name}"),
                    None => write!(f, "enum"),
                }
            }
            CType::Pointer(ty) => {
                let mut child_context: RepresentationContext = (*context).clone();
                child_context.n_ptr += 1;
                ty.repr(f, &child_context)
            }
            CType::Array(ty, size) => {
                // Change this later.
                let child_context: RepresentationContext = Default::default();
                ty.repr(f, &child_context)?;
                if *size as u32 == 0 {
                    match &context.var_name {
                        Some(name) => write!(f, " {}[]", name),
                        None => panic!("Variable must have a name"),
                    }
                } else {
                    match &context.var_name {
                        Some(name) => write!(f, " {}[{}]", name, size),
                        None => panic!("Variable must have a name"),
                    }
                }
            }
            CType::FunctionPtr(func_info) => {
                let mut child_context: RepresentationContext = Default::default();
                func_info.ret.repr(f, &child_context)?;
                match &context.var_name {
                    Some(name) => write!(f, " (*{})(", name)?,
                    None => panic!("Variable must have a name"),
                }

                for (i, arg) in func_info.args.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    arg.repr(f, &child_context)?;
                }

                if func_info.args.len() == 0 {
                    CType::Void.repr(f, &child_context)?;
                }
                write!(f, ")")
            }
        }
    }
}

impl Debug for CType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr(
            f,
            &crate::crepr::RepresentationContext {
                indent: 1,
                indent_string: "\t".into(),
                include_newline: true,
                include_comments: true,
                ..Default::default()
            },
        )
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
        CFuncPtrInfo { args: types[0..last_idx].to_vec(), ret: Box::new(types[last_idx].clone()) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CStructInfo {
    pub name: String,
}

impl From<&VariantDef> for CStructInfo {
    fn from(value: &VariantDef) -> Self {
        // TODO: Change this. to_string uses unsafe!!!
        CStructInfo { name: value.name.to_string() }
    }
}
impl From<&String> for CStructInfo {
    fn from(value: &String) -> Self {
        CStructInfo { name: value.clone() }
    }
}

// TODO: This is not yet done.
impl<'tcx> From<&Ty<'tcx>> for CType {
    fn from(ty: &Ty) -> Self {
        match ty.kind() {
            rustc_middle::ty::Bool => CType::Bool,
            rustc_middle::ty::Char => CType::Char,
            rustc_middle::ty::Str => CType::Array(Box::new(CType::Char), 0),
            rustc_middle::ty::Uint(u) => {
                CType::UInt(CUIntTy::from(u.bit_width().unwrap_or(CUIntTy::DEFAULT_BIT_WIDTH)))
            }
            rustc_middle::ty::Int(i) => {
                CType::Int(CIntTy::from(i.bit_width().unwrap_or(CIntTy::DEFAULT_BIT_WIDTH)))
            }
            rustc_middle::ty::Float(float) => CType::Float(CFloatTy::from(float.bit_width())),
            rustc_middle::ty::FnPtr(s) => {
                CType::FunctionPtr(Box::new(CFuncPtrInfo::from(s.skip_binder())))
            }
            rustc_middle::ty::Ref(_, ty, _) => CType::Pointer(Box::new(CType::from(ty))),
            rustc_middle::ty::Array(ty, size) => {
                // TODO: Move value extraction to utils::try_usize or think of something better
                CType::Array(
                    Box::new(CType::from(ty)),
                    size.try_to_scalar().unwrap().to_u64().unwrap().try_into().unwrap(),
                )
            }
            rustc_middle::ty::Slice(ty) => CType::from(ty),
            rustc_middle::ty::Adt(adt, _) => match adt.adt_kind() {
                rustc_middle::ty::AdtKind::Struct => {
                    CType::Struct(CStructInfo::from(adt.variants().iter().next().unwrap()))
                }
                rustc_middle::ty::AdtKind::Union => CType::Union,
                rustc_middle::ty::AdtKind::Enum => CType::Enum,
            },
            _ => {
                println!("printing unknown type: {:?}", ty.kind());
                CType::Unit
            }
        }
    }
}
