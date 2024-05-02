use crate::bb::{self, BasicBlock};
use crate::crepr::{indent, Expression, Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::stmt::Statement;
use crate::ty::{CStructInfo, CType};
use crate::{base::OngoingCodegen, definition::CVarDecl};
use rustc_middle::ty::{Instance, Tuple};
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
            indent(f, context)?;
            write!(f, "bb{}: {{\n", i)?;

            bb.indented_repr(f, context)?;

            indent(f, context)?;
            write!(f, "}}\n")?;
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
    _ongoing_codegen: &mut OngoingCodegen,
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

        let c_var = CVarDef::new(name, CType::from(&ty));
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
        let c_ty: CType;

        match ty.kind() {
            Tuple(t) => {
                let struct_name = _ongoing_codegen.context.get_struct_name(t);
                c_ty = CType::Struct(CStructInfo::from(&struct_name));
            }
            _ => {
                c_ty = CType::from(&ty);
            }
        }
        let c_var = CVarDef::new(name, c_ty);
        c_fn.add_var_decl(CVarDecl::new(c_var, None));
    }
}

#[allow(unused_variables)]
pub fn handle_fn<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    inst: &Instance<'tcx>,
) {
    let mir = tcx.instance_mir(inst.def);
    let mut c_fn = CFunction::new(inst.to_string(), CType::from(&mir.return_ty()));

    // Pring mir of function for debugging
    print_mir(tcx, mir);

    // Handle local variables
    handle_decls(ongoing_codegen, mir, &mut c_fn);

    trace!("{:?}", c_fn);

    // Handle basic blocks
    bb::handle_bbs(tcx, ongoing_codegen, mir, &mut c_fn);

    let mut bb = BasicBlock::new();
    bb.push(Statement::from_expression(Expression::Return {
        value: Box::new(Expression::Variable { local: 0 }),
    }));

    // equivalent to return var0, since we are not handling return values yet
    c_fn.push_bb(bb);

    // If is main prefix with "_"
    if c_fn.is_main() {
        c_fn.name = format!("_{}", c_fn.name);
    }

    ongoing_codegen.context.get_mut_functions().push(c_fn);
}
