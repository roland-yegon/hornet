use crate::ast::*;

pub struct Codegen {
    // Emits LLVM IR
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {}
    }
}

impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    pub fn generate(&mut self, program: &Program) -> String {
        let mut ir = String::new();
        ir.push_str("; Hornet Generated LLVM IR\n");
        ir.push_str("declare i32 @printf(i8*, ...)\n\n");
        
        for _stmt in &program.statements {
            // Traverse AST and append IR
        }
        
        ir
    }
}
