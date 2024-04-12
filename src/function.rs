use crate::base::OngoingCodegen;
use crate::definition::CVarDef;
use crate::ty::CType;
use rustc_middle::{
    mir::{BasicBlockData, Rvalue, StatementKind},
    ty::{print::with_no_trimmed_paths, Instance},
};
use std::fmt;
use std::io::Write;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CFunction {
    name: String,
    signature: Vec<CVarDef>,
    body: FnBody,
    return_ty: CType,
}

impl CFunction {
    pub fn new(name: String, signature: Vec<CVarDef>, return_ty: CType) -> Self {
        Self { name, signature, body: FnBody::new(String::new()), return_ty }
    }

    pub fn is_main(&self) -> bool {
        self.name == "main"
    }

    pub fn push(&mut self, line: &str, newline: bool, indent: usize) {
        self.body.push(line, newline, indent);
    }

    pub fn as_prototype(&self) -> String {
        let mut prototype = format!("{} {}(", self.return_ty, self.name);
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                prototype.push_str(", ");
            }
            prototype.push_str(&arg.to_string());
        }
        prototype.push_str(");");
        prototype
    }

    #[allow(dead_code)]
    pub fn validate_fn(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FnBody {
    body: String,
}

impl fmt::Display for FnBody {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{{\n {} \n}}\n\n", self.body)
    }
}

impl FnBody {
    pub fn new(body: String) -> Self {
        Self { body }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.body.is_empty()
    }

    pub fn push(&mut self, line: &str, newline: bool, indent: usize) {
        self.body.push_str(&("    ".repeat(indent) + line));
        if newline {
            self.body.push('\n');
        }
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.body.clear();
    }
}

impl fmt::Display for CFunction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}(", self.return_ty, self.name)?;
        for (i, arg) in self.signature.iter().enumerate() {
            if i > 0 {
                write!(f, ", ")?;
            }
            write!(f, "{}", arg)?;
        }
        write!(f, ") ")?;

        write!(f, "{}", self.body)
    }
}

#[allow(unused_variables)]
pub fn handle_fn<'tcx>(
    tcx: rustc_middle::ty::TyCtxt<'tcx>,
    ongoing_codegen: &mut OngoingCodegen,
    inst: &Instance<'tcx>,
) {
    let mir = tcx.instance_mir(inst.def);

    let blocks = &mir.basic_blocks;
    for (last_bb_id, block_data) in blocks.into_iter().enumerate() {
        let block_data: &BasicBlockData = block_data;

        let statements = &block_data.statements;
        with_no_trimmed_paths!({
            for stmt in statements {
                writeln!(std::io::stdout(), "Statement: {:?}", stmt).unwrap();
                writeln!(std::io::stdout(), "Statement Kind: {:?}", stmt.kind).unwrap();
                match &stmt.kind {
                    StatementKind::Assign(val) => {
                        let place = &val.0;
                        let rvalue = &val.1;
                        writeln!(std::io::stdout(), "Place: {:?}", place).unwrap();
                        writeln!(std::io::stdout(), "Rvalue: {:?}", rvalue).unwrap();

                        match rvalue {
                            Rvalue::Repeat(operand, len) => {
                                writeln!(std::io::stdout(), "Repeat",).unwrap();
                            }
                            Rvalue::Ref(a, b, c) => {
                                writeln!(std::io::stdout(), "Ref",).unwrap();
                            }
                            Rvalue::ThreadLocalRef(region) => {
                                writeln!(std::io::stdout(), "ThreadLocalRef",).unwrap();
                            }
                            Rvalue::AddressOf(a, b) => {
                                writeln!(std::io::stdout(), "AddressOf",).unwrap();
                            }
                            Rvalue::Len(a) => {
                                writeln!(std::io::stdout(), "Len",).unwrap();
                            }
                            Rvalue::Cast(kind, operand, ty) => {
                                writeln!(std::io::stdout(), "Cast",).unwrap();
                            }
                            Rvalue::BinaryOp(op, operand1) => {
                                writeln!(std::io::stdout(), "BinaryOp",).unwrap();
                            }
                            Rvalue::CheckedBinaryOp(op, operand1) => {
                                writeln!(std::io::stdout(), "CheckedBinaryOp",).unwrap();
                            }
                            Rvalue::NullaryOp(op, ty) => {
                                writeln!(std::io::stdout(), "NullaryOp",).unwrap();
                            }
                            Rvalue::UnaryOp(op, operand) => {
                                writeln!(std::io::stdout(), "UnaryOp",).unwrap();
                            }
                            Rvalue::Discriminant(place) => {
                                writeln!(std::io::stdout(), "Discriminant",).unwrap();
                            }
                            Rvalue::Aggregate(kind, operands) => {
                                writeln!(std::io::stdout(), "Aggregate",).unwrap();
                            }
                            Rvalue::ShallowInitBox(kind, operands) => {
                                writeln!(std::io::stdout(), "ShallowInitBox",).unwrap();
                            }
                            Rvalue::CopyForDeref(kind) => {
                                writeln!(std::io::stdout(), "CopyForDeref",).unwrap();
                            }
                            Rvalue::Use(operand) => match operand.constant() {
                                Some(constant) => match constant.const_ {
                                    rustc_middle::mir::Const::Unevaluated(c, t) => {
                                        writeln!(
                                            std::io::stdout(),
                                            "Const: {:?} {:?}",
                                            tcx.const_eval_poly(c.def),
                                            t
                                        )
                                        .unwrap();
                                    }
                                    _ => {}
                                },
                                None => {
                                    writeln!(std::io::stdout(), "Use: {:?}", operand).unwrap();
                                }
                            },
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}
