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

    var_name: Option<String>,
    pub cur_fn: Option<&'ctx CFunction>,
    n_ptr: u8,
}

impl RepresentationContext<'_> {
    pub fn get_variable_name_option(&mut self) -> Option<String> {
        self.var_name.take()
    }

    pub fn get_ptr_count(&mut self) -> u8 {
        let ret = self.n_ptr;
        self.n_ptr = 0;
        ret
    }

    pub fn increment_ptr_count(&mut self) {
        self.n_ptr += 1;
    }
    
    pub fn set_variable_name(&mut self, name: String) {
        self.var_name = Some(name);
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
        self.repr(
            f,
            &mut RepresentationContext {
                indent: context.indent + 1,
                ..context.clone()
            },
        )
    }

    fn default_repr_str(&self) -> String {
        let mut s = String::new();
        self.default_repr(&mut s).unwrap();
        s
    }

    fn repr_str(&self, context: &mut RepresentationContext) -> String {
        let mut s = String::new();
        self.repr(&mut s, context).unwrap();
        s
    }

    fn newline(&self, f: &mut (dyn fmt::Write), context: &RepresentationContext) -> fmt::Result {
        if context.include_newline {
            write!(f, "\n")?;
        }
        Ok(())
    }
}

pub fn indent(f: &mut (dyn fmt::Write), context: &RepresentationContext) -> fmt::Result {
    write!(
        f,
        "{}",
        context.indent_string.as_str().repeat(context.indent)
    )
}
