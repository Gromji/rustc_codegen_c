use crate::bb::{self, BasicBlock};
use crate::crepr::{indent, Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::expression::Expression;
use crate::ty::CType;
use crate::{base::OngoingCodegen, definition::CVarDecl};
use rustc_const_eval::interpret::ConstAllocation;
use rustc_middle::mir::interpret::{AllocId, GlobalAlloc};
use rustc_middle::ty::{self, Instance, SymbolName, TyCtxt, TypeFoldable};
// use stable_mir::mir::alloc::{AllocId, GlobalAlloc};
use std::collections::HashSet;
use std::fmt::{self, Debug};

use tracing::{debug, trace, warn};

#[derive(Clone, PartialEq, Eq)]
pub struct CFunction {
    name: String,
    signature: Vec<CVarDef>,
    local_decl: Vec<CVarDecl>,
    basic_blocks: Vec<BasicBlock>,
    is_main: bool,
    return_ty: CType,
}

pub struct CodegenFunctionCx<'tcx, 'ccx> {
    pub tcx: rustc_middle::ty::TyCtxt<'tcx>,
    pub mir: &'ccx rustc_middle::mir::Body<'tcx>,
    pub ongoing_codegen: &'ccx mut OngoingCodegen,

    instance: Instance<'tcx>,
    pub crate_num: usize,

    pub(crate) ty_to_c: &'ccx mut std::collections::HashMap<ty::Ty<'tcx>, CType>,
    pub(crate) alloc_to_c: &'ccx mut std::collections::HashMap<AllocId, Expression>,
}

impl<'tcx> CodegenFunctionCx<'tcx, '_> {
    pub fn monomorphize<T>(&self, value: T) -> T
    where
        T: Copy + TypeFoldable<TyCtxt<'tcx>>,
    {
        self.instance.instantiate_mir_and_normalize_erasing_regions(
            self.tcx,
            ty::ParamEnv::reveal_all(),
            ty::EarlyBinder::bind(value),
        )
    }

    pub fn ty_for_local(&self, local: rustc_middle::mir::Local) -> ty::Ty<'tcx> {
        self.monomorphize(self.mir.local_decls[local].ty)
    }

    #[allow(dead_code)]
    pub fn ctype_from_cache(&self, ty: &ty::Ty<'tcx>) -> Option<CType> {
        self.ty_to_c.get(ty).cloned()
    }

    fn handle_cosnt_alloc(&mut self, alloc: ConstAllocation, alloc_id: AllocId) -> Expression {
        let inner_alloc = alloc.inner();

        let alloc_bytes: Vec<u8> = inner_alloc
            .inspect_with_uninit_and_ptr_outside_interpreter(0..inner_alloc.len())
            .into();

        let ptrs = inner_alloc.provenance().ptrs();

        let mut ptr_declrs: Vec<(usize, Expression)> = Vec::new();

        for (offset, prov) in ptrs.iter() {
            let offset = u32::try_from(offset.bytes_usize()).unwrap();
            // Check if this allocation is a function
            let reloc_target_alloc = self.tcx.global_alloc(prov.alloc_id());

            if let GlobalAlloc::Function(finstance) = reloc_target_alloc {
                let fn_name = format_fn_name(&self.tcx.symbol_name(finstance));

                ptr_declrs.push((offset as usize, Expression::Constant { value: fn_name }));
            } else {
                warn!("Not a function alloc: {:?}", reloc_target_alloc);

                ptr_declrs.push((
                    offset as usize,
                    Expression::Constant { value: format!("ptr_{}", offset) },
                ));
            }
        }

        let alloc_name = format!("ALLOC_{}_CRATE_{}", alloc_id.0, self.crate_num);

        let static_alloc =
            crate::alloc::StaticAllocation::new(alloc_name.clone(), alloc_bytes, ptr_declrs);

        self.ongoing_codegen.context.add_static(static_alloc);

        Expression::Constant { value: format!("&{}", alloc_name.clone()) }
    }

    pub fn handle_global_decl(&mut self, alloc: AllocId) -> Expression {
        if self.alloc_to_c.contains_key(&alloc) {
            return self.alloc_to_c[&alloc].clone();
        }

        let global_alloc = self.tcx.global_alloc(alloc);

        let c_alloc = match global_alloc {
            GlobalAlloc::Memory(const_alloc) => self.handle_cosnt_alloc(const_alloc, alloc),

            _ => {
                panic!("Global alloc not handled: {:?}", global_alloc);
            }
        };

        self.alloc_to_c.insert(alloc, c_alloc.clone());

        c_alloc
    }
}

impl Representable for CFunction {
    fn repr(&self, f: &mut (dyn fmt::Write), context: &mut RepresentationContext) -> fmt::Result {
        let mut new_context = context.clone();
        new_context.cur_fn = Some(&self);
        self.return_ty.repr(f, &mut new_context)?;
        write!(f, " {}(", self.name)?;
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.repr(f, &mut new_context)?;
        }
        write!(f, ") ")?;

        write!(f, "{{\n")?;
        for decl in &self.local_decl {
            indent(f, &new_context)?;
            decl.repr(f, &mut new_context)?;
            write!(f, "\n")?;
        }

        for bb in self.basic_blocks.iter() {
            bb.repr(f, &mut new_context)?;
        }

        write!(f, "}}")
    }
}

impl Debug for CFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.default_repr(f)
    }
}

