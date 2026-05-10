use crate::ast::*;
use std::collections::HashMap;

pub struct Codegen {
    next_id: usize,
    globals: Vec<String>,
    ir: String,
    locals: Vec<HashMap<String, (String, String)>>,
    allocas: Vec<Vec<String>>,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen {
            next_id: 0,
            globals: Vec::new(),
            ir: String::new(),
            locals: Vec::new(),
            allocas: Vec::new(),
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

    fn emit_expr(&mut self, expr: &Expr) -> Result<(String, String), String> {
        match expr {
            Expr::Literal(Literal::Number(value)) => Ok(("i64".to_string(), value.to_string())),
            Expr::Literal(Literal::String(value)) => Ok(self.emit_string_literal(value)),
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
                    match name.as_str() {
                        "print" => {
                            let (arg_ty, arg_val) = self.emit_expr(&args[0])?;
                            if arg_ty == "i8*" {
                                self.emit_line(format!("  call i32 (i8*, ...) @printf(i8* {}, i8* null)", arg_val));
                            } else {
                                let (_, fmt_ptr) = self.emit_string_literal("%ld\n");
                                self.emit_line(format!("  call i32 (i8*, ...) @printf(i8* {}, i64 {})", fmt_ptr, arg_val));
                            }
                            Ok(("i64".to_string(), "0".to_string()))
                        }
                        _ => Err(format!("Unsupported call target: {}", name)),
                    }
                } else {
                    Err("Unsupported call expression".to_string())
                }
            }
            Expr::MemberAccess { .. } | Expr::Range { .. } | Expr::List(_) | Expr::NamedArg { .. } | Expr::Map(_) | Expr::IndexAccess { .. } => {
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

    fn emit_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
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
            Stmt::FunctionDef { name, params, body } => {
                let param_defs = params.iter().enumerate().map(|(index, _)| format!("i64 %arg{}", index)).collect::<Vec<_>>().join(", ");
                let header = format!("define void @{}({}) {{", name, param_defs);
                self.emit_line(header);
                self.enter_scope();
                for (index, param) in params.iter().enumerate() {
                    let ptr = self.allocate_local(param, "i64");
                    self.emit_line(format!("  store i64 %arg{}, i64* {}", index, ptr));
                }
                self.emit_block(body)?;
                self.emit_line("  ret void");
                self.exit_scope();
                self.emit_line("}");
                Ok(())
            }
            _ => Ok(()),
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
            if let Stmt::FunctionDef { .. } = stmt {
                self.emit_stmt(stmt).expect("function codegen");
            } else {
                main_body.push(stmt.clone());
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
