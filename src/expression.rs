use crate::{
    bb::BasicBlockIdentifier,
    crepr::{indent, Representable, RepresentationContext},
    fatptr::{FAT_PTR_DATA_FIELD, FAT_PTR_NAME},
    ty::CType,
};
use std::{
    fmt,
    ops::{Add, BitAnd, BitOr, Div, Mul, Sub},
};

#[allow(dead_code)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BinOpType {
    Add,
    Sub,
    Mul,
    CheckedAdd,
    CheckedSub,
    CheckedMul,
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
            rustc_middle::mir::BinOp::AddWithOverflow => BinOpType::CheckedAdd,
            rustc_middle::mir::BinOp::SubWithOverflow => BinOpType::CheckedSub,
            rustc_middle::mir::BinOp::MulWithOverflow => BinOpType::CheckedMul,
        }
    }
}

impl Representable for BinOpType {
    fn repr(&self, f: &mut (dyn fmt::Write), _context: &mut RepresentationContext) -> fmt::Result {
        match self {
            BinOpType::Add => write!(f, "+"),
            BinOpType::Sub => write!(f, "-"),
            BinOpType::Mul => write!(f, "*"),
            BinOpType::CheckedAdd => write!(f, "+"),
            BinOpType::CheckedSub => write!(f, "-"),
            BinOpType::CheckedMul => write!(f, "*"),
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
            BinOpType::CheckedAdd => write!(f, "checked_add"),
            BinOpType::CheckedSub => write!(f, "checked_sub"),
            BinOpType::CheckedMul => write!(f, "checked_mul"),
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
    fn repr(&self, f: &mut (dyn fmt::Write), _context: &mut RepresentationContext) -> fmt::Result {
        match self {
            UnaryOpType::Neg => write!(f, "-"),
            UnaryOpType::Not => write!(f, "!"),
            UnaryOpType::BitNot => write!(f, "~"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VariableAccess {
    Reference,
    Dereference,
    Field { name: String },
    Index { expression: Expression },
    Cast { ty: CType },
    FatPtrDereference { ty: CType },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Constant {
        value: String,
    },
    Variable {
        local: usize,
        access: Vec<VariableAccess>,
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

    /*
       typedef struct {
           int a;
           int b;
       } TestStruct;

       TestStruct test = { .a = 1, .b = 2 };

       as far as I am aware, this is a GCC extension, not sure if all compilers allow it;
    */
    NamedStruct {
        name: Box<Expression>,
        fields: Vec<(String, Expression)>,
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
    InlineAsm {
        asm: String,
    },
    Cast {
        ty: CType,
        value: Box<Expression>,
    },
}
impl Expression {
    /// Returns Expression::Assignment
    pub fn assign(&self, rhs: Box<Expression>) -> Expression {
        Expression::Assignment {
            lhs: Box::new(self.clone()),
            rhs,
        }
    }
    pub fn gt(&self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Gt,
            lhs: Box::new(self.clone()),
            rhs,
        })
    }
    pub fn lt(&self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Lt,
            lhs: Box::new(self.clone()),
            rhs,
        })
    }
    pub fn neq(&self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Ne,
            lhs: Box::new(self.clone()),
            rhs,
        })
    }
    pub fn equ(&self, rhs: Box<Expression>) -> Box<Expression> {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Eq,
            lhs: Box::new(self.clone()),
            rhs,
        })
    }
    pub fn constant(value: &String) -> Box<Expression> {
        Box::new(Expression::Constant {
            value: value.clone(),
        })
    }
    pub fn arr_vari(local: usize, idx: usize) -> Box<Expression> {
        Box::new(Expression::Variable {
            local,
            access: vec![VariableAccess::Index {
                expression: Expression::const_int(idx as i128),
            }],
        })
    }
    pub fn vari(local: usize) -> Box<Expression> {
        Box::new(Expression::unbvari(local))
    }
    pub fn unbvari(local: usize) -> Expression {
        Expression::Variable {
            local,
            access: Vec::new(),
        }
    }
    pub fn strct(name: Box<Expression>, fields: Vec<Expression>) -> Box<Expression> {
        Box::new(Expression::Struct { name, fields })
    }

