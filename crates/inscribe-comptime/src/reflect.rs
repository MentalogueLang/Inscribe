use std::collections::HashMap;

use inscribe_mir::{LocalId, MirFunction, MirProgram};

pub fn qualified_function_name(function: &MirFunction) -> String {
    function
        .receiver
        .as_ref()
        .map(|receiver| format!("{receiver}.{}", function.name))
        .unwrap_or_else(|| function.name.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionReflection {
    pub qualified_name: String,
    pub param_count: usize,
    pub return_type: String,
    pub local_count: usize,
    pub block_count: usize,
    pub is_declaration: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalReflection {
    pub id: LocalId,
    pub name: String,
    pub ty: String,
    pub mutable: bool,
    pub temp: bool,
}

#[derive(Debug)]
pub struct MirReflection<'a> {
    program: &'a MirProgram,
    by_name: HashMap<String, usize>,
}

impl<'a> MirReflection<'a> {
    pub fn new(program: &'a MirProgram) -> Self {
        let by_name = program
            .functions
            .iter()
            .enumerate()
            .map(|(index, function)| (qualified_function_name(function), index))
            .collect();

        Self { program, by_name }
    }

    pub fn program(&self) -> &'a MirProgram {
        self.program
    }

    pub fn functions(&self) -> impl Iterator<Item = &'a MirFunction> {
        self.program.functions.iter()
    }

    pub fn callable_names(&self) -> Vec<String> {
        self.program
            .functions
            .iter()
            .map(qualified_function_name)
            .collect()
    }

    pub fn function(&self, name: &str) -> Option<&'a MirFunction> {
        self.by_name
            .get(name)
            .map(|index| &self.program.functions[*index])
    }

    pub fn describe_function(&self, name: &str) -> Option<FunctionReflection> {
        let function = self.function(name)?;
        Some(FunctionReflection {
            qualified_name: qualified_function_name(function),
            param_count: function.signature.params.len(),
            return_type: function.signature.return_type.display_name(),
            local_count: function.locals.len(),
            block_count: function.blocks.len(),
            is_declaration: function.is_declaration,
        })
    }

    pub fn locals(&self, name: &str) -> Option<Vec<LocalReflection>> {
        let function = self.function(name)?;
        Some(
            function
                .locals
                .iter()
                .map(|local| LocalReflection {
                    id: local.id,
                    name: local.name.clone(),
                    ty: local.ty.display_name(),
                    mutable: local.mutable,
                    temp: local.temp,
                })
                .collect(),
        )
    }
}
