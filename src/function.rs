use crate::bb::{self, BasicBlock};
use crate::crepr::{indent, Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::ty::CType;
use crate::{base::OngoingCodegen, definition::CVarDecl};
use rustc_middle::ty::{self, Instance, SymbolName, TyCtxt, TypeFoldable};
use std::collections::HashSet;
use std::fmt::{self, Debug};

use tracing::{debug, trace};

#[derive(Clone, PartialEq, Eq)]
pub struct CFunction {
    name: String,
    signature: Vec<CVarDef>,
    local_decl: Vec<CVarDecl>,
    basic_blocks: Vec<BasicBlock>,
    return_ty: CType,
}

pub struct CodegenFunctionCx<'tcx, 'ccx> {
    pub tcx: rustc_middle::ty::TyCtxt<'tcx>,
    pub mir: &'tcx rustc_middle::mir::Body<'tcx>,
    pub ongoing_codegen: &'ccx mut OngoingCodegen,
    pub instance: Instance<'tcx>,

    pub(crate) ty_to_c: &'ccx mut std::collections::HashMap<ty::Ty<'tcx>, CType>,
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

    pub fn ctype_from_cache(&self, ty: &ty::Ty<'tcx>) -> Option<CType> {
        self.ty_to_c.get(ty).cloned()
    }
}

impl Representable for CFunction {
    fn repr(&self, f: &mut (dyn fmt::Write), context: &RepresentationContext) -> fmt::Result {
        let mut new_context = context.clone();
        new_context.cur_fn = Some(&self);
        self.return_ty.repr(f, &new_context)?;
        write!(f, " {}(", self.name)?;
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.repr(f, &new_context)?;
        }
        write!(f, ") ")?;

        write!(f, "{{\n")?;
        for decl in &self.local_decl {
            indent(f, &new_context)?;
            decl.repr(f, &new_context)?;
            write!(f, "\n")?;
        }

        for bb in self.basic_blocks.iter() {
            bb.repr(f, &new_context)?;
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
            return_ty: return_ty,
        }
    }

    pub fn is_main(&self) -> bool {
        self.name == "main"
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn push_bb(&mut self, bb: BasicBlock) {
        self.basic_blocks.push(bb);
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
) {
    let mir = tcx.instance_mir(inst.def);
    let mut fn_cx = CodegenFunctionCx {
        tcx,
        ongoing_codegen,
        instance: inst,
        mir,
        ty_to_c: rust_to_c_map,
    };

    let mut c_fn = CFunction::new(
        format_fn_name(&tcx.symbol_name(inst)),
        fn_cx.rust_to_c_type(&mir.return_ty()),
    );

    // Pring mir of function for debugging
    print_mir(tcx, mir);

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
