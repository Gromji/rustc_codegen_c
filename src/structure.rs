use crate::definition::CVarDef;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CStruct {
    name: String,
    fields: Vec<CVarDef>,
}

impl CStruct {
    pub fn new(name: String) -> Self {
        Self { name, fields: Vec::new() }
    }

    pub fn push(&mut self, field: CVarDef) {
        self.fields.push(field);
    }

    #[allow(dead_code)]
    pub fn validate_struct(&self) -> bool {
        todo!("TODO: Would be a good idea to have some kind of validation")
    }
}

impl std::fmt::Display for CStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut definition = format!("struct {} {{\n", self.name);
        for field in &self.fields {
            definition.push_str(&format!("    {};\n", field));
        }
        definition.push_str("};");

        write!(f, "{}", definition)
    }
}
