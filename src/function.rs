use crate::crepr::{Representable, RepresentationContext};
use crate::definition::CVarDef;
use crate::stmt::handle_stmt;
use crate::ty::{CStructInfo, CType};
use crate::{base::OngoingCodegen, definition::CVarDecl};
use rustc_middle::{mir::BasicBlockData, ty::{Instance, Tuple}};
use std::collections::HashSet;
use std::fmt::{self, Debug};

use tracing::{debug, trace};

#[derive(Clone, PartialEq, Eq)]
pub struct CFunction {
    name: String,
    signature: Vec<CVarDef>,
    body: FnBody,
    return_ty: CType,
}

impl Representable for CFunction {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        self.return_ty.repr(f, _context)?;
        write!(f, " {}(", self.name)?;
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            arg.repr(f, _context)?;
        }
        write!(f, ") ")?;

        self.body.repr(f, _context)
    }
}

impl Debug for CFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr(
            f,
            &RepresentationContext {
                indent: 1,
                indent_string: "\t".into(),
                include_newline: true,
                include_comments: true,
                ..Default::default()
            },
        )
    }
}

impl CFunction {
    pub fn new(name: String, return_ty: CType) -> Self {
        Self { name: name, signature: Vec::new(), body: FnBody::new(), return_ty: return_ty }
    }

    pub fn is_main(&self) -> bool {
        self.name == "main"
    }

    pub fn push(&mut self, line: &str, newline: bool, indent: usize) {
        self.body.push(line, newline, indent);
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
        self.body.add_local_var(var);
    }

    #[allow(dead_code)]
    pub fn validate_fn(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

#[derive(Clone, PartialEq, Eq)]
pub struct FnBody {
    local_decl: Vec<CVarDecl>,
    body: String,
}

impl Representable for FnBody {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        write!(f, "{{\n")?;
        for decl in &self.local_decl {
            write!(f, "{}", _context.indent_string.as_str().repeat(_context.indent))?;
            decl.repr(f, _context)?;
            write!(f, "\n")?;
        }
        write!(f, "{}", self.body)?;
        write!(f, "}}")
    }
}

impl Debug for FnBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.repr(
            f,
            &RepresentationContext {
                indent: 1,
                indent_string: "\t".into(),
                include_newline: true,
                include_comments: true,
                ..Default::default()
            },
        )
    }
}

impl FnBody {
    pub fn new() -> Self {
        Self { local_decl: Vec::new(), body: String::new() }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    // TODO: Probably not a good idea to push string lines directly
    // We should change this as soon as possible.
    // Also, hardcoded indent string will cause problems, since RepresentationContext also defines indendation_string.
    pub fn push(&mut self, line: &str, newline: bool, indent: usize) {
        self.body.push_str(&("    ".repeat(indent) + line));
        if newline {
            self.body.push('\n');
        }
    }

    pub fn add_local_var(&mut self, var: CVarDecl) {
        self.local_decl.push(var);
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.body.clear();
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
            _ => {c_ty = CType::from(&ty);}
        }
        let c_var = CVarDef::new(name, c_ty);
        c_fn.add_var_decl(CVarDecl::new(c_var, None));
    }
}

fn handle_bbs<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    mir: &rustc_middle::mir::Body<'tcx>,
    c_fn: &mut CFunction,
) {
    let blocks = &mir.basic_blocks;
    for (_last_bb_id, block_data) in blocks.into_iter().enumerate() {
        let block_data: &BasicBlockData = block_data;

        let statements: &Vec<rustc_middle::mir::Statement<'_>> = &block_data.statements;

        // Print basic block for debugging. TODO should probably depend on a cli argument.
        c_fn.body.push(&format!("// Basic Block: {:?}", block_data), true, 1);

        for stmt in statements {
            handle_stmt(tcx, ongoing_codegen, stmt, c_fn);
        }
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
    handle_bbs(tcx, ongoing_codegen, mir, &mut c_fn);

    // Add return statement
    c_fn.push("return var0;", true, 1);

    // If is main prefix with "_"
    if c_fn.is_main() {
        c_fn.name = format!("_{}", c_fn.name);
    }

    ongoing_codegen.context.get_mut_functions().push(c_fn);
}
