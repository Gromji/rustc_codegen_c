pub struct Prefix {
    code: String,
}

impl Prefix {
    pub fn new() -> Self {
        Self { code: String::new() }
    }

    pub fn push(&mut self, code: &str, newline: bool) {
        self.code.push_str(code);
        if newline {
            self.code.push_str("\n");
        }
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }
}

pub fn build_prefix(prefix: &mut Prefix) {
    prefix.push(&prefix_includes().as_str(), true);
    prefix.push(&prefix_functions().as_str(), true);
    prefix.push(&prefix_structs().as_str(), true);
}

// Greedy list of includes
fn prefix_includes() -> String {
    [
        "#include <stdio.h>",
        "#include <stdlib.h>",
        "#include <math.h>",
        "#include <string.h>",
        "#include <ctype.h>",
        "#include <time.h>",
        "#include <stddef.h>",
        "#include <limits.h>",
        "#include <float.h>",
        "#include <stdbool.h>",
        "#include <assert.h>",
        "#include <errno.h>",
        "#include <signal.h>",
        "#include <setjmp.h>",
        "#include <stdarg.h>",
        "#include <locale.h>",
        "#include <wchar.h>",
        "#include <complex.h>",
    ]
    .join("\n")
}

fn prefix_functions() -> String {
    "// TODO: Implement prefix functions here!".to_string()
}

fn prefix_structs() -> String {
    "// TODO: Implement prefix structs here!".to_string()
}