impl CFunction {
    pub fn new(name: String, return_ty: CType) -> Self {
        Self {
            name: name,
            signature: Vec::new(),
            local_decl: Vec::new(),
            basic_blocks: Vec::new(),
            is_main: false,
            return_ty: return_ty,
        }
    }

    pub fn is_main(&self) -> bool {
        self.is_main
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn push_bb(&mut self, bb: BasicBlock) {
        self.basic_blocks.push(bb);
    }

    pub fn clear_bb(&mut self) {
        self.basic_blocks.clear();
    }

    pub fn as_prototype(&self) -> String {
        let mut prototype = format!("{:?} {}(", self.return_ty, self.name);
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                prototype.push_str(", ");
            }
            prototype.push_str(format!("{:?}", &arg).as_str());
        }
        prototype.push_str(");");
        prototype
    }

    pub fn add_signature_var(&mut self, var: CVarDef) {
        self.signature.push(var);
    }

    pub fn add_var_decl(&mut self, var: CVarDecl) {
        self.local_decl.push(var);
    }

    pub fn get_local_var(&self, idx: usize) -> &CVarDef {
        for local_var in &self.local_decl {
            if local_var.get_id() == idx {
                return local_var.get_var();
            }
        }
        for sig_var in &self.signature {
            if sig_var.get_id() == idx {
                return sig_var;
            }
        }
        panic!("Local variable with id {} not found", idx);
    }

    #[allow(dead_code)]
    pub fn get_local_var_name(&self, idx: usize) -> String {
        self.get_local_var(idx).get_name()
    }

    #[allow(dead_code)]
    pub fn validate_fn(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

fn print_mir<'tcx>(tcx: rustc_middle::ty::TyCtxt<'tcx>, mir: &rustc_middle::mir::Body<'tcx>) {
    let mut buf = Vec::new();
    rustc_middle::mir::pretty::write_mir_fn(tcx, mir, &mut |_, _| Ok(()), &mut buf).unwrap();

    debug!("{}", &String::from_utf8_lossy(&buf).into_owned());
}

fn handle_decls<'tcx>(ctx: &mut CodegenFunctionCx<'tcx, '_>, c_fn: &mut CFunction) {
    let local_decls = &ctx.mir.local_decls;

    // TODO: Maybe use debug_info to get variable names
    //let debug_info = &mir.var_debug_info;
    //for info in debug_info.iter().enumerate() {
    //    writeln!(std::io::stdout(), "Debug Info: {:?}", info).unwrap();
    //}

    // Create set of usize
    let mut set: HashSet<usize> = HashSet::new();

    ctx.mir.args_iter().for_each(|arg| {
        let ty = local_decls[arg].ty;
        let name = format!("var{}", arg.index());

        // add index to set
        set.insert(arg.index());

        let c_var = CVarDef::new(arg.index(), name, ctx.rust_to_c_type(&ty));

        c_fn.add_signature_var(c_var);
    });

    for (idx, decl) in local_decls.into_iter().enumerate() {
        let decl: &rustc_middle::mir::LocalDecl = decl;
        // check if idx is in set
        if set.contains(&idx) {
            continue;
        }

        let ty = decl.ty;
        let name = format!("var{}", idx);
        let c_ty = ctx.rust_to_c_type(&ty);
        let c_var = CVarDef::new(idx, name, c_ty);
        c_fn.add_var_decl(CVarDecl::new(c_var, None));
    }
}

/*TODO(Luka) this sort of function name extraction is not ideal, but might be necessary to avoid collisions, needs more thought, should also look at other codegens*/
pub fn format_fn_name(name: &SymbolName) -> String {
    let mut name = name.name.to_string();
    name = name.replace('.', "_").replace('$', "_");
    name
}

#[allow(unused_variables)]
pub fn handle_fn<'tcx, 'ccx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    inst: Instance<'tcx>,
    rust_to_c_map: &'ccx mut std::collections::HashMap<ty::Ty<'tcx>, CType>,
    alloc_to_c_map: &'ccx mut std::collections::HashMap<AllocId, Expression>,
    crate_num: usize,
) {
    // this resolves generic parameters to concrete types
    let mono_mir = inst.instantiate_mir_and_normalize_erasing_regions(
        tcx,
        ty::ParamEnv::reveal_all(),
        ty::EarlyBinder::bind(tcx.instance_mir(inst.def).clone()),
    );

    let mut fn_cx = CodegenFunctionCx {
        tcx,
        ongoing_codegen,
        instance: inst,
        mir: &mono_mir,
        ty_to_c: rust_to_c_map,
        crate_num,
        alloc_to_c: alloc_to_c_map,
    };

    let mut c_fn = CFunction::new(
        format_fn_name(&tcx.symbol_name(inst)),
        fn_cx.rust_to_c_type(&mono_mir.return_ty()),
    );

    c_fn.is_main = inst.to_string() == "main";

    // Pring mir of function for debugging
    print_mir(tcx, &mono_mir);

    // Handle local variables
    handle_decls(&mut fn_cx, &mut c_fn);

    trace!("{:?}", c_fn);

    // Handle basic blocks
    bb::handle_bbs(&mut fn_cx, &mut c_fn);

    // If is main prefix with "_"
    if c_fn.is_main() {
        c_fn.name = format!("_{}", c_fn.name);
    }

    fn_cx.ongoing_codegen.context.get_mut_functions().push(c_fn);
}
