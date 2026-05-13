use crate::ast::*;
use std::collections::{HashMap, HashSet};
use std::fs;

pub struct Codegen {
    next_id: usize,
    globals: Vec<String>,
    ir: String,
    locals: Vec<HashMap<String, (String, String)>>,
    allocas: Vec<Vec<String>>,
    record_defs: HashMap<String, Vec<(String, String)>>,
    imported_modules: HashSet<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            next_id: 0,
            globals: Vec::new(),
            ir: String::new(),
            locals: Vec::new(),
            allocas: Vec::new(),
            record_defs: HashMap::new(),
            imported_modules: HashSet::new(),
        }
    }
}


impl Default for Codegen {
    fn default() -> Self {
        Self::new()
    }
}

impl Codegen {
    fn fresh(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("%t{}", id)
    }

    fn fresh_label(&mut self, prefix: &str) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("{}.{}", prefix, id)
    }

    fn fresh_global(&mut self) -> String {
        let id = self.next_id;
        self.next_id += 1;
        format!("@.str.{}", id)
    }

    fn enter_scope(&mut self) {
        self.locals.push(HashMap::new());
        self.allocas.push(Vec::new());
    }

    fn exit_scope(&mut self) {
        self.locals.pop();
        self.allocas.pop();
    }

    fn current_scope(&mut self) -> &mut HashMap<String, (String, String)> {
        self.locals.last_mut().expect("scope stack must not be empty")
    }

    fn record_fields(&self, name: &str) -> Option<&Vec<(String, String)>> {
        self.record_defs.get(name)
    }

    fn emit_line(&mut self, line: impl AsRef<str>) {
        self.ir.push_str(line.as_ref());
        self.ir.push('\n');
    }

    fn allocate_local(&mut self, name: &str, ty: &str) -> String {
        let ptr = self.fresh();
        let alloca = format!("  {} = alloca {}", ptr, ty);
        self.allocas.last_mut().unwrap().push(alloca);
        self.current_scope().insert(name.to_string(), (ty.to_string(), ptr.clone()));
        ptr
    }

    fn lookup_local(&self, name: &str) -> Option<&(String, String)> {
        for scope in self.locals.iter().rev() {
            if let Some(entry) = scope.get(name) {
                return Some(entry);
            }
        }
        None
    }

    fn emit_string_literal(&mut self, value: &str) -> (String, String) {
        let global_name = self.fresh_global();
        let bytes = value.as_bytes();
        let escaped = bytes.iter().map(|b| format!("\\{:02X}", b)).collect::<String>();
        let len = bytes.len() + 1;
        self.globals.push(format!(
            "{} = private unnamed_addr constant [{} x i8] c\"{}00\"",
            global_name,
            len,
            escaped
        ));
        let target = self.fresh();
        let gep = format!(
            "  {} = getelementptr inbounds [{} x i8], [{} x i8]* {}, i32 0, i32 0",
            target,
            len,
            len,
            global_name
        );
        self.ir.push_str(&gep);
        self.ir.push('\n');
        ("i8*".to_string(), target)
    }

    fn type_to_llvm(&self, type_name: &str) -> String {
        match type_name {
            "Int" => "i64".to_string(),
            "Float" => "double".to_string(),
            "String" => "i8*".to_string(),
            "Bool" => "i1".to_string(),
            "Unit" => "void".to_string(),
            _ => format!("%{}", type_name), // Custom types like records
        }
    }

    fn emit_expr(&mut self, expr: &Expr) -> Result<(String, String), String> {
        match expr {
            Expr::Literal(Literal::Int(value)) => Ok(("i64".to_string(), value.to_string())),
            Expr::Literal(Literal::Float(value)) => Ok(("double".to_string(), value.to_string())),
            Expr::Literal(Literal::String(value)) => Ok(self.emit_string_literal(value)),
            Expr::Literal(Literal::Bool(b)) => Ok(("i1".to_string(), if *b { "1" } else { "0" }.to_string())),
            Expr::Literal(Literal::Unit) => Ok(("void".to_string(), "".to_string())),
            Expr::UnaryOp { op, operand } => {
                let (ty, val) = self.emit_expr(operand)?;
                match op.as_str() {
                    "-" if ty == "i64" => {
                        let target = self.fresh();
                        self.emit_line(format!("  {} = sub i64 0, {}", target, val));
                        Ok(("i64".to_string(), target))
                    }
                    "-" if ty == "double" => {
                        let target = self.fresh();
                        self.emit_line(format!("  {} = fneg double {}", target, val));
                        Ok(("double".to_string(), target))
                    }
                    "not" => {
                        let target = self.fresh();
                        self.emit_line(format!("  {} = xor i1 {}, 1", target, val));
                        Ok(("i1".to_string(), target))
                    }
                    _ => Err(format!("Unknown unary operator: {}", op)),
                }
            }
            Expr::Identifier(name) => {
                let entry = self.lookup_local(name).ok_or_else(|| format!("Undefined variable: {}", name))?;
                let ty = entry.0.clone();
                let ptr = entry.1.clone();
                let target = self.fresh();
                self.emit_line(format!("  {} = load {}, {}* {}", target, ty, ty, ptr));
                Ok((ty, target))
            }
            Expr::BinaryOp { left, op, right } => {
                let (left_ty, left_val) = self.emit_expr(left)?;
                let (_, right_val) = self.emit_expr(right)?;
                let target = self.fresh();
                match op.as_str() {
                    "+" | "-" | "*" | "/" | "%" => {
                        let op_ir = match op.as_str() {
                            "+" => "add",
                            "-" => "sub",
                            "*" => "mul",
                            "/" => "sdiv",
                            "%" => "srem",
                            _ => unreachable!(),
                        };
                        self.emit_line(format!("  {} = {} {} {}, {}", target, op_ir, left_ty, left_val, right_val));
                        Ok((left_ty, target))
                    }
                    "and" => {
                        self.emit_line(format!("  {} = and i64 {}, {}", target, left_val, right_val));
                        Ok(("i64".to_string(), target))
                    }
                    "or" => {
                        self.emit_line(format!("  {} = or i64 {}, {}", target, left_val, right_val));
                        Ok(("i64".to_string(), target))
                    }
                    "not" => {
                        let cmp = self.fresh();
                        self.emit_line(format!("  {} = icmp eq i64 {}, 0", cmp, right_val));
                        let zext = self.fresh();
                        self.emit_line(format!("  {} = zext i1 {} to i64", zext, cmp));
                        Ok(("i64".to_string(), zext))
                    }
                    "==" | "!=" | "<" | ">" | "<=" | ">=" => {
                        let cmp_op = match op.as_str() {
                            "==" => "eq",
                            "!=" => "ne",
                            "<" => "slt",
                            ">" => "sgt",
                            "<=" => "sle",
                            ">=" => "sge",
                            _ => unreachable!(),
                        };
                        self.emit_line(format!("  {} = icmp {} {} {}, {}", target, cmp_op, left_ty, left_val, right_val));
                        let bool_target = self.fresh();
                        self.emit_line(format!("  {} = zext i1 {} to i64", bool_target, target));
                        Ok(("i64".to_string(), bool_target))
                    }
                    _ => Err(format!("Unsupported binary operator: {}", op)),
                }
            }
            Expr::Call { target, args } => {
                if let Expr::Identifier(name) = &**target {
                    if name == "print" {
                        let (arg_ty, arg_val) = self.emit_expr(&args[0])?;
                        if arg_ty == "i8*" {
                            self.emit_line(format!("  call i32 (i8*, ...) @printf(i8* {}, i8* null)", arg_val));
                        } else {
                            let (_, fmt_ptr) = self.emit_string_literal("%ld\n");
                            self.emit_line(format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})", fmt_ptr, arg_val));
                        }
                        Ok(("i64".to_string(), "0".to_string()))
                    } else if let Some(fields) = self.record_fields(name) {
                        if args.len() != fields.len() {
                            return Err(format!("Record constructor '{}' expects {} arguments, got {}", name, fields.len(), args.len()));
                        }
                        let mut arg_pairs = Vec::new();
                        for arg in args {
                            let (arg_ty, arg_val) = self.emit_expr(arg)?;
                            arg_pairs.push(format!("{} {}", arg_ty, arg_val));
                        }
                        let target_val = self.fresh();
                        self.emit_line(format!("  {} = call %{} @\"{}\"({})", target_val, name, name, arg_pairs.join(", ")));
                        Ok((format!("%{}", name), target_val))
                    } else {
                        Err(format!("Unsupported call target: {}", name))
                    }
                } else {
                    Err("Unsupported call expression".to_string())
                }
            }
            Expr::MemberAccess { object, member } => {
                let (obj_ty, obj_val) = self.emit_expr(object)?;
                if let Some(record_name) = obj_ty.strip_prefix('%') {
                    if let Some(fields) = self.record_fields(record_name) {
                        if let Some((index, (_, field_type))) = fields.iter().enumerate().find(|(_, (name, _))| name == member) {
                            let llvm_field_ty = self.type_to_llvm(field_type);
                            let target = self.fresh();
                            self.emit_line(format!("  {} = extractvalue {} {}, {}", target, obj_ty, obj_val, index));
                            return Ok((llvm_field_ty, target));
                        }
                        return Err(format!("Record '{}' has no field named '{}'", record_name, member));
                    }
                }
                Err(format!("Unsupported member access on type {}", obj_ty))
            }
            Expr::Range { .. } | Expr::List(_) | Expr::NamedArg { .. } | Expr::Map(_) | Expr::IndexAccess { .. } => {
                Err("Unsupported expression construct for code generation".to_string())
            }
        }
    }

    fn emit_if(&mut self, condition: &Expr, then_branch: &[Stmt], else_ifs: &[(Expr, Vec<Stmt>)], else_branch: &Option<Vec<Stmt>>) -> Result<(), String> {
        let (_, cond_val) = self.emit_expr(condition)?;
        let cond_bool = self.fresh();
        self.emit_line(format!("  {} = icmp ne i64 {}, 0", cond_bool, cond_val));

        let then_label = self.fresh_label("then");
        let else_label = self.fresh_label("else");
        let end_label = self.fresh_label("end");

        self.emit_line(format!("  br i1 {}, label %{}, label %{}", cond_bool, then_label, else_label));
        self.emit_line(format!("{}:", then_label));
        self.emit_block(then_branch)?;
        self.emit_line(format!("  br label %{}", end_label));

        self.emit_line(format!("{}:", else_label));
        if let Some((first_elif_cond, first_elif_body)) = else_ifs.first() {
            self.emit_if(first_elif_cond, first_elif_body, &else_ifs[1..], else_branch)?;
        } else if let Some(branch) = else_branch {
            self.emit_block(branch)?;
        }
        self.emit_line(format!("  br label %{}", end_label));

        self.emit_line(format!("{}:", end_label));
        Ok(())
    }

    fn emit_while(&mut self, condition: &Expr, body: &[Stmt]) -> Result<(), String> {
        let loop_label = self.fresh_label("loop");
        let body_label = self.fresh_label("body");
        let end_label = self.fresh_label("end");

        self.emit_line(format!("  br label %{}", loop_label));
        self.emit_line(format!("{}:", loop_label));
        let (_, cond_val) = self.emit_expr(condition)?;
        let cond_bool = self.fresh();
        self.emit_line(format!("  {} = icmp ne i64 {}, 0", cond_bool, cond_val));
        self.emit_line(format!("  br i1 {}, label %{}, label %{}", cond_bool, body_label, end_label));

        self.emit_line(format!("{}:", body_label));
        self.emit_block(body)?;
        self.emit_line(format!("  br label %{}", loop_label));

        self.emit_line(format!("{}:", end_label));
        Ok(())
    }

    fn emit_for(&mut self, iterator: &str, iterable: &Expr, body: &[Stmt]) -> Result<(), String> {
        if let Expr::Range { start, end, inclusive } = iterable {
            let (start_ty, start_val) = self.emit_expr(start)?;
            let (end_ty, end_val) = self.emit_expr(end)?;
            if start_ty != "i64" || end_ty != "i64" {
                return Err("Range bounds must be integers".to_string());
            }

            let iter_ptr = self.allocate_local(iterator, "i64");
            self.emit_line(format!("  store i64 {}, i64* {}", start_val, iter_ptr));

            let loop_label = self.fresh_label("for.loop");
            let body_label = self.fresh_label("for.body");
            let end_label = self.fresh_label("for.end");

            self.emit_line(format!("  br label %{}", loop_label));
            self.emit_line(format!("{}:", loop_label));
            let iter_val = self.fresh();
            self.emit_line(format!("  {} = load i64, i64* {}", iter_val, iter_ptr));
            let cmp_target = self.fresh();
            let cmp_op = if *inclusive { "sle" } else { "slt" };
            self.emit_line(format!("  {} = icmp {} i64 {}, {}", cmp_target, cmp_op, iter_val, end_val));
            self.emit_line(format!("  br i1 {}, label %{}, label %{}", cmp_target, body_label, end_label));

            self.emit_line(format!("{}:", body_label));
            self.emit_block(body)?;
            let next_val = self.fresh();
            self.emit_line(format!("  {} = add i64 {}, 1", next_val, iter_val));
            self.emit_line(format!("  store i64 {}, i64* {}", next_val, iter_ptr));
            self.emit_line(format!("  br label %{}", loop_label));

            self.emit_line(format!("{}:", end_label));
            Ok(())
        } else {
            Err("Only range iterables are supported in for loops".to_string())
        }
    }

    fn import_module(&mut self, module_name: &str) -> Result<(), String> {
        if self.imported_modules.contains(module_name) {
            return Ok(());
        }
        self.imported_modules.insert(module_name.to_string());

        let module_path = format!("{}.hn", module_name);
        let source = fs::read_to_string(&module_path).map_err(|e| format!("Could not load module '{}': {}", module_name, e))?;
        let mut lexer = crate::lexer::Lexer::new(&source);
        let tokens = lexer.tokenize().map_err(|e| format!("Lex error in module '{}': {}", module_name, e))?;
        let mut parser = crate::parser::Parser::new(tokens);
        let module_program = parser.parse().map_err(|e| format!("Parse error in module '{}': {}", module_name, e))?;

        for module_stmt in &module_program.statements {
            self.emit_stmt(module_stmt)?;
        }
        Ok(())
    }

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::Let { name, value } => {
                let (ty, val) = self.emit_expr(value)?;
                let ptr = self.allocate_local(name, &ty);
                self.emit_line(format!("  store {} {}, {}* {}", ty, val, ty, ptr));
                Ok(())
            }
            Stmt::Assignment { lhs, value } => {
                if let Expr::Identifier(name) = lhs {
                    let (ty, val) = self.emit_expr(value)?;
                    let ptr = if let Some((_, ptr)) = self.lookup_local(name) {
                        ptr.clone()
                    } else {
                        self.allocate_local(name, &ty)
                    };
                    self.emit_line(format!("  store {} {}, {}* {}", ty, val, ty, ptr));
                    Ok(())
                } else {
                    Err("Only identifier assignments are supported in codegen".to_string())
                }
            }
            Stmt::Expr(expr) => {
                self.emit_expr(expr).map(|_| ())
            }
            Stmt::Return(expr) => {
                let (ty, val) = self.emit_expr(expr)?;
                if ty == "i64" {
                    self.emit_line(format!("  ret i64 {}", val));
                } else {
                    self.emit_line("  ret void");
                }
                Ok(())
            }
            Stmt::If { condition, then_branch, else_ifs, else_branch } => {
                self.emit_if(condition, then_branch, else_ifs, else_branch)
            }
            Stmt::While { condition, body } => self.emit_while(condition, body),
            Stmt::For { iterator, iterable, body } => self.emit_for(iterator, iterable, body),
            Stmt::Loop { body } => {
                // [[PHASE BLOCKED: loop codegen not yet implemented]]
                self.emit_block(body)
            }
            Stmt::Break => {
                // [[PHASE BLOCKED: break codegen not yet implemented]]
                Ok(())
            }
            Stmt::Continue => {
                // [[PHASE BLOCKED: continue codegen not yet implemented]]
                Ok(())
            }
            Stmt::FunctionDef { name, params, body, return_type: _ } => {
                let param_defs = params.iter().enumerate().map(|(index, _)| format!("i64 %arg{}", index)).collect::<Vec<_>>().join(", ");
                let header = format!("define void @{}({}) {{", name, param_defs);
                self.emit_line(header);
                self.enter_scope();
                for (index, (param_name, _)) in params.iter().enumerate() {
                    let ptr = self.allocate_local(param_name, "i64");
                    self.emit_line(format!("  store i64 %arg{}, i64* {}", index, ptr));
                }
                self.emit_block(body)?;
                self.emit_line("  ret void");
                self.exit_scope();
                self.emit_line("}");
                Ok(())
            }
            Stmt::RecordDef { name, fields } => {
                self.record_defs.insert(name.clone(), fields.clone());

                // Define the LLVM struct type
                let field_types: Vec<String> = fields.iter()
                    .map(|(_, ty)| self.type_to_llvm(ty))
                    .collect();
                self.emit_line(&format!("%{} = type {{ {} }}", name, field_types.join(", ")));

                // Create constructor function
                let param_types: Vec<String> = field_types.clone();
                let param_list: Vec<String> = param_types.iter()
                    .enumerate()
                    .map(|(i, ty)| format!("{} %arg{}", ty, i))
                    .collect();

                self.emit_line(&format!("define %{} @\"{}\"({}) {{", name, name, param_list.join(", ")));
                self.enter_scope();

                // Allocate the struct
                let struct_ptr = self.fresh();
                self.emit_line(&format!("  {} = alloca %{}", struct_ptr, name));

                // Store each field
                for (i, (_, field_type)) in fields.iter().enumerate() {
                    let gep = self.fresh();
                    self.emit_line(&format!("  {} = getelementptr %{}, %{}* {}, i32 0, i32 {}", gep, name, name, struct_ptr, i));
                    self.emit_line(&format!("  store {} %arg{}, {}* {}", self.type_to_llvm(field_type), i, self.type_to_llvm(field_type), gep));
                }

                // Load the struct and return it
                let result = self.fresh();
                self.emit_line(&format!("  {} = load %{}, %{}* {}", result, name, name, struct_ptr));
                self.emit_line(&format!("  ret %{} {}", name, result));

                self.exit_scope();
                self.emit_line("}");
                Ok(())
            }
            Stmt::Use(module_name) => {
                self.import_module(module_name)
            }
            Stmt::Match { .. } => {
                // [[PHASE BLOCKED: match codegen not yet implemented]]
                Ok(())
            }
        }
    }

    fn emit_block(&mut self, statements: &[Stmt]) -> Result<(), String> {
        for stmt in statements {
            self.emit_stmt(stmt)?;
        }
        Ok(())
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.ir.push_str("; Hornet Generated LLVM IR\n");
        self.ir.push_str("declare i32 @printf(i8*, ...)\n\n");
        self.enter_scope();
        let mut main_body = Vec::new();
        for stmt in &program.statements {
            match stmt {
                Stmt::FunctionDef { .. } | Stmt::RecordDef { .. } => {
                    self.emit_stmt(stmt).expect("function or record codegen");
                }
                _ => main_body.push(stmt.clone()),
            }
        }

        self.emit_line("define i32 @main() {");
        self.emit_block(&main_body).expect("main codegen");
        self.emit_line("  ret i32 0");
        self.emit_line("}");
        self.exit_scope();

        let mut output = self.globals.join("\n");
        if !output.is_empty() {
            output.push('\n');
        }
        output.push('\n');
        output.push_str(&self.ir);
        output
    }
}
