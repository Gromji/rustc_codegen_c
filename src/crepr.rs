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

pub trait Representable {
    fn repr(&self, f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result;
    fn default_repr(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            &RepresentationContext {
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
        f: &mut fmt::Formatter<'_>,
        context: &RepresentationContext,
    ) -> fmt::Result {
        self.repr(f, &RepresentationContext { indent: context.indent + 1, ..context.clone() })
    }
}

pub fn indent(f: &mut fmt::Formatter<'_>, context: &RepresentationContext) -> fmt::Result {
    write!(f, "{}", context.indent_string.as_str().repeat(context.indent))
}
