use crate::ty::CType;
use std::fmt::{self, Debug};

// TODO we could pass more information to this context, such as the current function, to allow for more context-aware representations
#[derive(Debug, Clone, Default)]
pub struct RepresentationContext {
    pub indent: usize,
    pub indent_string: String,

    pub include_newline: bool,
    pub include_comments: bool,

    pub var_name: Option<String>,
    pub n_ptr: u8,
}

pub trait Representable {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result;
    fn default_repr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
    fn indented_repr(
        &self,
        f: &mut fmt::Formatter<'_>,
        context: &RepresentationContext,
    ) -> fmt::Result {
        self.repr(f, &RepresentationContext { indent: context.indent + 1, ..context.clone() })
    }
}

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
    Constant { value: String },
    Variable { local: usize, idx: Option<usize> }, // TODO this might not be an appropriate representation, especially if we plan to add debug info into the mix
    Assignment { lhs: Box<Expression>, rhs: Box<Expression> },
    BinaryOp { op: BinOpType, lhs: Box<Expression>, rhs: Box<Expression> },
    CheckedBinaryOp { op: BinOpType, lhs: Box<Expression>, rhs: Box<Expression> },
    UnaryOp { op: UnaryOpType, val: Box<Expression> },
    Struct { name: String, fields: Vec<Expression> },
    Array { fields: Vec<Expression> },
    Return { value: Box<Expression> },
    NoOp {},
}

impl Representable for Expression {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
        match self {
            Expression::Constant { value } => {
                write!(f, "{}", value)
            }

            Expression::Variable { local, idx } => match idx {
                Some(idx) => {
                    write!(f, "var{}[{}]", local, idx)
                }
                None => {
                    write!(f, "var{}", local)
                }
            },

            Expression::Assignment { lhs, rhs } => {
                // {} = {}
                lhs.repr(f, context)?;
                write!(f, " = ")?;
                rhs.repr(f, context)?;
                Ok(())
            }

            Expression::CheckedBinaryOp { op, lhs, rhs } => {
                /*
                   we could handle these functions with __builtin_add_overflow, __builtin_sub_overflow, __builtin_mul_overflow,
                   however these are GCC extensions which are not available in all compilers.
                */
                match op {
                    BinOpType::Add | BinOpType::Sub | BinOpType::Mul => {
                        lhs.repr(f, context)?;
                        write!(f, " ")?;
                        op.repr(f, context)?;
                        write!(f, " ")?;
                        rhs.repr(f, context)?;
                        Ok(())
                    }

                    _ => {
                        unreachable!("CheckedBinaryOp doesn't exist for type {:?}", op)
                    }
                }
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
                write!(f, "(struct {}){{ ", name)?;
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
        }
    }
}

pub fn indent(f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
    write!(f, "{}", context.indent_string.as_str().repeat(context.indent))
}