    pub fn const_int(value: i128) -> Expression {
        Expression::Constant {
            value: value.to_string(),
        }
    }

    pub fn fatptr(data: Expression, meta: Expression) -> Expression {
        Expression::Struct {
            name: Box::new(Expression::Constant {
                value: FAT_PTR_NAME.to_string(),
            }),
            fields: vec![data, meta],
        }
    }
}
impl Add for Box<Expression> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Add,
            lhs: self,
            rhs,
        })
    }
}
impl Sub for Box<Expression> {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Sub,
            lhs: self,
            rhs,
        })
    }
}
impl Mul for Box<Expression> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Mul,
            lhs: self,
            rhs,
        })
    }
}
impl Div for Box<Expression> {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Div,
            lhs: self,
            rhs,
        })
    }
}
impl BitOr for Box<Expression> {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::Or,
            lhs: self,
            rhs,
        })
    }
}
impl BitAnd for Box<Expression> {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self {
        Box::new(Expression::BinaryOp {
            op: BinOpType::And,
            lhs: self,
            rhs,
        })
    }
}

impl Representable for Expression {
    fn repr(&self, f: &mut (dyn fmt::Write), context: &mut RepresentationContext) -> fmt::Result {
        match self {
            Expression::Constant { value } => {
                write!(f, "{}", value)
            }

            Expression::Variable { local, access } => {
                if context.cur_fn.is_none() {
                    panic!("No current function set in context");
                }

                let var = context.cur_fn.unwrap().get_local_var(*local);

                let mut var_repr = var.get_name();

                for access in access {
                    match access {
                        VariableAccess::Reference => {
                            var_repr = format!("&{}", var_repr);
                        }

                        VariableAccess::Dereference => {
                            var_repr = format!("(*{})", var_repr);
                        }

                        VariableAccess::Field { name } => {
                            var_repr = format! {"{}.{}", var_repr, name};
                        }

                        VariableAccess::Index { expression } => {
                            var_repr = format! {"{}[{}]", var_repr, expression.repr_str(context)};
                        }

                        VariableAccess::Cast { ty } => {
                            var_repr = format!("(({}){})", ty.repr_str(context), var_repr);
                        }

                        VariableAccess::FatPtrDereference { ty } => {
                            let mut ch_ctx = context.clone();
                            ch_ctx.var_name = Some("".to_string());

                            var_repr = format!(
                                "(({}) ({}.{}))",
                                ty.repr_str(&ch_ctx),
                                var_repr,
                                FAT_PTR_DATA_FIELD
                            );
                        }
                    }
                }

                write!(f, "{}", var_repr)
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

            Expression::NamedStruct { name, fields } => {
                // (struct {}){ {} } (eg. (struct struct_name) { {1}, {2} })
                write!(f, "(")?;
                name.repr(f, context)?;
                write!(f, "){{ ")?;
                for (i, (field_name, field)) in fields.iter().enumerate() {
                    if i != 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, ".{} = ", field_name)?;
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

            Expression::SwitchJump {
                value,
                cases,
                default,
            } => {
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

                indent(f, context)?;
                write!(f, "default: goto ")?;
                default.repr(f, context)?;
                write!(f, ";")?;

                if context.include_newline {
                    write!(f, "\n")?;
                }

                indent(f, context)?;
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

            Expression::InlineAsm { asm } => {
                write!(f, "asm(\"{}\")", asm)
            }
            Expression::Cast { ty, value } => {
                write!(f, "(")?;
                ty.repr(f, context)?;
                write!(f, ")")?;
                value.repr(f, context)
            }
        }
    }
}
