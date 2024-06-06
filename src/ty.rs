use crate::crepr::{Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::function::CodegenFunctionCx;
use crate::structure::{CComposite, CStructDef, CTaggedUnionDef};
use crate::utils;
use rustc_hir::def_id::DefId;
use rustc_middle::ty::{ParamEnv, Ty};
use std::fmt::{self, Debug};
use tracing::{debug, debug_span, error};

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
    Struct(CCompositeInfo),
    TaggedUnion(CCompositeInfo),
    // enums in rust are not the same as enums in C, they are more like tagged unions
    Union(CCompositeInfo),

    // Luka: I doubt plain enums will be useful for any rust type. Enums with no fields are still tagged unions in rust from what I can tell
    Enum,
    Pointer(Box<CType>),
    Array(Box<CType>, usize),
    FunctionPtr(Box<CFuncPtrInfo>),
}

impl CType {
    pub fn as_composite_info(&self) -> CCompositeInfo {
        match self {
            CType::Struct(info) | CType::Union(info) | CType::TaggedUnion(info) => info.clone(),
            _ => panic!("Not a composite type"),
        }
    }
}

impl Representable for CType {
    fn repr(
        &self,
        f: &mut (dyn fmt::Write),
        context: &crate::crepr::RepresentationContext,
    ) -> fmt::Result {
        match self {
            // Custom struct for Rust's Unit type
            CType::Unit => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("__Unit{}", ptrs);
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

            CType::TaggedUnion(info) | CType::Struct(info) | CType::Union(info) => {
                let struct_name = &info.name;
                let ptrs = "*".repeat(context.n_ptr.into());
                let c_type = format!("{struct_name}{ptrs}");
                match &context.var_name {
                    Some(name) => write!(f, "{c_type} {name}"),
                    None => write!(f, "{c_type}"),
                }
            }
            // Incorrectly implemented, needs fix!
            CType::Enum => {
                let ptrs = "*".repeat(context.n_ptr.into());
                let _c_type = format!("enum{}", ptrs);
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

                if context.n_ptr > 0 {
                    write!(f, "{} {}", "*".repeat(context.n_ptr.into()), context.get_variable_name())
                } else if *size as u32 == 0 {
                    write!(f, " {}[]", context.get_variable_name())
                } else {
                    write!(f, " {}[{}]", context.get_variable_name(), size)
                }
            }
            CType::FunctionPtr(func_info) => {
                let child_context: RepresentationContext = Default::default();
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
        self.default_repr(f)
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
            CUIntTy::UInt128 => "__uint128_t",
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CCompositeInfo {
    pub name: String,
    pub ctx_idx: usize, // index into the context's struct list
}

impl CCompositeInfo {
    pub fn new(name: &String, ctx_idx: usize) -> Self {
        Self {
            name: name.clone(),
            ctx_idx,
        }
    }
}

impl<'tcx> CodegenFunctionCx<'tcx, '_> {
    pub fn rust_to_c_type(&mut self, ty: &Ty<'tcx>) -> CType {
        if self.ty_to_c.contains_key(ty) {
            return self.ty_to_c[ty].clone();
        }

        let ctype = self.rust_to_c_type_internal(ty);

        self.ty_to_c.insert(*ty, ctype.clone());
        return ctype;
    }

    /// This function might need to be changed further down the line to ensure that the names are truly unique and that we dont exceed any size lim
    /// TODO: cosnider hashing? would make the generated file a lot less readable though, so perhaps set to a flag so it doesn't annoy us during development
    fn composite_name(&self, def: DefId, field_names: &Vec<String>) -> String {
        // it's improtant to preserve information before :: to avoid conflicts with types from other crates, if we ever get around to that

        let mut name = self.tcx.def_path_str(def).replace("::", "__");

        for generic_name in field_names {
            name.push_str(&format!("_{}", generic_name));
        }

        return name;
    }

    // this is will also need to be changed
    fn wrapper_union_name(&self, def: DefId, field_names: &Vec<String>) -> String {
        let name = self.composite_name(def, field_names);

        "__WRAPPER_UNION_".to_string() + &name
    }

    fn tuple_name(&self, fields: &Vec<CType>) -> String {
        if fields.len() == 0 {
            // not really necesasary but I've gotten used to the name
            return "__Unit".to_string();
        }

        let mut name = String::from("t_");

        for ty in fields {
            name.push_str(&format!("{:?}", ty));
        }

        return name.replace("*", "__PTR__");
    }

    fn rust_to_c_type_internal(&mut self, ty: &Ty<'tcx>) -> CType {
        let _span = debug_span!("rust_to_c_type").entered();

        match ty.kind() {
            rustc_middle::ty::Tuple(types) => {
                let field_types: Vec<CType> =
                    types.iter().map(|x| self.rust_to_c_type(&x)).collect();

                // Luka: from what I can tell, rust treats tuples of size 1 as just the type itself. (e.g (i32, ) = i32))
                if field_types.len() == 1 {
                    debug!("Tuple with singular field: {:?}, unboxed", field_types);
                    return field_types[0].clone();
                }

                let c_struct = CStructDef {
                    name: self.tuple_name(&field_types),
                    fields: field_types
                        .iter()
                        .enumerate()
                        .map(|(idx, ty)| CVarDef::new(idx, format!("field_{idx}"), ty.clone()))
                        .collect(),
                };

                let struct_info = self
                    .ongoing_codegen
                    .context
                    .add_composite(&CComposite::Struct(c_struct));

                return CType::Struct(struct_info);
            }

            rustc_middle::ty::Adt(adt_def, generic_fields) => match adt_def.adt_kind() {
                rustc_middle::ty::AdtKind::Struct => {
                    let field_names: Vec<String> = generic_fields
                        .iter()
                        .map(|x| format!("{}", x.as_type().unwrap()))
                        .collect();

                    let c_struct = CStructDef {
                        name: self.composite_name(adt_def.did(), &field_names),
                        fields: adt_def
                            .all_fields()
                            .enumerate()
                            .map(|(idx, field)| {
                                CVarDef::new(
                                    idx,
                                    field.name.to_string(),
                                    self.rust_to_c_type(&field.ty(self.tcx, generic_fields)),
                                )
                            })
                            .collect(),
                    };

                    let struct_info = self
                        .ongoing_codegen
                        .context
                        .add_composite(&CComposite::Struct(c_struct));

                    return CType::Struct(struct_info);
                }

                rustc_middle::ty::AdtKind::Union => {
                    let field_names: Vec<String> = generic_fields
                        .iter()
                        .map(|x| format!("{}", x.as_type().unwrap()))
                        .collect();

                    let c_struct = CStructDef {
                        name: self.composite_name(adt_def.did(), &field_names),
                        fields: adt_def
                            .all_fields()
                            .enumerate()
                            .map(|(idx, field)| {
                                CVarDef::new(
                                    idx,
                                    field.name.to_string(),
                                    self.rust_to_c_type(&field.ty(self.tcx, generic_fields)),
                                )
                            })
                            .collect(),
                    };

                    let struct_info = self
                        .ongoing_codegen
                        .context
                        .add_composite(&CComposite::Union(c_struct));

                    return CType::Union(struct_info);
                },

                rustc_middle::ty::AdtKind::Enum => {
                    let mut variant_infos: Vec<CVarDef> = Vec::new();

                    for (idx, variant) in adt_def.variants().iter().enumerate() {
                        let variant_fields: Vec<CType> = variant
                            .fields
                            .iter()
                            .map(|field| {
                                let ty = field.ty(self.tcx, generic_fields);
                                self.rust_to_c_type(&ty)
                            })
                            .collect();

                        // build and save structs for each of the enum variants
                        let c_struct = CStructDef {
                            name: self.composite_name(
                                variant.def_id,
                                &variant_fields.iter().map(|x| format!("{:?}", x)).collect(),
                            ),
                            fields: variant_fields
                                .iter()
                                .enumerate()
                                .map(|(idx, ty)| {
                                    CVarDef::new(idx, format!("field_{idx}"), ty.clone())
                                })
                                .collect(),
                        };

                        let composite_info = self
                            .ongoing_codegen
                            .context
                            .add_composite(&CComposite::Struct(c_struct));

                        // save each struct as a CVarDef for the union
                        variant_infos.push(CVarDef::new(
                            idx,
                            format!("variant_{}", idx),
                            CType::Struct(composite_info),
                        ));
                    }

                    // create the actual union as a type
                    let union_def = CStructDef {
                        name: self.wrapper_union_name(
                            adt_def.did(),
                            &variant_infos
                                .iter()
                                .map(|x| x.get_name().clone())
                                .collect::<Vec<String>>(),
                        ),
                        fields: variant_infos,
                    };

                    let union_info = self
                        .ongoing_codegen
                        .context
                        .add_composite(&CComposite::Union(union_def));

                    let discr_type = self.rust_to_c_type(&ty.discriminant_ty(self.tcx));

                    let generic_field_names: Vec<String> = generic_fields
                        .iter()
                        .map(|x| format!("{}", x.as_type().unwrap()))
                        .collect();

                    let tagged_union_def = CTaggedUnionDef::new(
                        self.composite_name(adt_def.did(), &generic_field_names),
                        discr_type,
                        CType::Union(union_info),
                    );

                    let tagged_union_info = self
                        .ongoing_codegen
                        .context
                        .add_composite(&CComposite::TaggedUnion(tagged_union_def));

                    return CType::TaggedUnion(tagged_union_info);
                }
            },

            rustc_middle::ty::Closure(_def, args) => {
                let closure = args.as_closure();
                self.rust_to_c_type(&closure.tupled_upvars_ty())
            }

            rustc_middle::ty::Ref(_, ty, _) => CType::Pointer(Box::new(self.rust_to_c_type(ty))),

            rustc_middle::ty::Slice(ty) => self.rust_to_c_type(ty),

            rustc_middle::ty::Array(ty, size) => CType::Array(
                Box::new(self.rust_to_c_type(ty)),
                utils::const_to_usize(size),
            ),

            rustc_middle::ty::FnPtr(s) => {
                let sig = self
                    .tcx
                    .normalize_erasing_late_bound_regions(ParamEnv::reveal_all(), *s);

                let inputs: Vec<CType> = sig
                    .inputs()
                    .iter()
                    .map(|ty| self.rust_to_c_type(ty))
                    .collect();

                let output = self.rust_to_c_type(&sig.output());

                CType::FunctionPtr(Box::new(CFuncPtrInfo {
                    args: inputs,
                    ret: Box::new(output),
                }))
            }

            _ => CType::from(ty),
        }
    }
}

// Do not use this directly, use rust_to_c_type instead.
/// TODO: might aswell get rid of this from, it's a source of bugs
impl<'tcx> From<&Ty<'tcx>> for CType {
    fn from(ty: &Ty) -> Self {
        match ty.kind() {
            rustc_middle::ty::Bool => CType::Bool,
            rustc_middle::ty::Char => CType::Char,
            rustc_middle::ty::Str => CType::Array(Box::new(CType::Char), 0),
            rustc_middle::ty::Uint(u) => CType::UInt(CUIntTy::from(
                u.bit_width().unwrap_or(CUIntTy::DEFAULT_BIT_WIDTH),
            )),
            rustc_middle::ty::Int(i) => CType::Int(CIntTy::from(
                i.bit_width().unwrap_or(CIntTy::DEFAULT_BIT_WIDTH),
            )),
            rustc_middle::ty::Float(float) => CType::Float(CFloatTy::from(float.bit_width())),

            _ => {
                error!("printing unknown type: {:?}", ty);
                CType::Void
            }
        }
    }
}
