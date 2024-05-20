use crate::bb::{self, BasicBlock};
use crate::crepr::{indent, Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::ty::{rust_to_c_type, CType};
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
}

impl Representable for CFunction {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
        self.return_ty.repr(f, context)?;
        write!(f, " {}(", self.name)?;
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.repr(f, context)?;
        }
        write!(f, ") ")?;

        write!(f, "{{\n")?;
        for decl in &self.local_decl {
            indent(f, context)?;
            decl.repr(f, context)?;
            write!(f, "\n")?;
        }

        for (i, bb) in self.basic_blocks.iter().enumerate() {
            bb.repr(f, context)?;
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

    pub fn get_local_var(&self, idx: usize) -> &CVarDecl {
        &self.local_decl[idx]
    }
    pub fn get_local_var_name(&self, idx: usize) -> String {
        self.local_decl[idx].get_name()
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

fn handle_decls<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    mir: &rustc_middle::mir::Body<'tcx>,
    c_fn: &mut CFunction,
) {
    let local_decls = &mir.local_decls;

    // TODO: Maybe use debug_info to get variable names
    //let debug_info = &mir.var_debug_info;
    //for info in debug_info.iter().enumerate() {
    //    writeln!(std::io::stdout(), "Debug Info: {:?}", info).unwrap();
    //}

    // Create set of usize
    let mut set: HashSet<usize> = HashSet::new();

    mir.args_iter().for_each(|arg| {
        let ty = local_decls[arg].ty;
        let name = format!("var{}", arg.index());

        // add index to set
        set.insert(arg.index());

        let c_var = CVarDef::new(name, rust_to_c_type(tcx, ongoing_codegen, &ty));
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
        let c_ty = rust_to_c_type(tcx, ongoing_codegen, &ty);
        let c_var = CVarDef::new(name, c_ty);
        c_fn.add_var_decl(CVarDecl::new(c_var, None));
    }
}

/*TODO(Luka) this sort of function name extraction is not ideal, but might be necessary to avoid collisions, needs more thought, should also look at other codegens*/
pub fn format_fn_name(name: &SymbolName) -> String {
    let mut name = name.to_string();
    name = name.replace('.', "_").replace('$', "_");
    name
}

#[allow(unused_variables)]
pub fn handle_fn<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    inst: Instance<'tcx>,
) {
    let mir = tcx.instance_mir(inst.def);
    let fn_cx =
        CodegenFunctionCx { tcx: tcx, ongoing_codegen: ongoing_codegen, instance: inst, mir };

    let mut c_fn = CFunction::new(
        format_fn_name(&tcx.symbol_name(inst)),
        rust_to_c_type(fn_cx.tcx, fn_cx.ongoing_codegen, &mir.return_ty()),
    );

    // Pring mir of function for debugging
    print_mir(tcx, mir);

    // Handle local variables
    handle_decls(tcx, fn_cx.ongoing_codegen, mir, &mut c_fn);

    trace!("{:?}", c_fn);

    // Handle basic blocks
    bb::handle_bbs(&fn_cx, &mut c_fn);

    // If is main prefix with "_"
    if c_fn.is_main() {
        c_fn.name = format!("_{}", c_fn.name);
    }

    fn_cx.ongoing_codegen.context.get_mut_functions().push(c_fn);
}
