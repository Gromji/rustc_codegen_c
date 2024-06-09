use std::{
    env,
    fmt::{self, Debug},
};

use crate::function::CFunction;

// TODO we could pass more information to this context, such as the current function, to allow for more context-aware representations
#[derive(Debug, Clone, Default)]
pub struct RepresentationContext<'ctx> {
    pub indent: usize,
    pub indent_string: String,

    pub include_newline: bool,
    pub include_comments: bool,

    pub var_name: Option<String>,
    pub cur_fn: Option<&'ctx CFunction>,
    pub n_ptr: u8,
}

impl RepresentationContext<'_> {
    pub fn get_variable_name(&mut self) -> String {
        self.var_name.take().expect("Expected variable name in representation context")
    }

    pub fn get_variable_name_option(&mut self) -> Option<String> {
        self.var_name.take()
    }
}

pub trait Representable {
    fn repr(&self, f: &mut (dyn fmt::Write), context: &mut RepresentationContext) -> fmt::Result;

    fn default_repr(&self, f: &mut (dyn fmt::Write)) -> fmt::Result {
        let include = true;
        let comments_env = match env::var("C_CODEGEN_COMMENTS") {
            Ok(val) => val,
            Err(_) => String::from("INCLUDE"),
        };

        let include = match comments_env.as_str() {
            "EXCLUDE" => false,
            "INCLUDE" => true,
            _ => include,
        };

        self.repr(
            f,
            &mut RepresentationContext {
                indent: 1,
                indent_string: "\t".into(),
                include_newline: true,
                include_comments: include,
                ..Default::default()
            },
        )
    }
    fn indented_repr(
        &self,
        f: &mut (dyn fmt::Write),
        context: &mut RepresentationContext,
    ) -> fmt::Result {
        self.repr(f, &mut RepresentationContext { indent: context.indent + 1, ..context.clone() })
    }

    fn repr_str(&self, context: &mut RepresentationContext) -> String {
        let mut s = String::new();
        self.repr(&mut s, context).unwrap();
        s
    }
}

pub fn indent(f: &mut (dyn fmt::Write), context: &RepresentationContext) -> fmt::Result {
    write!(f, "{}", context.indent_string.as_str().repeat(context.indent))
}
