use crate::{
    bb::BasicBlockIdentifier,
    crepr::{indent, Representable, RepresentationContext},
};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpType {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl From<&rustc_middle::mir::BinOp> for BinOpType {
    fn from(value: &rustc_middle::mir::BinOp) -> Self {
        match value {
            rustc_middle::mir::BinOp::Add => BinOpType::Add,
            rustc_middle::mir::BinOp::Sub => BinOpType::Sub,
            rustc_middle::mir::BinOp::Mul => BinOpType::Mul,
            rustc_middle::mir::BinOp::Div => BinOpType::Div,
            rustc_middle::mir::BinOp::Rem => BinOpType::Mod,
            rustc_middle::mir::BinOp::BitAnd => BinOpType::And,
            rustc_middle::mir::BinOp::BitOr => BinOpType::Or,
            rustc_middle::mir::BinOp::BitXor => BinOpType::Xor,
            rustc_middle::mir::BinOp::Shl => BinOpType::Shl,
            rustc_middle::mir::BinOp::Shr => BinOpType::Shr,
            rustc_middle::mir::BinOp::Eq => BinOpType::Eq,
            rustc_middle::mir::BinOp::Ne => BinOpType::Ne,
            rustc_middle::mir::BinOp::Lt => BinOpType::Lt,
            rustc_middle::mir::BinOp::Le => BinOpType::Le,
            rustc_middle::mir::BinOp::Gt => BinOpType::Gt,
            rustc_middle::mir::BinOp::Ge => BinOpType::Ge,
            rustc_middle::mir::BinOp::AddUnchecked => BinOpType::Add,
            rustc_middle::mir::BinOp::SubUnchecked => BinOpType::Sub,
            rustc_middle::mir::BinOp::MulUnchecked => BinOpType::Mul,
            rustc_middle::mir::BinOp::ShlUnchecked => BinOpType::Shl,
            rustc_middle::mir::BinOp::ShrUnchecked => BinOpType::Shr,
            rustc_middle::mir::BinOp::Cmp => BinOpType::Eq,

            rustc_middle::mir::BinOp::Offset => BinOpType::Add, // TODO this is a guess
        }
    }
}

impl Representable for BinOpType {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        match self {
            BinOpType::Add => write!(f, "+"),
            BinOpType::Sub => write!(f, "-"),
            BinOpType::Mul => write!(f, "*"),
            BinOpType::Div => write!(f, "/"),
            BinOpType::Mod => write!(f, "%"),
            BinOpType::And => write!(f, "&"),
            BinOpType::Or => write!(f, "|"),
            BinOpType::Xor => write!(f, "^"),
            BinOpType::Shl => write!(f, "<<"),
            BinOpType::Shr => write!(f, ">>"),
            BinOpType::Eq => write!(f, "=="),
            BinOpType::Ne => write!(f, "!="),
            BinOpType::Lt => write!(f, "<"),
            BinOpType::Le => write!(f, "<="),
            BinOpType::Gt => write!(f, ">"),
            BinOpType::Ge => write!(f, ">="),
        }
    }
}
impl fmt::Display for BinOpType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinOpType::Add => write!(f, "add"),
            BinOpType::Sub => write!(f, "sub"),
            BinOpType::Mul => write!(f, "mul"),
            BinOpType::Div => write!(f, "div"),
            BinOpType::Mod => write!(f, "mod"),
            BinOpType::And => write!(f, "and"),
            BinOpType::Or => write!(f, "or"),
            BinOpType::Xor => write!(f, "xor"),
            BinOpType::Shl => write!(f, "shl"),
            BinOpType::Shr => write!(f, "shr"),
            BinOpType::Eq => write!(f, "eq"),
            BinOpType::Ne => write!(f, "ne"),
            BinOpType::Lt => write!(f, "lt"),
            BinOpType::Le => write!(f, "le"),
            BinOpType::Gt => write!(f, "gt"),
            BinOpType::Ge => write!(f, "ge"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum UnaryOpType {
    Neg,
    Not,
    BitNot,
}

impl Representable for UnaryOpType {
    fn repr(&self, f: &mut fmt::Formatter<'_>, _context: &RepresentationContext) -> fmt::Result {
        match self {
            UnaryOpType::Neg => write!(f, "-"),
            UnaryOpType::Not => write!(f, "!"),
            UnaryOpType::BitNot => write!(f, "~"),
        }
    }
}

pub struct Constant {
    pub value: String, // TODO this is a copout, we should have a proper representation for constants
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Constant {
        value: String,
    },
    Variable {
        local: usize,
        idx: Option<usize>,
    }, // TODO this might not be an appropriate representation, especially if we plan to add debug info into the mix
    Assignment {
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    BinaryOp {
        op: BinOpType,
        lhs: Box<Expression>,
        rhs: Box<Expression>,
    },
    UnaryOp {
        op: UnaryOpType,
        val: Box<Expression>,
    },
    Struct {
        name: Box<Expression>,
        fields: Vec<Expression>,
    },
    Array {
        fields: Vec<Expression>,
    },
    Goto {
        target: BasicBlockIdentifier,
    },
    Return {
        value: Box<Expression>,
    },
    SwitchJump {
        value: Box<Expression>,
        cases: Vec<(Box<Expression>, BasicBlockIdentifier)>,
        default: BasicBlockIdentifier,
    },
    NoOp {},
    FnCall {
        function: Box<Expression>,
        args: Vec<Expression>,
    },
}

impl Representable for Expression {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
        match self {
            Expression::Constant { value } => {
                write!(f, "{}", value)
            }

            Expression::Variable { local, idx } => {
                let name = match context.cur_fn {
                    Some(cur_fn) => cur_fn.get_local_var_name(*local),
                    None => format!("var{}", local), // This should never be the case.
                };
                match idx {
                    Some(idx) => {
                        write!(f, "{}[{}]", name, idx)
                    }
                    None => {
                        write!(f, "{}", name)
                    }
                }
            }

            Expression::Assignment { lhs, rhs } => {
                // {} = {}
                lhs.repr(f, context)?;
                write!(f, " = ")?;
                rhs.repr(f, context)?;
                Ok(())
            }
            Expression::BinaryOp { op, lhs, rhs } => {
                // {} {} {} (eg. {1} {+} {5})
                lhs.repr(f, context)?;
                write!(f, " ")?;
                op.repr(f, context)?;
                write!(f, " ")?;
                rhs.repr(f, context)?;
                Ok(())
            }

            Expression::UnaryOp { op, val } => {
                // {}{} (eg. {-}{5})
                op.repr(f, context)?;
                val.repr(f, context)?;
                Ok(())
            }

            Expression::Struct { name, fields } => {
                // (struct {}){ {} } (eg. (struct struct_name) { {1}, {2} })
                write!(f, "(")?;
                name.repr(f, context)?;
                write!(f, "){{ ")?;
                for (i, field) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    field.repr(f, context)?;
                }
                write!(f, " }}")
            }
            Expression::Array { fields } => {
                // {}, {}, ..., {}; (eg. {var1[0]=1}, {var1[1]=2}, {var1[2]=5};)
                for (i, field) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    field.repr(f, context)?;
                }
                Ok(())
            }

            Expression::NoOp {} => {
                if context.include_comments {
                    write!(f, "/* NoOp */")
                } else {
                    write!(f, "")
                }
            }

            Expression::Return { value } => {
                write!(f, "return ")?;
                value.repr(f, context)?;
                Ok(())
            }

            Expression::Goto { target } => {
                write!(f, "goto ")?;
                target.repr(f, context)?;
                Ok(())
            }

            Expression::SwitchJump { value, cases, default } => {
                write!(f, "switch (")?;
                value.repr(f, context)?;
                write!(f, ")")?;
                write!(f, " {{\n")?;

                for (case, target) in cases {
                    indent(f, context)?;
                    write!(f, "case ")?;
                    case.repr(f, context)?;
                    write!(f, ": goto ")?;
                    target.repr(f, context)?;
                    write!(f, ";")?;
                    if context.include_newline {
                        write!(f, "\n")?;
                    }
                }

                indent(f, context);
                write!(f, "default: goto ")?;
                default.repr(f, context)?;
                write!(f, ";")?;

                if context.include_newline {
                    write!(f, "\n")?;
                }

                indent(f, context);
                write!(f, "}}")?;
                Ok(())
            }

            Expression::FnCall { function, args } => {
                function.repr(f, context)?;
                write!(f, "(")?;
                for (i, arg) in args.iter().enumerate() {
                    arg.repr(f, context)?;
                    if i < args.len() - 1 {
                        write!(f, ", ")?;
                    }
                }
                write!(f, ")")?;
                Ok(())
            }
        }
    }
}
